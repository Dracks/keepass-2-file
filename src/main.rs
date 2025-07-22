use clap::Parser;
use std::error::Error;

mod app;
use app::commands::Cli;
use app::execute;
use app::IOLogs;

struct Console {}

impl IOLogs for Console {
    fn log(&self, msg: String) {
        println!("{msg}");
    }

    fn error(&self, msg: String) {
        eprintln!("{msg}");
    }

    fn read(&self, prompt: String, secure: bool) -> std::io::Result<String> {
        if secure {
            rpassword::prompt_password(prompt)
        } else {
            println!("{prompt}");
            let mut input = String::new();
            std::io::stdin().read_line(&mut input).unwrap();
            Ok(input.trim().to_string())
        }
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Cli::parse();

    execute(args, &Console {})
}
