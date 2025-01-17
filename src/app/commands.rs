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
        dot_env: String,

        #[arg(short, long, help="Overwrite the glogal keepass file")]
        keepass: Option<String>
    }
}

#[derive(Debug, Subcommand)]
pub enum ConfigCommands {
    // #[command(help="Set the default keepass db file")]
    SetDefaultKpDb {
        url: Option<String>
    },
    // #[command(help="Get the default keepass db file configured")]
    GetKpDb {}
}
