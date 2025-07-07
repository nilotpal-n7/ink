use std::path::Path;

pub fn is_in_ink(path: &Path) -> bool {
    path.components().any(|c| c.as_os_str() == ".ink")
}
