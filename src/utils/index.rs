use std::collections::HashMap;
use std::fs::{read, read_to_string, File};
use std::io::{BufWriter, Write};
use std::path::PathBuf;
use anyhow::Result;

use crate::utils::hash::hash_object;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct IndexEntry {
    pub path: PathBuf,
    pub hash: String,
}

#[derive(Default)]
pub struct Index {
    pub entries: HashMap<PathBuf, IndexEntry>,
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
        let data = read(path)?;
        let hash = hash_object(&data)?;

        let rel_path = path.strip_prefix(".").unwrap_or(path);

        index.add(IndexEntry {
            path: rel_path.to_path_buf(),
            hash,
        });
    }

    index.save()
}
