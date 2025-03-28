use clap::Parser;
use std::error::Error;

mod app;
use app::execute;
use app::commands::Cli;


fn main() -> Result<(), Box<dyn Error>> {
    let args = Cli::parse();

    execute(args)
}
