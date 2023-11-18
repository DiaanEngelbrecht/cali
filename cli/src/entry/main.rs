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

#[cfg(test)]
mod tests {
    use assert_cmd::Command;

    struct Cleanup;

    impl Drop for Cleanup {
        fn drop(&mut self) {
            std::fs::remove_dir_all("cali_test").expect("Couldn't clean up the test project");
        }
    }

    #[test]
    fn scaffolds_and_compiles() {
        let mut cmd = Command::cargo_bin("cali").unwrap();
        let assert = cmd.arg("new").arg("cali_test").assert();
        let _cleanup = Cleanup;
        assert.success();

        let cargo_check_command = std::process::Command::new("cargo")
            .arg("check")
            .arg("--manifest-path")
            .arg("./cali_test/Cargo.toml")
            .output();

        assert!(cargo_check_command.is_ok());
        if let Ok(output) = cargo_check_command {
            let code = output.status.code().unwrap();
            if code > 0 {
                let str_result = std::str::from_utf8(&output.stderr).unwrap();
                println!("Error output was {}", str_result);
            }
            assert!(code == 0);
        }
    }
}
