use std::io;

fn main() {
    println!(
        "
Nice to meet you!
What is your name?"
    );
    let mut name = String::new();
    io::stdin()
        .read_line(&mut name)
        .expect("Failed to read line");
    name = name.trim().to_string();

    println!("\nHello, {}\n", name);

    println!("What is your age?");

    let mut age = String::new();
    io::stdin()
        .read_line(&mut age)
        .expect("Failed to read line");
    print!("\nYou are {} years old!", age);
}
