// CLI Setup

use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(name = "JukeBoxCLI", author, version)]
#[command(about, long_about = None)]
pub struct Cli {
    #[arg(short, long, action = clap::ArgAction::Count)]
    pub verbose: u8,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    #[command(about = "Run a simple probing of data")]
    Probe,
    #[command(about = "Communicate with a JukeBox device")]
    Commune,
    #[command(about = "Test desktop GUI")]
    Gui,
}
