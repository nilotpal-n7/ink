use std::fmt::Debug;
use std::fs::{read_to_string, File};
use std::io::{BufWriter, Write};
use std::path::Path;
use anyhow::Result;

use crate::commands::branch::read_current_branch;
use crate::commands::commit::read_current_commit;

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct LogEntry {
    pub parent_hash: String,
    pub current_hash: String,
    pub author: String,
    pub time: String,
    pub log_type: String,
    pub message: String,
}

#[allow(dead_code)]
#[derive(Debug)]
pub struct Log {
    pub entries: Vec<LogEntry>,
}

impl Log {
    pub fn add_log(parent_hash: &str, log_type: &str, message: &str) -> Result<()> {
        let root = Path::new(".");
        let log_head = root.join("logs").join("HEAD");
        let current_branch = read_current_branch()?;
        let log_ref_head = root.join("logs").join("refs").join(current_branch);
        let log_file: File;
        let log_ref_file: File;

        if log_head.exists() {
            log_file = File::open(log_head)?;
        } else {
            log_file = File::create(log_head)?;
        }

        if log_ref_head.exists() {
            log_ref_file = File::open(log_ref_head)?;
        } else {
            log_ref_file = File::create(log_ref_head)?;
        }

        let mut log_writer = BufWriter::new(log_file);
        let mut log_ref_writer = BufWriter::new(log_ref_file);

        let current_hash = read_current_commit()?;
        let author = "Nilotpal Gupta";
        let time = "12:00 +0530"; //SystemTime::now();
        let log = format!("{} {} {} {} {}: {}",parent_hash, current_hash, author, time, log_type, message);

        writeln!(log_writer, "{}", log)?;
        if log_type == "commit" {
            writeln!(log_ref_writer, "{}", log)?;
        }
        
        Ok(())
    }

    pub fn read_log() -> Result<Self> {
        let root = Path::new(".");
        let log_head = root.join("logs").join("HEAD");
        let mut entries = Vec::new();

        if log_head.exists() {
            for line in read_to_string(log_head)?.lines() {
                let mut parts = line.split(" ");
                let parent_hash = parts.next().unwrap_or("").to_string();
                let current_hash = parts.next().unwrap_or("").to_string();
                let author = parts.next().unwrap_or("").to_string();
                let time = parts.next().unwrap_or("").to_string();
                let log_type = parts.next().unwrap_or("").to_string();
                let message = parts.next().unwrap_or("").to_string();
                entries.push(LogEntry {parent_hash, current_hash, author, time, log_type, message});
            }
        }
        
        Ok(Log {entries})
    }
}
