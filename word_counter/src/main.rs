use std::fs::File;
use std::io::{self, BufRead, Write};
use std::path::Path;
use clap::{Parser, ValueEnum};
use serde::Serialize;
use walkdir::WalkDir;
use indicatif::{ProgressBar, ProgressStyle};
use rayon::prelude::*;
use std::sync::Mutex;

#[derive(Parser)]
#[command(name = "word_counter")]
#[command(about = "A tool to count characters in text files")]
struct Args {
    #[arg(short, long, num_args = 1.., required = true)]
    input: Vec<String>,

    #[arg(short, long)]
    output: Option<String>,

    #[arg(short, long, value_enum, default_value = "text")]
    format: OutputFormat,

    #[arg(short, long)]
    summary: bool,

    #[arg(short, long)]
    recursive: bool,

    #[arg(short = 'x', long, num_args = 0..)]
    extensions: Vec<String>,
}

#[derive(Clone, ValueEnum, PartialEq)]
enum OutputFormat {
    Text,
    Json,
    Csv,
}

#[derive(Serialize)]
struct LineResult {
    line_number: usize,
    content: String,
    char_count: usize,
}

#[derive(Serialize)]
struct Summary {
    total_lines: usize,
    total_chars: usize,
    average_chars_per_line: f64,
}

struct FileProcessingResult {
    results: Vec<LineResult>,
    chars: usize,
    lines: usize,
}

fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>>
where
    P: AsRef<Path>,
{
    let file = File::open(filename)?;
    Ok(io::BufReader::new(file).lines())
}

fn collect_files(args: &Args) -> Vec<String> {
    let mut files = Vec::new();
    let extensions: std::collections::HashSet<String> = args.extensions.iter().cloned().collect();

    for input in &args.input {
        let path = Path::new(input);
        if path.is_file() {
            if extensions.is_empty() || has_valid_extension(path, &extensions) {
                files.push(input.clone());
            }
        } else if path.is_dir() && args.recursive {
            for entry in WalkDir::new(path).into_iter().filter_map(|e| e.ok()) {
                if entry.file_type().is_file() && (extensions.is_empty() || has_valid_extension(entry.path(), &extensions)) {
                    files.push(entry.path().to_string_lossy().to_string());
                }
            }
        }
    }
    files
}

fn has_valid_extension(path: &Path, extensions: &std::collections::HashSet<String>) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| extensions.contains(ext))
        .unwrap_or(false)
}

fn process_file(filename: &str, args: &Args) -> FileProcessingResult {
    let mut file_results = Vec::new();
    let mut file_chars = 0;
    let mut file_lines = 0;

    match read_lines(filename) {
        Ok(lines) => {
            for (line_number, line) in lines.enumerate() {
                match line {
                    Ok(content) => {
                        let char_count = content.chars().filter(|c| !c.is_whitespace()).count();
                        file_chars += char_count;
                        file_lines += 1;

                        file_results.push(LineResult {
                            line_number: line_number + 1,
                            content: content.clone(),
                            char_count,
                        });

                        if args.format == OutputFormat::Text && args.output.is_none() {
                            println!("File: {} - Line {}: {} - Char count: {}", filename, line_number + 1, content, char_count);
                        }
                    }
                    Err(e) => eprintln!("Error reading line {} in {}: {}", line_number + 1, filename, e),
                }
            }
        }
        Err(e) => eprintln!("Error reading file {}: {}", filename, e),
    }

    FileProcessingResult {
        results: file_results,
        chars: file_chars,
        lines: file_lines,
    }
}

fn main() {
    let args = Args::parse();
    let files = collect_files(&args);

    if files.is_empty() {
        eprintln!("No valid files found to process.");
        return;
    }

    let pb = ProgressBar::new(files.len() as u64);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {pos}/{len} ({eta})")
            .unwrap()
            .progress_chars("#>-"),
    );

    let pb_mutex = Mutex::new(pb);

    let file_results: Vec<FileProcessingResult> = files
        .par_iter()
        .map(|filename| {
            let result = process_file(filename, &args);
            {
                let pb = pb_mutex.lock().unwrap();
                pb.inc(1);
            }
            result
        })
        .collect();

    let pb = pb_mutex.into_inner().unwrap();
    pb.finish_with_message("Processing complete");

    let mut all_results = Vec::new();
    let mut total_chars = 0;
    let mut total_lines = 0;

    for result in file_results {
        all_results.extend(result.results);
        total_chars += result.chars;
        total_lines += result.lines;
    }

    match args.format {
        OutputFormat::Text => {
            if let Some(output_file) = &args.output {
                let mut file = File::create(output_file).expect("Failed to create output file");
                for result in &all_results {
                    writeln!(file, "Line {}: {} - Char count: {}", result.line_number, result.content, result.char_count).unwrap();
                }
            }
        }
        OutputFormat::Json => {
            let json = serde_json::to_string_pretty(&all_results).unwrap();
            if let Some(output_file) = &args.output {
                std::fs::write(output_file, &json).expect("Failed to write JSON");
            } else {
                println!("{}", json);
            }
        }
        OutputFormat::Csv => {
            let mut csv = String::new();
            csv.push_str("line_number,content,char_count\n");
            for result in &all_results {
                csv.push_str(&format!("{},{},{}\n", result.line_number, result.content.replace(",", "\\,"), result.char_count));
            }
            if let Some(output_file) = &args.output {
                std::fs::write(output_file, &csv).expect("Failed to write CSV");
            } else {
                print!("{}", csv);
            }
        }
    }

    if args.summary {
        let average = if total_lines > 0 { total_chars as f64 / total_lines as f64 } else { 0.0 };
        let summary = Summary {
            total_lines,
            total_chars,
            average_chars_per_line: average,
        };
        if args.format == OutputFormat::Json {
            let json = serde_json::to_string_pretty(&summary).unwrap();
            println!("Summary:\n{}", json);
        } else {
            println!("Summary: Total lines: {}, Total chars: {}, Average chars per line: {:.2}", total_lines, total_chars, average);
        }
    }
}