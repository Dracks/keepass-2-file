use commands::{Cli, Commands, ConfigCommands, NameOrPath};
use config::GlobalConfig;
use handlebars::{build_handlebars, LibHandlebars};
use keepass::{Database, DatabaseKey};
use std::{
    collections::HashMap,
    error::Error,
    fs::File,
    path::{Path, PathBuf},
};

pub mod commands;
pub mod config;
pub mod handlebars;
mod test_config;

fn join_relative(current: &Path, file: String) -> PathBuf {
    if let Some(without_rel_path) = file.strip_prefix("./") {
        join_relative(current, String::from(without_rel_path))
    } else if let Some(without_parent_path) = file.strip_prefix("../") {
        match current.parent() {
            Some(parent) => join_relative(parent, String::from(without_parent_path)),
            None => panic!("File has more parent relative that current parents"),
        }
    } else {
        current.join(file)
    }
}

fn get_output_path(template: &String, output: String, relative_to_input: bool) -> String {
    let template_dir_buf = std::env::current_dir().unwrap();
    let mut template_dir: &Path = template_dir_buf.as_path();

    if output.starts_with('/') {
        return output.to_string();
    } else if relative_to_input {
        let template_path = Path::new(&template);
        template_dir = template_path.parent().unwrap_or(template_dir);
    }
    join_relative(template_dir, output)
        .to_str()
        .unwrap()
        .to_string()
}

fn get_absolute_path(path: String) -> String {
    if path.starts_with('/') {
        path
    } else {
        let current_buff = std::env::current_dir().unwrap();
        let current_dir = current_buff.as_path();

        join_relative(current_dir, path)
            .to_str()
            .unwrap()
            .to_string()
    }
}

fn open_keepass_db(
    keepass_path: String,
    password: Option<String>,
) -> Result<Database, Box<dyn Error>> {
    let password = match password {
        Option::None => rpassword::prompt_password("Enter the KeePass database password: ")?,
        Option::Some(pwd) => pwd,
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
    vars: &HashMap<String, String>,
) -> Result<(), Box<dyn Error>> {
    handlebars.register_template_file(&name, template_path)?;
    println!("{:?}", vars);

    let rendered = handlebars
        .render(&name, vars)
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
        Commands::Config(config_command) => {
            match config_command {
                ConfigCommands::SetDefaultKpDb { url } => {
                    println!("Setting default KeePass DB URL: {:?}", url);
                    config.config.keepass = Some(url);
                    config.save()?;
                }
                ConfigCommands::GetKpDb => match config.config.keepass {
                    Some(url) => println!("Current file is {}", url),
                    None => println!(
                        "The current configuration '{}' doesn't contain a default keepass db",
                        config.get_file()
                    ),
                },
                ConfigCommands::ListFiles => {
                    let templates = config.config.get_templates();
                    if templates.is_empty() {
                        println!("No templates defined");
                    } else {
                        println!("Configured templates:");
                        for template in templates {
                            println!("\t {} -> {}", template.template_path, template.output_path)
                        }
                    }
                }
                ConfigCommands::AddFile {
                    name,
                    template,
                    output,
                    relative_to_input,
                } => {
                    let output_path = get_output_path(&template, output, relative_to_input);
                    config.config.add_template(
                        name,
                        get_absolute_path(template),
                        get_absolute_path(output_path),
                    );
                    config.save()?;
                }
                ConfigCommands::Prune => {
                    let templates = config.config.get_templates();
                    for template in templates {
                        println!("{:?}", template);
                        if !Path::new(&template.template_path).exists() {
                            println!(
                                "Template {} does not exist, removing from config",
                                template.template_path
                            );
                            config
                                .config
                                .delete_template(template.template_path, template.output_path);
                        }
                    }
                    config.save()?;
                }
                ConfigCommands::Delete { template } => {
                    match template {
                        NameOrPath::Name { name } => {
                            config.config.delete_templates(name);
                        }
                        NameOrPath::Paths { path, output } => {
                            config.config.delete_template(path, output);
                        }
                    }
                    config.save()?;
                }
                ConfigCommands::ListVariables => {
                    let variables = config.config.get_vars();
                    if variables.is_empty() {
                        println!("No variables defined");
                    } else {
                        println!("Variables:");
                        for (key, value) in variables {
                            println!("\t{} = {}", key, value);
                        }
                    }
                }
                ConfigCommands::AddVariables { variables } => {
                    for variable in variables {
                        if let Some((key, value)) = variable.split_once('=') {
                            if key.len() > 0 {
                                config.config.add_var(key.to_string(), value.to_string());
                            } else {
                                eprintln!("Malformed variable: \"{variable}\": variable name cannot be empty");
                            }
                        } else {
                            eprintln!("Malformed variable \"{variable}\": please use var=content");
                        }
                    }
                    config.save()?;
                }
                ConfigCommands::DeleteVariables { variables } => {
                    for variable in variables {
                        config.config.del_var(variable.to_string());
                    }
                    config.save()?;
                }
            }
        }
        Commands::Build {
            template,
            keepass,
            output,
            relative_to_input,
            password,
        } => {
            println!("Building template file: {}", template);
            println!("KeePass file: {:?}", keepass);
            let variables = config.config.get_vars();

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

            render_and_save_template(
                &mut handlebars,
                template.clone(),
                template,
                output_path,
                &variables,
            )?;
        }
        Commands::BuildAll { password } => {
            let variables = config.config.get_vars();
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
                    Some(ref name) => name.clone(),
                    None => template.template_path.clone(),
                };
                let result = render_and_save_template(
                    &mut handlebars,
                    name,
                    template.template_path.clone(),
                    template.output_path.clone(),
                    &variables,
                );
                if let Err(err) = result {
                    let name = match template.name {
                        Some(name) => name,
                        None => {
                            format!("{} => {}", template.template_path, template.output_path)
                        }
                    };
                    println!("Skipping template {} because of: {:?}", name, err)
                }
            }
        }
    }
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;

    use test_config::tests::TestConfig;

    #[test]
    fn test_join_relative_basic() {
        let test_path = Path::new("/home/users/devel/");
        let relative_path = String::from("./../.././file");
        let result = join_relative(&test_path, relative_path);
        assert_eq!(result.to_str().unwrap(), "/home/file");
    }

    #[test]
    fn test_adding_new_template() {
        let test = TestConfig::create();
        let result = execute(Cli {
            command: Commands::Config(ConfigCommands::AddFile {
                name: Some(String::from("New template")),
                template: String::from("./test_resources/file-with-error"),
                output: String::from("./tmp/error"),
                relative_to_input: true,
            }),
            config: Some(String::from(test.get_file_path())),
        });

        println!("{:?}", result);
        assert!(result.is_ok());

        let out_config = test.get();

        let templates = out_config.config.get_templates();
        assert_eq!(templates.len(), 4);
        assert_eq!(templates[3].name, Some(String::from("New template")));
    }

    #[test]
    fn test_adding_existing_template_will_replace_it() {
        let test = TestConfig::create();
        let result = execute(Cli {
            command: Commands::Config(ConfigCommands::AddFile {
                name: Some(String::from("New name")),
                template: String::from("./test_resources/.env.example"),
                output: String::from("./test_resources/tmp/.env"),
                relative_to_input: false,
            }),
            config: Some(String::from(test.get_file_path())),
        });

        println!("{:?}", result);
        assert!(result.is_ok());

        let out_config = test.get();

        let templates = out_config.config.get_templates();
        assert_eq!(templates.len(), 3);
        assert_eq!(templates[1].name, Some(String::from("New name")));
    }

    #[test]
    fn test_rendering_invalid_handlebars_template() {
        let ret = execute(Cli {
            command: Commands::Build {
                template: String::from("test_resources/file-with-error"),
                relative_to_input: false,
                output: String::from("test_outputs/file-with-error"),
                keepass: Some(String::from("test_resources/test_db.kdbx")),
                password: Some(String::from("MyTestPass")),
            },
            config: None,
        });
        if let Err(error) = ret {
            assert_eq!(error.to_string(),"Failed to render template: Error rendering \"test_resources/file-with-error\" line 2, col 15: Helper not found keepass-2-file")
        }
    }

    #[test]
    fn test_prune_command() {
        let test = TestConfig::create();
        let result = execute(Cli {
            config: Some(String::from(test.get_file_path())),
            command: Commands::Config(ConfigCommands::Prune),
        });
        println!("{:?}", result);
        assert!(result.is_ok());

        let out_config = test.get();

        let templates = out_config.config.get_templates();
        assert_eq!(templates.len(), 2);
        assert_eq!(templates[0].name, Some(String::from("valid")));
    }

    #[test]
    fn test_delete_command() {
        let test = TestConfig::create();
        let result = execute(Cli {
            config: Some(String::from(test.get_file_path())),
            command: Commands::Config(ConfigCommands::Delete {
                template: NameOrPath::Name {
                    name: String::from("other"),
                },
            }),
        });
        println!("{:?}", result);
        assert!(result.is_ok());

        let out_config = test.get();

        let templates = out_config.config.get_templates();
        assert_eq!(templates.len(), 2);
        assert_eq!(templates[1].name, Some(String::from("valid")));
    }

    #[test]
    fn test_build_all_but_skip_invalid() {
        let test = TestConfig::create();
        let result = execute(Cli {
            config: Some(String::from(test.get_file_path())),
            command: Commands::BuildAll {
                password: Some(String::from("MyTestPass")),
            },
        });
        println!("{:?}", result);
        assert!(result.is_ok());

        assert!(Path::new("test_resources/tmp/.env").exists());
    }

    #[test]
    fn test_add_variables() {
        let test = TestConfig::create();
        let result = execute(Cli {
            config: Some(String::from(test.get_file_path())),
            command: Commands::Config(ConfigCommands::AddVariables {
                variables: Vec::from([
                    String::from("var1=Some variable"),
                    String::from("email=j@k.com"),
                ]),
            }),
        });
        println!("{:?}", result);
        assert!(result.is_ok());

        let variables = test.get().config.get_vars();
        assert_eq!(variables.len(), 2);
        assert_eq!(variables.get("var1"), Some(&String::from("Some variable")));
        assert_eq!(variables.get("email"), Some(&String::from("j@k.com")));
    }

    #[test]
    fn test_delete_variables() {
        let test = TestConfig::create_with_vars();
        let result = execute(Cli {
            config: Some(String::from(test.get_file_path())),
            command: Commands::Config(ConfigCommands::DeleteVariables {
                variables: Vec::from([String::from("something")]),
            }),
        });
        println!("{:?}", result);
        assert!(result.is_ok());

        let variables = test.get().config.get_vars();
        assert_eq!(variables.len(), 1);
        assert_eq!(variables.get("email"), Some(&String::from("j@k.com")));

        let templates = test.get().config.get_templates();
        assert_eq!(templates.len(), 1);
        assert_eq!(templates[0].name, Some(String::from("valid")));
    }

    #[test]
    fn test_add_variables_with_malformed_input() {
        let test = TestConfig::create();
        let result = execute(Cli {
            config: Some(String::from(test.get_file_path())),
            command: Commands::Config(ConfigCommands::AddVariables {
                variables: Vec::from([
                    String::from("no_equals_sign"), // Missing '=' character
                    String::from("connection_string=postgres://user:password=@localhost:5432/mydb"), // Contains multiple '=' chars
                    String::from("key="),   // Empty value
                    String::from("=value"), // Empty key
                ]),
            }),
        });
        assert!(result.is_ok());

        let variables = test.get().config.get_vars();

        // Only 3 variables should be added (the one without '=' is skipped)
        assert_eq!(variables.len(), 2);

        // The connection string is truncated at the first '='
        assert_eq!(
            variables.get("connection_string"),
            Some(&String::from(
                "postgres://user:password=@localhost:5432/mydb"
            ))
        );
        // Should have been: "postgres://user:password@localhost:5432/mydb"

        // Empty values and keys are stored as is
        assert_eq!(variables.get("key"), Some(&String::from("")));
    }

    #[test]
    fn test_something_is_a_variable() {
        let test = TestConfig::create_with_vars();
        let result = execute(Cli {
            config: Some(String::from(test.get_file_path())),
            command: Commands::Build {
                template: String::from("./test_resources/with_variables"),
                output: String::from("./test_resources/tmp/with_variables"),
                keepass: None,
                password: Some(String::from("MyTestPass")),
                relative_to_input: false,
            },
        });
        println!("{:?}", result);
        assert!(result.is_ok());

        let output_file_path = String::from("test_resources/tmp/with_variables");
        let file_contents = std::fs::read_to_string(output_file_path).unwrap();
        assert_eq!(file_contents.trim(), "SOMETHING=\"is a variable\"");
    }
}
