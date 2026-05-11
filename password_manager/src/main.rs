use clap::Parser;
use rand::Rng;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {}

fn main() {
    let _args = Args::parse();
    let charset = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";
    let mut rng = rand::thread_rng();
    let password: String = (0..12)
        .map(|_| {
            let idx = rng.gen_range(0..charset.len());
            charset.chars().nth(idx).unwrap()
        })
        .collect();

    println!("{}", password);
}
