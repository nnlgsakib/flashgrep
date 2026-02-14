use crate::FlashgrepResult;
use std::path::PathBuf;

/// Manages the .flashgrep directory structure
#[derive(Debug, Clone)]
pub struct FlashgrepPaths {
    root: PathBuf,
}

impl FlashgrepPaths {
    /// Create a new FlashgrepPaths instance for a given repository root
    pub fn new(repo_root: &PathBuf) -> Self {
        Self {
            root: repo_root.join(crate::FLASHGREP_DIR),
        }
    }

    /// Get the root .flashgrep directory path
    pub fn root(&self) -> &PathBuf {
        &self.root
    }

    /// Get the path to the metadata database
    pub fn metadata_db(&self) -> PathBuf {
        self.root.join("metadata.db")
    }

    /// Get the path to the config file
    pub fn config_file(&self) -> PathBuf {
        self.root.join("config.json")
    }

    /// Get the path to the text index directory (Tantivy)
    pub fn text_index_dir(&self) -> PathBuf {
        self.root.join("text_index")
    }

    /// Get the path to the logs directory
    pub fn logs_dir(&self) -> PathBuf {
        self.root.join("logs")
    }

    /// Get the path to the vectors directory (for future use)
    pub fn vectors_dir(&self) -> PathBuf {
        self.root.join("vectors")
    }

    /// Get the path to the Unix socket (if using Unix sockets)
    pub fn socket_path(&self) -> PathBuf {
        self.root.join("mcp.sock")
    }

    /// Check if the flashgrep directory exists
    pub fn exists(&self) -> bool {
        self.root.exists()
    }

    /// Create the flashgrep directory structure
    pub fn create(&self) -> FlashgrepResult<()> {
        std::fs::create_dir_all(&self.root)?;
        std::fs::create_dir_all(self.text_index_dir())?;
        std::fs::create_dir_all(self.logs_dir())?;
        std::fs::create_dir_all(self.vectors_dir())?;
        Ok(())
    }

    /// Remove the entire flashgrep directory
    pub fn remove(&self) -> FlashgrepResult<()> {
        if self.root.exists() {
            std::fs::remove_dir_all(&self.root)?;
        }
        Ok(())
    }

    /// Get the size of the flashgrep directory in bytes
    pub fn size_bytes(&self) -> u64 {
        fn dir_size(path: &std::path::Path) -> u64 {
            let mut size = 0;
            if let Ok(entries) = std::fs::read_dir(path) {
                for entry in entries.flatten() {
                    let metadata = entry.metadata();
                    if let Ok(metadata) = metadata {
                        if metadata.is_file() {
                            size += metadata.len();
                        } else if metadata.is_dir() {
                            size += dir_size(&entry.path());
                        }
                    }
                }
            }
            size
        }
        dir_size(&self.root)
    }
}

/// Find the repository root by looking for .flashgrep directory or .git
pub fn find_repo_root(start_path: &PathBuf) -> Option<PathBuf> {
    let mut current = start_path.clone();

    loop {
        // Check for .flashgrep directory
        if current.join(crate::FLASHGREP_DIR).exists() {
            return Some(current);
        }

        // Check for .git directory as fallback
        if current.join(".git").exists() {
            return Some(current);
        }

        // Move up one directory
        match current.parent() {
            Some(parent) => current = parent.to_path_buf(),
            None => break,
        }
    }

    None
}

/// Get the current working directory as repository root, or use the provided path
pub fn get_repo_root(path: Option<&PathBuf>) -> FlashgrepResult<PathBuf> {
    match path {
        Some(p) => {
            let canonical = p.canonicalize()?;
            if canonical.is_dir() {
                Ok(canonical)
            } else {
                Err(crate::FlashgrepError::Config(format!(
                    "Path is not a directory: {}",
                    p.display()
                )))
            }
        }
        None => std::env::current_dir().map_err(Into::into),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_flashgrep_paths() {
        let temp_dir = TempDir::new().unwrap();
        let paths = FlashgrepPaths::new(&temp_dir.path().to_path_buf());

        assert!(paths.root().ends_with(".flashgrep"));
        assert!(paths.metadata_db().ends_with(".flashgrep/metadata.db"));
        assert!(paths.config_file().ends_with(".flashgrep/config.json"));
        assert!(paths.text_index_dir().ends_with(".flashgrep/text_index"));
    }

    #[test]
    fn test_create_and_remove() -> FlashgrepResult<()> {
        let temp_dir = TempDir::new()?;
        let paths = FlashgrepPaths::new(&temp_dir.path().to_path_buf());

        assert!(!paths.exists());

        paths.create()?;
        assert!(paths.exists());
        assert!(paths.text_index_dir().exists());
        assert!(paths.logs_dir().exists());

        paths.remove()?;
        assert!(!paths.exists());

        Ok(())
    }

    #[test]
    fn test_find_repo_root() -> FlashgrepResult<()> {
        let temp_dir = TempDir::new()?;
        let repo_root = temp_dir.path().to_path_buf();

        // Create .flashgrep directory
        std::fs::create_dir_all(repo_root.join(".flashgrep"))?;

        // Create a subdirectory
        let subdir = repo_root.join("src/nested");
        std::fs::create_dir_all(&subdir)?;

        // Should find the root from the subdirectory
        let found = find_repo_root(&subdir);
        assert!(found.is_some(), "Should find repo root");
        // Compare file names since canonicalization may differ on Windows
        assert_eq!(found.unwrap().file_name(), repo_root.file_name());

        Ok(())
    }
}
