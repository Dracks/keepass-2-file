use keepass::{ Database, DatabaseKey};
use std::error::Error;
use std::fs::File;
use clap::Parser;

mod app;
use app::handlebars::build_handlebars;
use app::config::GlobalConfig;
use app::commands::{Cli, Commands, ConfigCommands};

fn main() -> Result<(), Box<dyn Error>> {
    let args = Cli::parse();

    println!("Config path: {:?}", args.config);
    let home = std::env::var("HOME").expect("HOME environment variable not set");
    let config_path = args.config.unwrap_or_else(|| {
        format!("{}/.config/env-generator.yaml", home)
    });

    // Load and parse the configuration file
    let mut config = GlobalConfig::new(&config_path);

    match args.commands {
        Commands::Config(config_command) => {
            match config_command {
                ConfigCommands::SetDefaultKpDb { url } => {
                    println!("Setting default KeePass DB URL: {:?}", url);
                    config.config.keepass = url;
                    config.save();
                }
                ConfigCommands::GetKpDb {  } => {
                    match config.config.keepass {
                        Some(url) => println!("Current file is {}", url),
                        None => println!("The current configuration '{}' doesn't contain a default keepass db", config.get_file() ),
                    }
                }
            }
        }
        Commands::Build { dot_env, keepass } => {
            println!("Building with .env file: {}", dot_env);
            println!("KeePass file: {:?}", keepass);

            let keepass = match keepass {
                Some(url) => url,
                None => match config.config.keepass {
                    Some(url) => url,
                    None => {
                        println!("No keepass file configured in global config or passed as parameter");
                        return Err("No keepass file configured in global config or passed as parameter".into());
                    }
                }
            };

            let password = rpassword::prompt_password("Enter the KeePass database password: ").unwrap();


            let mut file = File::open(keepass)?;
            let template = std::fs::read_to_string("test_resources/.env.example").unwrap();
            let key = DatabaseKey::new().with_password(&password);
            let db = Database::open(&mut file, key)?;

            let handlebars = build_handlebars(db);

            println!(
                "{}",
                handlebars.render_template(&template, &())?
            );
        }
    }
    Ok(())
}
