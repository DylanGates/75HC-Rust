use std::fs;
use std::io;
use std::fs::OpenOptions;

fn read_file_lines(filename: &str) -> Result<Vec<String>, io::Error> {
    let content = fs::read_to_string(filename)?;
    Ok(content.lines().map(|line| line.to_string()).collect())
}

fn main() {
    let filename = "input.txt";

    let _file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(filename)
        .expect("Failed to open file");

    println!("File has been created: {:?}", filename);

    if filename.is_empty() {
        eprintln!("Error: No filename provided.");
        return;
    }

    match read_file_lines(filename) {
        Ok(lines) => {
            for line in lines.iter() {
                println!("{}", line);
            }
        }
        Err(e) => eprintln!("Error reading file: {}", e),
    }
}
