use crate::config::paths::{get_repo_root, FlashgrepPaths};
use crate::config::Config;
use crate::index::engine::Indexer;
use crate::mcp::stdio::McpStdioServer;
use crate::watcher::registry::{kill_process, is_process_alive, WatcherRegistry};
use crate::watcher::FileWatcher;
use crate::FlashgrepResult;
use clap::{Parser, Subcommand};
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
        Commands::Stats { path } => {
            let repo_root = get_repo_root(path.as_ref())?;
            
            if !FlashgrepPaths::new(&repo_root).exists() {
                println!("⚠ No index found. Run 'flashgrep index' first.");
                return Ok(());
            }
            
            let indexer = Indexer::new(repo_root)?;
            let stats = indexer.get_stats()?;
            
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
}
