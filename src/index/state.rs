use crate::FlashgrepResult;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};
use tracing::{debug, info, warn};

/// Current version of the index state format
pub const INDEX_STATE_VERSION: u32 = 1;

/// Represents the persisted state of the file index
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexState {
    /// Version of the index state format
    pub version: u32,

    /// Timestamp when the index was last updated
    pub last_updated: i64,

    /// Map of file paths to their metadata
    pub files: HashMap<PathBuf, FileMetadata>,
}

/// Metadata for a single file in the index
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FileMetadata {
    /// File size in bytes
    pub size: u64,

    /// Last modification time (Unix timestamp)
    pub mtime: i64,

    /// SHA-256 hash of file content (first 8KB for performance)
    pub content_hash: String,
}

impl IndexState {
    /// Create a new empty index state
    pub fn new() -> Self {
        Self {
            version: INDEX_STATE_VERSION,
            last_updated: chrono::Utc::now().timestamp(),
            files: HashMap::new(),
        }
    }

    /// Load index state from a file
    pub fn load(path: &Path) -> FlashgrepResult<Self> {
        if !path.exists() {
            debug!("Index state file not found at {:?}, creating new", path);
            return Ok(Self::new());
        }

        let content = std::fs::read_to_string(path)?;
        let state: IndexState = serde_json::from_str(&content).map_err(|e| {
            warn!("Failed to parse index state, creating new: {}", e);
            crate::FlashgrepError::Index(format!("Invalid index state: {}", e))
        })?;

        // Check version compatibility
        if state.version > INDEX_STATE_VERSION {
            warn!(
                "Index state version {} is newer than supported ({}), may be incompatible",
                state.version, INDEX_STATE_VERSION
            );
        }

        info!("Loaded index state with {} files", state.files.len());
        Ok(state)
    }

    /// Save index state to a file atomically
    pub fn save(&self, path: &Path) -> FlashgrepResult<()> {
        // Create parent directory if needed
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        // Write to temporary file first for atomic operation
        let temp_path = path.with_extension("tmp");
        let content = serde_json::to_string_pretty(self)?;
        std::fs::write(&temp_path, content)?;

        // Atomic rename
        std::fs::rename(&temp_path, path)?;

        debug!("Saved index state with {} files", self.files.len());
        Ok(())
    }

    /// Update or add a file to the index
    pub fn update_file(&mut self, path: PathBuf, metadata: FileMetadata) {
        self.files.insert(path, metadata);
        self.last_updated = chrono::Utc::now().timestamp();
    }

    /// Remove a file from the index
    pub fn remove_file(&mut self, path: &Path) {
        self.files.remove(path);
        self.last_updated = chrono::Utc::now().timestamp();
    }

    /// Check if a file exists in the index
    pub fn has_file(&self, path: &Path) -> bool {
        self.files.contains_key(path)
    }

    /// Get metadata for a file
    pub fn get_file(&self, path: &Path) -> Option<&FileMetadata> {
        self.files.get(path)
    }

    /// Compare current file metadata with stored metadata
    pub fn is_file_changed(&self, path: &Path, metadata: &FileMetadata) -> bool {
        match self.files.get(path) {
            Some(stored) => stored != metadata,
            None => true,
        }
    }

    /// Get all file paths in the index
    pub fn get_all_paths(&self) -> Vec<PathBuf> {
        self.files.keys().cloned().collect()
    }

    /// Compact the index by removing entries that no longer exist on disk
    pub fn compact(&mut self, root: &Path) -> usize {
        let to_remove: Vec<PathBuf> = self
            .files
            .keys()
            .filter(|path| {
                let full_path = root.join(path);
                !full_path.exists()
            })
            .cloned()
            .collect();

        let removed_count = to_remove.len();
        for path in to_remove {
            self.files.remove(&path);
        }

        if removed_count > 0 {
            self.last_updated = chrono::Utc::now().timestamp();
            info!("Compacted index: removed {} stale entries", removed_count);
        }

        removed_count
    }

    /// Get the number of files in the index
    pub fn len(&self) -> usize {
        self.files.len()
    }

    /// Check if the index is empty
    pub fn is_empty(&self) -> bool {
        self.files.is_empty()
    }
}

impl Default for IndexState {
    fn default() -> Self {
        Self::new()
    }
}

/// Thread-safe wrapper for IndexState
pub struct ThreadSafeIndexState {
    inner: Arc<RwLock<IndexState>>,
}

impl ThreadSafeIndexState {
    /// Create a new thread-safe index state
    pub fn new() -> Self {
        Self {
            inner: Arc::new(RwLock::new(IndexState::new())),
        }
    }

    /// Load from a file
    pub fn load(path: &Path) -> FlashgrepResult<Self> {
        let state = IndexState::load(path)?;
        Ok(Self {
            inner: Arc::new(RwLock::new(state)),
        })
    }

    /// Save to a file
    pub fn save(&self, path: &Path) -> FlashgrepResult<()> {
        let state = self
            .inner
            .read()
            .map_err(|_| crate::FlashgrepError::Index("Failed to acquire read lock".to_string()))?;
        state.save(path)
    }

    /// Update a file in the index
    pub fn update_file(&self, path: PathBuf, metadata: FileMetadata) -> FlashgrepResult<()> {
        let mut state = self.inner.write().map_err(|_| {
            crate::FlashgrepError::Index("Failed to acquire write lock".to_string())
        })?;
        state.update_file(path, metadata);
        Ok(())
    }

    /// Remove a file from the index
    pub fn remove_file(&self, path: &Path) -> FlashgrepResult<()> {
        let mut state = self.inner.write().map_err(|_| {
            crate::FlashgrepError::Index("Failed to acquire write lock".to_string())
        })?;
        state.remove_file(path);
        Ok(())
    }

    /// Check if a file has changed
    pub fn is_file_changed(&self, path: &Path, metadata: &FileMetadata) -> FlashgrepResult<bool> {
        let state = self
            .inner
            .read()
            .map_err(|_| crate::FlashgrepError::Index("Failed to acquire read lock".to_string()))?;
        Ok(state.is_file_changed(path, metadata))
    }

    /// Check if a file exists in the index
    pub fn has_file(&self, path: &Path) -> FlashgrepResult<bool> {
        let state = self
            .inner
            .read()
            .map_err(|_| crate::FlashgrepError::Index("Failed to acquire read lock".to_string()))?;
        Ok(state.has_file(path))
    }

    /// Get all file paths
    pub fn get_all_paths(&self) -> FlashgrepResult<Vec<PathBuf>> {
        let state = self
            .inner
            .read()
            .map_err(|_| crate::FlashgrepError::Index("Failed to acquire read lock".to_string()))?;
        Ok(state.get_all_paths())
    }

    /// Compact the index
    pub fn compact(&self, root: &Path) -> FlashgrepResult<usize> {
        let mut state = self.inner.write().map_err(|_| {
            crate::FlashgrepError::Index("Failed to acquire write lock".to_string())
        })?;
        Ok(state.compact(root))
    }

    /// Get the number of files
    pub fn len(&self) -> FlashgrepResult<usize> {
        let state = self
            .inner
            .read()
            .map_err(|_| crate::FlashgrepError::Index("Failed to acquire read lock".to_string()))?;
        Ok(state.len())
    }
}

impl Default for ThreadSafeIndexState {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for ThreadSafeIndexState {
    fn clone(&self) -> Self {
        Self {
            inner: Arc::clone(&self.inner),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_index_state_new() {
        let state = IndexState::new();
        assert_eq!(state.version, INDEX_STATE_VERSION);
        assert!(state.files.is_empty());
    }

    #[test]
    fn test_index_state_update_and_get() {
        let mut state = IndexState::new();
        let path = PathBuf::from("test.rs");
        let metadata = FileMetadata {
            size: 100,
            mtime: 1234567890,
            content_hash: "abc123".to_string(),
        };

        state.update_file(path.clone(), metadata.clone());
        assert!(state.has_file(&path));
        assert_eq!(state.get_file(&path), Some(&metadata));
    }

    #[test]
    fn test_index_state_save_and_load() -> FlashgrepResult<()> {
        let temp_dir = TempDir::new()?;
        let path = temp_dir.path().join("index-state.json");

        let mut state = IndexState::new();
        state.update_file(
            PathBuf::from("test.rs"),
            FileMetadata {
                size: 100,
                mtime: 1234567890,
                content_hash: "abc123".to_string(),
            },
        );

        state.save(&path)?;
        let loaded = IndexState::load(&path)?;

        assert_eq!(loaded.len(), 1);
        assert!(loaded.has_file(&PathBuf::from("test.rs")));

        Ok(())
    }

    #[test]
    fn test_index_state_compact() -> FlashgrepResult<()> {
        let temp_dir = TempDir::new()?;
        let root = temp_dir.path();

        // Create a test file
        std::fs::write(root.join("exists.rs"), "fn main() {}")?;

        let mut state = IndexState::new();
        state.update_file(
            PathBuf::from("exists.rs"),
            FileMetadata {
                size: 100,
                mtime: 1234567890,
                content_hash: "abc123".to_string(),
            },
        );
        state.update_file(
            PathBuf::from("deleted.rs"),
            FileMetadata {
                size: 100,
                mtime: 1234567890,
                content_hash: "def456".to_string(),
            },
        );

        assert_eq!(state.len(), 2);

        let removed = state.compact(root);
        assert_eq!(removed, 1);
        assert_eq!(state.len(), 1);
        assert!(state.has_file(&PathBuf::from("exists.rs")));
        assert!(!state.has_file(&PathBuf::from("deleted.rs")));

        Ok(())
    }

    #[test]
    fn test_thread_safe_index_state() -> FlashgrepResult<()> {
        let state = ThreadSafeIndexState::new();

        state.update_file(
            PathBuf::from("test.rs"),
            FileMetadata {
                size: 100,
                mtime: 1234567890,
                content_hash: "abc123".to_string(),
            },
        )?;

        assert!(state.has_file(&PathBuf::from("test.rs"))?);
        assert_eq!(state.len()?, 1);

        Ok(())
    }
}
