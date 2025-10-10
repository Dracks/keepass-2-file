use clap::{command, Parser, Subcommand};
use clap_complete::Shell;

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Cli {
    /// Set a custom local configuration
    #[arg(long, value_name = "Config")]
    pub config: Option<String>,

    #[arg(long, action = clap::ArgAction::SetTrue, help = "Suppress warning output (errors still shown)")]
    pub disable_warnings: bool,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    /// administer the configuration
    #[command(subcommand)]
    Config(ConfigCommands),

    /// Generate shell completions
    Completion {
        /// The shell to generate completions for
        #[arg(value_enum)]
        shell: Shell,
    },

    /// Build a template to the selected output
    Build {
        template: String,

        #[arg(
            short,
            long,
            help = "when output is a relative path, it will make it relative to the folder of template when enabled or relative to current when disabled"
        )]
        relative_to_input: bool,

        output: String,

        #[arg(short, long, help = "Overwrite the global keepass file")]
        keepass: Option<String>,

        #[arg(short, long, help = "Add or overwrite variables into the build")]
        vars: Vec<String>,
    },

    /// Build all the templates in the configuration
    BuildAll {
        #[arg(short, long, help = "Add or overwrite variables into the build")]
        vars: Vec<String>,
    },
}

#[derive(Debug, Subcommand)]
pub enum NameOrPath {
    Name { name: String },
    Paths { path: String, output: String },
}

#[derive(Debug, Subcommand)]
pub enum ConfigCommands {
    /// Set the default KeePass file
    SetDefaultKpDb { url: String },

    /// Get the current KeePass file
    GetKpDb,

    /// List the templates inside the configuration
    ListFiles,

    /// Add a template into the config
    AddFile {
        #[arg(long)]
        name: Option<String>,

        template: String,
        output: String,
        #[arg(
            short,
            long,
            help = "when output is a relative path, it will make it relative to the folder of template when enabled or relative to current when disabled"
        )]
        relative_to_input: bool,
    },

    /// Deletes all templates that the source doesn't exists
    Prune,

    /// Deletes a template
    Delete {
        #[command(subcommand)]
        template: NameOrPath,
    },

    /// List default variables in the config
    ListVariables,

    /// Add a default variable
    AddVariables {
        #[arg(num_args=1.., help="Variables defined as var=value, when existing will be overwrited")]
        variables: Vec<String>,
    },

    /// Delete a variable variable
    DeleteVariables {
        #[arg(num_args=1..)]
        variables: Vec<String>,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cli_build_all_variables() {
        let cli = Cli::parse_from([
            "kp2f",
            "build-all",
            "-v",
            "email=something",
            "-v",
            "email2=j@k.com",
        ]);

        assert!(
            matches!(cli.command, Commands::BuildAll { vars } if vars == Vec::from(["email=something".to_string(), "email2=j@k.com".to_string()]))
        );
    }

    #[test]
    fn test_cli_build_variables() {
        let cli = Cli::parse_from([
            "kp2f",
            "build",
            "file.env.example",
            "-r",
            ".env",
            "-v",
            "email=something",
            "-v",
            "email2=j@k.com",
        ]);

        match cli.command {
            Commands::Build {
                template,
                relative_to_input,
                output,
                keepass,
                vars,
            } => {
                assert_eq!(
                    vars,
                    Vec::from(["email=something".to_string(), "email2=j@k.com".to_string()])
                );
                assert_eq!(output, ".env");
                assert_eq!(relative_to_input, true);
                assert_eq!(template, "file.env.example");
                assert_eq!(keepass, None);
            }
            _ => assert!(false),
        }
    }
}
