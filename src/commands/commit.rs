use std::fs::{read_to_string, write};
use std::path::Path;
use anyhow::{ anyhow, Result };

use crate::commands;
use crate::utils::enums::AddMode;
use crate::utils::object::{create_commit, create_tree };

pub fn run(message: String, a: bool) -> Result<()> {
    if a {
        commands::add::run(AddMode::Update)?
    }

    let tree_hash = create_tree()?;
    let parent_hash = read_current_commit().ok(); // Allow first commit
    let comment_hash = create_commit(&tree_hash, parent_hash.as_deref(), &message, "Nilotpal Gupta")?;
    update_head(&comment_hash)?;

    Ok(())
}

pub fn read_current_commit() -> Result<String> {
    let root = Path::new(".mygit");
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

pub fn update_head(new_hash: &str) -> Result<()> {
    let root = Path::new(".mygit");
    let head_path = root.join("HEAD");

    let head_contents = read_to_string(&head_path)?.trim().to_string();

    if head_contents.starts_with("ref:") {
        // Symbolic ref, like: "ref: refs/heads/main"
        let ref_path = head_contents.trim_start_matches("ref:").trim();
        let full_ref_path = root.join(ref_path);
        write(full_ref_path, format!("{}\n", new_hash))?;
    } else {
        // Detached HEAD – write the hash directly to HEAD
        write(head_path, format!("{}\n", new_hash))?;
    }

    Ok(())
}
