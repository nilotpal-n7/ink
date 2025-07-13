use std::fs::{read, read_to_string, write};
use std::path::Path;
use anyhow::{ anyhow, Result };

use crate::commands;
use crate::commands::branch::read_current_branch;
use crate::utils::enums::AddMode;
use crate::utils::log::Log;
use crate::utils::object::{create_commit, create_tree };
use crate::utils::zip::decompress;

pub fn run(message: String, a: bool) -> Result<()> {
    if a {
        commands::add::run(AddMode::Update)?
    }

    let tree_hash = create_tree()?;

    // Try reading the previous commit's tree hash
    let parent_hash = read_current_commit()?;

    if parent_hash != "0000000000000000000000000000000000000000000000000000000000000000" {
        let parent_tree = read_tree_of_commit(&parent_hash)?;
        if tree_hash == parent_tree {
            println!("Nothing to commit — working tree matches last commit.");
            return Ok(());
        }
    }

    let commit_hash = create_commit(&tree_hash, &parent_hash, &message, "Nilotpal Gupta")?;
    update_current_commit(&commit_hash)?;
    let mut log = Log::load()?;
    log.add(parent_hash, "commit".to_string(), message)?;
    log.save()?;
    
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
            return Ok(String::from("0000000000000000000000000000000000000000000000000000000000000000"));
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
    
    let (_, body) = std::str::from_utf8(&decompressed)?
        .split_once('\0')
        .ok_or_else(|| anyhow!("Invalid commit object format: no null byte"))?;

    for line in body.lines() {
        if line.starts_with("tree ") {
            return Ok(line[5..].trim().to_string());
        }
    }
    Err(anyhow!("No tree found in commit {}", commit_hash))
}

pub fn get_branch_commit(branch: &str) -> Result<String> {
    let path = Path::new(".ink").join("refs").join("heads").join(branch);
    if !path.exists() {
        return Ok("0000000000000000000000000000000000000000000000000000000000000000".to_string());
    }

    let content = std::fs::read_to_string(path)?;
    Ok(content.trim().to_string())
}
