use clap::Parser;
use dirs;
use keepass::{Database, DatabaseKey};
use std::error::Error;
use std::fs::File;

mod app;
use app::commands::{Cli, Commands, ConfigCommands};
use app::config::GlobalConfig;
use app::handlebars::build_handlebars;

fn get_output_path(template: String, output: String, relative_to_input: bool) -> String {
    let template_dir_buf = std::env::current_dir().unwrap();
    let mut template_dir: &std::path::Path = template_dir_buf.as_path();

    if output.starts_with('/') {
        return output.to_string();
    } else if relative_to_input {
        let template_path = std::path::Path::new(&template);
        template_dir = template_path.parent().unwrap_or(&template_dir);
    }
    template_dir.join(output).to_str().unwrap().to_string()
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Cli::parse();

    println!("Config path: {:?}", args.config);
    let home = dirs::home_dir()
        .ok_or("Could not determine home directory")?
        .to_str()
        .ok_or("Home directory is not valid UTF-8")?
        .to_string();
    let config_path = args
        .config
        .unwrap_or_else(|| format!("{}/.config/env-generator.yaml", home));

    // Load and parse the configuration file
    let mut config = GlobalConfig::new(&config_path)?;

    match args.commands {
        Commands::Config(config_command) => match config_command {
            ConfigCommands::SetDefaultKpDb { url } => {
                println!("Setting default KeePass DB URL: {:?}", url);
                config.config.keepass = Some(url);
                config.save()?;
            }
            ConfigCommands::GetKpDb {} => match config.config.keepass {
                Some(url) => println!("Current file is {}", url),
                None => println!(
                    "The current configuration '{}' doesn't contain a default keepass db",
                    config.get_file()
                ),
            },
        },
        Commands::Build {
            template,
            keepass,
            output,
            relative_to_input,
        } => {
            println!("Building with .env file: {}", template);
            println!("KeePass file: {:?}", keepass);

            let keepass = match keepass {
                Some(url) => url,
                None => {
                    match config.config.keepass {
                        Some(url) => url,
                        None => {
                            println!("No keepass file configured in global config or passed as parameter");
                            return Err("No keepass file configured in global config or passed as parameter".into());
                        }
                    }
                }
            };

            let password =
                rpassword::prompt_password("Enter the KeePass database password: ").unwrap();

            let mut file = File::open(keepass).expect("Keepass db file not found");
            let template_contents =
                std::fs::read_to_string(template.clone()).expect("template file cannot be found");
            let key = DatabaseKey::new().with_password(&password);
            let db = Database::open(&mut file, key)
                .expect("Database cannot be opened, maybe password is wrong?");

            let handlebars = build_handlebars(db);
            let rendered = handlebars
                .render_template(&template_contents, &())
                .map_err(|e| format!("Failed to render template: {}", e))?;
            let output_path = get_output_path(template, output, relative_to_input);

            std::fs::write(output_path.clone(), rendered).map_err(|e| format!("Failed to write output file {}: {}", output_path, e))?;

            println!("file overwrited {} generated ", output_path)
        }
    }
    Ok(())
}
