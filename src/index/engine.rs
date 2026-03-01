use crate::chunking::Chunker;
use crate::config::paths::FlashgrepPaths;
use crate::config::Config;
use crate::db::models::{Chunk, ChunkVector, FileMetadata, Symbol};
use crate::db::Database;
use crate::index::scanner::{FileScanner, FlashgrepIgnore};
use crate::neural::{embed_text, embed_texts, is_model_cached, EMBEDDING_MODEL_ID};
use crate::symbols::SymbolDetector;
use crate::FlashgrepResult;
use indicatif::{ProgressBar, ProgressStyle};
use rayon::prelude::*;
use std::io::IsTerminal;
use std::path::{Path, PathBuf};
use tantivy::schema::*;
use tantivy::{Index, IndexWriter, Term};
use tracing::{debug, error, info};

struct FileIndexPlan {
    file_path: PathBuf,
    metadata: FileMetadata,
}

struct PreparedFileIndex {
    file_path: PathBuf,
    metadata: FileMetadata,
    chunks: Vec<Chunk>,
    symbols: Vec<Symbol>,
    vectors: Vec<ChunkVector>,
}

/// Main indexing engine
pub struct Indexer {
    paths: FlashgrepPaths,
    semantic_vectors_enabled: bool,
    db: Database,
    index: Index,
    writer: IndexWriter,
    config: Config,
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
            semantic_vectors_enabled: is_model_cached(&paths).unwrap_or(false),
            paths,
            db,
            index,
            writer,
            config,
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
    fn create_or_open_index(index_dir: &Path) -> FlashgrepResult<Index> {
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
    pub fn index_file(&mut self, file_path: &Path) -> FlashgrepResult<bool> {
        debug!("Checking file: {}", file_path.display());

        if !self.semantic_vectors_enabled {
            self.semantic_vectors_enabled = is_model_cached(&self.paths).unwrap_or(false);
        }

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
        self.db.delete_file_vectors(file_path)?;

        // Insert/update file record
        self.db.insert_file(&metadata)?;

        // Chunk the file
        let chunks = self
            .chunker
            .chunk_file(file_path.to_path_buf(), &content, last_modified);

        // Collect all symbols from all chunks
        let mut all_symbols = Vec::new();

        // Index each chunk and collect symbols
        for chunk in &chunks {
            // Detect symbols
            let symbols = self.symbol_detector.detect_in_chunk(
                &chunk.content,
                file_path.to_path_buf(),
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

        // Build and persist semantic vectors for each chunk.
        if !chunks.is_empty() && self.semantic_vectors_enabled {
            let mut vectors = Vec::with_capacity(chunks.len());
            for chunk in &chunks {
                let embedding = match embed_text(&self.paths, &chunk.content) {
                    Ok(v) => v,
                    Err(err) => {
                        error!(
                            "Failed to embed chunk for {}:{}-{}: {}",
                            chunk.file_path.display(),
                            chunk.start_line,
                            chunk.end_line,
                            err
                        );
                        self.semantic_vectors_enabled = false;
                        vectors.clear();
                        break;
                    }
                };
                if embedding.is_empty() {
                    continue;
                }
                vectors.push(ChunkVector {
                    id: None,
                    file_path: chunk.file_path.clone(),
                    start_line: chunk.start_line,
                    end_line: chunk.end_line,
                    content_hash: chunk.content_hash.clone(),
                    embedding,
                    model_id: EMBEDDING_MODEL_ID.to_string(),
                    last_modified: chunk.last_modified,
                });
            }
            self.db.upsert_chunk_vectors_batch(&vectors)?;
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
    pub fn index_repository(&mut self, repo_root: &Path) -> FlashgrepResult<IndexStats> {
        info!("Starting repository indexing: {}", repo_root.display());

        let scanner = FileScanner::new(repo_root.to_path_buf(), self.config.clone());
        let files: Vec<_> = scanner.scan().collect();
        let total_files = files.len();

        info!("Found {} files to check", total_files);

        let progress = Self::create_progress_bar(total_files as u64);
        let mut indexed = 0;
        let mut skipped = 0;
        let mut failed = 0;
        let mut plans = Vec::new();

        for file_path in &files {
            match FileMetadata::from_path(file_path) {
                Ok(metadata) => match self.db.needs_reindex(file_path, metadata.last_modified) {
                    Ok(true) => plans.push(FileIndexPlan {
                        file_path: file_path.clone(),
                        metadata,
                    }),
                    Ok(false) => {
                        skipped += 1;
                        if let Some(pb) = &progress {
                            pb.inc(1);
                        }
                    }
                    Err(err) => {
                        failed += 1;
                        error!(
                            "Failed to check {} for reindex eligibility: {}",
                            file_path.display(),
                            err
                        );
                        if let Some(pb) = &progress {
                            pb.inc(1);
                        }
                    }
                },
                Err(err) => {
                    failed += 1;
                    error!(
                        "Failed to read metadata for {}: {}",
                        file_path.display(),
                        err
                    );
                    if let Some(pb) = &progress {
                        pb.inc(1);
                    }
                }
            }
        }

        let paths_for_workers = self.paths.clone();
        let semantic_vectors_enabled = self.semantic_vectors_enabled;
        let progress_for_workers = progress.clone();
        let prepared_results: Vec<(PathBuf, FlashgrepResult<PreparedFileIndex>)> = plans
            .into_par_iter()
            .map_init(
                || (Chunker::new(), SymbolDetector::new()),
                |(chunker, symbol_detector), plan| {
                    let path = plan.file_path.clone();
                    let result = Self::prepare_file_for_indexing(
                        &paths_for_workers,
                        semantic_vectors_enabled,
                        chunker,
                        symbol_detector,
                        plan,
                    );
                    if let Some(pb) = &progress_for_workers {
                        pb.inc(1);
                    }
                    (path, result)
                },
            )
            .collect();

        for (file_path, prepared_result) in prepared_results {
            match prepared_result {
                Ok(prepared) => {
                    if let Err(err) = self.persist_prepared_file(prepared) {
                        error!("Failed to index {}: {}", file_path.display(), err);
                        failed += 1;
                    } else {
                        indexed += 1;
                    }
                }
                Err(err) => {
                    error!("Failed to index {}: {}", file_path.display(), err);
                    failed += 1;
                }
            }
        }

        if let Some(pb) = &progress {
            pb.finish_with_message(format!(
                "Indexed {}/{} files ({} skipped, {} failed)",
                indexed, total_files, skipped, failed
            ));
        }

        // Commit the Tantivy writer
        self.writer.commit()?;

        info!(
            "Indexing complete: {} indexed, {} skipped (unchanged), {} failed",
            indexed, skipped, failed
        );

        self.get_stats()
    }

    fn create_progress_bar(total_files: u64) -> Option<ProgressBar> {
        if !std::io::stdout().is_terminal() {
            return None;
        }

        let progress_bar = ProgressBar::new(total_files);
        progress_bar.set_style(
            ProgressStyle::with_template(
                "{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {pos}/{len} ({percent}%) ETA {eta_precise}",
            )
            .unwrap()
            .progress_chars("#>-"),
        );
        progress_bar.enable_steady_tick(std::time::Duration::from_millis(100));
        Some(progress_bar)
    }

    fn prepare_file_for_indexing(
        paths: &FlashgrepPaths,
        semantic_vectors_enabled: bool,
        chunker: &Chunker,
        symbol_detector: &SymbolDetector,
        plan: FileIndexPlan,
    ) -> FlashgrepResult<PreparedFileIndex> {
        let content = std::fs::read_to_string(&plan.file_path)?;
        let chunks = chunker.chunk_file(
            plan.file_path.clone(),
            &content,
            plan.metadata.last_modified,
        );

        let mut symbols = Vec::new();
        for chunk in &chunks {
            symbols.extend(symbol_detector.detect_in_chunk(
                &chunk.content,
                plan.file_path.clone(),
                chunk.start_line,
            ));
        }

        let mut vectors = Vec::new();
        if semantic_vectors_enabled && !chunks.is_empty() {
            let chunk_contents: Vec<String> =
                chunks.iter().map(|chunk| chunk.content.clone()).collect();
            let embeddings = match embed_texts(paths, &chunk_contents) {
                Ok(embeddings) => embeddings,
                Err(err) => {
                    error!(
                        "Failed to embed chunks for {}: {}",
                        plan.file_path.display(),
                        err
                    );
                    Vec::new()
                }
            };

            vectors.reserve(chunks.len());
            for (chunk, embedding) in chunks.iter().zip(embeddings.into_iter()) {
                if embedding.is_empty() {
                    continue;
                }

                vectors.push(ChunkVector {
                    id: None,
                    file_path: chunk.file_path.clone(),
                    start_line: chunk.start_line,
                    end_line: chunk.end_line,
                    content_hash: chunk.content_hash.clone(),
                    embedding,
                    model_id: EMBEDDING_MODEL_ID.to_string(),
                    last_modified: chunk.last_modified,
                });
            }
        }

        Ok(PreparedFileIndex {
            file_path: plan.file_path,
            metadata: plan.metadata,
            chunks,
            symbols,
            vectors,
        })
    }

    fn persist_prepared_file(&mut self, prepared: PreparedFileIndex) -> FlashgrepResult<()> {
        self.db.delete_file_chunks(&prepared.file_path)?;
        self.db.delete_file_symbols(&prepared.file_path)?;
        self.db.delete_file_vectors(&prepared.file_path)?;
        self.db.insert_file(&prepared.metadata)?;

        for chunk in &prepared.chunks {
            self.add_chunk_to_tantivy(chunk)?;
        }

        if !prepared.chunks.is_empty() {
            self.db.insert_chunks_batch(&prepared.chunks)?;
        }

        if !prepared.vectors.is_empty() {
            self.db.upsert_chunk_vectors_batch(&prepared.vectors)?;
        }

        if !prepared.symbols.is_empty() {
            self.db.insert_symbols_batch(&prepared.symbols)?;
        }

        Ok(())
    }

    /// Get index statistics
    pub fn get_stats(&self) -> FlashgrepResult<IndexStats> {
        self.db.get_stats()
    }

    /// Check if an index exists at the given path
    pub fn index_exists(repo_root: &Path) -> bool {
        let paths = FlashgrepPaths::new(repo_root);
        paths.exists()
    }

    /// Clear the entire index
    pub fn clear_index(&mut self) -> FlashgrepResult<()> {
        info!("Clearing index...");

        // Clear Tantivy index (text search)
        self.writer.delete_all_documents()?;
        self.writer.commit()?;
        info!("Text index cleared");

        // Clear metadata database (file records, chunks, symbols)
        self.db.clear_all()?;
        info!("Metadata database cleared");

        let vectors_dir = self.paths.vectors_dir();
        if vectors_dir.exists() {
            let _ = std::fs::remove_dir_all(&vectors_dir);
        }
        std::fs::create_dir_all(&vectors_dir)?;

        info!("Index cleared successfully");
        Ok(())
    }

    /// Remove one file from both Tantivy and metadata store.
    pub fn remove_file_from_index(&mut self, file_path: &Path) -> FlashgrepResult<()> {
        let schema = self.index.schema();
        let file_path_field = schema.get_field("file_path").unwrap();
        self.writer.delete_term(Term::from_field_text(
            file_path_field,
            &file_path.to_string_lossy(),
        ));
        self.writer.commit()?;
        self.db.delete_file(file_path)?;
        Ok(())
    }

    /// Remove indexed files that are now ignored by ignore patterns.
    /// Returns (removed, kept) counts.
    pub fn reconcile_ignored_files(
        &mut self,
        repo_root: &Path,
        ignore_patterns: &FlashgrepIgnore,
    ) -> FlashgrepResult<(usize, usize)> {
        let indexed_files = self.db.get_all_files()?;
        let mut to_remove = Vec::new();
        let mut kept = 0usize;

        for path in indexed_files {
            if ignore_patterns.is_ignored(&path, repo_root) {
                to_remove.push(path);
            } else {
                kept += 1;
            }
        }

        if to_remove.is_empty() {
            return Ok((0, kept));
        }

        let removed = self.db.delete_files_bulk(&to_remove)?;

        // Rebuild index content to guarantee text index and metadata consistency
        // regardless of existing schema term/tokenization behavior.
        self.writer.delete_all_documents()?;
        self.writer.commit()?;
        self.db.clear_all()?;
        let _ = self.index_repository(repo_root)?;

        Ok((removed, kept))
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
