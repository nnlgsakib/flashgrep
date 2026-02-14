use crate::chunking::Chunker;
use crate::config::paths::FlashgrepPaths;
use crate::config::Config;
use crate::db::models::{Chunk, FileMetadata};
use crate::db::Database;
use crate::index::scanner::FileScanner;
use crate::symbols::SymbolDetector;
use crate::FlashgrepResult;
use std::path::PathBuf;
use tantivy::schema::*;
use tantivy::{Index, IndexWriter};
use tracing::{debug, error, info};

/// Main indexing engine
pub struct Indexer {
    db: Database,
    index: Index,
    writer: IndexWriter,
    config: Config,
    paths: FlashgrepPaths,
    symbol_detector: SymbolDetector,
    chunker: Chunker,
}

impl Indexer {
    /// Create a new indexer for the given repository root
    pub fn new(repo_root: PathBuf) -> FlashgrepResult<Self> {
        let paths = FlashgrepPaths::new(&repo_root);

        // Create directories if they don't exist
        if !paths.exists() {
            paths.create()?;
        }

        // Load or create config
        let config = if paths.config_file().exists() {
            Config::from_file(&paths.config_file())?
        } else {
            let default = Config::default();
            default.to_file(&paths.config_file())?;
            default
        };

        // Open database
        let db = Database::open(&paths.metadata_db())?;

        // Create or open Tantivy index
        let index = Self::create_or_open_index(&paths.text_index_dir())?;
        let writer = index.writer(50_000_000)?; // 50MB buffer

        Ok(Self {
            db,
            index,
            writer,
            config,
            paths,
            symbol_detector: SymbolDetector::new(),
            chunker: Chunker::new(),
        })
    }

    /// Create the Tantivy index schema
    fn create_schema() -> Schema {
        let mut schema_builder = Schema::builder();

        // File path field
        schema_builder.add_text_field("file_path", TEXT | STORED);

        // Content field (tokenized for search)
        schema_builder.add_text_field("content", TEXT | STORED);

        // Start and end line numbers
        schema_builder.add_u64_field("start_line", STORED | FAST);
        schema_builder.add_u64_field("end_line", STORED | FAST);

        // Content hash for deduplication
        schema_builder.add_text_field("content_hash", STRING | STORED);

        // Last modified timestamp
        schema_builder.add_u64_field("last_modified", FAST);

        schema_builder.build()
    }

    /// Create or open the Tantivy index
    fn create_or_open_index(index_dir: &PathBuf) -> FlashgrepResult<Index> {
        let schema = Self::create_schema();

        if index_dir.exists() && index_dir.join("meta.json").exists() {
            // Open existing index
            Ok(Index::open_in_dir(index_dir)?)
        } else {
            // Create new index
            std::fs::create_dir_all(index_dir)?;
            Ok(Index::create_in_dir(index_dir, schema)?)
        }
    }

    /// Index a single file with batch inserts for better performance
    /// Skips files that haven't changed since last indexing
    pub fn index_file(&mut self, file_path: &PathBuf) -> FlashgrepResult<bool> {
        debug!("Checking file: {}", file_path.display());

        // Get file metadata first to check modification time
        let metadata = FileMetadata::from_path(file_path)?;
        let last_modified = metadata.last_modified;

        // Check if file needs reindexing
        if !self.db.needs_reindex(file_path, last_modified)? {
            debug!("Skipping unchanged file: {}", file_path.display());
            return Ok(false); // File unchanged, skipped
        }

        debug!("Indexing file: {}", file_path.display());

        // Read file content
        let content = std::fs::read_to_string(file_path)?;

        // Delete existing chunks and symbols for this file
        self.db.delete_file_chunks(file_path)?;
        self.db.delete_file_symbols(file_path)?;

        // Insert/update file record
        self.db.insert_file(&metadata)?;

        // Chunk the file
        let chunks = self
            .chunker
            .chunk_file(file_path.clone(), &content, last_modified);

        // Collect all symbols from all chunks
        let mut all_symbols = Vec::new();

        // Index each chunk and collect symbols
        for chunk in &chunks {
            // Detect symbols
            let symbols = self.symbol_detector.detect_in_chunk(
                &chunk.content,
                file_path.clone(),
                chunk.start_line,
            );
            all_symbols.extend(symbols);

            // Add to Tantivy index
            self.add_chunk_to_tantivy(chunk)?;
        }

        // Batch insert chunks (much faster than individual inserts)
        if !chunks.is_empty() {
            self.db.insert_chunks_batch(&chunks)?;
        }

        // Batch insert symbols (much faster than individual inserts)
        if !all_symbols.is_empty() {
            self.db.insert_symbols_batch(&all_symbols)?;
        }

        Ok(true) // File was indexed
    }

    /// Add a chunk to the Tantivy index
    fn add_chunk_to_tantivy(&mut self, chunk: &Chunk) -> FlashgrepResult<()> {
        let schema = self.index.schema();
        let file_path_field = schema.get_field("file_path").unwrap();
        let content_field = schema.get_field("content").unwrap();
        let start_line_field = schema.get_field("start_line").unwrap();
        let end_line_field = schema.get_field("end_line").unwrap();
        let content_hash_field = schema.get_field("content_hash").unwrap();
        let last_modified_field = schema.get_field("last_modified").unwrap();

        let mut doc = Document::default();
        doc.add_text(file_path_field, chunk.file_path.to_string_lossy());
        doc.add_text(content_field, &chunk.content);
        doc.add_u64(start_line_field, chunk.start_line as u64);
        doc.add_u64(end_line_field, chunk.end_line as u64);
        doc.add_text(content_hash_field, &chunk.content_hash);
        doc.add_u64(last_modified_field, chunk.last_modified as u64);

        self.writer.add_document(doc)?;

        Ok(())
    }

    /// Index the entire repository
    /// Only reindexes files that have changed since last indexing
    pub fn index_repository(&mut self, repo_root: &PathBuf) -> FlashgrepResult<IndexStats> {
        info!("Starting repository indexing: {}", repo_root.display());

        let scanner = FileScanner::new(repo_root.clone(), self.config.clone());
        let files: Vec<_> = scanner.scan().collect();
        let total_files = files.len();

        info!("Found {} files to check", total_files);

        let mut indexed = 0;
        let mut skipped = 0;
        let mut failed = 0;

        for (i, file_path) in files.iter().enumerate() {
            if i % 100 == 0 {
                info!("Processed {}/{} files...", i, total_files);
            }

            match self.index_file(file_path) {
                Ok(true) => indexed += 1,
                Ok(false) => skipped += 1,
                Err(e) => {
                    error!("Failed to index {}: {}", file_path.display(), e);
                    failed += 1;
                }
            }
        }

        // Commit the Tantivy writer
        self.writer.commit()?;

        info!(
            "Indexing complete: {} indexed, {} skipped (unchanged), {} failed",
            indexed, skipped, failed
        );

        self.get_stats()
    }

    /// Get index statistics
    pub fn get_stats(&self) -> FlashgrepResult<IndexStats> {
        self.db.get_stats()
    }

    /// Check if an index exists at the given path
    pub fn index_exists(repo_root: &PathBuf) -> bool {
        let paths = FlashgrepPaths::new(repo_root);
        paths.exists()
    }

    /// Clear the entire index
    pub fn clear_index(&mut self) -> FlashgrepResult<()> {
        info!("Clearing index...");

        // Clear Tantivy index
        self.writer.delete_all_documents()?;
        self.writer.commit()?;

        // Clear database (recreate it)
        drop(std::mem::replace(
            &mut self.db,
            Database::open(&self.paths.metadata_db())?,
        ));

        info!("Index cleared");
        Ok(())
    }

    /// Get the database reference
    pub fn db(&self) -> &Database {
        &self.db
    }

    /// Get the Tantivy index
    pub fn tantivy_index(&self) -> &Index {
        &self.index
    }
}

use crate::db::models::IndexStats;

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_indexer_creation() -> FlashgrepResult<()> {
        let temp_dir = TempDir::new()?;
        let repo_root = temp_dir.path().to_path_buf();

        let indexer = Indexer::new(repo_root)?;
        assert!(indexer.get_stats()?.total_files == 0);

        Ok(())
    }

    #[test]
    fn test_index_file() -> FlashgrepResult<()> {
        let temp_dir = TempDir::new()?;
        let repo_root = temp_dir.path().to_path_buf();

        // Create a test file
        std::fs::write(repo_root.join("test.rs"), "fn main() {}\n")?;

        let mut indexer = Indexer::new(repo_root.clone())?;
        indexer.index_file(&repo_root.join("test.rs"))?;

        let stats = indexer.get_stats()?;
        assert_eq!(stats.total_files, 1);
        assert!(stats.total_chunks > 0);

        Ok(())
    }
}
