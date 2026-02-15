//! Flashgrep - High-performance local code indexing engine
//!
//! A fast, language-agnostic code search tool designed for LLM coding agents.
//! Provides full-text and structural search with minimal memory footprint.

pub mod chunking;
pub mod cli;
pub mod config;
pub mod db;
pub mod index;
pub mod mcp;
pub mod search;
pub mod symbols;
pub mod watcher;

use anyhow::Result;
use std::path::Path;

/// Version of the flashgrep binary
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Name of the flashgrep binary
pub const NAME: &str = env!("CARGO_PKG_NAME");

/// Default port for MCP server
pub const DEFAULT_MCP_PORT: u16 = 7777;

/// Default directory name for flashgrep data
pub const FLASHGREP_DIR: &str = ".flashgrep";

/// Maximum file size to index (2MB)
pub const MAX_FILE_SIZE: u64 = 2 * 1024 * 1024;

/// Maximum chunk size in lines
pub const MAX_CHUNK_LINES: usize = 300;

/// Initialize logging with tracing
pub fn init_logging() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();
}

/// Check if a path is within a flashgrep directory
pub fn is_in_flashgrep_dir(path: &Path) -> bool {
    path.components().any(|c| {
        if let std::path::Component::Normal(os_str) = c {
            os_str == FLASHGREP_DIR
        } else {
            false
        }
    })
}

/// Get the flashgrep directory path for a given repository root
pub fn get_flashgrep_dir(repo_root: &Path) -> std::path::PathBuf {
    repo_root.join(FLASHGREP_DIR)
}

#[derive(Debug, thiserror::Error)]
pub enum FlashgrepError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Database error: {0}")]
    Database(#[from] rusqlite::Error),

    #[error("Search error: {0}")]
    Search(String),

    #[error("Index not found at {0}")]
    IndexNotFound(std::path::PathBuf),

    #[error("Index state error: {0}")]
    Index(String),

    #[error("Invalid configuration: {0}")]
    Config(String),

    #[error("File watcher error: {0}")]
    FileWatcher(String),

    #[error("MCP server error: {0}")]
    McpServer(String),

    #[error("Task error: {0}")]
    Task(String),
}

impl From<anyhow::Error> for FlashgrepError {
    fn from(err: anyhow::Error) -> Self {
        FlashgrepError::Config(err.to_string())
    }
}

impl From<notify::Error> for FlashgrepError {
    fn from(err: notify::Error) -> Self {
        FlashgrepError::FileWatcher(err.to_string())
    }
}

impl From<tantivy::TantivyError> for FlashgrepError {
    fn from(err: tantivy::TantivyError) -> Self {
        FlashgrepError::Search(err.to_string())
    }
}

impl From<serde_json::Error> for FlashgrepError {
    fn from(err: serde_json::Error) -> Self {
        FlashgrepError::Config(err.to_string())
    }
}

impl From<tantivy::query::QueryParserError> for FlashgrepError {
    fn from(err: tantivy::query::QueryParserError) -> Self {
        FlashgrepError::Search(err.to_string())
    }
}

impl From<r2d2::Error> for FlashgrepError {
    fn from(err: r2d2::Error) -> Self {
        FlashgrepError::Database(rusqlite::Error::SqliteFailure(
            rusqlite::ffi::Error {
                code: rusqlite::ErrorCode::Unknown,
                extended_code: 0,
            },
            Some(err.to_string()),
        ))
    }
}

impl From<tokio::task::JoinError> for FlashgrepError {
    fn from(err: tokio::task::JoinError) -> Self {
        FlashgrepError::Task(err.to_string())
    }
}

impl FlashgrepError {
    pub fn exit_code(&self) -> i32 {
        match self {
            FlashgrepError::Io(_) => 1,
            FlashgrepError::Database(_) => 2,
            FlashgrepError::Search(_) => 3,
            FlashgrepError::IndexNotFound(_) => 4,
            FlashgrepError::Index(_) => 4,
            FlashgrepError::Config(_) => 5,
            FlashgrepError::FileWatcher(_) => 6,
            FlashgrepError::McpServer(_) => 7,
            FlashgrepError::Task(_) => 8,
        }
    }
}

pub type FlashgrepResult<T> = Result<T, FlashgrepError>;
