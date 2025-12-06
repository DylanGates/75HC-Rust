use std::fs::File;
use std::io::{self, BufRead};
use std::path::Path;
use std::fs::OpenOptions;

fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>>
where
    P: AsRef<Path>,
{
    let file = File::open(filename)?;
    Ok(io::BufReader::new(file).lines())
}

fn main() {
    let filename = "input.txt";

    let _file = OpenOptions::new()
        .write(true)
        .create(true)
        .open(filename)
        .expect("Failed to open file");

    println!("File has been created: {:?}", filename);

    if filename.is_empty() {
        eprintln!("Error: No filename provided.");
        return;
    }

    match read_lines(filename) {
        Ok(lines) => {
            for line in lines {
                match line {
                    Ok(content) => {
                        let char_count = content.chars().filter(|c| !c.is_whitespace()).count();
                        println!("Line: {} - Char count: {}", content, char_count);
                    }
                    Err(e) => eprintln!("Error reading line: {}", e),
                }
            }
        }
        Err(e) => eprintln!("Error reading file: {}", e),
    }
}