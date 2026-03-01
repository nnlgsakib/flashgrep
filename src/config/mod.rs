pub mod paths;

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ModelCacheScope {
    Local,
    Global,
}

/// Configuration for flashgrep
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Version of the configuration format
    pub version: String,

    /// Port for MCP server (if using TCP)
    pub mcp_port: u16,

    /// Use Unix socket instead of TCP (Unix only)
    #[serde(default = "default_use_unix_socket")]
    pub use_unix_socket: bool,

    /// Path to Unix socket (if use_unix_socket is true)
    #[serde(default = "default_socket_path")]
    pub socket_path: PathBuf,

    /// Maximum file size to index (in bytes)
    #[serde(default = "default_max_file_size")]
    pub max_file_size: u64,

    /// Maximum chunk size in lines
    #[serde(default = "default_max_chunk_lines")]
    pub max_chunk_lines: usize,

    /// File extensions to index
    #[serde(default = "default_extensions")]
    pub extensions: Vec<String>,

    /// Directories to ignore
    #[serde(default = "default_ignored_dirs")]
    pub ignored_dirs: Vec<String>,

    /// Debounce duration for file watcher in milliseconds
    #[serde(default = "default_debounce_ms")]
    pub debounce_ms: u64,

    /// Enable initial indexing on watcher start
    #[serde(default = "default_enable_initial_index")]
    pub enable_initial_index: bool,

    /// Progress logging interval for initial scan (number of files)
    #[serde(default = "default_progress_interval")]
    pub progress_interval: usize,

    /// Path to store index state (relative to .flashgrep directory)
    #[serde(default = "default_index_state_path")]
    pub index_state_path: PathBuf,

    /// Default model cache scope for neural model resolution
    #[serde(default = "default_model_cache_scope")]
    pub model_cache_scope: ModelCacheScope,

    /// Absolute or repo-relative path to a shared model cache when scope is global
    #[serde(default = "default_global_model_cache_path_opt")]
    pub global_model_cache_path: Option<PathBuf>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            version: env!("CARGO_PKG_VERSION").to_string(),
            mcp_port: crate::DEFAULT_MCP_PORT,
            use_unix_socket: default_use_unix_socket(),
            socket_path: default_socket_path(),
            max_file_size: default_max_file_size(),
            max_chunk_lines: default_max_chunk_lines(),
            extensions: default_extensions(),
            ignored_dirs: default_ignored_dirs(),
            debounce_ms: default_debounce_ms(),
            enable_initial_index: default_enable_initial_index(),
            progress_interval: default_progress_interval(),
            index_state_path: default_index_state_path(),
            model_cache_scope: default_model_cache_scope(),
            global_model_cache_path: default_global_model_cache_path_opt(),
        }
    }
}

impl Config {
    /// Load configuration from a file
    pub fn from_file(path: &Path) -> anyhow::Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let mut config: Config = serde_json::from_str(&content)?;
        config.normalize_model_paths(path)?;
        Ok(config)
    }

    /// Save configuration to a file
    pub fn to_file(&self, path: &Path) -> anyhow::Result<()> {
        let content = serde_json::to_string_pretty(self)?;
        std::fs::write(path, content)?;
        Ok(())
    }

    /// Get the default configuration file path within a flashgrep directory
    pub fn default_path(flashgrep_dir: &Path) -> PathBuf {
        flashgrep_dir.join("config.json")
    }

    fn normalize_model_paths(&mut self, config_path: &Path) -> anyhow::Result<()> {
        if let Some(path) = self.global_model_cache_path.clone() {
            if path.as_os_str().is_empty() {
                anyhow::bail!(
                    "global_model_cache_path cannot be empty; set a valid path or remove it"
                );
            }

            let normalized = if path.is_absolute() {
                path
            } else {
                let repo_root = config_path
                    .parent()
                    .and_then(|p| p.parent())
                    .ok_or_else(|| {
                        anyhow::anyhow!(
                            "Cannot normalize global_model_cache_path; invalid config location {}",
                            config_path.display()
                        )
                    })?;
                repo_root.join(path)
            };

            self.global_model_cache_path = Some(normalized);
        }

        Ok(())
    }
}

fn default_use_unix_socket() -> bool {
    cfg!(unix)
}

fn default_socket_path() -> PathBuf {
    PathBuf::from(".flashgrep/mcp.sock")
}

fn default_max_file_size() -> u64 {
    crate::MAX_FILE_SIZE
}

fn default_max_chunk_lines() -> usize {
    crate::MAX_CHUNK_LINES
}

fn default_extensions() -> Vec<String> {
    vec![
        "go".to_string(),
        "rs".to_string(),
        "js".to_string(),
        "ts".to_string(),
        "py".to_string(),
        "sol".to_string(),
        "json".to_string(),
        "md".to_string(),
        "yaml".to_string(),
        "yml".to_string(),
        "toml".to_string(),
    ]
}

fn default_ignored_dirs() -> Vec<String> {
    vec![
        ".git".to_string(),
        "node_modules".to_string(),
        "target".to_string(),
        "dist".to_string(),
        "build".to_string(),
        "vendor".to_string(),
        ".flashgrep".to_string(),
    ]
}

fn default_debounce_ms() -> u64 {
    500
}

fn default_enable_initial_index() -> bool {
    true
}

fn default_progress_interval() -> usize {
    1000
}

fn default_index_state_path() -> PathBuf {
    PathBuf::from("index-state.json")
}

fn default_model_cache_scope() -> ModelCacheScope {
    ModelCacheScope::Local
}

pub fn default_global_model_cache_path() -> PathBuf {
    if let Some(base) = dirs::data_local_dir().or_else(dirs::data_dir) {
        return base.join("flashgrep").join("model-cache");
    }

    if let Some(home) = dirs::home_dir() {
        return home.join(".flashgrep").join("model-cache");
    }

    PathBuf::from(".flashgrep/model-cache")
}

fn default_global_model_cache_path_opt() -> Option<PathBuf> {
    Some(default_global_model_cache_path())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_default() {
        let config = Config::default();
        assert_eq!(config.version, env!("CARGO_PKG_VERSION"));
        assert_eq!(config.mcp_port, 7777);
        assert_eq!(config.max_file_size, 2 * 1024 * 1024);
        assert_eq!(config.max_chunk_lines, 300);
    }

    #[test]
    fn test_config_serialization() {
        let config = Config::default();
        let json = serde_json::to_string(&config).unwrap();
        let deserialized: Config = serde_json::from_str(&json).unwrap();
        assert_eq!(config.version, deserialized.version);
        assert_eq!(config.mcp_port, deserialized.mcp_port);
    }

    #[test]
    fn test_model_scope_default_is_local() {
        let config = Config::default();
        assert_eq!(config.model_cache_scope, ModelCacheScope::Local);
        assert_eq!(
            config.global_model_cache_path,
            Some(default_global_model_cache_path())
        );
    }

    #[test]
    fn test_normalize_relative_global_model_path() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let flashgrep_dir = temp_dir.path().join(".flashgrep");
        std::fs::create_dir_all(&flashgrep_dir).unwrap();
        let config_path = flashgrep_dir.join("config.json");

        let content = r#"{
  "version": "0.1.0",
  "mcp_port": 7777,
  "use_unix_socket": false,
  "socket_path": ".flashgrep/mcp.sock",
  "max_file_size": 2097152,
  "max_chunk_lines": 300,
  "extensions": ["rs"],
  "ignored_dirs": [".git"],
  "debounce_ms": 500,
  "enable_initial_index": true,
  "progress_interval": 1000,
  "index_state_path": "index-state.json",
  "model_cache_scope": "global",
  "global_model_cache_path": "shared-models"
}"#;

        std::fs::write(&config_path, content).unwrap();
        let config = Config::from_file(&config_path).unwrap();
        assert_eq!(config.model_cache_scope, ModelCacheScope::Global);
        assert_eq!(
            config.global_model_cache_path,
            Some(temp_dir.path().join("shared-models"))
        );
    }
}
