use clap::{Parser, Subcommand};
use deadcode_core::{DeadCode, ExecutionMode};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "deadcode", version = "0.1.0")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Run { file: PathBuf },
    Repl,
}

fn main() {
    let cli = Cli::parse();
    
    match cli.command {
        Commands::Run { file } => {
            println!("Running: {}", file.display());
            let dc = DeadCode::new(ExecutionMode::Interpret);
            match dc.run_file(&file) {
                Ok(value) => {
                    println!("Result: {}", value);
                }
                Err(e) => {
                    eprintln!("Error: {}", e);
                    std::process::exit(1);
                }
            }
        }
        Commands::Repl => {
            println!("REPL not yet implemented");
        }
    }
}
