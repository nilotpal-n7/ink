use std::fs::{read_to_string, write};
use std::str::FromStr;
use std::path::Path;
use anyhow::Result;
use sha2::{Digest, Sha256};
use blake3::Hasher;
use std::io::Write;

use crate::utils::enums::HashAlgo;

pub fn hash_object(data: &[u8]) -> Result<String> {
    let algo = load_hash_algo()?;
    
    let hash = match algo {
        HashAlgo::Sha256 => {
            let mut hasher = Sha256::new();
            hasher.update(data);
            format!("{:x}", hasher.finalize())
        }
        HashAlgo::Blake3 => {
            let mut hasher = Hasher::new();
            hasher.update(data);
            hasher.finalize().to_hex().to_string()
        }
    };
    
    Ok(hash)
}

pub fn load_hash_algo() -> Result<HashAlgo> {
    let path = Path::new(".ink/config");
    let contents = read_to_string(path)?;

    for line in contents.lines() {
        if let Some((key, value)) = line.split_once('=') {
            if key.trim() == "hash" {
                return Ok(HashAlgo::from_str(value.trim())?);
            }
        }
    }
    Ok(HashAlgo::default())
}

pub fn save_hash_algo(algo: HashAlgo) -> Result<()> {
    let path = Path::new(".ink/config");

    let mut lines = if path.exists() {
        read_to_string(path)?
            .lines()
            .map(|l| l.to_string())
            .collect::<Vec<_>>()
    } else {
        Vec::new()
    };

    let mut found = false;
    for line in lines.iter_mut() {
        if line.trim_start().starts_with("hash=") {
            *line = format!("hash={}", algo.to_string());
            found = true;
            break;
        }
    }

    if !found {
        lines.push(format!("hash={}", algo.to_string()));
    }

    let output = lines.join("\n") + "\n";
    write(path, output)?; // .as_bytes()

    Ok(())
}
