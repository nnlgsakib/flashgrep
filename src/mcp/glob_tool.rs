use crate::{FlashgrepError, FlashgrepResult};
use glob::{MatchOptions, Pattern};
use serde_json::{json, Value};
use std::cmp::Ordering;
use std::path::{Path, PathBuf};
use std::time::UNIX_EPOCH;
use walkdir::WalkDir;

pub fn glob_input_schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            "pattern": {"type": "string", "description": "Primary include glob pattern"},
            "path": {"type": "string", "description": "Root directory to search from"},
            "include": {"type": "array", "items": {"type": "string"}, "description": "Additional include glob patterns"},
            "exclude": {"type": "array", "items": {"type": "string"}, "description": "Exclude glob patterns"},
            "extensions": {"type": "array", "items": {"type": "string"}, "description": "File extensions (rs or .rs)"},
            "max_depth": {"type": "integer", "minimum": 0, "description": "Maximum traversal depth from root"},
            "recursive": {"type": "boolean", "description": "Whether traversal recurses into subdirectories"},
            "include_hidden": {"type": "boolean", "description": "Include hidden files/directories"},
            "follow_symlinks": {"type": "boolean", "description": "Follow symbolic links"},
            "case_sensitive": {"type": "boolean", "description": "Case-sensitive glob matching"},
            "sort_by": {"type": "string", "enum": ["path", "name", "modified", "size"]},
            "sort_order": {"type": "string", "enum": ["asc", "desc"]},
            "limit": {"type": "integer", "minimum": 1, "description": "Maximum number of results"}
        }
    })
}

pub fn run_glob(arguments: &Value) -> FlashgrepResult<Value> {
    let opts = GlobOptions::from_args(arguments)?;
    let mut matches = Vec::new();

    let mut walker = WalkDir::new(&opts.root).follow_links(opts.follow_symlinks);
    if let Some(max_depth) = opts.max_depth {
        walker = walker.max_depth(max_depth + 1);
    }

    let include_patterns = compile_patterns(&opts.includes)?;
    let exclude_patterns = compile_patterns(&opts.excludes)?;

    for entry in walker
        .into_iter()
        .filter_entry(|e| entry_allowed(e.path(), &opts.root, opts.include_hidden))
    {
        let entry = match entry {
            Ok(e) => e,
            Err(_) => continue,
        };

        if !entry.file_type().is_file() {
            continue;
        }

        let rel_path = relative_unix_path(entry.path(), &opts.root);
        if rel_path.is_empty() {
            continue;
        }

        if !matches_any(&rel_path, &include_patterns, opts.case_sensitive) {
            continue;
        }
        if matches_any(&rel_path, &exclude_patterns, opts.case_sensitive) {
            continue;
        }

        if !extension_allowed(entry.path(), &opts.extensions) {
            continue;
        }

        let metadata = entry.metadata().ok();
        let size = metadata.as_ref().map(|m| m.len()).unwrap_or(0);
        let modified = metadata
            .as_ref()
            .and_then(|m| m.modified().ok())
            .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
            .map(|d| d.as_secs())
            .unwrap_or(0);

        matches.push(GlobMatch {
            file_path: entry.path().to_string_lossy().to_string(),
            name: entry.file_name().to_string_lossy().to_string(),
            rel_path,
            size,
            modified,
        });
    }

    sort_matches(&mut matches, opts.sort_by, opts.sort_order);

    if let Some(limit) = opts.limit {
        matches.truncate(limit);
    }

    Ok(json!({
        "results": matches.iter().map(|m| json!({
            "file_path": m.file_path,
            "name": m.name,
            "relative_path": m.rel_path,
            "size": m.size,
            "modified_unix": m.modified
        })).collect::<Vec<_>>(),
        "total": matches.len(),
        "options": {
            "root": opts.root.to_string_lossy(),
            "includes": opts.includes,
            "excludes": opts.excludes,
            "extensions": opts.extensions,
            "max_depth": opts.max_depth,
            "recursive": opts.recursive,
            "include_hidden": opts.include_hidden,
            "follow_symlinks": opts.follow_symlinks,
            "case_sensitive": opts.case_sensitive,
            "sort_by": opts.sort_by.as_str(),
            "sort_order": opts.sort_order.as_str(),
            "limit": opts.limit,
        }
    }))
}

#[derive(Clone)]
struct GlobOptions {
    root: PathBuf,
    includes: Vec<String>,
    excludes: Vec<String>,
    extensions: Vec<String>,
    max_depth: Option<usize>,
    recursive: bool,
    include_hidden: bool,
    follow_symlinks: bool,
    case_sensitive: bool,
    sort_by: SortBy,
    sort_order: SortOrder,
    limit: Option<usize>,
}

impl GlobOptions {
    fn from_args(arguments: &Value) -> FlashgrepResult<Self> {
        let root = arguments
            .get("path")
            .and_then(Value::as_str)
            .map(PathBuf::from)
            .unwrap_or(std::env::current_dir()?);

        let pattern = arguments
            .get("pattern")
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|p| !p.is_empty())
            .map(ToString::to_string);

        let mut includes = vec_from_string_array(arguments.get("include"))?;
        if includes.is_empty() {
            includes.push(pattern.unwrap_or_else(|| "**/*".to_string()));
        }

        let excludes = vec_from_string_array(arguments.get("exclude"))?;
        let extensions = normalize_extensions(vec_from_string_array(arguments.get("extensions"))?);

        let recursive = arguments
            .get("recursive")
            .and_then(Value::as_bool)
            .unwrap_or(true);
        let user_max_depth = arguments
            .get("max_depth")
            .and_then(Value::as_u64)
            .map(|n| n as usize);
        let max_depth = if recursive {
            user_max_depth
        } else {
            Some(user_max_depth.unwrap_or(0))
        };

        let include_hidden = arguments
            .get("include_hidden")
            .and_then(Value::as_bool)
            .unwrap_or(false);
        let follow_symlinks = arguments
            .get("follow_symlinks")
            .and_then(Value::as_bool)
            .unwrap_or(false);
        let case_sensitive = arguments
            .get("case_sensitive")
            .and_then(Value::as_bool)
            .unwrap_or(true);
        let sort_by = SortBy::parse(arguments.get("sort_by").and_then(Value::as_str))?;
        let sort_order = SortOrder::parse(arguments.get("sort_order").and_then(Value::as_str))?;
        let limit = arguments
            .get("limit")
            .and_then(Value::as_u64)
            .map(|n| n as usize);

        if let Some(0) = limit {
            return Err(FlashgrepError::Config(
                "Invalid limit: must be greater than 0".to_string(),
            ));
        }
        if !root.exists() || !root.is_dir() {
            return Err(FlashgrepError::Config(format!(
                "Invalid path: '{}' is not a directory",
                root.display()
            )));
        }

        Ok(Self {
            root,
            includes,
            excludes,
            extensions,
            max_depth,
            recursive,
            include_hidden,
            follow_symlinks,
            case_sensitive,
            sort_by,
            sort_order,
            limit,
        })
    }
}

#[derive(Clone, Copy)]
enum SortBy {
    Path,
    Name,
    Modified,
    Size,
}

impl SortBy {
    fn parse(input: Option<&str>) -> FlashgrepResult<Self> {
        match input.unwrap_or("path") {
            "path" => Ok(Self::Path),
            "name" => Ok(Self::Name),
            "modified" => Ok(Self::Modified),
            "size" => Ok(Self::Size),
            other => Err(FlashgrepError::Config(format!(
                "Invalid sort_by '{}'. Expected one of: path, name, modified, size",
                other
            ))),
        }
    }

    fn as_str(&self) -> &'static str {
        match self {
            Self::Path => "path",
            Self::Name => "name",
            Self::Modified => "modified",
            Self::Size => "size",
        }
    }
}

#[derive(Clone, Copy)]
enum SortOrder {
    Asc,
    Desc,
}

impl SortOrder {
    fn parse(input: Option<&str>) -> FlashgrepResult<Self> {
        match input.unwrap_or("asc") {
            "asc" => Ok(Self::Asc),
            "desc" => Ok(Self::Desc),
            other => Err(FlashgrepError::Config(format!(
                "Invalid sort_order '{}'. Expected one of: asc, desc",
                other
            ))),
        }
    }

    fn as_str(&self) -> &'static str {
        match self {
            Self::Asc => "asc",
            Self::Desc => "desc",
        }
    }
}

struct GlobMatch {
    file_path: String,
    rel_path: String,
    name: String,
    size: u64,
    modified: u64,
}

fn vec_from_string_array(value: Option<&Value>) -> FlashgrepResult<Vec<String>> {
    let mut result = Vec::new();
    if let Some(array) = value.and_then(Value::as_array) {
        for v in array {
            let s = v
                .as_str()
                .ok_or_else(|| FlashgrepError::Config("Expected array of strings".to_string()))?;
            if !s.trim().is_empty() {
                result.push(s.trim().to_string());
            }
        }
    }
    Ok(result)
}

fn normalize_extensions(exts: Vec<String>) -> Vec<String> {
    exts.into_iter()
        .map(|e| e.trim().trim_start_matches('.').to_ascii_lowercase())
        .filter(|e| !e.is_empty())
        .collect()
}

fn compile_patterns(patterns: &[String]) -> FlashgrepResult<Vec<Pattern>> {
    patterns
        .iter()
        .map(|p| {
            Pattern::new(p)
                .map_err(|e| FlashgrepError::Config(format!("Invalid glob pattern '{}': {}", p, e)))
        })
        .collect()
}

fn entry_allowed(path: &Path, root: &Path, include_hidden: bool) -> bool {
    if include_hidden {
        return true;
    }

    let rel = path.strip_prefix(root).unwrap_or(path);

    !rel.components()
        .filter_map(|c| match c {
            std::path::Component::Normal(v) => Some(v.to_string_lossy().to_string()),
            _ => None,
        })
        .any(|c| c.starts_with('.'))
}

fn relative_unix_path(path: &Path, root: &Path) -> String {
    let rel = path.strip_prefix(root).unwrap_or(path);
    rel.to_string_lossy().replace('\\', "/")
}

fn matches_any(path: &str, patterns: &[Pattern], case_sensitive: bool) -> bool {
    let opts = MatchOptions {
        case_sensitive,
        require_literal_separator: false,
        require_literal_leading_dot: false,
    };
    patterns.iter().any(|p| p.matches_with(path, opts))
}

fn extension_allowed(path: &Path, extensions: &[String]) -> bool {
    if extensions.is_empty() {
        return true;
    }
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_ascii_lowercase());
    match ext {
        Some(e) => extensions.contains(&e),
        None => false,
    }
}

fn sort_matches(matches: &mut [GlobMatch], by: SortBy, order: SortOrder) {
    matches.sort_by(|a, b| {
        let ord = match by {
            SortBy::Path => a.rel_path.cmp(&b.rel_path),
            SortBy::Name => a
                .name
                .cmp(&b.name)
                .then_with(|| a.rel_path.cmp(&b.rel_path)),
            SortBy::Modified => a
                .modified
                .cmp(&b.modified)
                .then_with(|| a.rel_path.cmp(&b.rel_path)),
            SortBy::Size => a
                .size
                .cmp(&b.size)
                .then_with(|| a.rel_path.cmp(&b.rel_path)),
        };
        match order {
            SortOrder::Asc => ord,
            SortOrder::Desc => match ord {
                Ordering::Less => Ordering::Greater,
                Ordering::Greater => Ordering::Less,
                Ordering::Equal => Ordering::Equal,
            },
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn setup() -> (TempDir, PathBuf) {
        let temp = TempDir::new().expect("temp dir");
        let root = temp.path().to_path_buf();
        fs::create_dir_all(root.join("src/nested")).expect("create nested dirs");
        fs::create_dir_all(root.join("tests")).expect("create tests dir");
        fs::create_dir_all(root.join(".hidden")).expect("create hidden dir");
        fs::write(root.join("src/main.rs"), "fn main() {}\n").expect("write main");
        fs::write(root.join("src/lib.rs"), "pub fn lib() {}\n").expect("write lib");
        fs::write(root.join("src/nested/mod.rs"), "pub mod nested;\n").expect("write mod");
        fs::write(root.join("tests/test_main.rs"), "#[test] fn t() {}\n").expect("write test");
        fs::write(root.join(".hidden/secret.rs"), "secret\n").expect("write hidden");
        (temp, root)
    }

    #[test]
    fn filters_by_extension_and_exclude() {
        let (_tmp, root) = setup();
        let result = run_glob(&json!({
            "path": root,
            "pattern": "**/*",
            "extensions": ["rs"],
            "exclude": ["tests/**"]
        }))
        .expect("glob result");

        let paths = result["results"].as_array().expect("results array");
        assert!(paths.iter().all(|p| {
            p["relative_path"]
                .as_str()
                .expect("relative path")
                .ends_with(".rs")
        }));
        assert!(!paths.iter().any(|p| {
            p["relative_path"]
                .as_str()
                .expect("relative path")
                .starts_with("tests/")
        }));
    }

    #[test]
    fn respects_depth_bound() {
        let (_tmp, root) = setup();
        let result = run_glob(&json!({
            "path": root,
            "pattern": "**/*.rs",
            "max_depth": 1
        }))
        .expect("glob result");

        let paths = result["results"].as_array().expect("results array");
        assert!(!paths.iter().any(|p| {
            p["relative_path"]
                .as_str()
                .expect("relative path")
                .contains("nested/")
        }));
    }

    #[test]
    fn deterministic_sort_and_limit() {
        let (_tmp, root) = setup();
        let result = run_glob(&json!({
            "path": root,
            "pattern": "**/*.rs",
            "sort_by": "name",
            "sort_order": "asc",
            "limit": 2
        }))
        .expect("glob result");

        let paths = result["results"].as_array().expect("results array");
        assert_eq!(paths.len(), 2);
        let n0 = paths[0]["name"].as_str().expect("name 0");
        let n1 = paths[1]["name"].as_str().expect("name 1");
        assert!(n0 <= n1);
    }

    #[test]
    fn invalid_sort_option_returns_error() {
        let (_tmp, root) = setup();
        let err = run_glob(&json!({"path": root, "pattern": "**/*", "sort_by": "bad"}))
            .expect_err("expected invalid sort_by error");
        assert!(err.to_string().contains("Invalid sort_by"));
    }

    #[test]
    fn preserves_backward_compatible_defaults() {
        let (_tmp, root) = setup();
        let result = run_glob(&json!({"path": root, "pattern": "**/*.rs"})).expect("glob result");
        let paths = result["results"].as_array().expect("results array");
        assert!(!paths.is_empty());
    }
}
