use crate::config::Config;
use crate::index::scanner::{
    is_binary_file, is_oversized_file, should_ignore_directory, should_index_file, FlashgrepIgnore,
};
use crate::index::state::{FileMetadata, ThreadSafeIndexState};
use crate::FlashgrepResult;
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};
use std::time::SystemTime;
use tracing::{debug, info, warn};
use walkdir::WalkDir;

/// Maximum bytes to read for content hash (for performance)
const MAX_HASH_BYTES: usize = 8 * 1024; // 8KB

/// Initial scanner for building the file index on watcher startup
pub struct InitialScanner {
    root: PathBuf,
    config: Config,
    ignore_patterns: FlashgrepIgnore,
    progress_interval: usize,
    index_state: ThreadSafeIndexState,
}

/// Represents a file that needs to be processed
#[derive(Debug)]
pub struct ScannedFile {
    pub path: PathBuf,
    pub metadata: FileMetadata,
}

/// Metrics for scan performance
#[derive(Debug, Clone)]
pub struct ScanMetrics {
    /// Total duration of the scan
    pub duration_ms: u64,
    /// Files scanned per second
    pub files_per_second: f64,
    /// Total bytes processed
    pub bytes_processed: u64,
    /// Average file size
    pub avg_file_size_bytes: u64,
    /// Scan start time (Unix timestamp)
    pub start_time: i64,
    /// Scan end time (Unix timestamp)
    pub end_time: i64,
}

/// Result of the initial scan
#[derive(Debug)]
pub struct ScanResult {
    pub files_scanned: usize,
    pub files_added: usize,
    pub files_modified: usize,
    pub files_deleted: usize,
    pub errors: Vec<String>,
    pub metrics: Option<ScanMetrics>,
}

impl InitialScanner {
    /// Create a new initial scanner
    pub fn new(
        root: PathBuf,
        config: Config,
        ignore_patterns: FlashgrepIgnore,
        index_state: ThreadSafeIndexState,
    ) -> Self {
        let progress_interval = config.progress_interval;
        Self {
            root,
            config,
            ignore_patterns,
            progress_interval,
            index_state,
        }
    }

    /// Set the progress logging interval
    pub fn with_progress_interval(mut self, interval: usize) -> Self {
        self.progress_interval = interval;
        self
    }

    /// Perform the initial scan asynchronously
    pub async fn scan(&self) -> FlashgrepResult<ScanResult> {
        info!("Starting initial scan of {}", self.root.display());

        let start_time = std::time::Instant::now();
        let start_timestamp = chrono::Utc::now().timestamp();
        let mut total_bytes: u64 = 0;

        let mut result = ScanResult {
            files_scanned: 0,
            files_added: 0,
            files_modified: 0,
            files_deleted: 0,
            errors: Vec::new(),
            metrics: None,
        };

        // Load previous index for comparison
        let previous_paths = self.index_state.get_all_paths()?;
        let previous_paths_set: std::collections::HashSet<_> = previous_paths.iter().cloned().collect();
        let mut current_paths = std::collections::HashSet::new();

        // Scan all files
        let walker = WalkDir::new(&self.root)
            .follow_links(false) // Don't follow symlinks to avoid cycles
            .into_iter();

        for entry in walker {
            match entry {
                Ok(entry) => {
                    if !entry.file_type().is_file() {
                        continue;
                    }

                    let path = entry.path();

                    // Skip files in .flashgrep directory
                    if self.is_in_flashgrep_dir(path) {
                        continue;
                    }

                    // Check if file should be included
                    if !self.should_include(path) {
                        continue;
                    }

                    // Extract metadata
                    match self.extract_file_metadata(path).await {
                        Ok(metadata) => {
                            total_bytes += metadata.size;
                            let rel_path = self.relative_path(path);
                            current_paths.insert(rel_path.clone());

                            // Check if file is new or modified
                            let is_new = !previous_paths_set.contains(&rel_path);
                            let is_modified = if !is_new {
                                self.index_state.is_file_changed(&rel_path, &metadata)?
                            } else {
                                false
                            };

                            // Update index state
                            self.index_state.update_file(rel_path.clone(), metadata)?;

                            result.files_scanned += 1;
                            if is_new {
                                result.files_added += 1;
                                debug!("New file detected: {}", rel_path.display());
                            } else if is_modified {
                                result.files_modified += 1;
                                debug!("Modified file detected: {}", rel_path.display());
                            }

                            // Log progress
                            if result.files_scanned % self.progress_interval == 0 {
                                info!(
                                    "Initial indexing progress: {} files scanned",
                                    result.files_scanned
                                );
                            }
                        }
                        Err(e) => {
                            let msg = format!("Failed to process {}: {}", path.display(), e);
                            warn!("{}", msg);
                            result.errors.push(msg);
                        }
                    }
                }
                Err(e) => {
                    let msg = format!("Failed to read directory entry: {}", e);
                    warn!("{}", msg);
                    result.errors.push(msg);
                }
            }
        }

        // Detect deleted files
        for path in &previous_paths {
            if !current_paths.contains(path) {
                result.files_deleted += 1;
                self.index_state.remove_file(path)?;
                debug!("Deleted file detected: {}", path.display());
            }
        }

        let end_time = std::time::Instant::now();
        let duration = end_time.duration_since(start_time);
        let duration_ms = duration.as_millis() as u64;
        let files_per_second = if duration_ms > 0 {
            (result.files_scanned as f64) / (duration_ms as f64 / 1000.0)
        } else {
            0.0
        };
        let avg_file_size = if result.files_scanned > 0 {
            total_bytes / result.files_scanned as u64
        } else {
            0
        };

        result.metrics = Some(ScanMetrics {
            duration_ms,
            files_per_second,
            bytes_processed: total_bytes,
            avg_file_size_bytes: avg_file_size,
            start_time: start_timestamp,
            end_time: chrono::Utc::now().timestamp(),
        });

        info!(
            "Initial scan complete: {} scanned, {} added, {} modified, {} deleted (took {:?}, {:.1} files/sec)",
            result.files_scanned, result.files_added, result.files_modified, result.files_deleted,
            duration, files_per_second
        );

        Ok(result)
    }

    /// Check if a path is within the .flashgrep directory
    fn is_in_flashgrep_dir(&self, path: &Path) -> bool {
        path.components().any(|c| {
            if let std::path::Component::Normal(name) = c {
                name == ".flashgrep"
            } else {
                false
            }
        })
    }

    /// Check if a file should be included in the scan
    fn should_include(&self, path: &Path) -> bool {
        // Check ignore patterns
        if self.ignore_patterns.is_ignored(path, &self.root) {
            return false;
        }

        // Check ignored directories
        for component in path.components() {
            if let std::path::Component::Normal(name) = component {
                if let Some(name_str) = name.to_str() {
                    if should_ignore_directory(name_str, &self.config) {
                        return false;
                    }
                }
            }
        }

        // Check file extension
        if !should_index_file(path, &self.config) {
            return false;
        }

        // Check file size
        match is_oversized_file(path, self.config.max_file_size) {
            Ok(true) => return false,
            Ok(false) => {}
            Err(_) => return false,
        }

        // Check if binary
        match is_binary_file(path) {
            Ok(true) => return false,
            Ok(false) => {}
            Err(_) => return false,
        }

        true
    }

    /// Extract metadata from a file
    async fn extract_file_metadata(&self, path: &Path) -> FlashgrepResult<FileMetadata> {
        let metadata = tokio::fs::metadata(path).await.map_err(|e| {
            crate::FlashgrepError::Io(e)
        })?;

        let size = metadata.len();
        
        let mtime = metadata
            .modified()
            .ok()
            .and_then(|t| t.duration_since(SystemTime::UNIX_EPOCH).ok())
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0);

        // Compute content hash (first 8KB only for performance)
        let content_hash = self.compute_content_hash(path).await?;

        Ok(FileMetadata {
            size,
            mtime,
            content_hash,
        })
    }

    /// Compute SHA-256 hash of file content (first 8KB only)
    async fn compute_content_hash(&self, path: &Path) -> FlashgrepResult<String> {
        let content = tokio::fs::read(path).await.map_err(|e| {
            crate::FlashgrepError::Io(e)
        })?;

        let hash_input = if content.len() > MAX_HASH_BYTES {
            &content[..MAX_HASH_BYTES]
        } else {
            &content
        };

        let hash = Sha256::digest(hash_input);
        Ok(hex::encode(hash))
    }

    /// Get the relative path from the repository root
    fn relative_path(&self, path: &Path) -> PathBuf {
        path.strip_prefix(&self.root)
            .unwrap_or(path)
            .to_path_buf()
    }
}

/// Run initial scan in background and return a channel for results
pub async fn run_initial_scan(
    root: PathBuf,
    config: Config,
    ignore_patterns: FlashgrepIgnore,
    index_state: ThreadSafeIndexState,
) -> FlashgrepResult<ScanResult> {
    let scanner = InitialScanner::new(root, config, ignore_patterns, index_state);
    scanner.scan().await
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_initial_scan() -> FlashgrepResult<()> {
        let temp_dir = TempDir::new()?;
        let root = temp_dir.path().to_path_buf();

        // Create test files
        std::fs::create_dir_all(root.join("src"))?;
        std::fs::write(root.join("src/main.rs"), "fn main() {}")?;
        std::fs::write(root.join("readme.md"), "# Readme")?;

        let config = Config::default();
        let ignore_patterns = FlashgrepIgnore::from_root(&root);
        let index_state = ThreadSafeIndexState::new();

        let scanner = InitialScanner::new(root.clone(), config, ignore_patterns, index_state);
        let result = scanner.scan().await?;

        assert_eq!(result.files_scanned, 2);
        assert_eq!(result.files_added, 2);
        assert_eq!(result.files_modified, 0);
        assert_eq!(result.files_deleted, 0);

        // Verify metrics are populated
        assert!(result.metrics.is_some());
        let metrics = result.metrics.unwrap();
        assert!(metrics.duration_ms > 0);
        assert!(metrics.files_per_second > 0.0);
        assert!(metrics.bytes_processed > 0);
        assert!(metrics.avg_file_size_bytes > 0);

        Ok(())
    }

    #[tokio::test]
    async fn test_scan_metrics() -> FlashgrepResult<()> {
        let temp_dir = TempDir::new()?;
        let root = temp_dir.path().to_path_buf();

        // Create test files with known sizes (use .md extension which is in default extensions)
        std::fs::write(root.join("small.md"), "a")?; // 1 byte
        std::fs::write(root.join("medium.md"), "hello world")?; // 11 bytes

        let config = Config::default();
        let ignore_patterns = FlashgrepIgnore::from_root(&root);
        let index_state = ThreadSafeIndexState::new();

        let scanner = InitialScanner::new(root.clone(), config, ignore_patterns, index_state);
        let result = scanner.scan().await?;

        assert!(result.metrics.is_some());
        let metrics = result.metrics.unwrap();

        // Verify basic metrics
        // Note: duration_ms might be 0 for very fast scans on small repos
        assert!(metrics.end_time >= metrics.start_time);
        assert_eq!(result.files_scanned, 2);
        assert_eq!(metrics.bytes_processed, 12); // 1 + 11 bytes
        assert_eq!(metrics.avg_file_size_bytes, 6); // 12 / 2

        Ok(())
    }

    #[tokio::test]
    async fn test_scan_respects_ignore_patterns() -> FlashgrepResult<()> {
        let temp_dir = TempDir::new()?;
        let root = temp_dir.path().to_path_buf();

        // Create test files
        std::fs::create_dir_all(root.join("src"))?;
        std::fs::write(root.join("src/main.rs"), "fn main() {}")?;
        std::fs::write(root.join("temp.txt"), "temporary")?;

        // Create ignore file
        std::fs::write(root.join(".flashgrepignore"), "temp*.txt\n")?;

        let config = Config::default();
        let ignore_patterns = FlashgrepIgnore::from_root(&root);
        let index_state = ThreadSafeIndexState::new();

        let scanner = InitialScanner::new(root.clone(), config, ignore_patterns, index_state);
        let result = scanner.scan().await?;

        assert_eq!(result.files_scanned, 1); // Only main.rs
        assert!(!result.errors.iter().any(|e| e.contains("temp.txt")));

        Ok(())
    }

    #[tokio::test]
    async fn test_scan_detects_modifications() -> FlashgrepResult<()> {
        let temp_dir = TempDir::new()?;
        let root = temp_dir.path().to_path_buf();

        // Create test file
        std::fs::write(root.join("test.rs"), "fn test() {}")?;

        let config = Config::default();
        let ignore_patterns = FlashgrepIgnore::from_root(&root);
        let index_state = ThreadSafeIndexState::new();

        // First scan
        let scanner = InitialScanner::new(root.clone(), config.clone(), ignore_patterns.clone(), index_state.clone());
        let result = scanner.scan().await?;
        assert_eq!(result.files_added, 1);

        // Modify file
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        std::fs::write(root.join("test.rs"), "fn test_modified() {}")?;

        // Second scan
        let scanner = InitialScanner::new(root.clone(), config, ignore_patterns, index_state);
        let result = scanner.scan().await?;
        assert_eq!(result.files_modified, 1);

        Ok(())
    }
}
