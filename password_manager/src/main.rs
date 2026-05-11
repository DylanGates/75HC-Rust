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

    /// Include special characters
    #[arg(short, long, default_value_t = false)]
    special: bool,

    /// Exclude ambiguous characters (l, 1, O, 0, etc.)
    #[arg(short, long, default_value_t = false)]
    exclude_ambiguous: bool,

    /// Custom special characters
    #[arg(short, long)]
    custom_special: Option<String>,

    /// Minimum number of digits
    #[arg(long, default_value_t = 0)]
    min_digits: usize,

    /// Minimum number of special characters
    #[arg(long, default_value_t = 0)]
    min_special: usize,

    /// Number of passwords to generate
    #[arg(short, long, default_value_t = 1)]
    count: usize,

    /// Show password entropy
    #[arg(short, long, default_value_t = false)]
    entropy: bool,
}

fn main() {
    let args = Args::parse();
    let mut charset = String::new();
    let mut digit_set = String::new();
    let mut special_set = String::new();
    
    if args.lowercase {
        charset.push_str("abcdefghijklmnopqrstuvwxyz");
    }
    if args.uppercase {
        charset.push_str("ABCDEFGHIJKLMNOPQRSTUVWXYZ");
    }
    if args.numbers {
        let digits = "0123456789";
        charset.push_str(digits);
        digit_set.push_str(digits);
    }
    if args.special {
        let specials = if let Some(ref custom) = args.custom_special {
            custom.as_str()
        } else {
            "!@#$%^&*()-_=+[]{}|;:,.<>?"
        };
        charset.push_str(specials);
        special_set.push_str(specials);
    }

    if args.exclude_ambiguous {
        let ambiguous = "l1O0I|";
        charset.retain(|c| !ambiguous.contains(c));
        digit_set.retain(|c| !ambiguous.contains(c));
        special_set.retain(|c| !ambiguous.contains(c));
    }

    if charset.is_empty() {
        eprintln!("Error: Character set is empty. Please enable at least one character type.");
        return;
    }

    let charset_size = charset.chars().count();
    let entropy_per_char = (charset_size as f64).log2();
    let total_entropy = entropy_per_char * args.length as f64;

    let mut rng = rand::thread_rng();

    for _ in 0..args.count {
        let mut password = String::new();
        loop {
            password = (0..args.length)
                .map(|_| {
                    let idx = rng.gen_range(0..charset.len());
                    charset.chars().nth(idx).unwrap()
                })
                .collect();

            let digit_count = password.chars().filter(|c| digit_set.contains(*c)).count();
            let special_count = password.chars().filter(|c| special_set.contains(*c)).count();
            if digit_count >= args.min_digits && special_count >= args.min_special {
                break;
            }
        }
        
        if args.entropy {
            println!("{} [Entropy: {:.2} bits]", password, total_entropy);
        } else {
            println!("{}", password);
        }
    }
}
