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
        h: Option<String>,

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
    }
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Init { h, z } => {
            if let Some(hash) = h {
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
    }
    Ok(())
}
