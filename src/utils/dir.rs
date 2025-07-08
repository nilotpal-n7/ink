use std::fs::{read_dir, remove_dir};
use std::path::Path;
use anyhow::Result;

pub fn is_in_ink(path: &Path) -> bool {
    path.components().any(|c| c.as_os_str() == ".ink")
}

pub fn remove_empty_parents_up_to(path: &Path, stop_at: &Path) -> Result<()> {
    let mut current = path.parent();

    while let Some(dir) = current {
        if dir == stop_at {
            break;
        }

        // Only remove if directory is empty
        if read_dir(dir)?.next().is_none() {
            remove_dir(dir)?;
        } else {
            break; // Stop as soon as we hit a non-empty directory
        }

        current = dir.parent();
    }

    Ok(())
}
