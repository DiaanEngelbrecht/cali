use cali_cli::scaffold::{controller::sync_protos_with_controllers, store::create_store};
use clap::{Parser, Subcommand};

/// Cali CLI
/// Create a new application with New
/// Scaffold into an existing application with Generate
#[derive(Parser, Debug)]
#[command(name = "Cali CLI")]
#[command(author = "Diaan Engelbrecht")]
#[command(version, about = "CLI utilities for the Cali Framework", long_about = None)]
struct Cli {
    #[command(subcommand)]
    commands: Option<Commands>,
}

#[derive(Subcommand, Debug)]
enum Commands {
    New {
        name: String,
    },
    Generate {
        #[command(subcommand)]
        target: GenerateTarget,
    },
}

#[derive(Subcommand, Debug)]
enum GenerateTarget {
    Controllers,
    Store { name: String },
}

fn main() {
    let cli = Cli::parse();

    if let Some(Commands::New { name }) = &cli.commands {
        cali_cli::scaffold::new::create_app(name);
    }

    if let Some(Commands::Generate { target }) = &cli.commands {
        match target {
            GenerateTarget::Controllers => sync_protos_with_controllers(),
            GenerateTarget::Store { name } => create_store(name.clone()),
        }
    }
}
