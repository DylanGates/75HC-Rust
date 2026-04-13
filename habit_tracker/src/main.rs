use clap::Parser;
use habit_tracker::cli::{Cli, CommandHandler};

fn main() {
    let cli = Cli::parse();
    let mut handler = CommandHandler::new();
    handler.execute(cli.command);
}