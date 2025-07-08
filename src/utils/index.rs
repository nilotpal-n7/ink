use std::collections::HashMap;
use std::fs::{create_dir_all, read_to_string, write, File, read};
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};
use anyhow::Result;
use bincode::config::standard;
use bincode::serde::{decode_from_slice, encode_to_vec};
use serde::{Serialize, Deserialize};

use crate::commands::branch::read_current_branch;
use crate::utils::object::create_blob;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct IndexEntry {
    pub path: PathBuf,
    pub hash: String,
}

#[derive(Default, Serialize, Deserialize)]
pub struct Index {
    pub entries: HashMap<PathBuf, IndexEntry>,
}

impl Index {
    pub fn save_for_branch(&self, branch: &str) -> Result<()> {
        let dir = Path::new(".ink/refs/INDEXES");
        create_dir_all(&dir)?;
        let path = dir.join(branch);

        let encoded = encode_to_vec(self, standard())?;
        write(path, encoded)?;
        Ok(())
    }

    pub fn load_for_branch(branch: &str) -> Result<Self> {
        let path = Path::new(".ink/refs/INDEXES").join(branch);
        let bytes = read(path)?;

        let (index, _): (Index, _) = decode_from_slice(&bytes, standard())?;
        Ok(index)
    }

    pub fn exists_for_branch(branch: &str) -> bool {
        Path::new(".ink/refs/INDEXES").join(branch).exists()
    }
}

impl Index {
    pub fn load() -> Result<Self> {
        let path = PathBuf::from(".ink/index");
        let mut entries = HashMap::new();

        if path.exists() {
            for line in read_to_string(&path)?.lines() {
                let mut parts = line.splitn(2, ' ');
                let hash = parts.next().unwrap_or("").to_string();
                let path_str = parts.next().unwrap_or("");
                let path = PathBuf::from(path_str);
                entries.insert(path.clone(), IndexEntry { path, hash });
            }
        }

        Ok(Index { entries })
    }

    pub fn save(&self) -> Result<()> {
        let file = File::create(".ink/index")?;
        let mut writer = BufWriter::new(file);

        for entry in self.entries.values() {
            writeln!(writer, "{} {}", entry.hash, entry.path.display())?;
        }

        Ok(())
    }

    pub fn add(&mut self, entry: IndexEntry) {
        self.entries.insert(entry.path.clone(), entry);
    }

    pub fn remove(&mut self, path: &PathBuf) {
        self.entries.remove(path);
    }

    #[allow(dead_code)]
    pub fn get(&self, path: &PathBuf) -> Option<&IndexEntry> {
        self.entries.get(path)
    }

    pub fn tracked_files(&self) -> Vec<std::path::PathBuf> {
        self.entries.values().map(|entry| entry.path.clone()).collect()
    }
}

pub fn add_files_to_index(files: &[PathBuf]) -> Result<()> {
    let mut index = Index::load()?;

    for path in files {
        let hash = create_blob(path.clone())?;
        let rel_path = path.strip_prefix(".").unwrap_or(path);

        index.add(IndexEntry {
            path: rel_path.to_path_buf(),
            hash,
        });
    }

    index.save()
}

pub fn save_index_for_current_branch() -> Result<()> {
    let index = Index::load()?;
    let branch = read_current_branch()?;
    index.save_for_branch(&branch)
}
