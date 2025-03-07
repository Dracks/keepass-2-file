use clap::{Parser, Subcommand};

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Cli {
    #[arg(value_name = "GLOBAL Config")]
    pub config: Option<String>,

    #[command(subcommand)]
    pub commands: Commands,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    #[command(subcommand)]
    Config(ConfigCommands),

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

    BuildAll,
}

#[derive(Debug, Subcommand)]
pub enum ConfigCommands {
    SetDefaultKpDb {
        url: String,
    },

    GetKpDb,

    ListFiles,

    AddFile {
        template: String,
        output: String,
        #[arg(
            short,
            long,
            help = "when output is a relative path, it will make it relative to the folder of template when enabled or relative to current when disabled"
        )]
        relative_to_input: bool,
    },
}
