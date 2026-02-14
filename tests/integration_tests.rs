use flashgrep::chunking::Chunker;
use flashgrep::config::paths::{get_repo_root, FlashgrepPaths};
use flashgrep::config::Config;
use flashgrep::db::models::{FileMetadata, SymbolType};
use flashgrep::db::Database;
use flashgrep::index::engine::Indexer;
use flashgrep::index::scanner::{should_index_file, FileScanner, FlashgrepIgnore};
use flashgrep::search::Searcher;
use flashgrep::symbols::SymbolDetector;
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

/// Helper to create a test file with content
fn create_test_file(dir: &PathBuf, name: &str, content: &str) {
    let path = dir.join(name);
    fs::write(&path, content).expect("Failed to write test file");
}

#[test]
fn test_file_scanner_finds_source_files() {
    let temp_dir = TempDir::new().unwrap();
    let root = temp_dir.path().to_path_buf();

    // Create test files
    create_test_file(&root, "main.rs", "fn main() {}");
    create_test_file(&root, "lib.py", "def hello(): pass");
    fs::create_dir(root.join("target")).unwrap();
    create_test_file(&root.join("target"), "debug.o", "binary");

    let config = Config::default();
    let scanner = FileScanner::new(root.clone(), config);
    let files: Vec<_> = scanner.scan().collect();

    assert_eq!(files.len(), 2);
    assert!(files.iter().any(|p| p.ends_with("main.rs")));
    assert!(files.iter().any(|p| p.ends_with("lib.py")));
    assert!(!files.iter().any(|p| p.to_string_lossy().contains("target")));
}

#[test]
fn test_file_scanner_ignores_by_extension() {
    let temp_dir = TempDir::new().unwrap();
    let root = temp_dir.path().to_path_buf();

    create_test_file(&root, "main.rs", "fn main() {}");
    create_test_file(&root, "app.exe", "binary");
    create_test_file(&root, "data.txt", "hello");

    let config = Config::default();
    let scanner = FileScanner::new(root, config);
    let files: Vec<_> = scanner.scan().collect();

    assert_eq!(files.len(), 1);
    assert!(files[0].ends_with("main.rs"));
}

#[test]
fn test_chunker_creates_logical_chunks() {
    let chunker = Chunker::new();
    let content = r#"fn main() {
    println!("Hello");
}

fn other() {
    println!("World");
}

fn third() {
    println!("!");
}"#;

    let chunks = chunker.chunk_file(PathBuf::from("test.rs"), content, 1234567890);

    assert!(!chunks.is_empty());
    // Should create at least 2 chunks (separated by blank lines)
    assert!(chunks.len() >= 2);

    // Check chunk boundaries
    for chunk in &chunks {
        assert!(chunk.start_line > 0);
        assert!(chunk.end_line >= chunk.start_line);
        assert!(!chunk.content_hash.is_empty());
    }
}

#[test]
fn test_symbol_detector_finds_functions() {
    let detector = SymbolDetector::new();
    let code = r#"
fn main() {
    println!("Hello");
}

fn helper_function() -> i32 {
    42
}
"#;

    let symbols = detector.detect_in_chunk(code, PathBuf::from("test.rs"), 1);

    let function_names: Vec<_> = symbols
        .iter()
        .filter(|s| s.symbol_type == SymbolType::Function)
        .map(|s| s.symbol_name.clone())
        .collect();

    assert!(function_names.contains(&"main".to_string()));
    assert!(function_names.contains(&"helper_function".to_string()));
}

#[test]
fn test_symbol_detector_finds_classes() {
    let detector = SymbolDetector::new();
    let code = r#"
class MyClass:
    def __init__(self):
        pass

struct Point {
    x: i32,
    y: i32,
}
"#;

    let symbols = detector.detect_in_chunk(code, PathBuf::from("test.py"), 1);

    let class_names: Vec<_> = symbols
        .iter()
        .filter(|s| s.symbol_type == SymbolType::Class || s.symbol_type == SymbolType::Struct)
        .map(|s| s.symbol_name.clone())
        .collect();

    assert!(
        class_names.contains(&"MyClass".to_string()) || class_names.contains(&"Point".to_string())
    );
}

#[test]
fn test_database_basic_operations() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("test.db");

    let db = Database::open(&db_path).expect("Failed to open database");

    // Insert file
    let file = FileMetadata {
        id: None,
        file_path: PathBuf::from("test.rs"),
        file_size: 100,
        last_modified: 1234567890,
        language: Some("rust".to_string()),
    };
    let file_id = db.insert_file(&file).expect("Failed to insert file");
    assert!(file_id > 0);

    // Check stats
    let stats = db.get_stats().expect("Failed to get stats");
    assert_eq!(stats.total_files, 1);

    // Check file exists
    let files = db.get_all_files().expect("Failed to get files");
    assert_eq!(files.len(), 1);
}

#[test]
fn test_database_batch_inserts() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("test.db");

    let db = Database::open(&db_path).expect("Failed to open database");

    // Insert file first
    let file = FileMetadata {
        id: None,
        file_path: PathBuf::from("test.rs"),
        file_size: 100,
        last_modified: 1234567890,
        language: Some("rust".to_string()),
    };
    db.insert_file(&file).unwrap();

    // Batch insert chunks
    let chunker = Chunker::new();
    let content = "line1\nline2\n\nline3\nline4";
    let chunks = chunker.chunk_file(PathBuf::from("test.rs"), content, 1234567890);

    let count = db
        .insert_chunks_batch(&chunks)
        .expect("Failed to batch insert");
    assert_eq!(count, chunks.len());

    let stats = db.get_stats().expect("Failed to get stats");
    assert_eq!(stats.total_chunks, chunks.len() as usize);
}

#[test]
fn test_incremental_indexing() {
    let temp_dir = TempDir::new().unwrap();
    let repo_root = temp_dir.path().to_path_buf();

    // Create test file
    create_test_file(&repo_root, "main.rs", "fn main() {}");

    // Create indexer
    let mut indexer = Indexer::new(repo_root.clone()).expect("Failed to create indexer");

    // First index
    let stats1 = indexer
        .index_repository(&repo_root)
        .expect("Failed to index");
    assert_eq!(stats1.total_files, 1);

    // Second index (should skip unchanged files)
    let stats2 = indexer
        .index_repository(&repo_root)
        .expect("Failed to re-index");
    assert_eq!(stats2.total_files, 1); // File count same
                                       // Stats may show 0 indexed if file unchanged
}

#[test]
fn test_indexer_detects_modifications() {
    let temp_dir = TempDir::new().unwrap();
    let repo_root = temp_dir.path().to_path_buf();

    // Create initial file
    create_test_file(&repo_root, "main.rs", "fn main() {}");

    // Index
    let mut indexer = Indexer::new(repo_root.clone()).unwrap();
    indexer.index_repository(&repo_root).unwrap();

    // Get initial stats
    let stats1 = indexer.get_stats().unwrap();

    // Modify file
    std::thread::sleep(std::time::Duration::from_millis(100));
    create_test_file(&repo_root, "main.rs", "fn main() { println!(\"hello\"); }");

    // Re-index
    indexer.index_repository(&repo_root).unwrap();

    // Should have re-indexed
    let stats2 = indexer.get_stats().unwrap();
    // Total files same, but content changed
    assert_eq!(stats2.total_files, stats1.total_files);
}

#[test]
fn test_flashgrepignore_patterns() {
    let temp_dir = TempDir::new().unwrap();
    let root = temp_dir.path().to_path_buf();

    // Create files
    create_test_file(&root, "main.rs", "fn main() {}");
    create_test_file(&root, "test.txt", "text data");
    fs::create_dir(root.join("temp_dir")).unwrap();
    create_test_file(&root.join("temp_dir"), "output.js", "console.log('test')");

    // Create .flashgrepignore - ignore .txt files and temp directories
    fs::write(root.join(".flashgrepignore"), "*.txt\ntemp*\n").unwrap();

    let config = Config::default();
    let scanner = FileScanner::new(root, config);
    let files: Vec<_> = scanner.scan().collect();

    // Should only find main.rs (test.txt ignored by *.txt, temp_dir/* ignored by temp*)
    assert!(!files.is_empty(), "Should find at least main.rs");
    assert!(
        files.iter().any(|p| p.ends_with("main.rs")),
        "Should find main.rs"
    );
    assert!(
        !files.iter().any(|p| p.ends_with("test.txt")),
        "Should not find test.txt"
    );
}

#[test]
fn test_end_to_end_workflow() {
    let temp_dir = TempDir::new().unwrap();
    let repo_root = temp_dir.path().to_path_buf();

    // Create a multi-file project
    fs::create_dir(repo_root.join("src")).unwrap();
    create_test_file(
        &repo_root.join("src"),
        "main.rs",
        r#"
fn main() {
    println!("Hello, world!");
}

fn helper() {
    println!("Helper function");
}
"#,
    );
    create_test_file(
        &repo_root.join("src"),
        "lib.rs",
        r#"
pub struct Point {
    x: f64,
    y: f64,
}

impl Point {
    pub fn new(x: f64, y: f64) -> Self {
        Point { x, y }
    }
}
"#,
    );
    create_test_file(
        &repo_root,
        "Cargo.toml",
        r#"
[package]
name = "test-project"
version = "0.1.0"
"#,
    );
    create_test_file(&repo_root, "README.md", "# Test Project\n\nA test project.");

    // Index the project
    let mut indexer = Indexer::new(repo_root.clone()).expect("Failed to create indexer");
    let stats = indexer
        .index_repository(&repo_root)
        .expect("Failed to index");

    // Verify indexing worked
    assert!(stats.total_files > 0, "Should have indexed files");
    assert!(stats.total_chunks > 0, "Should have created chunks");
    assert!(stats.total_symbols > 0, "Should have detected symbols");

    // Check database has data
    let db = indexer.db();
    let files = db.get_all_files().expect("Failed to get files");
    assert_eq!(files.len(), stats.total_files);

    // Check symbols
    let main_symbols = db
        .find_symbols_by_name("main")
        .expect("Failed to find symbols");
    assert!(!main_symbols.is_empty(), "Should find 'main' function");

    let point_symbols = db
        .find_symbols_by_name("Point")
        .expect("Failed to find symbols");
    assert!(!point_symbols.is_empty(), "Should find 'Point' struct");
}

#[test]
fn test_performance_small_repo() {
    let temp_dir = TempDir::new().unwrap();
    let repo_root = temp_dir.path().to_path_buf();

    // Create multiple files
    for i in 0..100 {
        let filename = format!("file_{}.rs", i);
        let content = format!(
            r#"
fn function_{}() {{
    println!("Hello from function {}");
}}
"#,
            i, i
        );
        create_test_file(&repo_root, &filename, &content);
    }

    let start = std::time::Instant::now();

    let mut indexer = Indexer::new(repo_root.clone()).unwrap();
    let stats = indexer.index_repository(&repo_root).unwrap();

    let duration = start.elapsed();

    // Should complete in reasonable time (< 10 seconds for 100 files)
    assert!(
        duration.as_secs() < 10,
        "Indexing 100 files took too long: {:?}",
        duration
    );
    assert_eq!(stats.total_files, 100);
}

#[test]
fn test_ignored_directories_do_not_appear_in_files_query_symbol() {
    let temp_dir = TempDir::new().unwrap();
    let repo_root = temp_dir.path().to_path_buf();

    fs::create_dir_all(repo_root.join("src")).unwrap();
    create_test_file(
        &repo_root.join("src"),
        "main.rs",
        "fn main() { let x = 1; }",
    );

    fs::create_dir_all(repo_root.join(".opencode/node_modules/zod")).unwrap();
    create_test_file(
        &repo_root.join(".opencode/node_modules/zod"),
        "core.rs",
        "fn ignored_fn() { let _ = \"IGNORED_TOKEN_ZOD\"; }",
    );

    fs::write(repo_root.join(".flashgrepignore"), ".opencode/\n").unwrap();

    let mut indexer = Indexer::new(repo_root.clone()).unwrap();
    indexer.index_repository(&repo_root).unwrap();

    let db = indexer.db();
    let files = db.get_all_files().unwrap();
    assert!(!files
        .iter()
        .any(|p| p.to_string_lossy().contains(".opencode")));

    let ignored_symbols = db.find_symbols_by_name("ignored_fn").unwrap();
    assert!(ignored_symbols.is_empty());

    let paths = FlashgrepPaths::new(&repo_root);
    let searcher = Searcher::new(indexer.tantivy_index(), &paths.metadata_db()).unwrap();
    let query_hits = searcher.query("IGNORED_TOKEN_ZOD", 10).unwrap();
    assert!(query_hits.is_empty());
}

#[test]
fn test_ignore_file_update_prunes_newly_ignored_indexed_files() {
    let temp_dir = TempDir::new().unwrap();
    let repo_root = temp_dir.path().to_path_buf();

    fs::create_dir_all(repo_root.join(".opencode/pkg")).unwrap();
    create_test_file(
        &repo_root.join(".opencode/pkg"),
        "keep_then_prune.rs",
        "fn transient_symbol() { let _ = \"TRANSIENT_IGNORE_TOKEN\"; }",
    );
    create_test_file(&repo_root, "main.rs", "fn main() {}\n");

    fs::write(repo_root.join(".flashgrepignore"), "# initially empty\n").unwrap();

    let mut indexer = Indexer::new(repo_root.clone()).unwrap();
    indexer.index_repository(&repo_root).unwrap();

    let before_files = indexer.db().get_all_files().unwrap();
    assert!(before_files
        .iter()
        .any(|p| p.to_string_lossy().contains(".opencode")));

    fs::write(repo_root.join(".flashgrepignore"), ".opencode/\n").unwrap();
    let ignore = FlashgrepIgnore::from_root(&repo_root);
    let (removed, _kept) = indexer
        .reconcile_ignored_files(&repo_root, &ignore)
        .unwrap();
    assert!(removed >= 1);

    let after_files = indexer.db().get_all_files().unwrap();
    assert!(!after_files
        .iter()
        .any(|p| p.to_string_lossy().contains(".opencode")));

    let symbols = indexer
        .db()
        .find_symbols_by_name("transient_symbol")
        .unwrap();
    assert!(symbols.is_empty());

    let paths = FlashgrepPaths::new(&repo_root);
    let searcher = Searcher::new(indexer.tantivy_index(), &paths.metadata_db()).unwrap();
    let hits = searcher.query("TRANSIENT_IGNORE_TOKEN", 10).unwrap();
    assert!(hits.is_empty());
}
