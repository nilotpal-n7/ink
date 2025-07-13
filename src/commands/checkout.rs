use std::collections::{HashMap, HashSet};
use std::fs::{create_dir_all, read, remove_file, File};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::str::from_utf8;

use anyhow::{anyhow, Result};
use dashmap::DashMap;
use rayon::prelude::*;

use crate::commands;
use crate::commands::branch::{read_current_branch, update_current_branch};
use crate::commands::commit::{get_branch_commit, read_current_commit, read_tree_of_commit};
use crate::utils::dir::remove_empty_parents_up_to;
use crate::utils::hash::hash_object;
use crate::utils::index::{Index, IndexEntry};
use crate::utils::log::Log;
use crate::utils::zip::decompress;
use crate::utils::ignore::is_ignored;

pub fn run(b: bool, force: bool, name: String) -> Result<()> {
    // If -b flag set, create the branch now
    if b {
        commands::branch::run(Some(name.clone()))?;
    }

    let current_branch = read_current_branch()?;
    let current_commit = read_current_commit()?;
    let current_index = Index::load()?;
    current_index.save_for_branch(&current_branch)?;

    // Target commit and branch info
    let target_commit = get_branch_commit(&name)?;

    let current_tree = if &current_commit != "0000000000000000000000000000000000000000000000000000000000000000" {
        get_tree_entries(&read_tree_of_commit(&current_commit)?)?
    } else {
        HashMap::new()
    };

    let target_tree = get_tree_entries(&read_tree_of_commit(&target_commit)?)?;

    // Current index as map
    let index_map: HashMap<_, _> = current_index
        .entries
        .par_iter()
        .map(|(_, entry)| (entry.path.clone(), entry.hash.clone()))
        .collect();

    // Determine if this is a new branch (never checked out before)
    let is_new_branch = !Index::exists_for_branch(&name);

    // Union of all paths involved
    let all_paths: HashSet<_> = index_map
        .keys()
        .chain(current_tree.keys())
        .chain(target_tree.keys())
        .cloned()
        .collect();

    // Only check uncommitted changes if not --force and not a new branch
    if !force && !is_new_branch {
        all_paths.par_iter().try_for_each(|path| {
            if is_ignored(path) {
                return Ok(());
            }

            let index_hash = index_map.get(path);
            let current_hash = current_tree.get(path);
            let target_hash = target_tree.get(path);

            let clean = is_clean(path, index_hash, current_hash, target_hash)?;
            if !clean {
                return Err(anyhow!(
                    "Uncommitted changes in '{}', please commit or stash them first.",
                    path.display()
                ));
            }

            Ok(())
        })?;
    }

    // Proceed to clean/delete or restore
    all_paths.par_iter().try_for_each(|path| -> Result<()> {
        if is_ignored(path) {
            return Ok(());
        }

        let current_hash = current_tree.get(path);
        let target_hash = target_tree.get(path);

        match (current_hash, target_hash) {
            (Some(_), None) => {
                if path.exists() {
                    remove_file(path).ok();
                }
                remove_empty_parents_up_to(path, Path::new(".")).ok();
                Ok(())
            }
            (_, Some(tgt)) => {
                restore_blob(path, tgt).ok();
                Ok(())
            }
            _ => Ok(()),
        }
    })?;

    update_current_branch(&name)?;

    // Load or create new index
    let new_index = if is_new_branch {
        let entries: Vec<_> = target_tree
            .par_iter()
            .map(|(path, hash)| IndexEntry {
                path: path.clone(),
                hash: hash.clone(),
            })
            .collect();

        let mut idx = Index::default();
        for entry in entries {
            idx.add(entry);
        }
        idx
    } else {
        Index::load_for_branch(&name)?
    };

    new_index.save()?;
    println!("Switched to branch '{}'", name);

    let mut log = Log::load(Path::new(".ink/logs/HEAD"))?;
    log.log_checkout(current_commit.clone(), target_commit.clone(), format!("switched from '{}' to '{}'", current_branch, name))?;
    log.save()?;

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
        .lines()
        .filter_map(|line| {
            let (meta, name) = line.split_once('\t')?;
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
    let null_pos = data.iter().position(|&b| b == 0).ok_or_else(|| anyhow!("Invalid blob object"))?;
    let content = &data[(null_pos + 1)..];

    if let Some(parent) = path.parent() {
        create_dir_all(parent)?;
    }

    let mut file = File::create(path)?;
    file.write_all(content)?;
    Ok(())
}

pub fn is_clean(
    path: &Path,
    index_hash: Option<&String>,
    current_hash: Option<&String>,
    target_hash: Option<&String>,
) -> Result<bool> {
    if !path.exists() {
        return Ok(index_hash.is_none() && current_hash.is_none());
    }

    let data = read(path)?;
    let header = format!("blob {}\0", data.len());
    let full = [header.as_bytes(), &data].concat();
    let working_hash = hash_object(&full)?;

    match target_hash {
        #[allow(unused_variables)]
        Some(tgt) => {
            if Some(&working_hash) == index_hash {
                if index_hash != current_hash {
                    return Err(anyhow!(
                        "Uncommitted staged changes in '{}'", path.display()
                    ));
                } else {
                    return Ok(true);
                }
            }

            if Some(&working_hash) == current_hash {
                return Err(anyhow!(
                    "Uncommitted unstaged changes in '{}'", path.display()
                ));
            }

            Ok(false)
        }

        None => {
            match (index_hash, current_hash) {
                (Some(index), Some(current)) if index == current => Ok(true),
                (Some(_), _) => Err(anyhow!(
                    "Uncommitted staged file '{}' would be lost.", path.display()
                )),
                _ => Ok(false),
            }
        }
    }
}
