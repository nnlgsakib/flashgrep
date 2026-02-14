pub mod models;

use crate::FlashgrepResult;
use models::{Chunk, FileMetadata, IndexStats, Symbol};
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use std::path::PathBuf;
use tracing::debug;

/// Database wrapper with connection pooling
pub struct Database {
    pool: Pool<SqliteConnectionManager>,
}

impl Database {
    /// Open or create the database at the given path with optimizations
    pub fn open(path: &PathBuf) -> FlashgrepResult<Self> {
        let manager = SqliteConnectionManager::file(path);
        let pool = Pool::builder()
            .max_size(10) // Max 10 connections in pool
            .build(manager)?;

        let db = Self { pool };
        db.init_schema()?;
        db.optimize()?;

        Ok(db)
    }

    /// Initialize the database schema
    fn init_schema(&self) -> FlashgrepResult<()> {
        let conn = self.pool.get()?;

        // Enable foreign keys
        conn.pragma_update(None, "foreign_keys", "ON")?;

        // Create files table
        conn.execute(
            "CREATE TABLE IF NOT EXISTS files (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                file_path TEXT UNIQUE NOT NULL,
                file_size INTEGER NOT NULL,
                last_modified INTEGER NOT NULL,
                language TEXT
            )",
            [],
        )?;

        // Create chunks table
        conn.execute(
            "CREATE TABLE IF NOT EXISTS chunks (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                file_path TEXT NOT NULL,
                start_line INTEGER NOT NULL,
                end_line INTEGER NOT NULL,
                content_hash TEXT NOT NULL,
                content TEXT NOT NULL,
                last_modified INTEGER NOT NULL,
                FOREIGN KEY (file_path) REFERENCES files(file_path) ON DELETE CASCADE
            )",
            [],
        )?;

        // Create index on file_path for chunks
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_chunks_file_path ON chunks(file_path)",
            [],
        )?;

        // Create symbols table
        conn.execute(
            "CREATE TABLE IF NOT EXISTS symbols (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                symbol_name TEXT NOT NULL,
                file_path TEXT NOT NULL,
                line_number INTEGER NOT NULL,
                symbol_type TEXT NOT NULL,
                FOREIGN KEY (file_path) REFERENCES files(file_path) ON DELETE CASCADE
            )",
            [],
        )?;

        // Create indexes for symbols
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_symbols_name ON symbols(symbol_name)",
            [],
        )?;
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_symbols_file_path ON symbols(file_path)",
            [],
        )?;

        Ok(())
    }

    /// Insert or update a file record
    pub fn insert_file(&self, file: &FileMetadata) -> FlashgrepResult<i64> {
        let conn = self.pool.get()?;
        conn.execute(
            "INSERT OR REPLACE INTO files (file_path, file_size, last_modified, language)
             VALUES (?1, ?2, ?3, ?4)",
            (
                file.file_path.to_string_lossy().to_string(),
                file.file_size as i64,
                file.last_modified,
                file.language.as_ref(),
            ),
        )?;
        Ok(conn.last_insert_rowid())
    }

    /// Apply performance optimizations
    fn optimize(&self) -> FlashgrepResult<()> {
        let conn = self.pool.get()?;

        // Enable WAL mode for better concurrent access
        conn.pragma_update(None, "journal_mode", "WAL")?;

        // Increase cache size to ~100MB (in pages of 4KB)
        conn.pragma_update(None, "cache_size", -25000)?;

        // Set synchronous mode to NORMAL for better performance (still safe with WAL)
        conn.pragma_update(None, "synchronous", "NORMAL")?;

        // Enable memory-mapped I/O (256MB)
        conn.pragma_update(None, "mmap_size", 268435456)?;

        // Set temp store to memory for better performance
        conn.pragma_update(None, "temp_store", "MEMORY")?;

        debug!("SQLite optimizations applied");
        Ok(())
    }

    /// Batch insert chunks (much faster than individual inserts)
    pub fn insert_chunks_batch(&self, chunks: &[Chunk]) -> FlashgrepResult<usize> {
        let mut conn = self.pool.get()?;
        let tx = conn.transaction()?;

        let mut count = 0;
        {
            let mut stmt = tx.prepare(
                "INSERT INTO chunks (file_path, start_line, end_line, content_hash, content, last_modified)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6)"
            )?;

            for chunk in chunks {
                stmt.execute([
                    chunk.file_path.to_string_lossy().to_string(),
                    chunk.start_line.to_string(),
                    chunk.end_line.to_string(),
                    chunk.content_hash.clone(),
                    chunk.content.clone(),
                    chunk.last_modified.to_string(),
                ])?;
                count += 1;
            }
        }

        tx.commit()?;
        Ok(count)
    }

    /// Insert a single chunk (for backward compatibility)
    pub fn insert_chunk(&self, chunk: &Chunk) -> FlashgrepResult<i64> {
        let conn = self.pool.get()?;
        conn.execute(
            "INSERT INTO chunks (file_path, start_line, end_line, content_hash, content, last_modified)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            [
                chunk.file_path.to_string_lossy().to_string(),
                chunk.start_line.to_string(),
                chunk.content_hash.clone(),
                chunk.content.clone(),
                chunk.last_modified.to_string(),
            ],
        )?;
        Ok(conn.last_insert_rowid())
    }

    /// Batch insert symbols (much faster than individual inserts)
    pub fn insert_symbols_batch(&self, symbols: &[Symbol]) -> FlashgrepResult<usize> {
        let mut conn = self.pool.get()?;
        let tx = conn.transaction()?;

        let mut count = 0;
        {
            let mut stmt = tx.prepare(
                "INSERT INTO symbols (symbol_name, file_path, line_number, symbol_type)
                 VALUES (?1, ?2, ?3, ?4)",
            )?;

            for symbol in symbols {
                stmt.execute([
                    symbol.symbol_name.clone(),
                    symbol.file_path.to_string_lossy().to_string(),
                    symbol.line_number.to_string(),
                    symbol.symbol_type.to_string(),
                ])?;
                count += 1;
            }
        }

        tx.commit()?;
        Ok(count)
    }

    /// Insert a single symbol (for backward compatibility)
    pub fn insert_symbol(&self, symbol: &Symbol) -> FlashgrepResult<i64> {
        let conn = self.pool.get()?;
        conn.execute(
            "INSERT INTO symbols (symbol_name, file_path, line_number, symbol_type)
             VALUES (?1, ?2, ?3, ?4)",
            [
                symbol.symbol_name.clone(),
                symbol.file_path.to_string_lossy().to_string(),
                symbol.line_number.to_string(),
                symbol.symbol_type.to_string(),
            ],
        )?;
        Ok(conn.last_insert_rowid())
    }

    /// Delete all chunks for a file
    pub fn delete_file_chunks(&self, file_path: &PathBuf) -> FlashgrepResult<usize> {
        let conn = self.pool.get()?;
        let count = conn.execute(
            "DELETE FROM chunks WHERE file_path = ?1",
            [file_path.to_string_lossy().to_string()],
        )?;
        Ok(count)
    }

    /// Delete all symbols for a file
    pub fn delete_file_symbols(&self, file_path: &PathBuf) -> FlashgrepResult<usize> {
        let conn = self.pool.get()?;
        let count = conn.execute(
            "DELETE FROM symbols WHERE file_path = ?1",
            [file_path.to_string_lossy().to_string()],
        )?;
        Ok(count)
    }

    /// Delete a file and all its associated chunks and symbols
    pub fn delete_file(&self, file_path: &PathBuf) -> FlashgrepResult<()> {
        let conn = self.pool.get()?;
        conn.execute(
            "DELETE FROM files WHERE file_path = ?1",
            [file_path.to_string_lossy().to_string()],
        )?;
        Ok(())
    }

    /// Delete multiple files and all associated chunks/symbols in one transaction.
    /// Returns number of file records deleted.
    pub fn delete_files_bulk(&self, file_paths: &[PathBuf]) -> FlashgrepResult<usize> {
        if file_paths.is_empty() {
            return Ok(0);
        }

        let mut conn = self.pool.get()?;
        let tx = conn.transaction()?;
        let mut deleted = 0usize;
        {
            let mut stmt = tx.prepare("DELETE FROM files WHERE file_path = ?1")?;
            for path in file_paths {
                deleted += stmt.execute([path.to_string_lossy().to_string()])?;
            }
        }

        tx.commit()?;
        Ok(deleted)
    }

    /// Check if a file needs reindexing (returns true if file is new or modified)
    pub fn needs_reindex(
        &self,
        file_path: &PathBuf,
        current_modified: i64,
    ) -> FlashgrepResult<bool> {
        let conn = self.pool.get()?;
        let path_str = file_path.to_string_lossy().to_string();

        let stored_modified: Option<i64> = conn
            .query_row(
                "SELECT last_modified FROM files WHERE file_path = ?1",
                [&path_str],
                |row| row.get(0),
            )
            .ok();

        match stored_modified {
            None => Ok(true), // File not in database, needs indexing
            Some(stored) => Ok(stored != current_modified), // Reindex if modified
        }
    }

    /// Get index statistics
    pub fn get_stats(&self) -> FlashgrepResult<IndexStats> {
        let conn = self.pool.get()?;

        let total_files: usize =
            conn.query_row("SELECT COUNT(*) FROM files", [], |row| row.get(0))?;

        let total_chunks: usize =
            conn.query_row("SELECT COUNT(*) FROM chunks", [], |row| row.get(0))?;

        let total_symbols: usize =
            conn.query_row("SELECT COUNT(*) FROM symbols", [], |row| row.get(0))?;

        let last_update: Option<i64> = conn
            .query_row("SELECT MAX(last_modified) FROM files", [], |row| row.get(0))
            .ok();

        // Calculate index size (simplified - just database file size)
        let index_size_bytes = conn
            .query_row(
                "SELECT page_count * page_size FROM pragma_page_count(), pragma_page_size()",
                [],
                |row| row.get(0),
            )
            .unwrap_or(0);

        Ok(IndexStats {
            total_files,
            total_chunks,
            total_symbols,
            index_size_bytes,
            last_update,
        })
    }

    /// Find symbols by name
    pub fn find_symbols_by_name(&self, name: &str) -> FlashgrepResult<Vec<Symbol>> {
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare(
            "SELECT id, symbol_name, file_path, line_number, symbol_type FROM symbols
             WHERE symbol_name = ?1",
        )?;

        let symbols = stmt
            .query_map([name], |row| {
                Ok(Symbol {
                    id: row.get(0)?,
                    symbol_name: row.get(1)?,
                    file_path: PathBuf::from(row.get::<_, String>(2)?),
                    line_number: row.get::<_, i64>(3)? as usize,
                    symbol_type: parse_symbol_type(&row.get::<_, String>(4)?),
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(symbols)
    }

    /// Get all indexed file paths
    pub fn get_all_files(&self) -> FlashgrepResult<Vec<PathBuf>> {
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare("SELECT file_path FROM files")?;
        let files = stmt
            .query_map([], |row| {
                let path: String = row.get(0)?;
                Ok(PathBuf::from(path))
            })?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(files)
    }

    /// Run VACUUM to optimize database file size
    pub fn vacuum(&self) -> FlashgrepResult<()> {
        let conn = self.pool.get()?;
        conn.execute("VACUUM", [])?;
        debug!("Database vacuumed");
        Ok(())
    }

    /// Analyze tables for better query planning
    pub fn analyze(&self) -> FlashgrepResult<()> {
        let conn = self.pool.get()?;
        conn.execute("ANALYZE", [])?;
        debug!("Database analyzed");
        Ok(())
    }

    /// Clear all data from the database
    /// Deletes all records from files, chunks, and symbols tables
    pub fn clear_all(&self) -> FlashgrepResult<()> {
        let conn = self.pool.get()?;

        // Delete from child tables first (though CASCADE should handle this)
        conn.execute("DELETE FROM symbols", [])?;
        conn.execute("DELETE FROM chunks", [])?;
        conn.execute("DELETE FROM files", [])?;

        debug!("Database cleared: all tables emptied");
        Ok(())
    }
}

fn parse_symbol_type(s: &str) -> models::SymbolType {
    use models::SymbolType;
    match s {
        "function" => SymbolType::Function,
        "class" => SymbolType::Class,
        "struct" => SymbolType::Struct,
        "interface" => SymbolType::Interface,
        "import" => SymbolType::Import,
        "export" => SymbolType::Export,
        "route" => SymbolType::Route,
        "sql" => SymbolType::SqlQuery,
        "public" => SymbolType::Public,
        "private" => SymbolType::Private,
        other => SymbolType::Other(other.to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_database_creation() -> FlashgrepResult<()> {
        let temp_dir = TempDir::new()?;
        let db_path = temp_dir.path().join("test.db");
        let db = Database::open(&db_path)?;

        let stats = db.get_stats()?;
        assert_eq!(stats.total_files, 0);
        assert_eq!(stats.total_chunks, 0);

        Ok(())
    }

    #[test]
    fn test_batch_insert() -> FlashgrepResult<()> {
        let temp_dir = TempDir::new()?;
        let db_path = temp_dir.path().join("test.db");
        let db = Database::open(&db_path)?;

        // Insert file first (required for foreign key constraint)
        let file = FileMetadata {
            id: None,
            file_path: PathBuf::from("test.rs"),
            file_size: 100,
            last_modified: 1234567890,
            language: Some("rust".to_string()),
        };
        db.insert_file(&file)?;

        let chunks = vec![
            Chunk::new(
                PathBuf::from("test.rs"),
                1,
                10,
                "fn main() {}".to_string(),
                1234567890,
            ),
            Chunk::new(
                PathBuf::from("test.rs"),
                11,
                20,
                "fn other() {}".to_string(),
                1234567890,
            ),
        ];

        let count = db.insert_chunks_batch(&chunks)?;
        assert_eq!(count, 2);

        let stats = db.get_stats()?;
        assert_eq!(stats.total_chunks, 2);

        Ok(())
    }

    #[test]
    fn test_delete_files_bulk_is_idempotent() -> FlashgrepResult<()> {
        let temp_dir = TempDir::new()?;
        let db_path = temp_dir.path().join("test.db");
        let db = Database::open(&db_path)?;

        let file_a = FileMetadata {
            id: None,
            file_path: PathBuf::from("a.rs"),
            file_size: 10,
            last_modified: 123,
            language: Some("rust".to_string()),
        };
        let file_b = FileMetadata {
            id: None,
            file_path: PathBuf::from("b.rs"),
            file_size: 20,
            last_modified: 124,
            language: Some("rust".to_string()),
        };
        db.insert_file(&file_a)?;
        db.insert_file(&file_b)?;

        let deleted_first =
            db.delete_files_bulk(&[PathBuf::from("a.rs"), PathBuf::from("b.rs")])?;
        assert_eq!(deleted_first, 2);

        let deleted_second =
            db.delete_files_bulk(&[PathBuf::from("a.rs"), PathBuf::from("b.rs")])?;
        assert_eq!(deleted_second, 0);

        Ok(())
    }
}
