use crate::db::models::{SearchResult, Symbol};
use crate::db::Database;
use crate::neural::{cosine_similarity, embed_text, ensure_model_cached, EMBEDDING_MODEL_ID};
use crate::path_utils::{normalize_glob_pattern, normalize_path_for_matching};
use crate::FlashgrepError;
use crate::FlashgrepResult;
use glob::{MatchOptions, Pattern};
use regex::{Regex, RegexBuilder};
use serde_json::Value;
use std::path::{Path, PathBuf};
use tantivy::query::QueryParser;
use tantivy::{Index, IndexReader, ReloadPolicy};
use tracing::debug;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QueryMode {
    Smart,
    Literal,
    Regex,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QueryRetrievalMode {
    Lexical,
    Semantic,
    Hybrid,
}

#[derive(Debug, Clone)]
pub struct QueryOptions {
    pub text: String,
    pub fixed_patterns: Vec<String>,
    pub limit: usize,
    pub mode: QueryMode,
    pub case_sensitive: bool,
    pub include: Vec<String>,
    pub exclude: Vec<String>,
    pub context: usize,
    pub offset: usize,
    pub retrieval_mode: QueryRetrievalMode,
}

impl QueryOptions {
    pub fn new(text: String, limit: usize) -> Self {
        Self {
            text,
            fixed_patterns: Vec::new(),
            limit: limit.max(1),
            mode: QueryMode::Smart,
            case_sensitive: true,
            include: Vec::new(),
            exclude: Vec::new(),
            context: 0,
            offset: 0,
            retrieval_mode: QueryRetrievalMode::Lexical,
        }
    }

    pub fn from_mcp_args(args: &Value) -> FlashgrepResult<Self> {
        let text = args
            .get("text")
            .and_then(Value::as_str)
            .unwrap_or("")
            .trim()
            .to_string();
        let fixed_patterns = vec_from_str_array(args.get("fixed_strings"))?;
        let limit = args.get("limit").and_then(Value::as_u64).unwrap_or(10) as usize;
        let mut mode = match args.get("mode").and_then(Value::as_str).unwrap_or("smart") {
            "smart" => QueryMode::Smart,
            "literal" => QueryMode::Literal,
            "regex" => QueryMode::Regex,
            other => {
                return Err(FlashgrepError::Config(format!(
                    "Invalid query mode '{}'. Expected one of: smart, literal, regex",
                    other
                )))
            }
        };

        if !fixed_patterns.is_empty() {
            mode = QueryMode::Literal;
        }

        let flags = args
            .get("regex_flags")
            .and_then(Value::as_str)
            .unwrap_or("");
        if mode != QueryMode::Regex && !flags.is_empty() {
            return Err(FlashgrepError::Config(
                "regex_flags is only valid when mode=regex".to_string(),
            ));
        }

        let include = vec_from_str_array(args.get("include"))?;
        let exclude = vec_from_str_array(args.get("exclude"))?;
        let context = args.get("context").and_then(Value::as_u64).unwrap_or(0) as usize;
        let offset = args.get("offset").and_then(Value::as_u64).unwrap_or(0) as usize;

        let retrieval_mode = match args
            .get("retrieval_mode")
            .and_then(Value::as_str)
            .unwrap_or("lexical")
        {
            "lexical" => QueryRetrievalMode::Lexical,
            "semantic" => QueryRetrievalMode::Semantic,
            "hybrid" => QueryRetrievalMode::Hybrid,
            other => {
                return Err(FlashgrepError::Config(format!(
                    "Invalid retrieval_mode '{}'. Expected one of: lexical, semantic, hybrid",
                    other
                )))
            }
        };

        let case_sensitive = if mode == QueryMode::Regex {
            !flags.contains('i')
        } else {
            args.get("case_sensitive")
                .and_then(Value::as_bool)
                .unwrap_or(true)
        };

        Ok(Self {
            text,
            fixed_patterns,
            limit: limit.max(1),
            mode,
            case_sensitive,
            include,
            exclude,
            context,
            offset,
            retrieval_mode,
        })
    }
}

#[derive(Debug, Clone)]
pub struct QueryResponse {
    pub results: Vec<SearchResult>,
    pub truncated: bool,
    pub scanned_files: usize,
    pub next_offset: Option<usize>,
}

/// Search engine for querying the index
pub struct Searcher {
    reader: IndexReader,
    query_parser: QueryParser,
    db: Database,
    flashgrep_paths: crate::config::paths::FlashgrepPaths,
}

impl Searcher {
    /// Create a new searcher
    pub fn new(index: &Index, db_path: &Path) -> FlashgrepResult<Self> {
        let reader = index
            .reader_builder()
            .reload_policy(ReloadPolicy::OnCommit)
            .try_into()?;
        let schema = index.schema();

        // Create query parser for the content field
        let content_field = schema.get_field("content").unwrap();
        let query_parser = QueryParser::for_index(index, vec![content_field]);

        let db = Database::open(db_path)?;

        let repo_root = db_path
            .parent()
            .and_then(|p| p.parent())
            .map(|p| p.to_path_buf())
            .ok_or_else(|| {
                FlashgrepError::Config(format!(
                    "Cannot infer repository root from metadata DB path {}",
                    db_path.display()
                ))
            })?;

        Ok(Self {
            reader,
            query_parser,
            db,
            flashgrep_paths: crate::config::paths::FlashgrepPaths::new(&repo_root),
        })
    }

    /// Search the index with a text query
    pub fn query(&self, text: &str, limit: usize) -> FlashgrepResult<Vec<SearchResult>> {
        let options = QueryOptions::new(text.to_string(), limit);
        let response = self.query_with_options(&options)?;
        Ok(response.results)
    }

    pub fn query_with_options(&self, options: &QueryOptions) -> FlashgrepResult<QueryResponse> {
        debug!(
            "Searching for: {} (limit: {}, mode={:?}, retrieval_mode={:?})",
            options.text, options.limit, options.mode, options.retrieval_mode
        );

        if options.text.is_empty() {
            return Ok(QueryResponse {
                results: Vec::new(),
                truncated: false,
                scanned_files: 0,
                next_offset: None,
            });
        }

        match options.retrieval_mode {
            QueryRetrievalMode::Lexical => self.query_lexical(options),
            QueryRetrievalMode::Semantic => self.query_semantic(options),
            QueryRetrievalMode::Hybrid => self.query_hybrid(options),
        }
    }

    fn query_lexical(&self, options: &QueryOptions) -> FlashgrepResult<QueryResponse> {
        let searcher = self.reader.searcher();
        let schema = searcher.schema();
        let file_path_field = schema.get_field("file_path").unwrap();
        let content_field = schema.get_field("content").unwrap();
        let start_line_field = schema.get_field("start_line").unwrap();
        let end_line_field = schema.get_field("end_line").unwrap();

        let include_patterns = compile_patterns(&options.include)?;
        let exclude_patterns = compile_patterns(&options.exclude)?;
        let regex = compile_query_regex(options)?;

        let query_text = match options.mode {
            QueryMode::Smart => options.text.clone(),
            QueryMode::Literal => {
                let mut fixed = options.fixed_patterns.clone();
                if fixed.is_empty() && !options.text.is_empty() {
                    fixed.push(options.text.clone());
                }
                if fixed.is_empty() {
                    options.text.clone()
                } else {
                    fixed
                        .into_iter()
                        .map(|v| format!("\"{}\"", v.replace('"', "\\\"")))
                        .collect::<Vec<_>>()
                        .join(" OR ")
                }
            }
            QueryMode::Regex => options
                .text
                .split(|c: char| !c.is_alphanumeric())
                .find(|s| !s.is_empty())
                .unwrap_or(&options.text)
                .to_string(),
        };
        let query = self.query_parser.parse_query(&query_text)?;

        let target_count = options.offset.saturating_add(options.limit);
        let fetch_limit = target_count
            .saturating_mul(30)
            .max(target_count)
            .min(10_000);
        let top_docs = searcher.search(
            &query,
            &tantivy::collector::TopDocs::with_limit(fetch_limit),
        )?;

        let mut results = Vec::new();
        let mut scanned_files = 0usize;
        let mut matched = 0usize;

        for (score, doc_address) in top_docs {
            let doc = searcher.doc(doc_address)?;

            let file_path = doc
                .get_first(file_path_field)
                .and_then(|v| v.as_text())
                .map(PathBuf::from)
                .unwrap_or_default();

            if !path_matches(
                &file_path,
                &include_patterns,
                &exclude_patterns,
                options.case_sensitive,
            ) {
                continue;
            }

            let content = doc
                .get_first(content_field)
                .and_then(|v| v.as_text())
                .unwrap_or("")
                .to_string();

            if !matches_query(
                &content,
                &options.text,
                &options.fixed_patterns,
                options.case_sensitive,
                regex.as_ref(),
            ) {
                continue;
            }

            let start_line = doc
                .get_first(start_line_field)
                .and_then(|v| v.as_u64())
                .unwrap_or(0) as usize;

            let end_line = doc
                .get_first(end_line_field)
                .and_then(|v| v.as_u64())
                .unwrap_or(0) as usize;

            let preview = if options.context > 0 {
                render_context_preview(&file_path, start_line, end_line, options.context)
                    .unwrap_or_else(|| content.lines().take(3).collect::<Vec<_>>().join("\n"))
            } else {
                content.lines().take(3).collect::<Vec<_>>().join("\n")
            };

            scanned_files += 1;
            if matched < options.offset {
                matched += 1;
                continue;
            }

            results.push(SearchResult {
                file_path,
                start_line,
                end_line,
                symbol_name: None,
                relevance_score: score,
                preview,
                content: None,
            });

            if results.len() >= options.limit {
                break;
            }

            matched += 1;
        }

        let truncated = results.len() >= options.limit;
        let next_offset = if truncated {
            Some(options.offset.saturating_add(results.len()))
        } else {
            None
        };
        Ok(QueryResponse {
            results,
            truncated,
            scanned_files,
            next_offset,
        })
    }

    fn query_semantic(&self, options: &QueryOptions) -> FlashgrepResult<QueryResponse> {
        ensure_model_cached(&self.flashgrep_paths)?;

        let query_embedding = embed_text(&self.flashgrep_paths, &options.text)?;
        let include_patterns = compile_patterns(&options.include)?;
        let exclude_patterns = compile_patterns(&options.exclude)?;

        let mut candidates = self.db.get_semantic_chunks(EMBEDDING_MODEL_ID)?;
        candidates.retain(|chunk| {
            path_matches(
                &chunk.file_path,
                &include_patterns,
                &exclude_patterns,
                options.case_sensitive,
            )
        });

        let max_mtime = candidates
            .iter()
            .map(|c| c.last_modified)
            .max()
            .unwrap_or(0);

        let mut ranked: Vec<SearchResult> = candidates
            .iter()
            .map(|chunk| {
                let similarity = cosine_similarity(&query_embedding, &chunk.embedding);
                let path_depth = chunk.file_path.components().count() as f32;
                let depth_boost = 1.0 / (1.0 + path_depth.max(1.0));
                let recency_boost = if max_mtime > 0 {
                    ((chunk.last_modified as f32 / max_mtime as f32).clamp(0.0, 1.0)) * 0.05
                } else {
                    0.0
                };
                let score = similarity + (depth_boost * 0.05) + recency_boost;

                let pinpoint = pinpoint_best_line(&chunk.content, &options.text);
                let (start_line, end_line, preview, pinpoint_boost) =
                    if let Some((offset, line)) = pinpoint {
                        let exact_line = chunk.start_line.saturating_add(offset);
                        (
                            exact_line, exact_line, line,
                            0.03, // prefer chunks where we can pinpoint a concrete line
                        )
                    } else {
                        (
                            chunk.start_line,
                            chunk.end_line,
                            chunk.content.lines().take(3).collect::<Vec<_>>().join("\n"),
                            0.0,
                        )
                    };

                SearchResult {
                    file_path: chunk.file_path.clone(),
                    start_line,
                    end_line,
                    symbol_name: None,
                    relevance_score: score + pinpoint_boost,
                    preview,
                    content: None,
                }
            })
            .collect();

        ranked.sort_by(|a, b| {
            b.relevance_score
                .total_cmp(&a.relevance_score)
                .then_with(|| a.file_path.cmp(&b.file_path))
                .then_with(|| a.start_line.cmp(&b.start_line))
        });

        let scanned_files = ranked.len();
        let window: Vec<SearchResult> = ranked
            .into_iter()
            .skip(options.offset)
            .take(options.limit)
            .collect();
        let truncated = scanned_files > options.offset.saturating_add(window.len());
        let next_offset = if truncated {
            Some(options.offset.saturating_add(window.len()))
        } else {
            None
        };

        Ok(QueryResponse {
            results: window,
            truncated,
            scanned_files,
            next_offset,
        })
    }

    fn query_hybrid(&self, options: &QueryOptions) -> FlashgrepResult<QueryResponse> {
        let mut lexical_options = options.clone();
        lexical_options.retrieval_mode = QueryRetrievalMode::Lexical;
        lexical_options.limit = options.limit.saturating_mul(2).max(10);
        lexical_options.offset = 0;

        let mut semantic_options = options.clone();
        semantic_options.retrieval_mode = QueryRetrievalMode::Semantic;
        semantic_options.limit = options.limit.saturating_mul(2).max(10);
        semantic_options.offset = 0;

        let lexical = match self.query_lexical(&lexical_options) {
            Ok(result) => result,
            Err(FlashgrepError::Search(msg))
                if lexical_options.mode == QueryMode::Smart && msg.contains("Syntax Error") =>
            {
                let mut fallback = lexical_options.clone();
                fallback.mode = QueryMode::Literal;
                self.query_lexical(&fallback)?
            }
            Err(err) => return Err(err),
        };
        let semantic = self.query_semantic(&semantic_options)?;

        let mut merged: Vec<SearchResult> = Vec::new();
        for r in lexical.results {
            merged.push(SearchResult {
                relevance_score: r.relevance_score + 0.15,
                ..r
            });
        }
        for r in semantic.results {
            if let Some(existing) = merged.iter_mut().find(|m| {
                m.file_path == r.file_path
                    && m.start_line == r.start_line
                    && m.end_line == r.end_line
            }) {
                existing.relevance_score = existing.relevance_score.max(r.relevance_score + 0.2);
            } else {
                merged.push(r);
            }
        }

        merged.sort_by(|a, b| {
            b.relevance_score
                .total_cmp(&a.relevance_score)
                .then_with(|| a.file_path.cmp(&b.file_path))
                .then_with(|| a.start_line.cmp(&b.start_line))
        });

        let scanned_files = merged.len();
        let window: Vec<SearchResult> = merged
            .into_iter()
            .skip(options.offset)
            .take(options.limit)
            .collect();
        let truncated = scanned_files > options.offset.saturating_add(window.len());
        let next_offset = if truncated {
            Some(options.offset.saturating_add(window.len()))
        } else {
            None
        };

        Ok(QueryResponse {
            results: window,
            truncated,
            scanned_files,
            next_offset,
        })
    }

    /// Get a specific slice of a file by line range
    pub fn get_slice(
        &self,
        file_path: &Path,
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

fn vec_from_str_array(value: Option<&Value>) -> FlashgrepResult<Vec<String>> {
    let mut items = Vec::new();
    if let Some(values) = value.and_then(Value::as_array) {
        for item in values {
            let s = item
                .as_str()
                .ok_or_else(|| FlashgrepError::Config("Expected array of strings".to_string()))?;
            let trimmed = s.trim();
            if !trimmed.is_empty() {
                items.push(trimmed.to_string());
            }
        }
    }
    Ok(items)
}

fn compile_patterns(patterns: &[String]) -> FlashgrepResult<Vec<Pattern>> {
    patterns
        .iter()
        .map(|p| normalize_glob_pattern(p))
        .filter(|p| !p.is_empty())
        .map(|p| {
            Pattern::new(&p)
                .map_err(|e| FlashgrepError::Config(format!("Invalid glob pattern '{}': {}", p, e)))
        })
        .collect()
}

fn path_matches(
    path: &Path,
    include: &[Pattern],
    exclude: &[Pattern],
    case_sensitive: bool,
) -> bool {
    let normalized = normalize_path_for_matching(path);
    let opts = MatchOptions {
        case_sensitive,
        require_literal_separator: false,
        require_literal_leading_dot: false,
    };

    let include_ok = if include.is_empty() {
        true
    } else {
        include
            .iter()
            .any(|p| p.matches_with(&normalized, opts) || p.matches_path_with(path, opts))
    };
    if !include_ok {
        return false;
    }

    !exclude
        .iter()
        .any(|p| p.matches_with(&normalized, opts) || p.matches_path_with(path, opts))
}

fn pinpoint_best_line(content: &str, query: &str) -> Option<(usize, String)> {
    let query_norm = query.trim().to_ascii_lowercase();
    if query_norm.is_empty() {
        return None;
    }

    let query_terms: Vec<&str> = query_norm
        .split(|c: char| !c.is_alphanumeric() && c != '_')
        .filter(|s| !s.is_empty())
        .collect();

    let mut best: Option<(usize, i32, String)> = None;
    for (idx, line) in content.lines().enumerate() {
        let line_norm = line.to_ascii_lowercase();
        let mut score = 0i32;

        if line_norm.contains(&query_norm) {
            score += 6;
        }

        for term in &query_terms {
            if term.len() >= 3 && line_norm.contains(term) {
                score += 1;
            }
        }

        if score <= 0 {
            continue;
        }

        match &best {
            Some((_, best_score, _)) if score <= *best_score => {}
            _ => {
                best = Some((idx, score, line.trim().to_string()));
            }
        }
    }

    best.map(|(idx, _, line)| (idx, line))
}

fn compile_query_regex(options: &QueryOptions) -> FlashgrepResult<Option<Regex>> {
    match options.mode {
        QueryMode::Regex => {
            let mut builder = RegexBuilder::new(&options.text);
            builder.case_insensitive(!options.case_sensitive);
            let regex = builder.build().map_err(|e| {
                FlashgrepError::Config(format!("Invalid regex pattern '{}': {}", options.text, e))
            })?;
            Ok(Some(regex))
        }
        _ => Ok(None),
    }
}

fn matches_query(
    content: &str,
    text: &str,
    fixed_patterns: &[String],
    case_sensitive: bool,
    regex: Option<&Regex>,
) -> bool {
    if let Some(re) = regex {
        return re.is_match(content);
    }

    if !fixed_patterns.is_empty() {
        if case_sensitive {
            return fixed_patterns.iter().any(|p| content.contains(p));
        }

        let content_lower = content.to_lowercase();
        return fixed_patterns
            .iter()
            .map(|p| p.to_lowercase())
            .any(|p| content_lower.contains(&p));
    }

    if case_sensitive {
        content.contains(text)
    } else {
        content.to_lowercase().contains(&text.to_lowercase())
    }
}

fn render_context_preview(
    file_path: &Path,
    start_line: usize,
    end_line: usize,
    context: usize,
) -> Option<String> {
    let content = std::fs::read_to_string(file_path).ok()?;
    let lines: Vec<&str> = content.lines().collect();
    if lines.is_empty() {
        return None;
    }
    let start_idx = start_line.saturating_sub(1).saturating_sub(context);
    let end_idx = end_line
        .saturating_add(context)
        .min(lines.len())
        .max(start_line.min(lines.len()));
    if start_idx >= lines.len() || start_idx >= end_idx {
        return None;
    }
    Some(lines[start_idx..end_idx].join("\n"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn query_options_parse_defaults() {
        let opts = QueryOptions::from_mcp_args(&json!({"text": "main"})).expect("options");
        assert_eq!(opts.text, "main");
        assert!(opts.fixed_patterns.is_empty());
        assert_eq!(opts.limit, 10);
        assert_eq!(opts.mode, QueryMode::Smart);
        assert_eq!(opts.retrieval_mode, QueryRetrievalMode::Lexical);
        assert!(opts.case_sensitive);
    }

    #[test]
    fn query_options_accept_semantic_retrieval_mode() {
        let opts = QueryOptions::from_mcp_args(&json!({
            "text": "find auth middleware",
            "retrieval_mode": "semantic"
        }))
        .expect("options");
        assert_eq!(opts.retrieval_mode, QueryRetrievalMode::Semantic);
    }

    #[test]
    fn query_options_fixed_strings_force_literal_mode() {
        let opts = QueryOptions::from_mcp_args(&json!({
            "text": "main",
            "mode": "smart",
            "fixed_strings": ["foo", "bar"]
        }))
        .expect("options");
        assert_eq!(opts.mode, QueryMode::Literal);
        assert_eq!(opts.fixed_patterns.len(), 2);
    }

    #[test]
    fn query_options_reject_regex_flags_without_regex_mode() {
        let err = QueryOptions::from_mcp_args(&json!({
            "text": "main",
            "mode": "literal",
            "regex_flags": "i"
        }))
        .expect_err("expected validation error");
        assert!(err.to_string().contains("regex_flags"));
    }

    #[test]
    fn query_options_regex_mode_uses_case_from_flags() {
        let opts = QueryOptions::from_mcp_args(&json!({
            "text": "foo.*bar",
            "mode": "regex",
            "regex_flags": "i"
        }))
        .expect("options");
        assert!(!opts.case_sensitive);
    }

    #[test]
    fn query_options_accept_offset_for_continuation() {
        let opts = QueryOptions::from_mcp_args(&json!({
            "text": "main",
            "limit": 10,
            "offset": 25
        }))
        .expect("options");
        assert_eq!(opts.offset, 25);
    }

    #[test]
    fn matches_query_supports_multi_fixed_patterns_case_insensitive() {
        let fixed = vec!["HELLO".to_string(), "missing".to_string()];
        assert!(matches_query("say hello world", "", &fixed, false, None));
    }

    #[test]
    fn compile_patterns_normalizes_windows_style_separator() {
        let compiled = compile_patterns(&["src\\**\\*.rs".to_string()]).expect("patterns");
        assert_eq!(compiled.len(), 1);
        assert!(compiled[0].matches("src/main.rs"));
    }

    #[test]
    fn pinpoint_best_line_prefers_line_with_query_terms() {
        let content =
            "package main\nfn unrelated() {}\nflag.Int(\"dim\", 32, \"vector dimension\")\n";
        let pinpoint = pinpoint_best_line(content, "usage text2vec dim flag").expect("pinpoint");
        assert_eq!(pinpoint.0, 2);
        assert!(pinpoint.1.contains("flag.Int"));
    }
}
