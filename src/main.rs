use clap::Parser;
use keepass::{Database, DatabaseKey};
use std::error::Error;
use std::fs::File;

mod app;
use app::commands::{Cli, Commands, ConfigCommands};
use app::config::GlobalConfig;
use app::handlebars::build_handlebars;

fn get_output_path(template: &String, output: String, relative_to_input: bool) -> String {
    let template_dir_buf = std::env::current_dir().unwrap();
    let mut template_dir: &std::path::Path = template_dir_buf.as_path();

    if output.starts_with('/') {
        return output.to_string();
    } else if relative_to_input {
        let template_path = std::path::Path::new(&template);
        template_dir = template_path.parent().unwrap_or(template_dir);
    }
    template_dir.join(output).to_str().unwrap().to_string()
}

fn get_absolute_path(path: String) -> String {
    if path.starts_with('/') {
        path
    } else {
        let current_dir = std::env::current_dir().unwrap();
        current_dir.join(path).to_str().unwrap().to_string()
    }
}

fn open_keepass_db(keepass_path: String) -> Result<Database, Box<dyn Error>> {
    let password = rpassword::prompt_password("Enter the KeePass database password: ")?;
    let mut file = File::open(keepass_path).map_err(|_| "Keepass db file not found")?;
    let key = DatabaseKey::new().with_password(&password);
    Database::open(&mut file, key)
        .map_err(|_| "Database cannot be opened, maybe password is wrong?".into())
}

fn render_and_save_template(
    handlebars: &handlebars::Handlebars,
    template_path: String,
    output_path: String,
) -> Result<(), Box<dyn Error>> {
    let template_contents = std::fs::read_to_string(&template_path)
        .map_err(|_| format!("template file cannot be found: {}", template_path))?;

    let rendered = handlebars
        .render_template(&template_contents, &())
        .map_err(|e| format!("Failed to render template: {}", e))?;

    std::fs::write(&output_path, rendered)
        .map_err(|e| format!("Failed to write output file {}: {}", output_path, e))?;

    println!("file written: {}", output_path);
    Ok(())
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
        .unwrap_or_else(|| format!("{}/.config/keepass-2-file.yaml", home));

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
            ConfigCommands::ListFiles => {
                let templates = config.config.get_templates();
                for template in templates {
                    println!(
                        "template: {} -> {}",
                        template.template_path, template.output_path
                    )
                }
            }
            ConfigCommands::AddFile {
                template,
                output,
                relative_to_input,
            } => {
                let output_path = get_output_path(&template, output, relative_to_input);
                config
                    .config
                    .add_template(get_absolute_path(template), get_absolute_path(output_path));
                config.save()?;
            }
        },
        Commands::Build {
            template,
            keepass,
            output,
            relative_to_input,
        } => {
            println!("Building template file: {}", template);
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

            let db = open_keepass_db(keepass)?;

            let handlebars = build_handlebars(db);

            let output_path = get_output_path(&template, output, relative_to_input);

            render_and_save_template(&handlebars, template, output_path)?;
        }
        Commands::BuildAll => {
            let files = config.config.get_templates();
            let keepass = match config.config.keepass {
                Some(url) => url,
                None => {
                    println!("No keepass file configured in global config");
                    return Err(
                        "No keepass file configured in global config or passed as parameter".into(),
                    );
                }
            };

            println!(
                "Building all files ({}) with KeePass file: {:?}",
                files.len(),
                keepass
            );

            let db = open_keepass_db(keepass).expect("Database error");

            let handlebars = build_handlebars(db);

            for template in files {
                render_and_save_template(
                    &handlebars,
                    template.template_path,
                    template.output_path,
                )?;
            }
        }
    }
    Ok(())
}
