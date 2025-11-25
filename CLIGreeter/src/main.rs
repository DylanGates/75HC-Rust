use std::io;

enum Manner {
    Polite,
    Rude,
}

impl Manner {
    fn greet(&self, name: &str) -> String {
        match self {
            Manner::Polite => format!("\nHello, nice to meet you {}!", name.to_uppercase()),
            Manner::Rude => format!(
                "\nOh, it's you {}...,
---\nI guess we have to say hi.\n",
                name
            ),
        }
    }
}

fn check_greeting(name: &str) -> Manner {
    let first_char = name.chars().next().unwrap_or('\0');

    if first_char.is_alphabetic() && first_char.is_uppercase() {
        Manner::Polite
    } else {
        Manner::Rude
    }
}

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

    let manner = check_greeting(&name);

    let display_greeting = match manner {
        Manner::Polite => manner.greet(&name),
        Manner::Rude => manner.greet(&name),
    };

    println!("{}", display_greeting);

    println!("What is your age?");

    let mut age = String::new();
    io::stdin()
        .read_line(&mut age)
        .expect("Failed to read line");
    print!("\nYou are {} years old!", age);
}
