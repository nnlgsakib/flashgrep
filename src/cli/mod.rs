use crate::config::paths::{get_repo_root, FlashgrepPaths};
use crate::config::Config;
use crate::db::Database;
use crate::index::engine::Indexer;
use crate::mcp::stdio::McpStdioServer;
use crate::search::Searcher;
use crate::watcher::registry::{kill_process, is_process_alive, WatcherRegistry};
use crate::watcher::FileWatcher;
use crate::FlashgrepResult;
use clap::{Parser, Subcommand, ValueEnum};
use serde::Serialize;
use std::ffi::OsString;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use tokio::task;
use tracing::info;

/// Flashgrep CLI
#[derive(Parser)]
#[command(name = "flashgrep")]
#[command(about = "High-performance local code indexing engine")]
#[command(version = env!("CARGO_PKG_VERSION"))]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, ValueEnum)]
pub enum OutputMode {
    Text,
    Json,
}

#[derive(Debug, Serialize)]
struct CliResult {
    file_path: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    start_line: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    end_line: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    symbol_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    relevance_score: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    preview: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    content: Option<String>,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Index a repository
    Index {
        /// Path to the repository (defaults to current directory)
        #[arg(value_name = "PATH")]
        path: Option<PathBuf>,
        /// Force full re-index (ignore existing index)
        #[arg(short, long)]
        force: bool,
    },
    /// Start file watcher only
    Start {
        /// Path to the repository (defaults to current directory)
        #[arg(value_name = "PATH")]
        path: Option<PathBuf>,
        /// Start file watcher in background and return immediately
        #[arg(short = 'b', long = "background")]
        background: bool,
    },
    /// Stop file watcher
    Stop {
        /// Path to the repository (defaults to current directory)
        #[arg(value_name = "PATH")]
        path: Option<PathBuf>,
    },
    /// Show active background watchers
    Watchers,
    /// Indexed text search (grep-like)
    Query {
        /// Search text/query
        text: String,
        /// Path to the repository (defaults to current directory)
        #[arg(value_name = "PATH")]
        path: Option<PathBuf>,
        /// Maximum number of results
        #[arg(short, long, default_value_t = 20)]
        limit: usize,
        /// Output format
        #[arg(long, value_enum, default_value_t = OutputMode::Text)]
        output: OutputMode,
    },
    /// List indexed files (glob-like)
    Files {
        /// Optional substring filter for file paths
        #[arg(short, long)]
        filter: Option<String>,
        /// Path to the repository (defaults to current directory)
        #[arg(value_name = "PATH")]
        path: Option<PathBuf>,
        /// Maximum number of results
        #[arg(short, long, default_value_t = 200)]
        limit: usize,
        /// Output format
        #[arg(long, value_enum, default_value_t = OutputMode::Text)]
        output: OutputMode,
    },
    /// Find symbol definitions/usages
    Symbol {
        /// Symbol name to search
        symbol_name: String,
        /// Path to the repository (defaults to current directory)
        #[arg(value_name = "PATH")]
        path: Option<PathBuf>,
        /// Maximum number of results
        #[arg(short, long, default_value_t = 50)]
        limit: usize,
        /// Output format
        #[arg(long, value_enum, default_value_t = OutputMode::Text)]
        output: OutputMode,
    },
    /// Get line range from a file
    Slice {
        /// File path (absolute or relative to repository root)
        file_path: PathBuf,
        /// Start line (1-indexed)
        start_line: usize,
        /// End line (1-indexed, inclusive)
        end_line: usize,
        /// Path to the repository (defaults to current directory)
        #[arg(value_name = "PATH")]
        path: Option<PathBuf>,
        /// Output format
        #[arg(long, value_enum, default_value_t = OutputMode::Text)]
        output: OutputMode,
    },
    /// Start MCP server (TCP mode)
    Mcp {
        /// Path to the repository (defaults to current directory)
        #[arg(value_name = "PATH")]
        path: Option<PathBuf>,
        /// Port to listen on (default: 7777)
        #[arg(short, long)]
        port: Option<u16>,
        /// Log level (default: info)
        #[arg(short, long)]
        log_level: Option<String>,
    },
    /// Start MCP server (stdio mode for MCP clients)
    McpStdio {
        /// Path to the repository (defaults to current directory)
        #[arg(value_name = "PATH")]
        path: Option<PathBuf>,
    },
    /// Show index statistics
    Stats {
        /// Path to the repository (defaults to current directory)
        #[arg(value_name = "PATH")]
        path: Option<PathBuf>,
    },
    /// Clear the index for a repository
    Clear {
        /// Path to the repository (defaults to current directory)
        #[arg(value_name = "PATH")]
        path: Option<PathBuf>,
    },
}

/// Run the CLI
pub async fn run() -> FlashgrepResult<()> {
    let cli = Cli::parse();
    
    match cli.command {
        Commands::Index { path, force } => {
            let repo_root = get_repo_root(path.as_ref())?;
            info!("Indexing repository: {}", repo_root.display());
            
            let mut indexer = Indexer::new(repo_root.clone())?;
            
            if force {
                println!("Force re-indexing...");
                indexer.clear_index()?;
            }
            
            let stats = indexer.index_repository(&repo_root)?;
            
            println!("\n✓ Indexing complete!");
            println!("  Files indexed: {}", stats.total_files);
            println!("  Chunks created: {}", stats.total_chunks);
            println!("  Symbols detected: {}", stats.total_symbols);
            
            Ok(())
        }
        Commands::Start { path, background } => {
            let repo_root = get_repo_root(path.as_ref())?;
            let canonical_repo_root = WatcherRegistry::canonicalize_repo_path(&repo_root)?;
            info!("Starting file watcher for: {}", repo_root.display());
            
            // Check if index exists
            if !FlashgrepPaths::new(&canonical_repo_root).exists() {
                println!("⚠ No index found. Run 'flashgrep index' first.");
                return Ok(());
            }

            let mut registry = WatcherRegistry::load_default()?;
            let _ = registry.cleanup_stale()?;

            if let Some(existing) = registry.get(&canonical_repo_root)? {
                if is_process_alive(existing.pid) && existing.pid != std::process::id() {
                    println!(
                        "Watcher is already running for {} (PID {}).",
                        canonical_repo_root.display(),
                        existing.pid
                    );
                    print_active_watchers(&registry);
                    return Ok(());
                }
            }

            if background {
                match spawn_background_watcher(&canonical_repo_root) {
                    Ok(pid) => {
                        registry.upsert(&canonical_repo_root, pid)?;
                        println!("✓ Started background watcher");
                        println!("  Repository: {}", canonical_repo_root.display());
                        println!("  PID: {}", pid);
                        print_active_watchers(&registry);
                    }
                    Err(e) => {
                        eprintln!("✗ Failed to start background watcher: {}", e);
                        return Err(e);
                    }
                }
                return Ok(());
            }

            registry.upsert(&canonical_repo_root, std::process::id())?;
            
            println!("Starting file watcher...");
            println!("Repository: {}", canonical_repo_root.display());
            
            // Start file watcher
            let watcher_root = canonical_repo_root.clone();
            let watcher_handle = task::spawn_blocking(move || {
                let mut watcher = match FileWatcher::new(watcher_root) {
                    Ok(w) => w,
                    Err(e) => {
                        eprintln!("Failed to create file watcher: {}", e);
                        return;
                    }
                };
                
                println!("File watcher started");
                
                if let Err(e) = watcher.watch() {
                    eprintln!("File watcher error: {}", e);
                }
            });
            
            // Wait for file watcher to complete (or Ctrl+C)
            watcher_handle.await?;
            
            Ok(())
        }
        Commands::Stop { path } => {
            let repo_root = get_repo_root(path.as_ref())?;
            let canonical_repo_root = WatcherRegistry::canonicalize_repo_path(&repo_root)?;
            info!("Stopping file watcher for: {}", repo_root.display());

            let mut registry = WatcherRegistry::load_default()?;
            let removed = registry.cleanup_stale()?;
            if removed > 0 {
                println!("Removed {} stale watcher entr{}.", removed, if removed == 1 { "y" } else { "ies" });
            }

            match registry.get(&canonical_repo_root)? {
                Some(entry) => {
                    if is_process_alive(entry.pid) {
                        println!("Stopping watcher for {} (PID {})...", canonical_repo_root.display(), entry.pid);
                        kill_process(entry.pid)?;
                    }
                    let _ = registry.remove(&canonical_repo_root)?;
                    println!("✓ Watcher stopped for {}", canonical_repo_root.display());
                }
                None => {
                    println!("No active watcher found for {}", canonical_repo_root.display());
                }
            }

            print_active_watchers(&registry);
            
            Ok(())
        }
        Commands::Watchers => {
            let mut registry = WatcherRegistry::load_default()?;
            let _ = registry.cleanup_stale()?;
            print_active_watchers(&registry);
            Ok(())
        }
        Commands::Query {
            text,
            path,
            limit,
            output,
        } => {
            let (repo_root, searcher) = create_searcher(path.as_ref())?;
            let mut results = searcher.query(&text, limit.max(1))?;
            results.sort_by(|a, b| {
                b.relevance_score
                    .total_cmp(&a.relevance_score)
                    .then_with(|| a.file_path.cmp(&b.file_path))
                    .then_with(|| a.start_line.cmp(&b.start_line))
                    .then_with(|| a.end_line.cmp(&b.end_line))
            });
            results.truncate(limit.max(1));

            let rendered: Vec<CliResult> = results
                .into_iter()
                .map(|r| CliResult {
                    file_path: r.file_path.to_string_lossy().to_string(),
                    start_line: Some(r.start_line),
                    end_line: Some(r.end_line),
                    symbol_name: r.symbol_name,
                    relevance_score: Some(r.relevance_score),
                    preview: Some(r.preview),
                    content: r.content,
                })
                .collect();

            render_results(&rendered, output, &format!("query in {}", repo_root.display()))?;
            Ok(())
        }
        Commands::Files {
            filter,
            path,
            limit,
            output,
        } => {
            let (repo_root, searcher) = create_searcher(path.as_ref())?;
            let mut files = searcher.list_files()?;
            files.sort();

            if let Some(needle) = filter.as_ref() {
                let needle = needle.to_lowercase();
                files.retain(|p| p.to_string_lossy().to_lowercase().contains(&needle));
            }

            files.truncate(limit.max(1));
            let rendered: Vec<CliResult> = files
                .into_iter()
                .map(|p| CliResult {
                    file_path: p.to_string_lossy().to_string(),
                    start_line: None,
                    end_line: None,
                    symbol_name: None,
                    relevance_score: None,
                    preview: None,
                    content: None,
                })
                .collect();

            render_results(&rendered, output, &format!("files in {}", repo_root.display()))?;
            Ok(())
        }
        Commands::Symbol {
            symbol_name,
            path,
            limit,
            output,
        } => {
            let (repo_root, searcher) = create_searcher(path.as_ref())?;
            let mut symbols = searcher.get_symbol(&symbol_name)?;
            symbols.sort_by(|a, b| {
                a.file_path
                    .cmp(&b.file_path)
                    .then_with(|| a.line_number.cmp(&b.line_number))
                    .then_with(|| a.symbol_name.cmp(&b.symbol_name))
            });
            symbols.truncate(limit.max(1));

            let rendered: Vec<CliResult> = symbols
                .into_iter()
                .map(|s| CliResult {
                    file_path: s.file_path.to_string_lossy().to_string(),
                    start_line: Some(s.line_number),
                    end_line: Some(s.line_number),
                    symbol_name: Some(s.symbol_name),
                    relevance_score: None,
                    preview: Some(format!("type={}", s.symbol_type)),
                    content: None,
                })
                .collect();

            render_results(
                &rendered,
                output,
                &format!("symbol {} in {}", symbol_name, repo_root.display()),
            )?;
            Ok(())
        }
        Commands::Slice {
            file_path,
            start_line,
            end_line,
            path,
            output,
        } => {
            if start_line == 0 || end_line == 0 || start_line > end_line {
                return Err(crate::FlashgrepError::Config(
                    "Invalid line range. Use start_line >= 1 and end_line >= start_line".to_string(),
                ));
            }

            let (repo_root, searcher) = create_searcher(path.as_ref())?;
            let normalized_path = if file_path.is_absolute() {
                file_path
            } else {
                repo_root.join(file_path)
            };
            let content = searcher
                .get_slice(&normalized_path, start_line, end_line)?
                .ok_or_else(|| {
                    crate::FlashgrepError::Config(format!(
                        "Could not read slice for {}:{}-{}",
                        normalized_path.display(),
                        start_line,
                        end_line
                    ))
                })?;

            let rendered = vec![CliResult {
                file_path: normalized_path.to_string_lossy().to_string(),
                start_line: Some(start_line),
                end_line: Some(end_line),
                symbol_name: None,
                relevance_score: None,
                preview: None,
                content: Some(content),
            }];
            render_results(&rendered, output, "slice")?;
            Ok(())
        }
        Commands::Stats { path } => {
            let repo_root = get_repo_root(path.as_ref())?;
            
            if !FlashgrepPaths::new(&repo_root).exists() {
                println!("⚠ No index found. Run 'flashgrep index' first.");
                return Ok(());
            }

            let paths = FlashgrepPaths::new(&repo_root);
            let db = Database::open(&paths.metadata_db())?;
            let stats = db.get_stats()?;
            
            println!("\n📊 Index Statistics");
            println!("==================");
            println!("  Total files: {}", stats.total_files);
            println!("  Total chunks: {}", stats.total_chunks);
            println!("  Total symbols: {}", stats.total_symbols);
            println!("  Index size: {} MB", stats.index_size_bytes / 1024 / 1024);
            if let Some(last_update) = stats.last_update {
                let datetime = chrono::DateTime::from_timestamp(last_update, 0);
                if let Some(dt) = datetime {
                    println!("  Last update: {}", dt.format("%Y-%m-%d %H:%M:%S"));
                }
            }
            
            Ok(())
        }
        Commands::Mcp { path, port, log_level } => {
            let repo_root = get_repo_root(path.as_ref())?;
            info!("Starting MCP server for: {}", repo_root.display());
            
            // Check if index exists
            if !FlashgrepPaths::new(&repo_root).exists() {
                println!("⚠ No index found. Run 'flashgrep index' first.");
                return Ok(());
            }
            
            // Create config with optional overrides
            let mut config = if FlashgrepPaths::new(&repo_root).config_file().exists() {
                Config::from_file(&FlashgrepPaths::new(&repo_root).config_file())?
            } else {
                Config::default()
            };
            
            if let Some(p) = port {
                config.mcp_port = p;
            }
            
            // Set log level if specified
            if let Some(level) = log_level {
                // This is a simplification - in real code, you'd need to properly configure the logging
                println!("Log level: {}", level);
            }
            
            println!("Starting MCP server...");
            println!("Repository: {}", repo_root.display());
            println!("Port: {}", config.mcp_port);
            
            // Create MCP server instance
            let server = crate::mcp::McpServer::new(repo_root.clone())?;
            
            // Run server and wait for shutdown
            server.start().await?;
            
            Ok(())
        }
        Commands::McpStdio { path } => {
            let repo_root = get_repo_root(path.as_ref())?;
            info!("Starting MCP stdio server for: {}", repo_root.display());
            
            // Check if index exists
            if !FlashgrepPaths::new(&repo_root).exists() {
                eprintln!("⚠ No index found. Run 'flashgrep index' first.");
                return Ok(());
            }
            
            // Create and start stdio MCP server
            let server = McpStdioServer::new(repo_root)?;
            
            // Run server (this blocks on stdin)
            server.start()?;
            
            Ok(())
        }
        Commands::Clear { path } => {
            let repo_root = get_repo_root(path.as_ref())?;
            
            if !FlashgrepPaths::new(&repo_root).exists() {
                println!("⚠ No index found.");
                return Ok(());
            }
            
            print!("Are you sure you want to clear the index? [y/N]: ");
            use std::io::Write;
            std::io::stdout().flush()?;
            
            let mut input = String::new();
            std::io::stdin().read_line(&mut input)?;
            
            if input.trim().eq_ignore_ascii_case("y") {
                let mut indexer = Indexer::new(repo_root)?;
                indexer.clear_index()?;
                println!("✓ Index cleared");
            } else {
                println!("Cancelled");
            }
            
            Ok(())
        }
    }
}

fn print_active_watchers(registry: &WatcherRegistry) {
    let entries = registry.list();
    if entries.is_empty() {
        println!("Active watchers: 0");
        return;
    }

    println!("Active watchers: {}", entries.len());
    for entry in entries {
        println!("  - {} (PID {})", entry.repo_root, entry.pid);
    }
}

fn spawn_background_watcher(repo_root: &PathBuf) -> FlashgrepResult<u32> {
    let exe_path = std::env::current_exe()?;
    let args = vec![
        OsString::from("start"),
        OsString::from(repo_root.to_string_lossy().to_string()),
    ];

    spawn_process_for_background(&exe_path, &args, true)
}

fn spawn_process_for_background(
    executable: &std::path::Path,
    args: &[OsString],
    detached: bool,
) -> FlashgrepResult<u32> {
    let mut command = Command::new(executable);
    command
        .args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());

    if detached {
        #[cfg(windows)]
        {
            use std::os::windows::process::CommandExt;
            const CREATE_NEW_PROCESS_GROUP: u32 = 0x00000200;
            const DETACHED_PROCESS: u32 = 0x00000008;
            command.creation_flags(CREATE_NEW_PROCESS_GROUP | DETACHED_PROCESS);
        }
    }

    let child = command.spawn()?;
    Ok(child.id())
}

fn create_searcher(path: Option<&PathBuf>) -> FlashgrepResult<(PathBuf, Searcher)> {
    let repo_root = get_repo_root(path)?;
    let paths = FlashgrepPaths::new(&repo_root);
    if !paths.exists() {
        return Err(crate::FlashgrepError::Config(format!(
            "No index found for {}. Run 'flashgrep index' first.",
            repo_root.display()
        )));
    }

    let index = tantivy::Index::open_in_dir(paths.text_index_dir())?;
    let searcher = Searcher::new(&index, &paths.metadata_db())?;
    Ok((repo_root, searcher))
}

fn render_results(results: &[CliResult], output: OutputMode, label: &str) -> FlashgrepResult<()> {
    match output {
        OutputMode::Json => {
            println!("{}", serde_json::to_string(results)?);
        }
        OutputMode::Text => {
            println!("{}: {} result(s)", label, results.len());
            for r in results {
                let mut line = r.file_path.clone();
                if let (Some(start), Some(end)) = (r.start_line, r.end_line) {
                    line = format!("{}:{}-{}", line, start, end);
                }
                if let Some(name) = &r.symbol_name {
                    line = format!("{} symbol={}", line, name);
                }
                if let Some(score) = r.relevance_score {
                    line = format!("{} score={:.3}", line, score);
                }
                println!("- {}", line);
                if let Some(preview) = &r.preview {
                    println!("  {}", preview.replace('\n', "\\n"));
                }
                if let Some(content) = &r.content {
                    println!("  {}", content.replace('\n', "\\n"));
                }
            }
        }
    }
    Ok(())
}

/// Print help information about .flashgrepignore
pub fn print_ignore_help() {
    println!("
.flashgrepignore file format:
  The .flashgrepignore file uses gitignore-style patterns to exclude files and
  directories from indexing. Place this file in the root of your repository.

Pattern syntax:
  *       - Wildcard, matches any sequence of characters
  ?       - Matches a single character
  **/     - Matches any number of directory levels
  /       - Anchors pattern to root
  !       - Negates a pattern (re-includes previously excluded files)
  #       - Comment line (ignored)

Examples:
  # Ignore all log files
  *.log

  # Ignore the build directory
  build/

  # Ignore all .tmp files except important.tmp
  *.tmp
  !important.tmp

  # Ignore a specific file at root
  /config.local.json
");
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    #[test]
    fn parse_start_background_flag() {
        let cli = Cli::parse_from(["flashgrep", "start", "-b"]);
        match cli.command {
            Commands::Start { background, .. } => assert!(background),
            _ => panic!("expected start command"),
        }
    }

    #[test]
    fn background_spawn_failure_is_reported() {
        let bad_exe = std::path::PathBuf::from("definitely_missing_flashgrep_binary");
        let result = spawn_process_for_background(&bad_exe, &[], false);
        assert!(result.is_err());
    }

    #[test]
    fn background_spawn_success_returns_pid() -> FlashgrepResult<()> {
        let exe = std::env::current_exe()?;
        let args = vec![OsString::from("--version")];
        let pid = spawn_process_for_background(&exe, &args, false)?;
        assert!(pid > 0);
        Ok(())
    }

    #[test]
    fn parse_query_with_json_output() {
        let cli = Cli::parse_from(["flashgrep", "query", "main", "--output", "json"]);
        match cli.command {
            Commands::Query { text, output, .. } => {
                assert_eq!(text, "main");
                assert_eq!(output, OutputMode::Json);
            }
            _ => panic!("expected query command"),
        }
    }

    #[test]
    fn parse_files_with_filter_and_limit() {
        let cli = Cli::parse_from([
            "flashgrep",
            "files",
            "--filter",
            "tests",
            "--limit",
            "5",
        ]);
        match cli.command {
            Commands::Files { filter, limit, .. } => {
                assert_eq!(filter.as_deref(), Some("tests"));
                assert_eq!(limit, 5);
            }
            _ => panic!("expected files command"),
        }
    }

    #[test]
    fn parse_slice_requires_line_args() {
        let cli = Cli::try_parse_from(["flashgrep", "slice", "src/main.rs"]);
        assert!(cli.is_err());
    }

    #[test]
    fn render_json_is_valid() -> FlashgrepResult<()> {
        let data = vec![CliResult {
            file_path: "src/main.rs".to_string(),
            start_line: Some(1),
            end_line: Some(3),
            symbol_name: Some("main".to_string()),
            relevance_score: Some(1.0),
            preview: Some("fn main".to_string()),
            content: None,
        }];

        let encoded = serde_json::to_string(&data)?;
        let parsed: serde_json::Value = serde_json::from_str(&encoded)?;
        assert!(parsed.is_array());
        Ok(())
    }
}
