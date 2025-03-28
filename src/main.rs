use clap::Parser;
use std::error::Error;

mod app;
use app::commands::Cli;
use app::execute;

fn main() -> Result<(), Box<dyn Error>> {
    let args = Cli::parse();

    execute(args)
}
