use std::path::Path;
use anyhow::Result;
use std::fs::{create_dir, create_dir_all, write};

use crate::utils::enums::HashAlgo;
use crate::utils::hash::save_hash_algo;
use crate::utils::zip::save_is_zip;
use crate::utils::dir::hide_folder_windows;

pub fn run(h: HashAlgo, z: bool) -> Result<()> {
    let root: &Path = Path::new(".ink");

    if root.exists() {
        println!("Ink already initialized!");
        return Ok(());
        
    } else {
        create_dir(root)?;
        #[cfg(target_os = "windows")]
        hide_folder_windows(Path::new(".ink"));
    }

    create_dir_all(root.join("objects"))?;
    create_dir_all(root.join("refs").join("heads"))?;
    create_dir_all(root.join("refs").join("INDEXES"))?;

    write(root.join("index"), "")?;
    write(root.join("config"), "")?;
    write(root.join("HEAD"), "ref: refs/heads/main")?;
    write(root.join("refs").join("INDEXES").join("main"), "")?;

    save_hash_algo(h)?;
    save_is_zip(z)?;

    println!("Empty Ink dir initialized...");
    Ok(())
}
