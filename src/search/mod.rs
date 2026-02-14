use crate::db::models::{SearchResult, Symbol};
use crate::db::Database;
use crate::FlashgrepResult;
use std::path::PathBuf;
use tantivy::query::QueryParser;
use tantivy::{Index, IndexReader, ReloadPolicy};
use tracing::debug;

/// Search engine for querying the index
pub struct Searcher {
    reader: IndexReader,
    query_parser: QueryParser,
    db: Database,
}

impl Searcher {
    /// Create a new searcher
    pub fn new(index: &Index, db_path: &PathBuf) -> FlashgrepResult<Self> {
        let reader = index
            .reader_builder()
            .reload_policy(ReloadPolicy::OnCommit)
            .try_into()?;
        let schema = index.schema();

        // Create query parser for the content field
        let content_field = schema.get_field("content").unwrap();
        let query_parser = QueryParser::for_index(index, vec![content_field]);

        let db = Database::open(db_path)?;

        Ok(Self {
            reader,
            query_parser,
            db,
        })
    }

    /// Search the index with a text query
    pub fn query(&self, text: &str, limit: usize) -> FlashgrepResult<Vec<SearchResult>> {
        debug!("Searching for: {} (limit: {})", text, limit);

        let searcher = self.reader.searcher();
        let query = self.query_parser.parse_query(text)?;

        let schema = searcher.schema();
        let file_path_field = schema.get_field("file_path").unwrap();
        let content_field = schema.get_field("content").unwrap();
        let start_line_field = schema.get_field("start_line").unwrap();
        let end_line_field = schema.get_field("end_line").unwrap();

        let top_docs = searcher.search(&query, &tantivy::collector::TopDocs::with_limit(limit))?;

        let mut results = Vec::new();
        for (_score, doc_address) in top_docs {
            let doc = searcher.doc(doc_address)?;

            let file_path = doc
                .get_first(file_path_field)
                .and_then(|v| v.as_text())
                .map(PathBuf::from)
                .unwrap_or_default();

            let content = doc
                .get_first(content_field)
                .and_then(|v| v.as_text())
                .unwrap_or("")
                .to_string();

            let start_line = doc
                .get_first(start_line_field)
                .and_then(|v| v.as_u64())
                .unwrap_or(0) as usize;

            let end_line = doc
                .get_first(end_line_field)
                .and_then(|v| v.as_u64())
                .unwrap_or(0) as usize;

            // Create preview (first 3 lines)
            let preview = content.lines().take(3).collect::<Vec<_>>().join("\n");

            results.push(SearchResult {
                file_path,
                start_line,
                end_line,
                symbol_name: None,
                relevance_score: _score,
                preview,
                content: None,
            });
        }

        Ok(results)
    }

    /// Get a specific slice of a file by line range
    pub fn get_slice(
        &self,
        file_path: &PathBuf,
        start_line: usize,
        end_line: usize,
    ) -> FlashgrepResult<Option<String>> {
        if !file_path.exists() {
            return Ok(None);
        }

        let content = std::fs::read_to_string(file_path)?;
        let lines: Vec<&str> = content.lines().collect();

        let start = start_line.saturating_sub(1);
        let end = end_line.min(lines.len());

        if start >= lines.len() {
            return Ok(None);
        }

        let slice = lines[start..end].join("\n");
        Ok(Some(slice))
    }

    /// Find symbols by name
    pub fn get_symbol(&self, symbol_name: &str) -> FlashgrepResult<Vec<Symbol>> {
        self.db.find_symbols_by_name(symbol_name)
    }

    /// List all indexed files
    pub fn list_files(&self) -> FlashgrepResult<Vec<PathBuf>> {
        self.db.get_all_files()
    }
}
