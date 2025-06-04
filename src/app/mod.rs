use commands::{Cli, Commands, ConfigCommands, NameOrPath};
use config::GlobalConfig;
use errors_and_warnings::{ErrorCode, HelperErrors};
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
mod errors_and_warnings;
pub mod handlebars;
mod test_helpers;
mod tools;

pub trait IOLogs {
    fn log(&self, str: String);
    fn read(&self, str: String, secure: bool) -> std::io::Result<String>;
    fn error(&self, str: String);
}

impl ErrorCode {
    fn to_io_logs(&self, io: &dyn IOLogs) {
        match self {
            ErrorCode::MissingEntry(path) => {
                io.error(format!("Entry not found: {}", path.join("/")));
            }
            ErrorCode::MissingField(path, field) => io.error(format!(
                "Field not found: {} in path: {}",
                field,
                path.join("/")
            )),
            ErrorCode::NoPassword(path) => {
                io.error(format!(
                    "Entry doesn't contain a password: {}",
                    path.join("/")
                ));
            }
            ErrorCode::NoUsername(path) => {
                io.error(format!(
                    "Entry doesn't contain an username: {}",
                    path.join("/")
                ));
            }
            ErrorCode::NoUrl(path) => {
                io.error(format!("Entry doesn't contain an url: {}", path.join("/")));
            }
        }
    }
}

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
    let original_path = Path::new(&path);
    let absolute_path = original_path.canonicalize();

    match absolute_path {
        Ok(absoluted) => return String::from(absoluted.to_str().unwrap()),
        Err(_e) => {
            return String::from(
                std::env::current_dir()
                    .unwrap()
                    .join(original_path)
                    .to_str()
                    .unwrap()
                    .to_string(),
            );
        }
    }
}

fn open_keepass_db(keepass_path: String, io: &dyn IOLogs) -> Result<Database, Box<dyn Error>> {
    let password = io.read(String::from("Enter the KeePass database password: "), true)?;
    let mut file = File::open(keepass_path).map_err(|_| "Keepass db file not found")?;
    let key = DatabaseKey::new().with_password(&password);
    Database::open(&mut file, key)
        .map_err(|_| "Database cannot be opened, maybe password is wrong?".into())
}

fn render_and_save_template(
    handlebars: &mut LibHandlebars,
    io: &dyn IOLogs,
    name: String,
    template_path: String,
    output_path: String,
    vars: &HashMap<String, String>,
) -> Result<(), Box<dyn Error>> {
    handlebars.register_template_file(&name, template_path)?;

    let rendered = handlebars
        .render(&name, vars)
        .map_err(|e| format!("Failed to render template: {}", e))?;

    std::fs::write(&output_path, rendered)
        .map_err(|e| format!("Failed to write output file {}: {}", output_path, e))?;

    io.log(format!("file written: {}", output_path));
    Ok(())
}

fn parse_variables(io: &dyn IOLogs, variables: Vec<String>) -> HashMap<String, String> {
    let mut parsed_variables: HashMap<String, String> = HashMap::new();
    for variable in variables {
        if let Some((key, value)) = variable.split_once('=') {
            let key = key.trim();
            if !key.is_empty() {
                parsed_variables.insert(key.to_string(), value.to_string());
            } else {
                io.error(format!(
                    "Malformed variable: \"{variable}\": variable name cannot be empty"
                ));
            }
        } else {
            io.error(format!(
                "Malformed variable \"{variable}\": please use var=content"
            ));
        }
    }
    parsed_variables
}

pub fn execute(args: Cli, io: &dyn IOLogs) -> Result<(), Box<dyn Error>> {
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
                let path = Path::new(&url);
                if path.is_absolute() {
                    if path.exists() {
                        io.log(format!("Setting default KeePass DB URL: {:?}", url));
                        config.config.keepass = Some(url);
                        config.save()?;
                    } else {
                        io.error(format!(
                            "The following file doesn't exist or it can't be accessed: {:?}",
                            url
                        ));
                    }
                } else {
                    io.error("The file path is not absolute. It must follow this format: /Users/username/**/*.kdbx on Mac, or C:\\**\\*.kdbx on Windows".to_string());
                }
            }
            ConfigCommands::GetKpDb => match config.config.keepass {
                Some(url) => io.log(format!("Current file is {}", url)),
                None => io.log(format!(
                    "The current configuration '{}' doesn't contain a default keepass db",
                    config.get_file()
                )),
            },
            ConfigCommands::ListFiles => {
                let templates = config.config.get_templates();
                if templates.is_empty() {
                    io.log(String::from("No templates defined"));
                } else {
                    io.log(String::from("Configured templates:"));
                    for template in templates {
                        io.log(format!(
                            "\t {} -> {}",
                            template.template_path, template.output_path
                        ));
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
                    io.log(format!("{:?}", template));
                    if !Path::new(&template.template_path).exists() {
                        io.log(format!(
                            "Template {} does not exist, removing from config",
                            template.template_path
                        ));
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
                    io.log(String::from("No variables defined"));
                } else {
                    io.log(String::from("Variables:"));
                    let mut sorted_keys: Vec<&String> = variables.keys().collect();
                    sorted_keys.sort();
                    for key in sorted_keys {
                        io.log(format!(
                            "\t{} = {}",
                            key,
                            variables
                                .get(key)
                                .expect("Loading a key that should be there")
                        ));
                    }
                }
            }
            ConfigCommands::AddVariables { variables } => {
                let vars_hash = parse_variables(io, variables);
                for (key, value) in vars_hash {
                    config.config.add_var(key, value);
                }
                config.save()?;
            }
            ConfigCommands::DeleteVariables { variables } => {
                for variable in variables {
                    config.config.del_var(variable.to_string());
                }
                config.save()?;
            }
        },
        Commands::Build {
            template,
            keepass,
            output,
            relative_to_input,
            vars,
        } => {
            io.log(format!("Building template file: {}", template));
            io.log(format!("KeePass file: {:?}", keepass));
            let mut variables = config.config.get_vars();
            variables.extend(parse_variables(io, vars));

            let keepass = match keepass {
                Some(url) => url,
                None => {
                    match config.config.keepass {
                        Some(url) => url,
                        None => {
                            io.log(String::from("No keepass file configured in global config or passed as parameter"));
                            return Err("No keepass file configured in global config or passed as parameter".into());
                        }
                    }
                }
            };

            let db = open_keepass_db(keepass, io)?;
            let errors_and_warnings = HelperErrors::new();
            let mut handlebars = build_handlebars(db, &errors_and_warnings);

            let output_path = get_output_path(&template, output, relative_to_input);

            render_and_save_template(
                &mut handlebars,
                io,
                template.clone(),
                template,
                output_path,
                &variables,
            )?;

            let errors = errors_and_warnings.get_errors();
            if !errors.is_empty() {
                io.error("There were some errors processing".into());
                for error in errors {
                    error.to_io_logs(io);
                }
            }
        }
        Commands::BuildAll { vars } => {
            let mut variables = config.config.get_vars();
            variables.extend(parse_variables(io, vars));

            let files = config.config.get_templates();
            let keepass = match config.config.keepass {
                Some(url) => url,
                None => {
                    io.log(String::from("No keepass file configured in global config"));
                    return Err(
                        "No keepass file configured in global config or passed as parameter".into(),
                    );
                }
            };

            io.log(format!(
                "Building all files ({}) with KeePass file: {:?}",
                files.len(),
                keepass
            ));

            let db = open_keepass_db(keepass, io).expect("Database error");

            let mut errors_and_warnings = HelperErrors::new();
            let errors_collector = errors_and_warnings.clone();
            let mut handlebars = build_handlebars(db, &errors_collector);

            for template in files {
                let name = match template.name {
                    Some(ref name) => name.clone(),
                    None => template.template_path.clone(),
                };
                let result = render_and_save_template(
                    &mut handlebars,
                    io,
                    name.clone(),
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
                    io.log(format!("Skipping template {} because of: {:?}", name, err));
                }
                let errors = errors_and_warnings.get_errors();
                if !errors.is_empty() {
                    io.error(format!(
                        "There were some errors processing {}:",
                        template.template_path
                    ));
                    for error in errors {
                        error.to_io_logs(io);
                    }
                }
                errors_and_warnings.clean();
            }
        }
    }
    Ok(())
}
#[cfg(test)]
mod tests {

    use super::*;

    use clap::Parser;
    use std::fs;
    use test_helpers::tests::{IODebug, TestConfig};
    use tools::normalize_separators;

    #[test]
    fn test_join_relative_basic() {
        let test_path = Path::new("/home/users/devel/");
        let relative_path = String::from("./../.././file");
        let result = join_relative(&test_path, relative_path);
        assert_eq!(
            normalize_separators(result.to_str().unwrap()),
            normalize_separators("/home/file")
        );
    }

    #[test]
    fn test_empty_file_commands() {
        let test = TestConfig::create_empty_file();
        let io = IODebug::new();

        // Test get keepass with empty config
        let get_result = execute(
            Cli {
                command: Commands::Config(ConfigCommands::GetKpDb),
                config: Some(String::from(test.get_file_path())),
            },
            &io,
        );
        assert!(get_result.is_ok());

        // Test list templates with empty config
        let list_templates_result = execute(
            Cli {
                command: Commands::Config(ConfigCommands::ListFiles),
                config: Some(String::from(test.get_file_path())),
            },
            &io,
        );
        assert!(list_templates_result.is_ok());

        // Test list variables with empty config
        let list_vars_result = execute(
            Cli {
                command: Commands::Config(ConfigCommands::ListVariables),
                config: Some(String::from(test.get_file_path())),
            },
            &io,
        );
        assert!(list_vars_result.is_ok());

        let logs = io.get_logs();
        // Check that the appropriate empty messages are displayed
        assert!(logs
            .iter()
            .any(|log| log.contains("doesn't contain a default keepass db")));
        assert!(logs.iter().any(|log| log.contains("No templates defined")));
        assert!(logs.iter().any(|log| log.contains("No variables defined")));
    }

    #[test]
    fn test_set_default_keepass_file() {
        let test = TestConfig::create();
        let io = IODebug::new();
        let relative_path = Path::new("test_resources/test_db.kdbx");
        let absolute_path = fs::canonicalize(relative_path).unwrap();
        let absolute_path_string = absolute_path.to_str().unwrap();

        let result = execute(
            Cli {
                command: Commands::Config(ConfigCommands::SetDefaultKpDb {
                    url: String::from(absolute_path_string),
                }),
                config: Some(String::from(test.get_file_path())),
            },
            &io,
        );

        assert!(result.is_ok());

        let config = test.get();
        assert_eq!(
            config.config.keepass,
            Some(String::from(absolute_path_string))
        );

        let logs = io.get_logs();
        assert_eq!(
            logs[0],
            format!("Setting default KeePass DB URL: {:?}", absolute_path_string)
        );
    }

    #[test]
    fn test_get_keepass_file() {
        let test = TestConfig::create();
        let io = IODebug::new();

        // Then get it
        let get_result = execute(
            Cli {
                command: Commands::Config(ConfigCommands::GetKpDb),
                config: Some(String::from(test.get_file_path())),
            },
            &io,
        );

        assert!(get_result.is_ok());

        let logs = io.get_logs();
        assert!(logs[0].contains("/test_resources/test_db.kdbx"));
    }

    #[test]
    fn test_list_files() {
        let test = TestConfig::create();
        let io = IODebug::new();

        let result = execute(
            Cli {
                command: Commands::Config(ConfigCommands::ListFiles),
                config: Some(String::from(test.get_file_path())),
            },
            &io,
        );

        assert!(result.is_ok());

        let logs = io.get_logs();
        println!("{:?}", logs);
        assert_eq!(logs.len(), 4);
        assert_eq!(logs[0], "Configured templates:");
        assert!(logs[2].contains(&normalize_separators("/test_resources/.env.example -> ")));
    }

    #[test]
    fn test_adding_new_template() {
        let test = TestConfig::create();
        let io = IODebug::new();
        let result = execute(
            Cli {
                command: Commands::Config(ConfigCommands::AddFile {
                    name: Some(String::from("New template")),
                    template: String::from("./test_resources/file-with-error"),
                    output: String::from("./tmp/error"),
                    relative_to_input: true,
                }),
                config: Some(String::from(test.get_file_path())),
            },
            &io,
        );

        println!("{:?}", result);
        assert!(result.is_ok());

        let out_config = test.get();

        let templates = out_config.config.get_templates();
        assert_eq!(templates.len(), 4);
        assert_eq!(templates[3].name, Some(String::from("New template")));
    }

    #[test]
    fn test_adding_existing_template_will_replace_it() {
        let mut test = TestConfig::create_super_config();
        test.disable_auto_clean();
        let io = IODebug::new();
        let result = execute(
            Cli {
                command: Commands::Config(ConfigCommands::AddFile {
                    name: Some(String::from("New name")),
                    template: String::from("./test_resources/.env.example"),
                    output: String::from("./test_resources/tmp/.env"),
                    relative_to_input: false,
                }),
                config: Some(String::from(test.get_file_path())),
            },
            &io,
        );

        println!("{:?}", result);
        assert!(result.is_ok());

        let out_config = test.get();

        let templates = out_config.config.get_templates();
        assert_eq!(templates.len(), 3);
        assert_eq!(templates[2].name, Some(String::from("New name")));
    }

    #[test]
    fn test_rendering_invalid_handlebars_template() {
        let mut io = IODebug::new();
        io.add_stdin("MyTestPass".to_string());
        let ret = execute(
            Cli {
                command: Commands::Build {
                    template: String::from("test_resources/file-with-error"),
                    relative_to_input: false,
                    output: String::from("test_outputs/file-with-error"),
                    keepass: Some(String::from("test_resources/test_db.kdbx")),
                    vars: Vec::new(),
                },
                config: None,
            },
            &io,
        );
        if let Err(error) = ret {
            assert_eq!(error.to_string(),"Failed to render template: Error rendering \"test_resources/file-with-error\" line 2, col 15: Helper not found keepass-2-file")
        }
    }

    #[test]
    fn test_rendering_templates_with_invalid_data() {
        let mut io = IODebug::new();
        let test = TestConfig::create_with_errors();
        io.add_stdin("MyTestPass".to_string());
        let result = execute(
            Cli::parse_from([
                "kp2f",
                "--config",
                test.get_file_path().as_str(),
                "build-all",
            ]),
            &io,
        );
        println!("{:?}", result);
        assert!(result.is_ok());

        let errors = io.get_errors();
        println!("{:?}", errors);
        assert_eq!(errors.len(), 5);
        assert!(errors[0].contains("0-with-errors"));
        assert_eq!(errors[1], "Entry not found: invalid/entry");
        assert_eq!(errors[2], "Field not found: whatever in path: group1/test2");
        assert!(errors[3].contains("1-with-other-errors"));
    }

    #[test]
    fn test_prune_command() {
        let test = TestConfig::create();
        let io = IODebug::new();
        let result = execute(
            Cli {
                config: Some(String::from(test.get_file_path())),
                command: Commands::Config(ConfigCommands::Prune),
            },
            &io,
        );
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
        let io = IODebug::new();
        let result = execute(
            Cli {
                config: Some(String::from(test.get_file_path())),
                command: Commands::Config(ConfigCommands::Delete {
                    template: NameOrPath::Name {
                        name: String::from("other"),
                    },
                }),
            },
            &io,
        );
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
        let mut io = IODebug::new();
        io.add_stdin("MyTestPass".to_string());
        let result = execute(
            Cli {
                config: Some(String::from(test.get_file_path())),
                command: Commands::BuildAll { vars: Vec::new() },
            },
            &io,
        );
        println!("{:?}", result);
        assert!(result.is_ok());

        assert!(Path::new("test_resources/tmp/.env").exists());
    }

    #[test]
    fn test_add_variables() {
        let test = TestConfig::create();
        let io = IODebug::new();
        let result = execute(
            Cli {
                config: Some(String::from(test.get_file_path())),
                command: Commands::Config(ConfigCommands::AddVariables {
                    variables: Vec::from([
                        String::from("var1=Some variable"),
                        String::from("email=j@k.com"),
                    ]),
                }),
            },
            &io,
        );
        println!("{:?}", result);
        assert!(result.is_ok());

        let variables = test.get().config.get_vars();
        assert_eq!(variables.len(), 2);
        assert_eq!(variables.get("var1"), Some(&String::from("Some variable")));
        assert_eq!(variables.get("email"), Some(&String::from("j@k.com")));
    }

    #[test]
    fn test_list_variables() {
        let test = TestConfig::create_with_vars();
        let io = IODebug::new();

        let result = execute(
            Cli {
                config: Some(test.get_file_path()),
                command: Commands::Config(ConfigCommands::ListVariables),
            },
            &io,
        );

        println!("{:?}", result);
        assert!(result.is_ok());

        let logs = io.get_logs();
        assert_eq!(logs.len(), 3);
        assert_eq!(logs[0], "Variables:");
        assert_eq!(logs[1], "\temail = j@k.com");
        assert_eq!(logs[2], "\tsomething = is a variable");
    }

    #[test]
    fn test_delete_variables() {
        let test = TestConfig::create_with_vars();
        let io = IODebug::new();
        let result = execute(
            Cli {
                config: Some(String::from(test.get_file_path())),
                command: Commands::Config(ConfigCommands::DeleteVariables {
                    variables: Vec::from([String::from("something")]),
                }),
            },
            &io,
        );
        println!("{:?}", result);
        assert!(result.is_ok());

        let variables = test.get().config.get_vars();
        assert_eq!(variables.len(), 1);
        assert_eq!(variables.get("email"), Some(&String::from("j@k.com")));

        let templates = test.get().config.get_templates();
        assert_eq!(templates.len(), 2);
        assert_eq!(templates[0].name, Some(String::from("valid")));
    }

    #[test]
    fn test_add_variables_with_malformed_input() {
        let test = TestConfig::create();
        let io = IODebug::new();
        let result = execute(
            Cli {
                config: Some(String::from(test.get_file_path())),
                command: Commands::Config(ConfigCommands::AddVariables {
                    variables: Vec::from([
                        String::from("no_equals_sign"), // Missing '=' character
                        String::from(
                            "connection_string=postgres://user:password=@localhost:5432/mydb",
                        ), // Contains multiple '=' chars
                        String::from("key="),           // Empty value
                        String::from("=value"),         // Empty key
                    ]),
                }),
            },
            &io,
        );
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
        let mut io = IODebug::new();
        io.add_stdin("MyTestPass".to_string());
        let result = execute(
            Cli {
                config: Some(String::from(test.get_file_path())),
                command: Commands::Build {
                    template: String::from("./test_resources/with_variables"),
                    output: String::from("./test_resources/tmp/with_variables"),
                    keepass: None,
                    relative_to_input: false,
                    vars: Vec::from([String::from("email=j@k2.com")]),
                },
            },
            &io,
        );
        println!("{:?}", result);
        assert!(result.is_ok());

        let output_file_path = String::from("test_resources/tmp/with_variables");
        let file_contents = std::fs::read_to_string(output_file_path).unwrap();
        assert!(file_contents.contains("SOMETHING=\"is a variable\""));
        assert!(file_contents.contains("EMAIL=\"j@k2.com\""));
    }

    #[test]
    fn test_something_is_a_variable_build_all() {
        let test = TestConfig::create_with_vars();
        let mut io = IODebug::new();
        io.add_stdin("MyTestPass".to_string());
        let result = execute(
            Cli {
                config: Some(String::from(test.get_file_path())),
                command: Commands::BuildAll {
                    vars: Vec::from([String::from("email=j@k2.com")]),
                },
            },
            &io,
        );
        println!("{:?}", result);
        assert!(result.is_ok());

        let output_file_path = String::from("test_resources/tmp/with_variables");
        let file_contents = std::fs::read_to_string(output_file_path).unwrap();
        assert!(file_contents.contains("SOMETHING=\"is a variable\""));
        assert!(file_contents.contains("EMAIL=\"j@k2.com\""));
    }

    #[test]
    fn test_fix_windows() {
        let template = String::from("./test_resources/.env.example");
        println!("{}", template);

        let original_path = Path::new(&template);
        println!("{:?}", original_path);
        let absolute_path = original_path.canonicalize();
        println!("{:?}", absolute_path);

        assert!(false)
    }
}
