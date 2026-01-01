use clap::Parser;
use better_cp::cli::{Cli, Commands};

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Move(_args) => {
            eprintln!("Move operation not yet implemented");
        }
        Commands::Copy(_) => {
            eprintln!("Use better-cp for copy operations");
        }
    }
}
