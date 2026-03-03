pub mod paths;

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct NeuralProviderConfig {
    #[serde(default = "default_neural_provider")]
    pub provider: String,
    #[serde(default = "default_neural_base_url")]
    pub base_url: String,
    #[serde(default = "default_neural_model")]
    pub model: String,
    #[serde(default = "default_neural_api_key_env")]
    pub api_key_env: String,
    #[serde(default)]
    pub api_key: Option<String>,
    #[serde(default = "default_neural_timeout_ms")]
    pub timeout_ms: u64,
    #[serde(default = "default_neural_max_candidates")]
    pub max_candidates: usize,
}

impl Default for NeuralProviderConfig {
    fn default() -> Self {
        Self {
            provider: default_neural_provider(),
            base_url: default_neural_base_url(),
            model: default_neural_model(),
            api_key_env: default_neural_api_key_env(),
            api_key: None,
            timeout_ms: default_neural_timeout_ms(),
            max_candidates: default_neural_max_candidates(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct NeuralConfig {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default = "default_neural_config_initialized")]
    pub initialized: bool,
    #[serde(default)]
    pub provider: NeuralProviderConfig,
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

    /// Neural navigation and provider configuration
    #[serde(default)]
    pub neural: NeuralConfig,
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
            neural: NeuralConfig::default(),
        }
    }
}

impl Config {
    /// Load configuration from a file
    pub fn from_file(path: &Path) -> anyhow::Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let config: Config = serde_json::from_str(&content)?;
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

    pub fn resolve_neural_api_key(&self) -> Option<String> {
        if let Some(key) = self
            .neural
            .provider
            .api_key
            .as_ref()
            .map(|k| k.trim().to_string())
        {
            if !key.is_empty() {
                return Some(key);
            }
        }

        let env_key = self.neural.provider.api_key_env.trim();
        if env_key.is_empty() {
            return None;
        }
        if !looks_like_env_var_name(env_key) {
            return Some(env_key.to_string());
        }
        std::env::var(env_key)
            .ok()
            .map(|v| v.trim().to_string())
            .filter(|v| !v.is_empty())
    }

    pub fn validate_neural(&self) -> anyhow::Result<()> {
        if !self.neural.enabled {
            return Ok(());
        }

        if self.neural.provider.provider.trim().is_empty() {
            anyhow::bail!("neural.provider.provider cannot be empty when neural mode is enabled");
        }
        if self.neural.provider.base_url.trim().is_empty() {
            anyhow::bail!("neural.provider.base_url cannot be empty when neural mode is enabled");
        }
        if self.neural.provider.model.trim().is_empty() {
            anyhow::bail!("neural.provider.model cannot be empty when neural mode is enabled");
        }
        if self.resolve_neural_api_key().is_none() {
            let key_hint = if looks_like_env_var_name(&self.neural.provider.api_key_env) {
                format!(
                    "neural.provider.api_key or env var {}",
                    self.neural.provider.api_key_env
                )
            } else {
                "neural.provider.api_key or a valid API key env var name".to_string()
            };
            anyhow::bail!(
                "Neural mode is enabled but no API key resolved. Set {}",
                key_hint
            );
        }
        Ok(())
    }
}

fn looks_like_env_var_name(value: &str) -> bool {
    !value.is_empty()
        && value
            .chars()
            .all(|c| c.is_ascii_uppercase() || c.is_ascii_digit() || c == '_')
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

fn default_neural_provider() -> String {
    "openrouter".to_string()
}

fn default_neural_base_url() -> String {
    "https://openrouter.ai/api/v1".to_string()
}

fn default_neural_model() -> String {
    "arcee-ai/trinity-large-preview:free".to_string()
}

fn default_neural_api_key_env() -> String {
    "OPENROUTER_API_KEY".to_string()
}

fn default_neural_timeout_ms() -> u64 {
    5000
}

fn default_neural_max_candidates() -> usize {
    24
}

fn default_neural_config_initialized() -> bool {
    false
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
    fn neural_defaults_match_openrouter_profile() {
        let cfg = Config::default();
        assert_eq!(cfg.neural.provider.provider, "openrouter");
        assert_eq!(cfg.neural.provider.base_url, "https://openrouter.ai/api/v1");
        assert_eq!(
            cfg.neural.provider.model,
            "arcee-ai/trinity-large-preview:free"
        );
        assert_eq!(cfg.neural.provider.api_key_env, "OPENROUTER_API_KEY");
        assert!(!cfg.neural.enabled);
    }

    #[test]
    fn resolve_neural_api_key_prefers_inline_key_then_env() {
        let mut cfg = Config::default();
        cfg.neural.provider.api_key = Some("inline-key".to_string());
        assert_eq!(cfg.resolve_neural_api_key().as_deref(), Some("inline-key"));

        cfg.neural.provider.api_key = None;
        cfg.neural.provider.api_key_env = "FLASHGREP_TEST_NEURAL_KEY".to_string();
        std::env::set_var("FLASHGREP_TEST_NEURAL_KEY", "env-key");
        assert_eq!(cfg.resolve_neural_api_key().as_deref(), Some("env-key"));
        std::env::remove_var("FLASHGREP_TEST_NEURAL_KEY");
    }
}
