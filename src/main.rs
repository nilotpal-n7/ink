use std::path::PathBuf;
use std::process::Command as SysCommand;
use std::str::FromStr;
use clap::{Parser, Subcommand};
use anyhow::Result;

use crate::utils::enums::{AddMode, HashAlgo};

mod commands;
mod utils;

#[derive(Parser)]
#[command(name = "ink", about = "🧠 Ink: A Git-like VCS", long_about = None)]
struct MultiCli {
    /// Multi-command mode: pass multiple commands separated by ';;'
    #[arg(short, long)]
    multi: bool,

    /// The full command string when using -m
    #[arg()]
    commands: Vec<String>,
}

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

        #[arg(long)]
        force: bool,

        name: String,
    }
}

fn main() -> Result<()> {
    let raw_args: Vec<String> = std::env::args().collect();

    // First, check for -m or --multi manually
    if raw_args.len() >= 2 && (raw_args[1] == "-m" || raw_args[1] == "--multi") {
        let multi_cli = MultiCli::parse();
        let joined = multi_cli.commands.join(" ");
        let commands = joined.split("@").map(str::trim).filter(|c| !c.is_empty());

        for cmd in commands {
            let binary = &raw_args[0];
            let args: Vec<&str> = cmd.split_whitespace().collect();

            let status = SysCommand::new(binary).args(args).status()?;
            if !status.success() {
                return Err(anyhow::anyhow!("Command failed: {}", cmd));
            }
        }

        return Ok(());
    }

    // Otherwise, parse as usual
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
        Commands::Checkout { b, force, name } => commands::checkout::run(b, force, name)?,
    }

    Ok(())
}
