use std::collections::{BTreeMap, HashMap, HashSet};
use std::fs::{create_dir_all, read, write};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};
use anyhow::{ anyhow, Result };
use rayon::prelude::*;

use crate::utils::hash::hash_object;
use crate::utils::index::Index;
use crate::utils::zip::compress;

pub fn create_blob(path: PathBuf) -> Result<String> {
    let content = read(path)?;
    let header = format!("blob {}\0", content.len());
    let full = [header.as_bytes(), &content].concat();

    let hash = hash_object(&full)?;
    let obj_path = PathBuf::from(format!(".ink/objects/{}/{}", &hash[..2], &hash[2..]));

    if obj_path.exists() {
        return Ok(hash)
    }

    let compressed = compress(full)?;
    create_dir_all(obj_path.parent().unwrap())?;
    write(obj_path, compressed)?;

    Ok(hash)
}

/// Represents a single tree entry (mode, type, hash, filename)
#[derive(Debug, Clone)]
struct TreeEntry {
    mode: String,
    obj_type: String,
    hash: String,
    name: String,
}

pub fn create_tree() -> Result<String> {
    let index = Index::load()?;
    let mut dir_entries: HashMap<PathBuf, Vec<TreeEntry>> = HashMap::new();
    let mut all_dirs = HashSet::new();

    // Group file entries by their immediate parent directory
    for entry in index.entries.values() {
        let parent = entry.path.parent().unwrap_or(Path::new("")).to_path_buf();
        let name = entry.path.file_name().unwrap().to_string_lossy().to_string();

        dir_entries.entry(parent.clone()).or_default().push(TreeEntry {
            mode: "100644".into(),
            obj_type: "blob".into(),
            hash: entry.hash.clone(),
            name,
        });

        // Collect all ancestor directories
        let mut current = Some(entry.path.as_path());
        while let Some(dir) = current {
            if let Some(parent) = dir.parent() {
                all_dirs.insert(parent.to_path_buf());
                current = Some(parent);
            } else {
                break;
            }
        }
    }

    all_dirs.insert(PathBuf::from("")); // Ensure root dir is included

    // Group directories by depth
    let mut dirs_by_depth: BTreeMap<usize, Vec<PathBuf>> = BTreeMap::new();
    for dir in all_dirs {
        let depth = dir.components().count();
        dirs_by_depth.entry(depth).or_default().push(dir);
    }

    let mut tree_hashes: HashMap<PathBuf, String> = HashMap::new();

    for (_depth, dirs_at_depth) in dirs_by_depth.iter().rev() {
        // Extract entries first to avoid mutable borrow in parallel
        let mut entries_by_dir: HashMap<PathBuf, Vec<TreeEntry>> = HashMap::new();
        for dir in dirs_at_depth {
            if let Some(entries) = dir_entries.remove(dir) {
                entries_by_dir.insert(dir.clone(), entries);
            }
        }

        let results: Vec<(PathBuf, String)> = dirs_at_depth
            .par_iter()
            .map(|dir| {
                let mut entries = entries_by_dir.get(dir).cloned().unwrap_or_default();

                // Subtrees (already built)
                let sub_entries: Vec<TreeEntry> = tree_hashes
                    .iter()
                    .filter_map(|(subdir, hash)| {
                        if subdir.parent() == Some(dir) {
                            let name = subdir.file_name()?.to_string_lossy().to_string();
                            Some(TreeEntry {
                                mode: "040000".into(),
                                obj_type: "tree".into(),
                                hash: hash.clone(),
                                name,
                            })
                        } else {
                            None
                        }
                    })
                    .collect();

                entries.extend(sub_entries);
                entries.sort_by(|a, b| a.name.cmp(&b.name));

                // Encode tree object
                let mut tree_bytes = Vec::new();
                for entry in &entries {
                    let line = format!("{} {} {}\t{}\n", entry.mode, entry.obj_type, entry.hash, entry.name);
                    tree_bytes.extend_from_slice(line.as_bytes());
                }

                let header = format!("tree {}\0", tree_bytes.len());
                let full = [header.as_bytes(), &tree_bytes].concat();
                let hash = hash_object(&full).expect("hash failed");

                let obj_path = PathBuf::from(format!(".ink/objects/{}/{}", &hash[..2], &hash[2..]));
                if !obj_path.exists() {
                    let compressed = compress(full).expect("compress failed");
                    create_dir_all(obj_path.parent().unwrap()).expect("mkdir failed");
                    write(obj_path, compressed).expect("write failed");
                }

                (dir.clone(), hash)
            })
            .collect();

        for (dir, hash) in results {
            tree_hashes.insert(dir, hash);
        }
    }

    tree_hashes
        .get(&PathBuf::from(""))
        .cloned()
        .ok_or_else(|| anyhow!("No root tree created"))
}

/// Creates a commit object from a tree hash and returns the commit hash.
/// If parent is Some(hash), sets it as the commit's parent.
pub fn create_commit(tree: &str, parent: Option<&str>, message: &str, author: &str) -> Result<String> {
    let timestamp = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();

    let mut content = format!("tree {}\n", tree);
    if let Some(parent_hash) = parent {
        content += &format!("parent {}\n", parent_hash);
    }
    content += &format!(
        "author {} {} +0000\ncommitter {} {} +0000\n\n{}\n",
        author, timestamp, author, timestamp, message
    );

    let header = format!("commit {}\0", content.len());
    let full = [header.as_bytes(), content.as_bytes()].concat();
    let hash = hash_object(&full)?;

    let obj_path = PathBuf::from(format!(".ink/objects/{}/{}", &hash[..2], &hash[2..]));
    if !obj_path.exists() {
        let compressed = compress(full)?;
        create_dir_all(obj_path.parent().unwrap())?;
        write(&obj_path, compressed)?;
    }

    Ok(hash)
}
