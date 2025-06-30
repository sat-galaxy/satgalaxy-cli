#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

mod core;
mod glucose;
mod minisat;
mod parser;
mod utils;
use std::process::exit;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}
#[derive(Subcommand)]
enum Commands {
    /// Use minisat(2.2.0) solver
    /// https://github.com/niklasso/minisat
    Minisat(minisat::Arg),
    /// Use glucose(4.2.1) solver
    /// https://github.com/arminbiere/glucose
    Glucose(glucose::Arg),
}
fn main() {
    let cli = Cli::parse();
    let ret: Result<i32, anyhow::Error> = match cli.command {
        Commands::Minisat(arg) => arg.run(),
        Commands::Glucose(arg) => arg.run(),
    };

    match ret {
        Ok(code) => exit(code),
        Err(e) => eprintln!("c ERROR: {}", e),
    }
}
