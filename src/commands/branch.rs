use std::fs::{read_to_string, write};
use std::path::Path;
use anyhow::Result;

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
