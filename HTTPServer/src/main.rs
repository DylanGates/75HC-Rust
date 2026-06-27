use anyhow::{Context, Result};
use clap::Parser;
use mime_guess::from_path;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use tiny_http::{Header, Response, Server};

#[derive(Parser)]
#[command(name = "HTTPServer", about = "A tiny static file HTTP server")]
struct Cli {
    /// Port to listen on
    #[arg(short, long, default_value = "8080")]
    port: u16,

    /// Directory to serve files from
    #[arg(short, long, default_value = "./public")]
    dir: PathBuf,

    /// Custom 404 page (HTML file path)
    #[arg(short = 'e', long)]
    custom_404: Option<PathBuf>,

    /// Enable directory listing
    #[arg(short = 'l', long)]
    directory_listing: bool,
}

fn guess_mime(path: &Path) -> String {
    from_path(path).first_or_octet_stream().to_string()
}

fn build_dir_listing(path: &Path, request_path: &str) -> String {
    let mut html = String::new();
    html.push_str(&format!(
        "<!DOCTYPE html><html><head><meta charset='utf-8'><title>Directory: {}</title>",
        request_path
    ));
    html.push_str("<style>body{font-family:sans-serif;margin:2em}ul{list-style:none;padding:0}li{padding:4px 0}a{text-decoration:none;color:#0366d6}a:hover{text-decoration:underline}</style>");
    html.push_str("</head><body>");
    html.push_str(&format!("<h1>Directory: {}</h1><ul>", request_path));

    if request_path != "/" {
        let parent = Path::new(request_path).parent().unwrap_or(Path::new("/"));
        let parent_str = parent.to_string_lossy();
        let parent_str = if parent_str.ends_with('/') || parent_str.is_empty() {
            "/".to_string()
        } else {
            format!("{}/", parent_str)
        };
        html.push_str(&format!("<li><a href=\"{}\">..</a></li>", parent_str));
    }

    if let Ok(entries) = fs::read_dir(path) {
        let mut items: Vec<_> = entries
            .filter_map(|e| e.ok())
            .filter(|e| !e.file_name().to_string_lossy().starts_with('.'))
            .collect();
        items.sort_by_key(|e| e.file_name());

        for entry in items {
            let name = entry.file_name().to_string_lossy().to_string();
            let is_dir = entry.file_type().map(|t| t.is_dir()).unwrap_or(false);
            let suffix = if is_dir { "/" } else { "" };
            let href = if request_path.ends_with('/') {
                format!("{}{}", request_path, name)
            } else {
                format!("{}/{}", request_path, name)
            };
            html.push_str(&format!("<li><a href=\"{}\">{}{}</a></li>", href, name, suffix));
        }
    }

    html.push_str("</ul></body></html>");
    html
}

fn serve_file(path: &Path, request_path: &str, cli: &Cli) -> Response<std::io::Cursor<Vec<u8>>> {
    if !path.exists() {
        // Try index.html for directories
        if path.is_dir() {
            let index = path.join("index.html");
            if index.exists() {
                return serve_file_response(&index);
            }
            if cli.directory_listing {
                let html = build_dir_listing(path, request_path);
                let len = html.len();
                return Response::from_string(html)
                    .with_header(
                        Header::from_bytes(&b"Content-Type"[..], &b"text/html; charset=utf-8"[..])
                            .unwrap(),
                    )
                    .with_status_code(200);
            }
        }
        return serve_404(cli);
    }

    if path.is_dir() {
        let index = path.join("index.html");
        if index.exists() {
            return serve_file_response(&index);
        }
        if cli.directory_listing {
            let html = build_dir_listing(path, request_path);
            return Response::from_string(html)
                .with_header(
                    Header::from_bytes(&b"Content-Type"[..], &b"text/html; charset=utf-8"[..])
                        .unwrap(),
                )
                .with_status_code(200);
        }
        return serve_404(cli);
    }

    serve_file_response(path)
}

fn serve_file_response(path: &Path) -> Response<std::io::Cursor<Vec<u8>>> {
    match fs::read(path) {
        Ok(data) => {
            let mime = guess_mime(path);
            Response::from_data(data)
                .with_header(
                    Header::from_bytes(&b"Content-Type"[..], mime.as_bytes()).unwrap(),
                )
        }
        Err(_) => Response::from_string("500 Internal Server Error")
            .with_status_code(500),
    }
}

fn serve_404(cli: &Cli) -> Response<std::io::Cursor<Vec<u8>>> {
    if let Some(ref custom_path) = cli.custom_404 {
        if custom_path.exists() {
            return serve_file_response(custom_path);
        }
    }
    let body = "<!DOCTYPE html><html><head><meta charset='utf-8'><title>404 Not Found</title></head><body><h1>404 — Not Found</h1><p>The requested resource was not found on this server.</p></body></html>";
    Response::from_string(body)
        .with_header(
            Header::from_bytes(&b"Content-Type"[..], &b"text/html; charset=utf-8"[..]).unwrap(),
        )
        .with_status_code(404)
}

fn sanitize_path(base: &Path, requested: &str) -> PathBuf {
    // Remove query strings and fragments
    let clean = requested.split('?').next().unwrap_or(requested);
    let clean = clean.split('#').next().unwrap_or(clean);
    // Remove leading slash and normalize
    let clean = clean.trim_start_matches('/');
    base.join(clean)
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    let serve_dir = if cli.dir.is_relative() {
        std::env::current_dir()
            .context("Failed to get current directory")?
            .join(&cli.dir)
    } else {
        cli.dir.clone()
    };

    if !serve_dir.exists() {
        fs::create_dir_all(&serve_dir)
            .with_context(|| format!("Failed to create serve directory: {}", serve_dir.display()))?;
        println!("Created serve directory: {}", serve_dir.display());
    }

    let addr = format!("0.0.0.0:{}", cli.port);
    let server = Server::http(&addr)
        .map_err(|e| anyhow::anyhow!("Failed to bind to {addr}: {e}"))?;

    let request_count = Arc::new(AtomicUsize::new(0));

    println!("HTTPServer running on http://localhost:{}", cli.port);
    println!("Serving directory: {}", serve_dir.display());
    println!("Directory listing: {}", if cli.directory_listing { "enabled" } else { "disabled" });
    println!("Press Ctrl+C to stop.\n");

    for request in server.incoming_requests() {
        let count = request_count.fetch_add(1, Ordering::SeqCst) + 1;
        let url = request.url().to_string();
        let method = request.method().as_str().to_string();
        let path = sanitize_path(&serve_dir, &url);

        println!("[{count}] {method} {url} -> {}", path.display());

        let response = match method.as_str() {
            "GET" | "HEAD" => serve_file(&path, &url, &cli),
            _ => Response::from_string("405 Method Not Allowed")
                .with_header(
                    Header::from_bytes(&b"Content-Type"[..], &b"text/plain"[..]).unwrap(),
                )
                .with_status_code(405),
        };

        if let Err(e) = request.respond(response) {
            eprintln!("[{count}] Failed to send response: {e}");
        }
    }

    Ok(())
}
