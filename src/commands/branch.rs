use std::fs::{create_dir_all, read_to_string, write};
use std::path::Path;
use anyhow::Result;
use colored::Colorize;
use rayon::prelude::*;

use crate::commands::commit::read_current_commit;

pub fn run(name: Option<String>) -> Result<()> {
    match name {
        Some(n) => {
            let root = Path::new(".ink");
            let branch_path = root.join("refs").join("heads").join(&n);

            if branch_path.exists() {
                println!("Branch with name {} already exist", n);
                return Ok(());
            }

            let current_commit = read_current_commit()?;
            create_dir_all(branch_path.parent().unwrap())?;
            write(branch_path, current_commit)?;
            println!("Created branch '{}'", n);
        }

        None => {
            let root = Path::new(".ink");
            let branches_path = root.join("refs").join("heads");
            let current_branch = read_current_branch()?;
            let entries: Vec<_> = branches_path.read_dir()?.collect::<Result<_, _>>()?;

            entries.par_iter().for_each(|entry| {
                let name = entry.file_name().to_string_lossy().to_string();

                if name == current_branch {
                    println!("{}", format!("* {}", name.underline()).green()); // current branch
                } else {
                    println!("  {}", name);
                }
            });
        }
    }

    Ok(())
}

pub fn read_current_branch() -> Result<String> {
    let root = Path::new(".ink");
    let head_path = root.join("HEAD");

    let head_contents = read_to_string(&head_path)?.trim().to_string();
    let head_contents: Vec<&str> = head_contents.split("/").collect();
    let branch = head_contents[head_contents.len() - 1];

    Ok(branch.to_string())
}

pub fn update_current_branch(new_branch: &str) -> Result<()> {
    let root = Path::new(".ink");
    let head_path = root.join("HEAD");
    write(head_path, format!("ref: refs/heads/{}\n", new_branch))?;

    Ok(())
}
