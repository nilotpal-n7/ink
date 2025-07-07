use std::path::PathBuf;
use std::str::FromStr;
use anyhow::{ anyhow, Error, Result };

#[derive(Debug)]
pub enum AddMode {
    All,
    Update,
    Files(Vec<PathBuf>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HashAlgo {
    Sha256,
    Blake3,
}

impl Default for HashAlgo {
    fn default() -> Self {
        HashAlgo::Blake3
    }
}

impl ToString for HashAlgo {
    fn to_string(&self) -> String {
        match self {
            HashAlgo::Sha256 => "sha256".to_string(),
            HashAlgo::Blake3 => "blake3".to_string(),
        }
    }
}

impl FromStr for HashAlgo {
    type Err = Error;

    fn from_str(s: &str) -> Result<HashAlgo> {
        match s.to_ascii_lowercase().as_str() {
            "sha256" => Ok(HashAlgo::Sha256),
            "blake3" => Ok(HashAlgo::Blake3),
            _ => Err(anyhow!("Unsupported hash algorithm: {}", s)),
        }
    }
}
