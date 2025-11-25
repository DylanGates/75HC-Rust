use std::io;

fn main() {
    let mut name = String::new();
    io::stdin()
        .read_line(&mut name)
        .expect("Failed to read line");
    name = name.trim().to_string();

    println!("Hello, {}", name);
}
