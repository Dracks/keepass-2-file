use clap::{command, Parser, Subcommand};

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Cli {
    #[arg(long, value_name = "Config")]
    pub config: Option<String>,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    #[command(subcommand)]
    Config(ConfigCommands),

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

        #[arg(short, long, help = "Overwrite the glogal keepass file")]
        keepass: Option<String>,
    },

    /// Build all the templates in the configuration
    BuildAll {},
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
