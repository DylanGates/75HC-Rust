use clap::Parser;
use rand::Rng;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Length of the password
    #[arg(short, long, default_value_t = 12)]
    length: usize,

    /// Include uppercase letters
    #[arg(short, long, default_value_t = true)]
    uppercase: bool,

    /// Include lowercase letters
    #[arg(short, long, default_value_t = true)]
    lowercase: bool,

    /// Include numeric characters
    #[arg(short, long, default_value_t = true)]
    numbers: bool,
}

fn main() {
    let args = Args::parse();
    let mut charset = String::new();
    if args.lowercase {
        charset.push_str("abcdefghijklmnopqrstuvwxyz");
    }
    if args.uppercase {
        charset.push_str("ABCDEFGHIJKLMNOPQRSTUVWXYZ");
    }
    if args.numbers {
        charset.push_str("0123456789");
    }
    
    let mut rng = rand::thread_rng();
    let password: String = (0..args.length)
        .map(|_| {
            let idx = rng.gen_range(0..charset.len());
            charset.chars().nth(idx).unwrap()
        })
        .collect();

    println!("{}", password);
}
