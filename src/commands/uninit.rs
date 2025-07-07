use std::fs::remove_dir_all;
use std::path::Path;
use anyhow::Result;

pub fn run() -> Result<()> {
    let root = Path::new(".ink");

    if !root.exists() {
        println!("Ink dir not initialized!");
        return Ok(());
    }

    remove_dir_all(".ink")?;

    println!("Ink dir removed and uninitialized...");
    Ok(())
}
