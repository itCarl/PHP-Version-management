mod cli;
mod config;
mod core;
mod download;
mod system;
mod xampp;

use anyhow::Result;
use clap::Parser;
use cli::commands::{Cli, Commands};

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Init => cli::handlers::handle_init(),
        Commands::List => cli::handlers::handle_list(),
        Commands::Current => cli::handlers::handle_current(),
        Commands::Use { version } => cli::handlers::handle_use(&version),
        Commands::Install { version, nts, x86 } => {
            cli::handlers::handle_install(&version, nts, x86)
        }
        Commands::Uninstall { version } => cli::handlers::handle_uninstall(&version),
        Commands::Xampp { command } => cli::handlers::handle_xampp(command),
        Commands::Doctor => cli::handlers::handle_doctor(),
        Commands::Which => cli::handlers::handle_which(),
    }
}
