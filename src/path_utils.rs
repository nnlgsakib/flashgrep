use std::path::{Path, PathBuf};

pub fn normalize_path_for_matching(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

pub fn normalize_glob_pattern(pattern: &str) -> String {
    pattern.trim().replace('\\', "/")
}

pub fn resolve_path(repo_root: &Path, path: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        repo_root.join(path)
    }
}
