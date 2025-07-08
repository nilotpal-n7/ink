use std::collections::{HashMap, HashSet};
use std::fs::{create_dir_all, read, File};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::str::from_utf8;
use std::sync::Mutex;

use anyhow::{anyhow, Result};
use dashmap::DashMap;
use rayon::prelude::*;

use crate::commands;
use crate::commands::branch::update_current_branch;
use crate::commands::commit::{get_branch_commit, read_current_commit, read_tree_of_commit};
use crate::utils::hash::hash_object;
use crate::utils::index::{Index, IndexEntry};
use crate::utils::zip::decompress;

pub fn run(b: bool, name: String) -> Result<()> {
    let mut target_commit: Option<String> = None;

    if b {
        commands::branch::run(Some(name.clone()))?;
        // Avoid switching before tree is safely handled
        target_commit = read_current_commit().ok();
    } else {
        target_commit = Some(get_branch_commit(&name)?);
    }

    let target_commit = target_commit.ok_or_else(|| anyhow!("Target branch has no commit"))?;
    let index = Index::load()?;
    let index_map: HashMap<_, _> = index
        .entries
        .par_iter()
        .map(|(_, entry)| (entry.path.clone(), entry.hash.clone()))
        .collect();

    let current_commit = read_current_commit().ok();
    let current_tree = if let Some(commit) = &current_commit {
        get_tree_entries(&read_tree_of_commit(commit)?)?
    } else {
        HashMap::new()
    };

    let tree_hash = read_tree_of_commit(&target_commit)?;
    let target_tree = get_tree_entries(&tree_hash)?;

    let all_paths: HashSet<_> = index_map
        .keys()
        .chain(current_tree.keys())
        .chain(target_tree.keys())
        .cloned()
        .collect();

    all_paths.par_iter().try_for_each(|path| {
        let index_hash = index_map.get(path);
        let current_hash = current_tree.get(path);
        let target_hash = target_tree.get(path);

        let clean = is_clean(path, index_hash)?;

        match (clean, current_hash, target_hash) {
            (false, _, _) => Err(anyhow!(
                "Uncommitted changes in '{}', please commit or stash them first.",
                path.display()
            )),
            (true, Some(_), None) => {
                // Delete only if current had it and target doesn't
                if path.exists() {
                    std::fs::remove_file(path)?;
                }
                Ok(())
            }
            (true, _, Some(target_hash)) => restore_blob(path, target_hash),
            _ => Ok(()),
        }
    })?;

    // Only now it's safe to update HEAD
    update_current_branch(&name)?;

    let entries: Vec<IndexEntry> = target_tree
        .par_iter()
        .map(|(path, hash)| IndexEntry {
            path: path.clone(),
            hash: hash.clone(),
        })
        .collect();

    let new_index = Mutex::new(Index::default());
    entries.par_iter().for_each(|entry| {
        let mut idx = new_index.lock().unwrap();
        idx.add(entry.clone());
    });

    let index = new_index.into_inner().unwrap();
    index.save()?;

    println!("Switched to branch '{}'", name);
    Ok(())
}

pub fn get_tree_entries(tree_hash: &str) -> Result<HashMap<PathBuf, String>> {
    let out = DashMap::new();
    read_tree_recursive(PathBuf::new(), tree_hash, &out)?;
    Ok(out.into_iter().collect())
}

pub fn read_tree_recursive(
    prefix: PathBuf,
    hash: &str,
    out: &DashMap<PathBuf, String>,
) -> Result<()> {
    let obj_path = Path::new(".ink")
        .join("objects")
        .join(&hash[..2])
        .join(&hash[2..]);

    let compressed = read(&obj_path)?;
    let data = decompress(compressed)?;
    let null_pos = data.iter().position(|&b| b == 0).ok_or_else(|| anyhow!("Invalid tree object"))?;
    let content = &data[null_pos + 1..];
    let text = from_utf8(content)?;

    let subtasks: Vec<(PathBuf, String)> = text
        .par_lines()
        .filter_map(|line| {
            if line.trim().is_empty() {
                return None;
            }

            let mut parts = line.split_whitespace();
            let obj_type = parts.next()?;
            let obj_hash = parts.next()?;
            let name = line.split('\t').nth(1)?;

            let full_path = prefix.join(name);

            if obj_type == "blob" {
                out.insert(full_path, obj_hash.to_string());
                None
            } else if obj_type == "tree" {
                Some((full_path, obj_hash.to_string()))
            } else {
                None
            }
        })
        .collect();

    for (sub_prefix, sub_hash) in subtasks {
        read_tree_recursive(sub_prefix, &sub_hash, out)?;
    }

    Ok(())
}

pub fn restore_blob(path: &Path, hash: &str) -> Result<()> {
    let obj_path = Path::new(".ink")
        .join("objects")
        .join(&hash[..2])
        .join(&hash[2..]);

    let compressed = std::fs::read(&obj_path)?;
    let data = decompress(compressed)?;

    let null_pos = data
        .iter()
        .position(|&b| b == 0)
        .ok_or_else(|| anyhow!("Invalid blob object"))?;
    let content = &data[null_pos + 1..];

    if let Some(parent) = path.parent() {
        create_dir_all(parent)?;
    }

    let mut file = File::create(path)?;
    file.write_all(content)?;
    Ok(())
}

pub fn is_clean(path: &Path, index_hash: Option<&String>) -> Result<bool> {
    if !path.exists() {
        println!("{} missing on disk, index_hash = {:?}", path.display(), index_hash);
        return Ok(index_hash.is_none());
    }

    let data = std::fs::read(path)?;
    let header = format!("blob {}\0", data.len());
    let full = [header.as_bytes(), &data].concat();

    let working_hash = hash_object(&full)?;

    let clean = Some(&working_hash) == index_hash;

    if !clean {
        println!("--------------------------------------");
        println!("Uncommitted file detected: {}", path.display());
        println!("  Hash in working directory: {}", working_hash);
        println!("  Hash in index/commit:      {:?}", index_hash);
        println!("--------------------------------------");
    }

    Ok(clean)
}
