use std::fs::{read, read_to_string, write};
use std::path::Path;
use anyhow::{ anyhow, Result };

use crate::commands;
use crate::commands::branch::read_current_branch;
use crate::utils::enums::AddMode;
use crate::utils::object::{create_commit, create_tree };
use crate::utils::zip::decompress;

pub fn run(message: String, a: bool) -> Result<()> {
    if a {
        commands::add::run(AddMode::Update)?
    }

    let tree_hash = create_tree()?;

    // Try reading the previous commit's tree hash
    let parent_hash = read_current_commit().ok();
    if let Some(ref parent) = parent_hash {
        let parent_tree = read_tree_of_commit(parent)?;

        if tree_hash == parent_tree {
            println!("Nothing to commit — working tree matches last commit.");
            return Ok(());
        }
    }

    let comment_hash = create_commit(&tree_hash, parent_hash.as_deref(), &message, "Nilotpal Gupta")?;
    update_current_commit(&comment_hash)?;

    Ok(())
}

pub fn read_current_commit() -> Result<String> {
    let root = Path::new(".ink");
    let head_path = root.join("HEAD");

    let head_contents = read_to_string(&head_path)?;
    let head_contents = head_contents.trim();

    let commit_hash = if head_contents.starts_with("ref:") {
        // Follow the reference (e.g., "ref: refs/heads/main")
        let ref_path = head_contents.trim_start_matches("ref:").trim();
        let ref_file = root.join(ref_path);
        if !ref_file.exists() {
            return Err(anyhow!("Reference file {:?} does not exist", ref_file));
        }
        read_to_string(ref_file)?.trim().to_string()
    } else {
        // HEAD contains the commit hash directly (detached HEAD)
        head_contents.to_string()
    };

    Ok(commit_hash)
}

pub fn update_current_commit(new_hash: &str) -> Result<()> {
    let root = Path::new(".ink");
    let current_branch = read_current_branch()?;
    let head_path = root.join("refs").join("heads").join(current_branch);
    write(head_path, new_hash)?;

    Ok(())
}

pub fn read_tree_of_commit(commit_hash: &str) -> Result<String> {
    let commit_path = Path::new(".ink").join("objects").join(&commit_hash[..2]).join(&commit_hash[2..]); // use your existing logic
    let content = read(commit_path)?;
    let decompressed = decompress(content)?;
    let text = std::str::from_utf8(&decompressed)?;

    for line in text.lines() {
        if line.starts_with("tree ") {
            return Ok(line[5..].trim().to_string());
        }
    }
    Err(anyhow!("No tree found in commit {}", commit_hash))
}
