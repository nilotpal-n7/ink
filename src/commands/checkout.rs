use std::collections::{HashMap, HashSet};
use std::fs::{create_dir_all, read, remove_file, File};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::str::from_utf8;

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
    let current_commit = read_current_commit().ok();

    let target_commit = if b {
        let base_commit = current_commit.clone();
        commands::branch::run(Some(name.clone()))?;
        base_commit
    } else {
        Some(get_branch_commit(&name)?)
    };

    let target_commit = target_commit.ok_or_else(|| anyhow!("Target branch has no commit"))?;
    let index = Index::load()?;
    let index_map: HashMap<_, _> = index
        .entries
        .par_iter()
        .map(|(_, entry)| (entry.path.clone(), entry.hash.clone()))
        .collect();

    let prev_commit = current_commit.clone();
    let current_tree = if let Some(commit) = &prev_commit {
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

        let clean = is_clean(path, index_hash.or(current_hash), current_hash)?;

        match (clean, current_hash, target_hash) {
            (false, _, _) => Err(anyhow!(
                "Uncommitted changes in '{}', please commit or stash them first.",
                path.display()
            )),
            (true, Some(_), None) => {
                if path.exists() {
                    remove_file(path)?;
                }
                Ok(())
            }
            (true, _, Some(target_hash)) => restore_blob(path, target_hash),
            _ => Ok(()),
        }
    })?;

    update_current_branch(&name)?;

    let mut new_index = Index::default();

    if b {
        new_index.save_for_branch(&name)?;
    } else if Index::exists_for_branch(&name) {
        new_index = Index::load_for_branch(&name)?;
    } else {
        let entries: Vec<IndexEntry> = target_tree
            .par_iter()
            .map(|(path, hash)| IndexEntry {
                path: path.clone(),
                hash: hash.clone(),
            })
            .collect();

        for entry in entries {
            new_index.add(entry);
        }
    }

    new_index.save()?;

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
    let content = &data[(null_pos + 1)..];
    let text = from_utf8(content)?;

    let subtasks: Vec<(PathBuf, String)> = text
        .par_lines()
        .filter_map(|line| {
            if line.trim().is_empty() {
                return None;
            }

            let tab_split: Vec<&str> = line.split('\t').collect();
            if tab_split.len() != 2 {
                return None;
            }
            let meta = tab_split[0];
            let name = tab_split[1];
            let mut parts = meta.split_whitespace();
            let _mode = parts.next()?;
            let obj_type = parts.next()?;
            let obj_hash = parts.next()?;

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

    let compressed = read(&obj_path)?;
    let data = decompress(compressed)?;

    let null_pos = data
        .iter()
        .position(|&b| b == 0)
        .ok_or_else(|| anyhow!("Invalid blob object"))?;
    let content = &data[(null_pos + 1)..];

    if let Some(parent) = path.parent() {
        create_dir_all(parent)?;
    }

    let mut file = File::create(path)?;
    file.write_all(content)?;
    Ok(())
}

pub fn is_clean(path: &Path, ref_hash: Option<&String>, fallback_commit_hash: Option<&String>) -> Result<bool> {
    if !path.exists() {
        return Ok(ref_hash.is_none() && fallback_commit_hash.is_none());
    }

    let data = read(path)?;
    let header = format!("blob {}\0", data.len());
    let full = [header.as_bytes(), &data].concat();
    let working_hash = hash_object(&full)?;

    let clean = Some(&working_hash) == ref_hash || Some(&working_hash) == fallback_commit_hash;

    if !clean {
        println!("File {} is dirty:", path.display());
        println!("  Working:      {}", working_hash);
        println!("  Index/Ref:    {:?}", ref_hash);
        println!("  From commit:  {:?}", fallback_commit_hash);
    }

    Ok(clean)
}
