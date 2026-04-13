use clap::Parser;
use habit_tracker::cli::{Cli, CommandHandler};
use anyhow::Result;

fn main() -> Result<()> {
    let cli = Cli::parse();
    let mut handler = CommandHandler::new(cli.data_dir, cli.verbose)?;
    handler.execute(cli.command)?;
    Ok(())
}