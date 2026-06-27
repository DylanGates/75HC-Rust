use anyhow::Result;
use clap::Parser;

#[derive(Parser)]
#[command(name = "MiniCrate", about = "String utility CLI — demonstrates the mini-crate library")]
struct Cli {
    /// Input string to process
    input: String,

    /// Operations to perform (comma-separated or multiple flags)
    #[arg(short, long, default_value = "all")]
    operations: Vec<String>,

    /// Max chars for truncate operation
    #[arg(short = 'm', long, default_value = "20")]
    max_chars: usize,
}

fn run_op(op: &str, input: &str, max_chars: usize) {
    match op {
        "truncate" => println!("  truncate({max_chars}):  {}", mini_crate::truncate(input, max_chars)),
        "word_count" => println!("  word_count:          {}", mini_crate::word_count(input)),
        "palindrome" => println!("  is_palindrome:       {}", mini_crate::is_palindrome(input)),
        "vowels" => println!("  count_vowels:        {}", mini_crate::count_vowels(input)),
        "reverse" => println!("  reverse:             {}", mini_crate::reverse(input)),
        "snake_case" => println!("  to_snake_case:       {}", mini_crate::to_snake_case(input)),
        "camel_case" => println!("  to_camel_case:       {}", mini_crate::to_camel_case(input)),
        "all" => {
            run_op("truncate", input, max_chars);
            run_op("word_count", input, max_chars);
            run_op("palindrome", input, max_chars);
            run_op("vowels", input, max_chars);
            run_op("reverse", input, max_chars);
            run_op("snake_case", input, max_chars);
            run_op("camel_case", input, max_chars);
        }
        _ => eprintln!("  Unknown operation: {op}"),
    }
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    println!("Input: \"{}\"\n", cli.input);
    println!("Results:");

    for op in &cli.operations {
        run_op(op, &cli.input, cli.max_chars);
    }

    Ok(())
}
