use clap::{CommandFactory, Parser};
use clap_complete::generate;
use std::error::Error;
use std::io;

mod app;
use app::IOLogs;
use app::commands::{Cli, Commands};
use app::execute;

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

    if let Commands::Completion { shell } = args.command {
        let mut cmd = Cli::command();
        let name = cmd.get_name().to_string();
        generate(shell, &mut cmd, name, &mut io::stdout());
        Ok(())
    } else {
        execute(args, &Console {})
    }
}
