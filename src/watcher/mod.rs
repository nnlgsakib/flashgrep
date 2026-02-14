use crate::config::paths::FlashgrepPaths;
use crate::config::Config;
use crate::index::engine::Indexer;
use crate::index::scanner::{
    is_binary_file, is_oversized_file, should_ignore_directory, should_index_file, FlashgrepIgnore,
};
use crate::FlashgrepResult;
use notify::{Config as NotifyConfig, Event, RecommendedWatcher, RecursiveMode, Watcher};
use std::collections::HashMap;
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
}

impl FileWatcher {
    /// Create a new file watcher
    pub fn new(repo_root: PathBuf) -> FlashgrepResult<Self> {
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

        Ok(Self {
            repo_root,
            indexer,
            config,
            debounce_duration: Duration::from_millis(500),
            ignore_patterns,
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

    /// Start watching the repository
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

    /// Process file system events with debouncing
    fn process_events(&mut self, rx: Receiver<Event>) -> FlashgrepResult<()> {
        let mut pending_changes: HashMap<PathBuf, Instant> = HashMap::new();
        let mut last_update = Instant::now();

        loop {
            // Check for new events
            if let Ok(event) = rx.try_recv() {
                debug!("File event: {:?}", event);

                for path in event.paths {
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
                    self.handle_change(&path)?;
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

    /// Handle a single file change
    fn handle_change(&mut self, path: &PathBuf) -> FlashgrepResult<()> {
        if !path.exists() {
            // File was deleted
            info!("File deleted: {}", path.display());
            self.indexer.db().delete_file(path)?;
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
                    } else {
                        debug!("Skipped unchanged file: {}", path.display());
                    }
                }
                Err(e) => warn!("Failed to index {}: {}", path.display(), e),
            }
        }

        Ok(())
    }
}
