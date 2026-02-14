use crate::config::paths::{get_repo_root, FlashgrepPaths};
use crate::config::Config;
use crate::index::engine::Indexer;
use crate::mcp::McpServer;
use crate::mcp::stdio::McpStdioServer;
use crate::watcher::FileWatcher;
use crate::FlashgrepResult;
use clap::{Parser, Subcommand};
use std::path::PathBuf;
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
    },
    /// Stop file watcher
    Stop {
        /// Path to the repository (defaults to current directory)
        #[arg(value_name = "PATH")]
        path: Option<PathBuf>,
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
    },}

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
        Commands::Start { path } => {
            let repo_root = get_repo_root(path.as_ref())?;
            info!("Starting file watcher for: {}", repo_root.display());
            
            // Check if index exists
            if !FlashgrepPaths::new(&repo_root).exists() {
                println!("⚠ No index found. Run 'flashgrep index' first.");
                return Ok(());
            }
            
            println!("Starting file watcher...");
            println!("Repository: {}", repo_root.display());
            
            // Start file watcher
            let watcher_root = repo_root.clone();
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
            info!("Stopping file watcher for: {}", repo_root.display());
            
            println!("Stopping file watcher...");
            
            // Currently, we don't have a way to stop the file watcher gracefully
            // This would require implementing a process management system
            println!("Note: File watcher will stop when the process is terminated (Ctrl+C)");
            
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
