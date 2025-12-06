use std::fs::File;
use std::io::{self, BufRead, Write};
use std::path::Path;
use std::fs::OpenOptions;
use clap::{Parser, ValueEnum};
use serde::Serialize;

#[derive(Parser)]
#[command(name = "word_counter")]
#[command(about = "A tool to count characters in text files")]
struct Args {
    #[arg(short, long)]
    input: String,

    #[arg(short, long)]
    output: Option<String>,

    #[arg(short, long, value_enum, default_value = "text")]
    format: OutputFormat,

    #[arg(short, long)]
    summary: bool,
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

fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>>
where
    P: AsRef<Path>,
{
    let file = File::open(filename)?;
    Ok(io::BufReader::new(file).lines())
}

fn main() {
    let args = Args::parse();
    let filename = &args.input;

    if !Path::new(filename).exists() {
        OpenOptions::new()
            .write(true)
            .create(true)
            .open(filename)
            .expect("Failed to create file");
        println!("File '{}' has been created.", filename);
    }

    match read_lines(filename) {
        Ok(lines) => {
            let mut results = Vec::new();
            let mut total_chars = 0;
            let mut total_lines = 0;

            for (line_number, line) in lines.enumerate() {
                match line {
                    Ok(content) => {
                        let char_count = content.chars().filter(|c| !c.is_whitespace()).count();
                        total_chars += char_count;
                        total_lines += 1;

                        results.push(LineResult {
                            line_number: line_number + 1,
                            content: content.clone(),
                            char_count,
                        });

                        if args.format == OutputFormat::Text && args.output.is_none() {
                            println!("Line {}: {} - Char count: {}", line_number + 1, content, char_count);
                        }
                    }
                    Err(e) => eprintln!("Error reading line {}: {}", line_number + 1, e),
                }
            }

            match args.format {
                OutputFormat::Text => {
                    if let Some(output_file) = &args.output {
                        let mut file = File::create(output_file).expect("Failed to create output file");
                        for result in &results {
                            writeln!(file, "Line {}: {} - Char count: {}", result.line_number, result.content, result.char_count).unwrap();
                        }
                    }
                }
                OutputFormat::Json => {
                    let json = serde_json::to_string_pretty(&results).unwrap();
                    if let Some(output_file) = &args.output {
                        std::fs::write(output_file, &json).expect("Failed to write JSON");
                    } else {
                        println!("{}", json);
                    }
                }
                OutputFormat::Csv => {
                    let mut csv = String::new();
                    csv.push_str("line_number,content,char_count\n");
                    for result in &results {
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
        Err(e) => eprintln!("Error reading file: {}", e),
    }
}