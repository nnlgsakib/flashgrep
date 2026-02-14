use crate::db::models::Chunk;
use std::path::PathBuf;

/// Maximum number of lines per chunk
pub const MAX_CHUNK_LINES: usize = 300;

/// Chunks file content into logical blocks
pub struct Chunker;

impl Chunker {
    /// Create a new chunker
    pub fn new() -> Self {
        Self
    }

    /// Chunk a file's content
    pub fn chunk_file(&self, file_path: PathBuf, content: &str, last_modified: i64) -> Vec<Chunk> {
        let lines: Vec<&str> = content.lines().collect();
        let mut chunks = Vec::new();
        let mut current_start = 0;

        while current_start < lines.len() {
            let (chunk_end, chunk_lines) = self.find_chunk_boundary(&lines, current_start);

            let content = chunk_lines.join("\n");
            let chunk = Chunk::new(
                file_path.clone(),
                current_start + 1, // 1-indexed
                chunk_end,         // 1-indexed, inclusive
                content,
                last_modified,
            );

            chunks.push(chunk);
            current_start = chunk_end; // Next chunk starts where this one ended
        }

        chunks
    }

    /// Find the boundary for the next chunk
    fn find_chunk_boundary(&self, lines: &[&str], start: usize) -> (usize, Vec<String>) {
        let max_end = (start + MAX_CHUNK_LINES).min(lines.len());
        let mut end = start;
        let mut bracket_depth = 0;
        let mut last_blank_line = None;

        for i in start..max_end {
            let line = lines[i].trim();

            // Track bracket depth
            for c in line.chars() {
                match c {
                    '{' | '[' | '(' => bracket_depth += 1,
                    '}' | ']' | ')' => bracket_depth -= 1,
                    _ => {}
                }
            }

            // Track blank lines
            if line.is_empty() {
                last_blank_line = Some(i);
            }

            // If bracket depth returns to 0 and we have a blank line, consider splitting
            if bracket_depth == 0 && last_blank_line == Some(i) {
                end = i + 1;
                break;
            }

            end = i + 1;
        }

        // If we haven't found a good boundary and we're at max size, split anyway
        if end == start {
            end = max_end;
        }

        let chunk_lines: Vec<String> = lines[start..end].iter().map(|s| s.to_string()).collect();

        (end, chunk_lines)
    }

    /// Split content by blank lines only
    pub fn chunk_by_blank_lines(
        &self,
        file_path: PathBuf,
        content: &str,
        last_modified: i64,
    ) -> Vec<Chunk> {
        let lines: Vec<&str> = content.lines().collect();
        let mut chunks = Vec::new();
        let mut current_start = 0;
        let mut current_lines = Vec::new();

        for (i, line) in lines.iter().enumerate() {
            current_lines.push(line.to_string());

            // Split at blank lines or max size
            if (line.trim().is_empty() || current_lines.len() >= MAX_CHUNK_LINES)
                && !current_lines.is_empty()
            {
                let chunk_content = current_lines.join("\n");
                let chunk = Chunk::new(
                    file_path.clone(),
                    current_start + 1,
                    i + 1,
                    chunk_content,
                    last_modified,
                );
                chunks.push(chunk);
                current_start = i + 1;
                current_lines.clear();
            }
        }

        // Don't forget the last chunk
        if !current_lines.is_empty() {
            let chunk_content = current_lines.join("\n");
            let chunk = Chunk::new(
                file_path.clone(),
                current_start + 1,
                lines.len(),
                chunk_content,
                last_modified,
            );
            chunks.push(chunk);
        }

        chunks
    }

    /// Check if brackets are balanced in a line range
    pub fn is_bracket_balanced(lines: &[&str]) -> bool {
        let mut depth = 0i32;
        for line in lines {
            for c in line.chars() {
                match c {
                    '{' | '[' | '(' => depth += 1,
                    '}' | ']' | ')' => depth -= 1,
                    _ => {}
                }
            }
        }
        depth == 0
    }
}

impl Default for Chunker {
    fn default() -> Self {
        Self::new()
    }
}

/// Calculate content hash for deduplication
pub fn calculate_content_hash(content: &str) -> String {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(content.as_bytes());
    format!("{:x}", hasher.finalize())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chunk_file() {
        let chunker = Chunker::new();
        let content = r#"fn main() {
    println!("Hello");
}

fn other() {
    println!("World");
}"#;

        let chunks = chunker.chunk_file(PathBuf::from("test.rs"), content, 1234567890);

        assert!(!chunks.is_empty());
        assert!(chunks[0].content.contains("main"));
    }

    #[test]
    fn test_chunk_by_blank_lines() {
        let chunker = Chunker::new();
        let content = "line1\n\nline2\nline3\n\nline4";

        let chunks = chunker.chunk_by_blank_lines(PathBuf::from("test.txt"), content, 1234567890);

        assert!(chunks.len() >= 2);
    }

    #[test]
    fn test_bracket_balanced() {
        let lines = vec!["fn main() {", "    println!();", "}"];
        assert!(Chunker::is_bracket_balanced(&lines));

        let lines = vec!["fn main() {", "    println!();"];
        assert!(!Chunker::is_bracket_balanced(&lines));
    }

    #[test]
    fn test_content_hash() {
        let hash1 = calculate_content_hash("hello");
        let hash2 = calculate_content_hash("hello");
        let hash3 = calculate_content_hash("world");

        assert_eq!(hash1, hash2);
        assert_ne!(hash1, hash3);
        assert_eq!(hash1.len(), 64); // SHA256 hex length
    }
}
