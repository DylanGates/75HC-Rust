use std::io;

enum Manner {
    Polite,
    Rude,
}

impl Manner {
    fn greet(&self, name: &str) -> String {
        match self {
            Manner::Polite => format!("\nHello, nice to meet you {}!\n", name),
            Manner::Rude => format!(
                "\nOh, it's you {}...,
---\nI guess we have to say hi.\n",
                name
            ),
        }
    }
}

fn check_greeting(name: &str) -> Manner {
    if name.is_empty() {
        return Manner::Rude;
    }

    let mut is_start_of_word = true;

    let is_polite = name.chars().all(|c| {
        if c.is_whitespace() {
            is_start_of_word = true;
            true 
        } else if c.is_alphabetic() {
            if is_start_of_word {
                is_start_of_word = false;
                c.is_uppercase()
            } else {
                c.is_lowercase()
            }
        } else {
            false
        }
    });

    if is_polite {
        Manner::Polite
    } else {
        Manner::Rude
    }
}


fn main() {
    println!(
        "
Nice to meet you!
What is your name? (e.g., John or mary)"
    );
    let mut name_input = String::new();
    io::stdin()
        .read_line(&mut name_input)
        .expect("Failed to read line");

    let name = name_input.trim().to_string();

    let manner = check_greeting(&name);
    let display_greeting = manner.greet(&name);

    println!("{}", display_greeting);

    println!("What is your age?");

    let mut age = String::new();
    io::stdin()
        .read_line(&mut age)
        .expect("Failed to read line");

    match age.trim().parse::<u8>() {
        Ok(age) => {
            print!("\nYou are {} years old!", age);
        }
        Err(_) => {
            print!("\nThat's not a valid age! Please enter a number.");
        }
    }
}
