use std::fs::{read_to_string, File};
use std::io::{BufWriter, Write};
use std::path::Path;
use anyhow::Result;
use chrono::{DateTime, Local, Offset};

use crate::commands::branch::read_current_branch;
use crate::commands::commit::read_current_commit;

#[derive(Debug, Clone)]
pub struct LogEntry {
    pub parent_hash: String,
    pub current_hash: String,
    pub author_name: String,
    pub author_email: String,
    pub time: String,
    pub timezone: String,
    pub log_type: String,
    pub message: String,
}

#[derive(Debug, Clone)]
pub struct Log {
    pub entries: Vec<LogEntry>,
}

impl Log {
    pub fn load(path: &Path) -> Result<Self> {
        let mut entries = Vec::new();

        if path.exists() {
            for line in read_to_string(path)?.lines() {
                let parts: Vec<&str> = line.splitn(6, ' ').collect();
                if parts.len() < 6 { continue; }

                let parent_hash = parts[0].to_string();
                let current_hash = parts[1].to_string();
                let author_raw = parts[2].to_string();
                let email_raw = parts[3].trim_matches(&['<', '>'][..]).to_string();
                let time = parts[4].to_string();
                let remaining: Vec<&str> = parts[5].splitn(2, '\t').collect();
                let timezone = remaining.get(0).unwrap_or(&"+0000").to_string();
                let type_msg: Vec<&str> = remaining.get(1).unwrap_or(&"").splitn(2, ": ").collect();
                let log_type = type_msg.get(0).unwrap_or(&"").to_string();
                let message = type_msg.get(1).unwrap_or(&"").to_string();

                entries.push(LogEntry {
                    parent_hash,
                    current_hash,
                    author_name: author_raw,
                    author_email: email_raw,
                    time,
                    timezone,
                    log_type,
                    message,
                });
            }
        }

        Ok(Log { entries })
    }

    pub fn add(&mut self, parent_hash: String, current_hash: String, log_type: String, message: String) -> Result<()> {
        let (author_name, author_email) = read_author()?;

        let now: DateTime<Local> = Local::now();
        let timestamp = now.timestamp();
        let offset = now.offset().fix().local_minus_utc();
        let offset_str = format!("{:+03}{:02}", offset / 3600, (offset.abs() % 3600) / 60);

        let entry = LogEntry {
            parent_hash,
            current_hash,
            author_name,
            author_email,
            time: timestamp.to_string(),
            timezone: offset_str,
            log_type,
            message,
        };

        self.entries.push(entry);
        Ok(())
    }

    pub fn save(&self) -> Result<()> {
        let current_branch = read_current_branch()?;
        std::fs::create_dir_all(".ink/logs/refs/heads")?;

        let head_path = Path::new(".ink/logs/HEAD");
        let path = format!(".ink/logs/refs/heads/{}", current_branch);
        let branch_path = Path::new(&path);

        let head_file = File::create(head_path)?;
        let branch_file = File::create(branch_path)?;

        let mut head_writer = BufWriter::new(head_file);
        let mut branch_writer = BufWriter::new(branch_file);

        for entry in &self.entries {
            let log_line = format!(
                "{} {} {} <{}> {} {}	{}: {}",
                entry.parent_hash,
                entry.current_hash,
                entry.author_name,
                entry.author_email,
                entry.time,
                entry.timezone,
                entry.log_type,
                entry.message
            );

            writeln!(head_writer, "{}", log_line)?;

            if entry.log_type == "commit" || entry.log_type == "branch" {
                writeln!(branch_writer, "{}", log_line)?;
            }
        }

        Ok(())
    }

    pub fn log_checkout(&mut self, previous_hash: String, new_hash: String, message: String) -> Result<()> {
        self.add(previous_hash, new_hash, "checkout".to_string(), message)
    }

    pub fn log_branch(&mut self, parent_hash: String, new_hash: String, message: String) -> Result<()> {
        self.add(parent_hash, new_hash, "branch".to_string(), message)
    }

    pub fn log_commit(&mut self, parent_hash: String, message: String) -> Result<()> {
        let current_hash = read_current_commit()?;
        self.add(parent_hash, current_hash, "commit".to_string(), message)
    }
}

fn read_author() -> Result<(String, String)> {
    let config_path = Path::new(".ink/.inkconfig");
    let mut name = "Nilotpal Gupta".to_string();
    let mut email = "nilotpalgupta0701@gmail.com".to_string();

    if config_path.exists() {
        for line in read_to_string(config_path)?.lines() {
            if let Some(v) = line.strip_prefix("author=") {
                name = v.trim().to_string();
            } else if let Some(v) = line.strip_prefix("email=") {
                email = v.trim().to_string();
            }
        }
    }

    Ok((name, email))
}
