use std::fs::{create_dir_all, write};
use std::path::{Path, PathBuf};
use anyhow::{anyhow, Result};

use crate::commands::commit::read_current_commit;
use crate::utils::object::{read_blob_object, read_tree_object};

/// Entry point for restore command
pub fn run(target: PathBuf) -> Result<()> {
    let commit_hash = read_current_commit()?;
    if commit_hash == "0000000000000000000000000000000000000000000000000000000000000000" {
        return Err(anyhow!("No commits yet. Nothing to restore."));
    }

    let tree_hash = crate::commands::commit::read_tree_of_commit(&commit_hash)?;
    restore_tree(&tree_hash, Path::new(""), &target)?;

    Ok(())
}

/// Recursively restores a tree into the working directory
fn restore_tree(tree_hash: &str, target: &Path, restore_target: &Path) -> Result<()> {
    let entries = read_tree_object(tree_hash)?;

    for (entry_path, object_type, object_hash) in entries {
        let full_path = target.join(&entry_path);

        match object_type.as_str() {
            "blob" => {
                if restore_target == Path::new(".") || full_path.starts_with(restore_target) {
                    let content = read_blob_object(&object_hash)?;
                    if let Some(parent) = full_path.parent() {
                        create_dir_all(parent)?;
                    }
                    write(&full_path, content)?;
                    println!("Restored: {}", full_path.display());
                }
            }

            "tree" => {
                let new_target = if entry_path.as_os_str().is_empty() {
                    target.to_path_buf()
                } else {
                    target.join(&entry_path)
                };
                restore_tree(&object_hash, &new_target, restore_target)?;
            }

            _ => return Err(anyhow!("Unknown object type in tree: {}", object_type)),
        }
    }

    Ok(())
}
