use mimalloc::MiMalloc;
#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

mod minisat;
mod utils;
use std::{path::PathBuf, process::exit};

use clap::{Parser, Subcommand};

use validator::{Validate, ValidationError};

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}
#[derive(Subcommand)]
enum Commands {
    /// Use minisat(2.2.0) solver
    Minisat(minisat::Arg),
}
fn main() {
    let cli = Cli::parse();
    let ret = match cli.command {
        Commands::Minisat(arg) => {
            arg.validate().unwrap();
            arg.run()
        }
    };

    // match ret {
    //     Ok(code) => exit(code),
    //     Err(_) => todo!(),
    // }
}
