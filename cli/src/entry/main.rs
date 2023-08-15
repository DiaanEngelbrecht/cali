use std::{fs::File, io::Write, path::Path};

use clap::{Parser, Subcommand};
use flair_core::{
    protos::parser::get_proto_data,
    scaffolding::controller::{
        generate_controller_files_contents, generate_controller_mod_file_contents,
    },
};

/// Flair CLI
/// Create a new application with New
/// Scaffold into an existing application with Generate
#[derive(Parser, Debug)]
#[command(name = "Flair CLI")]
#[command(author = "Diaan Engelbrecht")]
#[command(version, about = "CLI utilities for the Flair Framework", long_about = None)]
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
}

fn main() {
    let cli = Cli::parse();

    if let Some(Commands::New { name }) = &cli.commands {
        flair_cli::scaffold::new::create_app(name);
    }

    if let Some(Commands::Generate { target }) = &cli.commands {
        match target {
            GenerateTarget::Controllers => {
                let path = Path::new("./interface/grpc/services");
                let proto_data = get_proto_data(&path).expect("Should have worked");
                let file_with_contents = generate_controller_files_contents(&proto_data);
                let mod_contents = generate_controller_mod_file_contents(&proto_data);

                for (file_name, file_contents) in file_with_contents.iter() {
                    // Check if file exists and if it doesn't auto gen contents
                    // let file_exists = Path::new(file_name).try_exists().unwrap_or(false);
                    // if !file_exists {
                    let mut file =
                        File::create(file_name).expect("Could not create controller file");
                    file.write_all(file_contents.as_bytes())
                        .expect("Could not write to controller file");
                    // }
                }
                let mut mod_file = File::create("./web/src/controllers/mod.rs")
                    .expect("Could not create controller file");

                mod_file
                    .write_all(&mod_contents)
                    .expect("Could not write body");
            }
        }
    }
}
