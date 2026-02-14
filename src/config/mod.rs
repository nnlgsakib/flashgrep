pub mod paths;

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

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
        }
    }
}

impl Config {
    /// Load configuration from a file
    pub fn from_file(path: &PathBuf) -> anyhow::Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let config: Config = serde_json::from_str(&content)?;
        Ok(config)
    }

    /// Save configuration to a file
    pub fn to_file(&self, path: &PathBuf) -> anyhow::Result<()> {
        let content = serde_json::to_string_pretty(self)?;
        std::fs::write(path, content)?;
        Ok(())
    }

    /// Get the default configuration file path within a flashgrep directory
    pub fn default_path(flashgrep_dir: &PathBuf) -> PathBuf {
        flashgrep_dir.join("config.json")
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

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

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
}
