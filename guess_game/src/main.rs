use std::io;

fn guess_number() {
    println!("Guess the number between 1 and 100!");

    let secret_number = rand::thread_rng().gen_range(1..=100);

    loop {
        println!("Please input your guess:");

        let mut input = String::new();
        io::stdin()
            .read_line(&mut input)
            .expect("Failed to read line");

        let guess = input.trim().parse::<u32>().expect("Please type a number!");

        if guess < secret_number {
            println!("Too small!");
        } else if guess > secret_number {
            println!("Too big!");
        } else {
            println!("You guessed it! The number was {}.", secret_number);
            break;
        }
    }
}

fn main() {
    guess_number();
}