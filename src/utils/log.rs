use std::fmt::Debug;
use std::fs::{read_to_string, File};
use std::io::{BufWriter, Write};
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};
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
#[derive(Debug, Clone)]
pub struct Log {
    pub entries: Vec<LogEntry>,
}

impl Log {
    #[allow(dead_code)]
    pub fn load() -> Result<Self> {
        let root = Path::new(".ink");
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

    pub fn add(&mut self, parent_hash: String, log_type: String, message: String ) -> Result<()> {
        let current_hash = read_current_commit()?;
        let timestamp = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
        let author = String::from("Nilotpal Gupta");

        let entry = LogEntry { parent_hash: parent_hash, current_hash: current_hash, author: author, time: timestamp.to_string(), log_type: log_type, message: message };
        self.entries.push(entry);

        Ok(())
    }

    pub fn save(&self) -> Result<()> {
        let current_branch = read_current_branch()?;
        let log_file = File::create(".ink/logs/HEAD")?;
        let log_ref_file = File::create(format!(".ink/logs/refs/heads/{}", current_branch))?;
        let mut log_writer = BufWriter::new(log_file);
        let mut log_ref_writer = BufWriter::new(log_ref_file);

        for entry in self.entries.clone() {
            let log = format!("{} {} {} {} +0000 {}: {}", entry.parent_hash, entry.current_hash, entry.author, entry.time, entry.log_type, entry.message);
            writeln!(log_writer, "{}", log)?;
            if entry.log_type == "commit" {
                writeln!(log_ref_writer, "{}", log)?;
            }
        }

        Ok(())
    }
}
