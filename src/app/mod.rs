use commands::{Cli, Commands, ConfigCommands};
use config::GlobalConfig;
use handlebars::{build_handlebars, LibHandlebars};
use keepass::{Database, DatabaseKey};
use std::{error::Error, fs::File};

pub mod commands;
pub mod config;
pub mod handlebars;


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

fn open_keepass_db(keepass_path: String, password: Option<String>) -> Result<Database, Box<dyn Error>> {
    let password = match password {
        Option::None => rpassword::prompt_password("Enter the KeePass database password: ")?,
        Option::Some(pwd) => pwd
    };
    let mut file = File::open(keepass_path).map_err(|_| "Keepass db file not found")?;
    let key = DatabaseKey::new().with_password(&password);
    Database::open(&mut file, key)
        .map_err(|_| "Database cannot be opened, maybe password is wrong?".into())
}

fn render_and_save_template(
    handlebars: &mut LibHandlebars,
    name: String,
    template_path: String,
    output_path: String,
) -> Result<(), Box<dyn Error>> {
    handlebars.register_template_file(&name, template_path)?;

    let rendered = handlebars
        .render(&name, &())
        .map_err(|e| format!("Failed to render template: {}", e))?;

    std::fs::write(&output_path, rendered)
        .map_err(|e| format!("Failed to write output file {}: {}", output_path, e))?;

    println!("file written: {}", output_path);
    Ok(())
}

pub fn execute(args: Cli) -> Result<(), Box<dyn Error>> {
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

    match args.command {
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
                name,
                template,
                output,
                relative_to_input,
            } => {
                let output_path = get_output_path(&template, output, relative_to_input);
                config
                    .config
                    .add_template(name, get_absolute_path(template), get_absolute_path(output_path));
                config.save()?;
            }
        },
        Commands::Build {
            template,
            keepass,
            output,
            relative_to_input,
            password
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

            let db = open_keepass_db(keepass, password)?;

            let mut handlebars = build_handlebars(db);

            let output_path = get_output_path(&template, output, relative_to_input);

            render_and_save_template(&mut handlebars, template.clone(), template, output_path)?;
        }
        Commands::BuildAll { password } => {
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

            let db = open_keepass_db(keepass, password).expect("Database error");

            let mut handlebars = build_handlebars(db);

            for template in files {
                let name = match template.name {
                    Some(name)=>name,
                    None => template.template_path.clone()
                };
                render_and_save_template(
                    &mut handlebars,
                    name,
                    template.template_path,
                    template.output_path,
                )?;
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rendering_invalid_handlebars_template() {
        let ret = execute(Cli{
            command: Commands::Build {
                template: String::from("test_resources/file-with-error"),
                relative_to_input: false,
                output: String::from("test_outputs/file-with-error"),
                keepass: Some(String::from("test_resources/test_db.kdbx")),
                password: Some(String::from("MyTestPass"))
            },
            config: None
        });
        match ret {
            Ok(_) => assert!(false, "It should fail"),
            Err(error) =>{
                assert_eq!(error.to_string(),"Failed to render template: Error rendering \"test_resources/file-with-error\" line 2, col 15: Helper not found keepass-2-file")
            }
        }
    }
}
