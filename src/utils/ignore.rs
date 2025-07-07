use std::fs;
use std::path::{Path, PathBuf};

pub fn is_ignored(path: &Path) -> bool {
    let ignore_path = PathBuf::from(".inkignore");
    if !ignore_path.exists() {
        return false;
    }

    if let Ok(content) = fs::read_to_string(ignore_path) {
        for pattern in content.lines().filter(|l| !l.trim().is_empty()) {
            if let Ok(glob) = glob::Pattern::new(pattern) {
                if glob.matches_path(path) {
                    return true;
                }
            }
        }
    }

    false
}
