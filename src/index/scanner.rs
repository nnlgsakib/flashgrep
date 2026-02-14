use crate::config::Config;
use crate::FlashgrepResult;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

/// Build a normalized repository-relative path for ignore checks.
/// Uses '/' separators across platforms.
pub fn normalize_repo_relative_path(path: &Path, root: &PathBuf) -> String {
    let rel = path.strip_prefix(root).unwrap_or(path);
    rel.components()
        .filter_map(|c| match c {
            std::path::Component::Normal(name) => Some(name.to_string_lossy().to_string()),
            _ => None,
        })
        .collect::<Vec<_>>()
        .join("/")
}

/// Default directories to ignore
pub const DEFAULT_IGNORED_DIRS: &[&str; 7] = &[
    ".git",
    "node_modules",
    "target",
    "dist",
    "build",
    "vendor",
    ".flashgrep",
];

/// Default file extensions to index
pub const DEFAULT_EXTENSIONS: &[&str; 11] = &[
    "go", "rs", "js", "ts", "py", "sol", "json", "md", "yaml", "yml", "toml",
];

/// Check if a directory should be ignored
pub fn should_ignore_directory(dir_name: &str, config: &Config) -> bool {
    config.ignored_dirs.contains(&dir_name.to_string()) || DEFAULT_IGNORED_DIRS.contains(&dir_name)
}

/// Check if a file should be indexed based on extension
pub fn should_index_file(path: &Path, config: &Config) -> bool {
    if let Some(ext) = path.extension() {
        let ext_str = ext.to_string_lossy().to_lowercase();
        config.extensions.contains(&ext_str)
    } else {
        false
    }
}

/// Check if a file appears to be binary
pub fn is_binary_file(path: &Path) -> FlashgrepResult<bool> {
    let content = std::fs::read(path)?;

    // Check for null bytes (common in binary files)
    if content.contains(&0) {
        return Ok(true);
    }

    // Check if content is valid UTF-8
    match String::from_utf8(content) {
        Ok(_) => Ok(false),
        Err(_) => Ok(true),
    }
}

/// Check if a file exceeds the size limit
pub fn is_oversized_file(path: &Path, max_size: u64) -> FlashgrepResult<bool> {
    let metadata = std::fs::metadata(path)?;
    Ok(metadata.len() > max_size)
}

/// Scans a repository for indexable files
pub struct FileScanner {
    root: PathBuf,
    config: Config,
    ignore_patterns: FlashgrepIgnore,
}

impl FileScanner {
    /// Create a new file scanner
    pub fn new(root: PathBuf, config: Config) -> Self {
        let ignore_patterns = FlashgrepIgnore::from_root(&root);
        Self {
            root,
            config,
            ignore_patterns,
        }
    }

    /// Scan the repository and return indexable files
    pub fn scan(&self) -> impl Iterator<Item = PathBuf> + '_ {
        WalkDir::new(&self.root)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
            .filter(move |e| self.should_include(e.path()))
            .map(|e| e.path().to_path_buf())
    }

    /// Check if a path should be included in the index
    fn should_include(&self, path: &Path) -> bool {
        // Check if it's in the flashgrep directory
        if path.components().any(|c| {
            if let std::path::Component::Normal(name) = c {
                name == ".flashgrep"
            } else {
                false
            }
        }) {
            return false;
        }

        // Check ignore patterns
        if self.ignore_patterns.is_ignored(path, &self.root) {
            return false;
        }

        // Check ignored directories from config/defaults on all path components
        for component in path.components() {
            if let std::path::Component::Normal(name) = component {
                if let Some(name_str) = name.to_str() {
                    if should_ignore_directory(name_str, &self.config) {
                        return false;
                    }
                }
            }
        }

        // Check if we should index this file type
        if !should_index_file(path, &self.config) {
            return false;
        }

        // Check file size
        match is_oversized_file(path, self.config.max_file_size) {
            Ok(true) => return false,
            Ok(false) => {}
            Err(_) => return false,
        }

        // Check if binary
        match is_binary_file(path) {
            Ok(true) => return false,
            Ok(false) => {}
            Err(_) => return false,
        }

        true
    }
}

/// Represents a .flashgrepignore file with gitignore-style patterns
#[derive(Debug, Default)]
pub struct FlashgrepIgnore {
    patterns: Vec<IgnorePattern>,
}

#[derive(Debug)]
struct IgnorePattern {
    pattern: String,
    is_negation: bool,
    is_directory_only: bool,
}

impl FlashgrepIgnore {
    /// Load ignore patterns from the root .flashgrepignore file
    pub fn from_root(root: &PathBuf) -> Self {
        let ignore_file = root.join(".flashgrepignore");
        if ignore_file.exists() {
            match Self::from_file(&ignore_file) {
                Ok(patterns) => patterns,
                Err(_) => Self::default(),
            }
        } else {
            Self::default()
        }
    }

    /// Load ignore patterns from a file
    pub fn from_file(path: &PathBuf) -> FlashgrepResult<Self> {
        let content = std::fs::read_to_string(path)?;
        let mut patterns = Vec::new();

        for line in content.lines() {
            let line = line.trim();

            // Skip empty lines and comments
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            let is_negation = line.starts_with('!');
            let is_directory_only = line.ends_with('/');

            let pattern = if is_negation { &line[1..] } else { line };

            let pattern = if is_directory_only {
                &pattern[..pattern.len() - 1]
            } else {
                pattern
            };

            patterns.push(IgnorePattern {
                pattern: pattern.to_string(),
                is_negation,
                is_directory_only,
            });
        }

        Ok(Self { patterns })
    }

    /// Check if a path is ignored
    pub fn is_ignored(&self, path: &Path, root: &PathBuf) -> bool {
        let relative_str = normalize_repo_relative_path(path, root);

        let mut ignored = false;

        for pattern in &self.patterns {
            let matches = if pattern.is_directory_only {
                Self::directory_match(&relative_str, &pattern.pattern)
            } else {
                Self::match_pattern(&relative_str, &pattern.pattern)
            };

            if matches {
                ignored = !pattern.is_negation;
            }
        }

        ignored
    }

    /// Match directory-only patterns against a normalized path.
    fn directory_match(path: &str, pattern: &str) -> bool {
        path == pattern || path.starts_with(&format!("{}/", pattern))
    }

    /// Match a path against a gitignore-style pattern
    fn match_pattern(path: &str, pattern: &str) -> bool {
        let path_parts: Vec<&str> = path.split('/').collect();
        let _pattern_parts: Vec<&str> = pattern.split('/').collect();

        // Simple glob matching
        if pattern.contains('*') || pattern.contains('?') {
            return Self::glob_match(path, pattern);
        }

        // Exact match or directory prefix match
        if pattern.starts_with('/') {
            // Anchored to root
            let anchored_pattern = &pattern[1..];
            path == anchored_pattern || path.starts_with(&format!("{}/", anchored_pattern))
        } else {
            // Match at any level
            path == pattern
                || path_parts.contains(&pattern)
                || path.starts_with(&format!("{}/", pattern))
                || path.ends_with(&format!("/{}", pattern))
        }
    }

    /// Simple glob pattern matching
    fn glob_match(path: &str, pattern: &str) -> bool {
        let mut pattern_chars = pattern.chars().peekable();
        let mut path_chars = path.chars().peekable();

        while let Some(p) = pattern_chars.next() {
            match p {
                '*' => {
                    // Match zero or more characters
                    if pattern_chars.peek().is_none() {
                        return true; // * at end matches everything
                    }
                    let next_p = pattern_chars.peek().copied().unwrap();
                    while let Some(c) = path_chars.next() {
                        if c == next_p {
                            break;
                        }
                    }
                }
                '?' => {
                    // Match exactly one character
                    if path_chars.next().is_none() {
                        return false;
                    }
                }
                c => {
                    // Match exact character
                    match path_chars.next() {
                        Some(pc) if pc == c => {}
                        _ => return false,
                    }
                }
            }
        }

        path_chars.next().is_none()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_should_ignore_directory() {
        let config = Config::default();
        assert!(should_ignore_directory(".git", &config));
        assert!(should_ignore_directory("node_modules", &config));
        assert!(!should_ignore_directory("src", &config));
    }

    #[test]
    fn test_should_index_file() {
        let config = Config::default();
        assert!(should_index_file(Path::new("test.rs"), &config));
        assert!(should_index_file(Path::new("test.py"), &config));
        assert!(!should_index_file(Path::new("test.exe"), &config));
    }

    #[test]
    fn test_is_oversized_file() -> FlashgrepResult<()> {
        let temp_dir = TempDir::new()?;
        let small_file = temp_dir.path().join("small.txt");
        std::fs::write(&small_file, "small")?;

        assert!(!is_oversized_file(&small_file, 100)?);
        assert!(is_oversized_file(&small_file, 1)?);

        Ok(())
    }

    #[test]
    fn test_flashgrep_ignore() -> FlashgrepResult<()> {
        let temp_dir = TempDir::new()?;
        let root = temp_dir.path().to_path_buf();

        // Create .flashgrepignore
        let ignore_content = r#"
# This is a comment
temp*
"#;
        std::fs::write(root.join(".flashgrepignore"), ignore_content)?;

        let ignore = FlashgrepIgnore::from_root(&root);

        // Test wildcard pattern
        assert!(
            ignore.is_ignored(&root.join("temp123"), &root),
            "temp* should be ignored"
        );
        assert!(
            ignore.is_ignored(&root.join("temp_file.rs"), &root),
            "temp* should be ignored"
        );
        assert!(
            !ignore.is_ignored(&root.join("src/main.rs"), &root),
            "src/main.rs should not be ignored"
        );

        Ok(())
    }

    #[test]
    fn test_file_scanner() -> FlashgrepResult<()> {
        let temp_dir = TempDir::new()?;
        let root = temp_dir.path().to_path_buf();

        // Create test files
        std::fs::create_dir_all(root.join("src"))?;
        std::fs::write(root.join("src/main.rs"), "fn main() {}")?;
        std::fs::write(root.join("readme.md"), "# Readme")?;
        std::fs::write(root.join("test.exe"), "binary")?; // Should be ignored
        std::fs::create_dir_all(root.join(".git"))?;
        std::fs::write(root.join(".git/config"), "git config")?; // Should be ignored

        let config = Config::default();
        let scanner = FileScanner::new(root.clone(), config);
        let files: Vec<_> = scanner.scan().collect();

        assert_eq!(files.len(), 2);
        assert!(files.iter().any(|p| p.ends_with("main.rs")));
        assert!(files.iter().any(|p| p.ends_with("readme.md")));

        Ok(())
    }

    #[test]
    fn test_directory_pattern_ignores_nested_files() -> FlashgrepResult<()> {
        let temp_dir = TempDir::new()?;
        let root = temp_dir.path().to_path_buf();

        std::fs::create_dir_all(root.join("src"))?;
        std::fs::write(root.join("src/main.rs"), "fn main() {}")?;

        std::fs::create_dir_all(root.join(".opencode/node_modules/zod"))?;
        std::fs::write(
            root.join(".opencode/node_modules/zod/core.js"),
            "export const z = 1;",
        )?;

        std::fs::write(root.join(".flashgrepignore"), ".opencode/\n")?;

        let config = Config::default();
        let scanner = FileScanner::new(root.clone(), config);
        let files: Vec<_> = scanner.scan().collect();

        assert!(files.iter().any(|p| p.ends_with("src/main.rs")));
        assert!(!files
            .iter()
            .any(|p| p.to_string_lossy().contains(".opencode")));

        Ok(())
    }
}
