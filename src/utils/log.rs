use std::fs::{read_to_string, OpenOptions};
use std::io::Write;
use std::path::Path;
use anyhow::Result;
use chrono::{DateTime, Local, Offset};

use crate::commands::branch::read_current_branch;
use crate::commands::commit::read_current_commit;

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

pub fn log_action(parent_hash: String, current_hash: String, log_type: &str, message: &str) -> Result<()> {
    let (author_name, author_email) = read_author()?;

    let now: DateTime<Local> = Local::now();
    let timestamp = now.timestamp();
    let offset = now.offset().fix().local_minus_utc();
    let offset_str = format!("{:+03}{:02}", offset / 3600, (offset.abs() % 3600) / 60);

    let log_line = format!(
        "{} {} {} <{}> {} {}	{}: {}\n",
        parent_hash,
        current_hash,
        author_name,
        author_email,
        timestamp,
        offset_str,
        log_type,
        message
    );

    std::fs::create_dir_all(".ink/logs/refs/heads")?;

    let current_branch = read_current_branch()?;
    let head_path = Path::new(".ink/logs/HEAD");
    let path = format!(".ink/logs/refs/heads/{}", current_branch);
    let branch_path = Path::new(&path);

    let mut head_file = OpenOptions::new().create(true).append(true).open(head_path)?;
    head_file.write_all(log_line.as_bytes())?;

    if log_type == "commit" || log_type == "branch" {
        let mut branch_file = OpenOptions::new().create(true).append(true).open(branch_path)?;
        branch_file.write_all(log_line.as_bytes())?;
    }

    Ok(())
}

pub fn log_commit(message: &str) -> Result<()> {
    let parent_hash = read_current_commit()?;
    log_action(parent_hash.clone(), parent_hash, "commit", message)
}

pub fn log_branch(branch_name: &str) -> Result<()> {
    let current_hash = read_current_commit()?;
    log_action(current_hash.clone(), current_hash, "branch", &format!("created branch '{}'", branch_name))
}

pub fn log_checkout(from: String, to: String, name: &str) -> Result<()> {
    log_action(from, to, "checkout", &format!("switched to '{}'", name))
}
