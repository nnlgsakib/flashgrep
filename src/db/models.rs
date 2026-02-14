use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Represents a chunk of a file for indexing
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Chunk {
    /// Unique identifier for the chunk
    pub id: Option<i64>,

    /// Path to the source file
    pub file_path: PathBuf,

    /// Starting line number (1-indexed)
    pub start_line: usize,

    /// Ending line number (1-indexed, inclusive)
    pub end_line: usize,

    /// SHA256 hash of the content
    pub content_hash: String,

    /// The actual content of the chunk
    pub content: String,

    /// Last modified timestamp of the source file
    pub last_modified: i64,
}

impl Chunk {
    /// Create a new chunk
    pub fn new(
        file_path: PathBuf,
        start_line: usize,
        end_line: usize,
        content: String,
        last_modified: i64,
    ) -> Self {
        let content_hash = calculate_hash(&content);
        Self {
            id: None,
            file_path,
            start_line,
            end_line,
            content_hash,
            content,
            last_modified,
        }
    }

    /// Get the number of lines in this chunk
    pub fn line_count(&self) -> usize {
        self.end_line.saturating_sub(self.start_line) + 1
    }
}

/// Represents a detected symbol in the code
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Symbol {
    /// Unique identifier for the symbol
    pub id: Option<i64>,

    /// Name of the symbol
    pub symbol_name: String,

    /// Path to the file containing the symbol
    pub file_path: PathBuf,

    /// Line number where the symbol is defined (1-indexed)
    pub line_number: usize,

    /// Type of symbol (function, class, import, etc.)
    pub symbol_type: SymbolType,
}

/// Types of symbols that can be detected
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum SymbolType {
    Function,
    Class,
    Struct,
    Interface,
    Import,
    Export,
    Route,
    SqlQuery,
    Public,
    Private,
    Other(String),
}

impl std::fmt::Display for SymbolType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SymbolType::Function => write!(f, "function"),
            SymbolType::Class => write!(f, "class"),
            SymbolType::Struct => write!(f, "struct"),
            SymbolType::Interface => write!(f, "interface"),
            SymbolType::Import => write!(f, "import"),
            SymbolType::Export => write!(f, "export"),
            SymbolType::Route => write!(f, "route"),
            SymbolType::SqlQuery => write!(f, "sql"),
            SymbolType::Public => write!(f, "public"),
            SymbolType::Private => write!(f, "private"),
            SymbolType::Other(s) => write!(f, "{}", s),
        }
    }
}

/// Metadata about an indexed file
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FileMetadata {
    /// Unique identifier for the file
    pub id: Option<i64>,

    /// Path to the file
    pub file_path: PathBuf,

    /// Size of the file in bytes
    pub file_size: u64,

    /// Last modified timestamp
    pub last_modified: i64,

    /// Detected programming language
    pub language: Option<String>,
}

impl FileMetadata {
    /// Detect language from file extension
    pub fn detect_language(path: &PathBuf) -> Option<String> {
        path.extension().and_then(|ext| ext.to_str()).map(|ext| {
            match ext.to_lowercase().as_str() {
                "rs" => "rust",
                "go" => "go",
                "js" => "javascript",
                "ts" => "typescript",
                "py" => "python",
                "sol" => "solidity",
                "json" => "json",
                "md" => "markdown",
                "yaml" | "yml" => "yaml",
                "toml" => "toml",
                _ => "unknown",
            }
            .to_string()
        })
    }

    /// Create metadata from a file path
    pub fn from_path(path: &PathBuf) -> anyhow::Result<Self> {
        let metadata = std::fs::metadata(path)?;
        let file_size = metadata.len();
        let last_modified = metadata
            .modified()?
            .duration_since(std::time::UNIX_EPOCH)?
            .as_secs() as i64;
        let language = Self::detect_language(path);

        Ok(Self {
            id: None,
            file_path: path.clone(),
            file_size,
            last_modified,
            language,
        })
    }
}

/// Result of a search query
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    /// Path to the file containing the match
    pub file_path: PathBuf,

    /// Starting line number (1-indexed)
    pub start_line: usize,

    /// Ending line number (1-indexed)
    pub end_line: usize,

    /// Name of the symbol if detected in this chunk
    pub symbol_name: Option<String>,

    /// Relevance score (higher is better)
    pub relevance_score: f32,

    /// Preview of the content (first few lines)
    pub preview: String,

    /// The actual content (if explicitly requested)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
}

/// Statistics about the index
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexStats {
    /// Total number of indexed files
    pub total_files: usize,

    /// Total number of chunks
    pub total_chunks: usize,

    /// Total number of detected symbols
    pub total_symbols: usize,

    /// Size of the index in bytes
    pub index_size_bytes: u64,

    /// Timestamp of the last index update
    pub last_update: Option<i64>,
}

/// Calculate SHA256 hash of content
fn calculate_hash(content: &str) -> String {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(content.as_bytes());
    hex::encode(hasher.finalize())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_chunk_creation() {
        let chunk = Chunk::new(
            PathBuf::from("test.rs"),
            1,
            10,
            "fn main() {}".to_string(),
            1234567890,
        );
        assert_eq!(chunk.start_line, 1);
        assert_eq!(chunk.end_line, 10);
        assert_eq!(chunk.line_count(), 10);
        assert!(!chunk.content_hash.is_empty());
    }

    #[test]
    fn test_symbol_type_display() {
        assert_eq!(SymbolType::Function.to_string(), "function");
        assert_eq!(SymbolType::Class.to_string(), "class");
    }

    #[test]
    fn test_detect_language() {
        assert_eq!(
            FileMetadata::detect_language(&PathBuf::from("test.rs")),
            Some("rust".to_string())
        );
        assert_eq!(
            FileMetadata::detect_language(&PathBuf::from("test.py")),
            Some("python".to_string())
        );
        assert_eq!(
            FileMetadata::detect_language(&PathBuf::from("test.unknown")),
            Some("unknown".to_string())
        );
    }

    #[test]
    fn test_file_metadata_from_path() -> anyhow::Result<()> {
        let mut temp_file = NamedTempFile::new()?;
        writeln!(temp_file, "test content")?;

        let metadata = FileMetadata::from_path(&temp_file.path().to_path_buf())?;
        assert!(metadata.file_size > 0);
        assert!(metadata.last_modified > 0);

        Ok(())
    }
}
