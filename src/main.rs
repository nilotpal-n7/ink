use std::path::PathBuf;
use std::str::FromStr;
use clap::{Parser, Subcommand};
use anyhow::Result;

use crate::utils::enums::{AddMode, HashAlgo};

mod commands;
mod utils;

#[derive(Parser)]
#[command(name = "ink", about = "🧠 Ink: A Git-like VCS", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Init {
        #[arg(short)]
        a: Option<String>,

        #[arg(short)]
        z: bool,
    },
    Uninit,
    Add {
        files: Vec<PathBuf>,

        #[arg(short)]
        u: bool,
    },
    Commit {
        message: String,

        #[arg(short)]
        a: bool
    },
    Branch {
        name: Option<String>
    },
    Checkout {
        #[arg(short)]
        b: bool,

        name: String,
    }
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Init { a, z } => {
            if let Some(hash) = a {
                commands::init::run(HashAlgo::from_str(&hash)?, z)?
            }
            commands::init::run(HashAlgo::default(), z)?
        },
        Commands::Uninit => commands::uninit::run()?,
        Commands::Add { files, u } => {
            if u {
                commands::add::run(AddMode::Update)?
            } else if files == vec![PathBuf::from(".")] {
                commands::add::run(AddMode::All)?
            } else {
                commands::add::run(AddMode::Files(files))?
            }
        },
        Commands::Commit { message, a } => commands::commit::run(message, a)?,
        Commands::Branch { name } => commands::branch::run(name)?,
        Commands::Checkout { b, name } => commands::checkout::run(b, name)?,
    }

    Ok(())
}
