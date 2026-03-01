use crate::db::models::{SearchResult, Symbol};
use crate::db::Database;
use crate::path_utils::{normalize_glob_pattern, normalize_path_for_matching};
use crate::FlashgrepError;
use crate::FlashgrepResult;
use glob::{MatchOptions, Pattern};
use regex::{Regex, RegexBuilder};
use serde_json::Value;
use std::path::PathBuf;
use tantivy::query::QueryParser;
use tantivy::{Index, IndexReader, ReloadPolicy};
use tracing::debug;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QueryMode {
    Smart,
    Literal,
    Regex,
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
        let options = QueryOptions::new(text.to_string(), limit);
        let response = self.query_with_options(&options)?;
        Ok(response.results)
    }

    pub fn query_with_options(&self, options: &QueryOptions) -> FlashgrepResult<QueryResponse> {
        debug!(
            "Searching for: {} (limit: {}, mode={:?})",
            options.text, options.limit, options.mode
        );

        if options.text.is_empty() {
            return Ok(QueryResponse {
                results: Vec::new(),
                truncated: false,
                scanned_files: 0,
                next_offset: None,
            });
        }

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
    path: &PathBuf,
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
    file_path: &PathBuf,
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
        assert!(opts.case_sensitive);
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
}
