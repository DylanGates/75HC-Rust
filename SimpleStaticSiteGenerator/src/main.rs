use anyhow::{Context, Result};
use chrono::NaiveDate;
use clap::Parser;
use pulldown_cmark::{html, Options, Parser as MdParser};
use serde::Deserialize;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::mpsc::{channel, RecvTimeoutError};
use std::time::Duration;

#[derive(Parser)]
#[command(name = "SimpleStaticSiteGenerator", about = "Convert Markdown files to HTML pages")]
struct Cli {
    /// Input directory with Markdown files
    #[arg(short, long, default_value = "./content")]
    input: PathBuf,

    /// Output directory for generated HTML
    #[arg(short, long, default_value = "./dist")]
    output: PathBuf,

    /// Path to header template (HTML snippet)
    #[arg(long)]
    header: Option<PathBuf>,

    /// Path to footer template (HTML snippet)
    #[arg(long)]
    footer: Option<PathBuf>,

    /// Watch for changes and rebuild
    #[arg(short, long)]
    watch: bool,
}

// ─── Frontmatter parsing ───────────────────────────────────────────────────────

#[derive(Debug, Default, Deserialize)]
struct Frontmatter {
    title: Option<String>,
    date: Option<String>,
    #[serde(flatten)]
    extra: HashMap<String, serde_json::Value>,
}

fn parse_frontmatter(content: &str) -> (Option<Frontmatter>, &str) {
    let content = content.trim();
    if !content.starts_with("---") {
        return (None, content);
    }

    // Find the closing ---
    let end = content[3..].find("\n---").map(|pos| pos + 3);
    match end {
        Some(end_pos) => {
            let fm_str = &content[3..end_pos].trim();
            let body = content[end_pos + 4..].trim();
            match serde_json::from_str::<Frontmatter>(fm_str) {
                // Try JSON first
                Ok(fm) => (Some(fm), body),
                Err(_) => {
                    // Try simple key: value parsing
                    let mut fm = Frontmatter::default();
                    for line in fm_str.lines() {
                        if let Some((key, value)) = line.split_once(':') {
                            let key = key.trim().to_lowercase();
                            let value = value.trim().trim_matches('"').to_string();
                            match key.as_str() {
                                "title" => fm.title = Some(value),
                                "date" => fm.date = Some(value),
                                _ => {}
                            }
                        }
                    }
                    (Some(fm), body)
                }
            }
        }
        None => (None, content),
    }
}

// ─── Markdown to HTML ──────────────────────────────────────────────────────────

fn md_to_html(markdown: &str) -> String {
    let options = Options::all();
    let parser = MdParser::new_ext(markdown, options);
    let mut html_output = String::new();
    html::push_html(&mut html_output, parser);
    html_output
}

// ─── Templates ─────────────────────────────────────────────────────────────────

fn load_template(path: &Option<PathBuf>) -> Result<Option<String>> {
    match path {
        Some(p) => {
            let content = std::fs::read_to_string(p)
                .with_context(|| format!("Failed to read template: {}", p.display()))?;
            Ok(Some(content))
        }
        None => Ok(None),
    }
}

fn wrap_html(
    body: &str,
    title: &str,
    header: &Option<String>,
    footer: &Option<String>,
) -> String {
    let h = header.as_deref().unwrap_or("");
    let f = footer.as_deref().unwrap_or("");
    format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="utf-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{title}</title>
    <style>
        body {{ font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif; max-width: 800px; margin: 0 auto; padding: 1em 2em; line-height: 1.6; color: #333; }}
        h1, h2, h3 {{ color: #1a1a1a; }}
        code {{ background: #f4f4f4; padding: 2px 5px; border-radius: 3px; }}
        pre {{ background: #f4f4f4; padding: 1em; border-radius: 5px; overflow-x: auto; }}
        blockquote {{ border-left: 4px solid #ddd; margin-left: 0; padding-left: 1em; color: #666; }}
        a {{ color: #0366d6; }}
        img {{ max-width: 100%; }}
        nav {{ margin-bottom: 2em; }}
        nav a {{ margin-right: 1em; }}
        footer {{ margin-top: 3em; padding-top: 1em; border-top: 1px solid #ddd; color: #888; }}
        .date {{ color: #888; font-size: 0.9em; }}
    </style>
</head>
<body>
    {h}
    <article>
        {body}
    </article>
    {f}
</body>
</html>"#,
    )
}

// ─── File processing ───────────────────────────────────────────────────────────

fn process_file(
    md_path: &Path,
    output_dir: &Path,
    header: &Option<String>,
    footer: &Option<String>,
) -> Result<()> {
    let content = std::fs::read_to_string(md_path)
        .with_context(|| format!("Failed to read: {}", md_path.display()))?;

    let (fm, body) = parse_frontmatter(&content);
    let html_body = md_to_html(body);

    let title = fm
        .as_ref()
        .and_then(|f| f.title.clone())
        .unwrap_or_else(|| {
            md_path
                .file_stem()
                .map(|s| s.to_string_lossy().to_string())
                .unwrap_or_else(|| "Untitled".to_string())
        });

    // Create output path
    let rel_path = md_path
        .parent()
        .and_then(|p| p.file_name())
        .map(|p| PathBuf::from(p))
        .unwrap_or_else(|| PathBuf::from("."));
    let out_dir = output_dir.join(&rel_path);
    std::fs::create_dir_all(&out_dir)
        .with_context(|| format!("Failed to create output dir: {}", out_dir.display()))?;

    let html_file_name = md_path
        .file_stem()
        .map(|s| format!("{}.html", s.to_string_lossy()))
        .unwrap_or_else(|| "index.html".to_string());
    let out_path = out_dir.join(html_file_name);

    let full_html = wrap_html(&html_body, &title, header, footer);
    std::fs::write(&out_path, &full_html)
        .with_context(|| format!("Failed to write: {}", out_path.display()))?;

    println!("  ✓ {} -> {}", md_path.display(), out_path.display());
    Ok(())
}

fn build_site(
    input_dir: &Path,
    output_dir: &Path,
    header: &Option<String>,
    footer: &Option<String>,
) -> Result<()> {
    if !input_dir.exists() {
        anyhow::bail!("Input directory does not exist: {}", input_dir.display());
    }

    // Clean output
    if output_dir.exists() {
        std::fs::remove_dir_all(output_dir)?;
    }
    std::fs::create_dir_all(output_dir)?;

    println!("Building site...");
    println!("  Input:  {}", input_dir.display());
    println!("  Output: {}", output_dir.display());

    let mut count = 0u32;
    let entries = walkdir(input_dir)?;
    for entry in entries {
        if entry.extension().map_or(false, |e| e == "md" || e == "markdown") {
            process_file(&entry, output_dir, header, footer)?;
            count += 1;
        }
    }

    // Generate index.html
    generate_index(input_dir, output_dir, header, footer)?;

    println!("\nDone! Generated {count} page(s).");
    Ok(())
}

fn walkdir(dir: &Path) -> Result<Vec<PathBuf>> {
    let mut entries = Vec::new();
    if dir.is_dir() {
        for entry in std::fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                entries.extend(walkdir(&path)?);
            } else {
                entries.push(path);
            }
        }
    }
    Ok(entries)
}

fn generate_index(
    input_dir: &Path,
    output_dir: &Path,
    header: &Option<String>,
    footer: &Option<String>,
) -> Result<()> {
    let entries = walkdir(input_dir)?;
    let md_files: Vec<_> = entries
        .iter()
        .filter(|p| p.extension().map_or(false, |e| e == "md" || e == "markdown"))
        .collect();

    if md_files.is_empty() {
        return Ok(());
    }

    let mut pages = Vec::new();
    for path in md_files {
        let content = std::fs::read_to_string(path)?;
        let (fm, _) = parse_frontmatter(&content);
        let title = fm.as_ref().and_then(|f| f.title.clone()).unwrap_or_default();
        let date_str = fm.as_ref().and_then(|f| f.date.clone());

        // Parse date for sorting
        let date_parsed = date_str
            .as_ref()
            .and_then(|d| NaiveDate::parse_from_str(d, "%Y-%m-%d").ok());

        let rel = path.strip_prefix(input_dir).unwrap_or(path);
        let link = rel.with_extension("html");

        pages.push((date_parsed, date_str, title, link));
    }

    // Sort by date descending
    pages.sort_by(|a, b| b.0.cmp(&a.0));

    let mut list_items = String::new();
    for (_, date_str, title, link) in &pages {
        let date_display = date_str
            .as_ref()
            .map(|d| format!(" <span class=\"date\">({d})</span>"))
            .unwrap_or_default();
        list_items.push_str(&format!(
            "<li><a href=\"{}\">{}</a>{}</li>\n",
            link.display(),
            if title.is_empty() {
                link.display().to_string()
            } else {
                title.clone()
            },
            date_display
        ));
    }

    let body = format!("<h1>Pages</h1>\n<ul>\n{}</ul>", list_items);
    let index_html = wrap_html(&body, "Home", header, footer);
    std::fs::write(output_dir.join("index.html"), &index_html)?;
    println!("  ✓ index.html generated with {} page(s)", pages.len());

    Ok(())
}

// ─── File watching ─────────────────────────────────────────────────────────────

fn watch_and_rebuild(
    input_dir: &Path,
    output_dir: &Path,
    header: &Option<String>,
    footer: &Option<String>,
) -> Result<()> {
    use notify::{Config, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};

    let (tx, rx) = channel();
    let mut watcher = RecommendedWatcher::new(tx, Config::default())?;
    watcher.watch(input_dir, RecursiveMode::Recursive)?;

    println!("Watching for changes in {}...", input_dir.display());

    loop {
        match rx.recv_timeout(Duration::from_secs(1)) {
            Ok(Ok(Event {
                kind: EventKind::Create(_) | EventKind::Modify(_) | EventKind::Remove(_),
                ..
            })) => {
                println!("\nChange detected. Rebuilding...");
                if let Err(e) = build_site(input_dir, output_dir, header, footer) {
                    eprintln!("Build error: {e}");
                }
                println!("\nWatching for changes...");
            }
            Ok(Ok(_)) => {}
            Ok(Err(e)) => eprintln!("Watch error: {e}"),
            Err(RecvTimeoutError::Timeout) => {}
            Err(RecvTimeoutError::Disconnected) => {
                anyhow::bail!("File watcher disconnected");
            }
        }
    }
}

// ─── Main ──────────────────────────────────────────────────────────────────────

fn main() -> Result<()> {
    let cli = Cli::parse();

    let header = load_template(&cli.header)?;
    let footer = load_template(&cli.footer)?;

    let input_dir = if cli.input.is_relative() {
        std::env::current_dir()?.join(&cli.input)
    } else {
        cli.input.clone()
    };

    let output_dir = if cli.output.is_relative() {
        std::env::current_dir()?.join(&cli.output)
    } else {
        cli.output.clone()
    };

    build_site(&input_dir, &output_dir, &header, &footer)?;

    if cli.watch {
        watch_and_rebuild(&input_dir, &output_dir, &header, &footer)?;
    }

    Ok(())
}
