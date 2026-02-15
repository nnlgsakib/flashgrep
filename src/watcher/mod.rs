pub mod registry;

use crate::config::paths::FlashgrepPaths;
use crate::config::Config;
use crate::index::engine::Indexer;
use crate::index::initial_scanner::{run_initial_scan, ScanResult};
use crate::index::scanner::{
    is_binary_file, is_oversized_file, should_ignore_directory, should_index_file, FlashgrepIgnore,
};
use crate::index::state::ThreadSafeIndexState;
use crate::FlashgrepResult;
use notify::{Config as NotifyConfig, Event, RecommendedWatcher, RecursiveMode, Watcher};
use std::collections::HashMap;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::PathBuf;
use std::sync::mpsc::{channel, Receiver};
use std::time::{Duration, Instant};
use tracing::{debug, info, warn};

/// File watcher for incremental indexing
pub struct FileWatcher {
    repo_root: PathBuf,
    indexer: Indexer,
    config: Config,
    debounce_duration: Duration,
    ignore_patterns: FlashgrepIgnore,
    lock_path: PathBuf,
    index_state: ThreadSafeIndexState,
    index_state_path: PathBuf,
}

/// Represents a change detected during initial scan
#[derive(Debug, Clone)]
pub enum SyntheticEvent {
    FileCreated(PathBuf),
    FileModified(PathBuf),
    FileDeleted(PathBuf),
}

impl FileWatcher {
    /// Create a new file watcher
    pub fn new(repo_root: PathBuf) -> FlashgrepResult<Self> {
        let lock_path = acquire_watcher_lock(&repo_root)?;
        let indexer = Indexer::new(repo_root.clone())?;

        // Load or create config
        let paths = FlashgrepPaths::new(&repo_root);
        let config = if paths.config_file().exists() {
            Config::from_file(&paths.config_file())?
        } else {
            Config::default()
        };

        // Load ignore patterns
        let ignore_patterns = FlashgrepIgnore::from_root(&repo_root);

        // Create default .flashgrepignore if it doesn't exist
        Self::create_default_ignore_file(&repo_root)?;

        // Load or create index state
        let index_state_path = paths.root().join(&config.index_state_path);
        let index_state = ThreadSafeIndexState::load(&index_state_path)?;

        Ok(Self {
            repo_root,
            indexer,
            config,
            debounce_duration: Duration::from_millis(500),
            ignore_patterns,
            lock_path,
            index_state,
            index_state_path,
        })
    }

    /// Create a default .flashgrepignore file if it doesn't exist
    fn create_default_ignore_file(repo_root: &PathBuf) -> FlashgrepResult<()> {
        let ignore_file = repo_root.join(".flashgrepignore");

        if !ignore_file.exists() {
            let default_content = r#"# Flashgrep ignore file
# Add patterns to exclude files/directories from indexing
# Uses gitignore-style syntax

# Build directories
target/
build/
dist/
node_modules/
vendor/

# IDE
.idea/
.vscode/
*.swp
*.swo

# OS
.DS_Store
Thumbs.db

# Logs
*.log

# Temporary files
*.tmp
*.temp
"#;

            std::fs::write(&ignore_file, default_content)?;
            info!("Created default .flashgrepignore file");
        }

        Ok(())
    }

    /// Perform initial scan and emit synthetic events for detected changes
    pub async fn perform_initial_scan(&mut self) -> FlashgrepResult<ScanResult> {
        info!("Starting initial index scan...");

        let result = run_initial_scan(
            self.repo_root.clone(),
            self.config.clone(),
            self.ignore_patterns.clone(),
            self.index_state.clone(),
        )
        .await?;

        // Emit synthetic events for detected changes
        // Note: In a full implementation, these would be sent through the event channel
        // For now, we just log them
        if result.files_added > 0 {
            info!("Detected {} files added while offline", result.files_added);
        }
        if result.files_modified > 0 {
            info!(
                "Detected {} files modified while offline",
                result.files_modified
            );
        }
        if result.files_deleted > 0 {
            info!(
                "Detected {} files deleted while offline",
                result.files_deleted
            );
        }

        // Save the updated index state
        self.index_state.save(&self.index_state_path)?;

        Ok(result)
    }

    /// Start watching the repository with optional initial scan
    pub async fn watch_with_initial_scan(&mut self) -> FlashgrepResult<()> {
        info!("Starting file watcher for: {}", self.repo_root.display());

        // Start the file system watcher immediately (non-blocking)
        let (tx, rx) = channel();

        let mut watcher = RecommendedWatcher::new(
            move |res: Result<Event, notify::Error>| {
                if let Ok(event) = res {
                    let _ = tx.send(event);
                }
            },
            NotifyConfig::default(),
        )?;

        watcher.watch(&self.repo_root, RecursiveMode::Recursive)?;

        info!("File watcher started, monitoring for changes...");

        // Perform initial scan if enabled
        if self.config.enable_initial_index {
            info!("Starting initial scan in background...");
            let scan_result = self.perform_initial_scan().await?;

            // Process synthetic events (files detected during scan)
            self.process_synthetic_changes(&scan_result)?;
        } else {
            info!("Initial indexing is disabled");
        }

        // Continue with normal event processing
        self.process_events(rx)?;

        Ok(())
    }

    /// Start watching the repository (legacy method without initial scan)
    pub fn watch(&mut self) -> FlashgrepResult<()> {
        info!("Starting file watcher for: {}", self.repo_root.display());

        let (tx, rx) = channel();

        let mut watcher = RecommendedWatcher::new(
            move |res: Result<Event, notify::Error>| {
                if let Ok(event) = res {
                    let _ = tx.send(event);
                }
            },
            NotifyConfig::default(),
        )?;

        watcher.watch(&self.repo_root, RecursiveMode::Recursive)?;

        info!("File watcher started, monitoring for changes...");

        self.process_events(rx)?;

        Ok(())
    }

    /// Process synthetic changes detected during initial scan
    fn process_synthetic_changes(&mut self, scan_result: &ScanResult) -> FlashgrepResult<()> {
        // In a full implementation, this would process the detected changes
        // and emit events through the same pipeline as real-time events
        // For now, we just ensure the files are properly indexed

        if scan_result.errors.is_empty() {
            return Ok(());
        }

        for error in &scan_result.errors {
            warn!("Initial scan error: {}", error);
        }

        Ok(())
    }

    /// Process file system events with debouncing
    fn process_events(&mut self, rx: Receiver<Event>) -> FlashgrepResult<()> {
        let mut pending_changes: HashMap<PathBuf, Instant> = HashMap::new();
        let mut last_update = Instant::now();

        loop {
            // Check for new events
            if let Ok(event) = rx.try_recv() {
                debug!("File event: {:?}", event);

                for path in event.paths {
                    if Self::is_ignore_file(&path) {
                        self.reload_ignore_patterns_and_reconcile()?;
                        continue;
                    }

                    // Skip if path should be ignored
                    if self.should_ignore_path(&path) {
                        debug!("Ignoring path: {}", path.display());
                        continue;
                    }

                    pending_changes.insert(path, Instant::now());
                }
            }

            // Process pending changes if debounce period has passed
            let now = Instant::now();
            if now.duration_since(last_update) >= self.debounce_duration {
                let ready_changes: Vec<_> = pending_changes
                    .iter()
                    .filter(|(_, instant)| now.duration_since(**instant) >= self.debounce_duration)
                    .map(|(path, _)| path.clone())
                    .collect();

                for path in ready_changes {
                    pending_changes.remove(&path);
                    if let Err(e) = self.handle_change(&path) {
                        warn!("Failed to handle change for {}: {}", path.display(), e);
                    }
                }

                last_update = now;
            }

            // Small sleep to prevent busy waiting
            std::thread::sleep(Duration::from_millis(10));
        }
    }

    /// Check if a path should be ignored by the file watcher
    fn should_ignore_path(&self, path: &PathBuf) -> bool {
        // Skip the .flashgrep directory
        if path.components().any(|c| {
            if let std::path::Component::Normal(name) = c {
                name == ".flashgrep"
            } else {
                false
            }
        }) {
            return true;
        }

        // Check if any parent directory is ignored
        for component in path.components() {
            if let std::path::Component::Normal(name) = component {
                if let Some(name_str) = name.to_str() {
                    if should_ignore_directory(name_str, &self.config) {
                        return true;
                    }
                }
            }
        }

        // Check ignore patterns
        if self.ignore_patterns.is_ignored(path, &self.repo_root) {
            return true;
        }

        // Skip binary files early (for file watcher efficiency)
        if path.is_file() {
            // Quick extension check first
            if !should_index_file(path, &self.config) {
                return true;
            }

            // Skip oversized files
            if let Ok(true) = is_oversized_file(path, self.config.max_file_size) {
                return true;
            }
        }

        false
    }

    fn is_ignore_file(path: &PathBuf) -> bool {
        path.file_name()
            .and_then(|s| s.to_str())
            .map(|n| n == ".flashgrepignore")
            .unwrap_or(false)
    }

    fn reload_ignore_patterns_and_reconcile(&mut self) -> FlashgrepResult<()> {
        self.ignore_patterns = FlashgrepIgnore::from_root(&self.repo_root);
        let (removed, kept) = self
            .indexer
            .reconcile_ignored_files(&self.repo_root, &self.ignore_patterns)?;
        info!(
            "Reloaded .flashgrepignore and reconciled index: {} removed, {} kept",
            removed, kept
        );
        Ok(())
    }

    /// Handle a single file change
    fn handle_change(&mut self, path: &PathBuf) -> FlashgrepResult<()> {
        if self.ignore_patterns.is_ignored(path, &self.repo_root) {
            debug!(
                "Path became ignored, pruning if indexed: {}",
                path.display()
            );
            self.indexer.remove_file_from_index(path)?;
            // Also update index state
            let rel_path = path.strip_prefix(&self.repo_root).unwrap_or(path);
            self.index_state.remove_file(rel_path)?;
            return Ok(());
        }

        if !path.exists() {
            // File was deleted
            info!("File deleted: {}", path.display());
            self.indexer.remove_file_from_index(path)?;
            // Update index state
            let rel_path = path.strip_prefix(&self.repo_root).unwrap_or(path);
            self.index_state.remove_file(rel_path)?;
        } else if path.is_file() {
            // Skip binary files during indexing
            if let Ok(true) = is_binary_file(path) {
                debug!("Skipping binary file: {}", path.display());
                return Ok(());
            }

            // File was created or modified
            info!("File changed: {}", path.display());
            match self.indexer.index_file(path) {
                Ok(indexed) => {
                    if indexed {
                        debug!("Successfully indexed: {}", path.display());
                        // Update index state with new metadata
                        self.update_index_state_for_file(path)?;
                    } else {
                        debug!("Skipped unchanged file: {}", path.display());
                    }
                }
                Err(e) => warn!("Failed to index {}: {}", path.display(), e),
            }
        }

        Ok(())
    }

    /// Update index state for a single file
    fn update_index_state_for_file(&mut self, path: &PathBuf) -> FlashgrepResult<()> {
        use crate::index::state::FileMetadata;
        use sha2::{Digest, Sha256};
        use std::time::SystemTime;

        let rel_path = path.strip_prefix(&self.repo_root).unwrap_or(path);

        let metadata = std::fs::metadata(path)?;
        let size = metadata.len();
        let mtime = metadata
            .modified()
            .ok()
            .and_then(|t| t.duration_since(SystemTime::UNIX_EPOCH).ok())
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0);

        // Compute content hash (first 8KB only)
        let content = std::fs::read(path)?;
        let hash_input = if content.len() > 8192 {
            &content[..8192]
        } else {
            &content
        };
        let content_hash = hex::encode(Sha256::digest(hash_input));

        let file_metadata = FileMetadata {
            size,
            mtime,
            content_hash,
        };

        self.index_state.update_file(rel_path.to_path_buf(), file_metadata)?;

        // Periodically save index state (every 100 changes)
        // In production, this should be debounced
        if self.index_state.len()? % 100 == 0 {
            self.index_state.save(&self.index_state_path)?;
        }

        Ok(())
    }
}

impl Drop for FileWatcher {
    fn drop(&mut self) {
        // Save index state on shutdown
        let _ = self.index_state.save(&self.index_state_path);
        let _ = std::fs::remove_file(&self.lock_path);
    }
}

fn acquire_watcher_lock(repo_root: &PathBuf) -> FlashgrepResult<PathBuf> {
    let paths = FlashgrepPaths::new(repo_root);
    std::fs::create_dir_all(paths.root())?;
    let lock_path = paths.root().join("watcher.lock");

    if lock_path.exists() {
        let stale = match std::fs::read_to_string(&lock_path) {
            Ok(raw) => match raw.trim().parse::<u32>() {
                Ok(pid) => !registry::is_process_alive(pid),
                Err(_) => true,
            },
            Err(_) => true,
        };

        if stale {
            let _ = std::fs::remove_file(&lock_path);
        } else {
            return Err(crate::FlashgrepError::FileWatcher(format!(
                "Watcher already running for {}",
                repo_root.display()
            )));
        }
    }

    match OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(&lock_path)
    {
        Ok(mut file) => {
            let pid = std::process::id();
            let _ = writeln!(file, "{}", pid);
            Ok(lock_path)
        }
        Err(err) if err.kind() == std::io::ErrorKind::AlreadyExists => {
            Err(crate::FlashgrepError::FileWatcher(format!(
                "Watcher already running for {}",
                repo_root.display()
            )))
        }
        Err(err) => Err(err.into()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_ignore_change_reloads_and_prunes_index() -> FlashgrepResult<()> {
        let temp_dir = TempDir::new()?;
        let repo_root = temp_dir.path().to_path_buf();

        std::fs::create_dir_all(repo_root.join(".opencode/pkg"))?;
        std::fs::write(
            repo_root.join(".opencode/pkg/ignored.rs"),
            "fn ignored_symbol() { let _ = \"WATCHER_IGNORE_TOKEN\"; }",
        )?;
        std::fs::write(repo_root.join("main.rs"), "fn main() {}\n")?;
        std::fs::write(repo_root.join(".flashgrepignore"), "# initially empty\n")?;

        let mut watcher = FileWatcher::new(repo_root.clone())?;
        watcher.indexer.index_repository(&repo_root)?;

        let before = watcher.indexer.db().get_all_files()?;
        assert!(before
            .iter()
            .any(|p| p.to_string_lossy().contains(".opencode")));

        std::fs::write(repo_root.join(".flashgrepignore"), ".opencode/\n")?;
        watcher.reload_ignore_patterns_and_reconcile()?;

        let after = watcher.indexer.db().get_all_files()?;
        assert!(!after
            .iter()
            .any(|p| p.to_string_lossy().contains(".opencode")));

        Ok(())
    }
}
