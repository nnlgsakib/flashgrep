use crate::config::paths::FlashgrepPaths;
use crate::config::Config;
use crate::db::Database;
use crate::search::Searcher;
use crate::FlashgrepResult;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{TcpListener, TcpStream};
use tracing::{debug, error, info};

/// MCP server for handling JSON-RPC requests
pub struct McpServer {
    config: Config,
    paths: FlashgrepPaths,
}

impl McpServer {
    /// Create a new MCP server
    pub fn new(repo_root: PathBuf) -> FlashgrepResult<Self> {
        let paths = FlashgrepPaths::new(&repo_root);
        let config = if paths.config_file().exists() {
            Config::from_file(&paths.config_file())?
        } else {
            Config::default()
        };
        
        Ok(Self { config, paths })
    }
    
    /// Start the MCP server
    pub async fn start(&self) -> FlashgrepResult<()> {
        let addr = format!("127.0.0.1:{}", self.config.mcp_port);
        let listener = TcpListener::bind(&addr).await?;
        
        info!("MCP server listening on: {}", addr);
        println!("MCP server listening on: {}", addr);
        
        loop {
            let (stream, addr) = listener.accept().await?;
            debug!("New connection from: {}", addr);
            
            let paths = self.paths.clone();
            tokio::spawn(async move {
                if let Err(e) = handle_connection(stream, paths).await {
                    error!("Connection error: {}", e);
                }
            });
        }
    }
}

async fn handle_connection(mut stream: TcpStream, paths: FlashgrepPaths) -> FlashgrepResult<()> {
    let (reader, mut writer) = stream.split();
    let mut reader = BufReader::new(reader);
    let mut line = String::new();
    
    // Open Tantivy index for searching
    let tantivy_index = match tantivy::Index::open_in_dir(paths.text_index_dir()) {
        Ok(idx) => Some(idx),
        Err(e) => {
            error!("Failed to open Tantivy index: {}", e);
            None
        }
    };
    
    while reader.read_line(&mut line).await? > 0 {
        debug!("Received: {}", line.trim());
        
        match serde_json::from_str::<JsonRpcRequest>(&line) {
            Ok(request) => {
                let response = handle_request(request, &paths, tantivy_index.as_ref()).await?;
                let response_json = serde_json::to_string(&response)?;
                writer.write_all(response_json.as_bytes()).await?;
                writer.write_all(b"\n").await?;
                writer.flush().await?;
            }
            Err(e) => {
                error!("Failed to parse JSON-RPC request: {}", e);
                let error_response = JsonRpcResponse {
                    jsonrpc: "2.0".to_string(),
                    id: None,
                    result: None,
                    error: Some(JsonRpcError {
                        code: -32700,
                        message: "Parse error".to_string(),
                        data: None,
                    }),
                };
                let response_json = serde_json::to_string(&error_response)?;
                writer.write_all(response_json.as_bytes()).await?;
                writer.write_all(b"\n").await?;
                writer.flush().await?;
            }
        }
        
        line.clear();
    }
    
    Ok(())
}

async fn handle_request(
    request: JsonRpcRequest, 
    paths: &FlashgrepPaths,
    tantivy_index: Option<&tantivy::Index>
) -> FlashgrepResult<JsonRpcResponse> {
    let result = match request.method.as_str() {
        "query" => {
            let text = request.params.get("text").and_then(|v| v.as_str()).unwrap_or("");
            let limit = request.params.get("limit").and_then(|v| v.as_u64()).unwrap_or(10) as usize;
            
            if text.is_empty() {
                Some(serde_json::json!({
                    "results": [],
                    "query": text,
                    "limit": limit,
                    "error": "Empty query"
                }))
            } else {
                // Perform actual search using Tantivy
                let search_results = if let Some(index) = tantivy_index {
                    let searcher = Searcher::new(index, &paths.metadata_db())?;
                    match searcher.query(text, limit) {
                        Ok(results) => {
                            let json_results: Vec<_> = results.iter().map(|r| {
                                serde_json::json!({
                                    "file_path": r.file_path.to_string_lossy(),
                                    "start_line": r.start_line,
                                    "end_line": r.end_line,
                                    "symbol_name": r.symbol_name,
                                    "relevance_score": r.relevance_score,
                                    "preview": r.preview,
                                })
                            }).collect();
                            serde_json::json!({
                                "results": json_results,
                                "query": text,
                                "limit": limit,
                                "total": results.len(),
                            })
                        }
                        Err(e) => {
                            error!("Search error: {}", e);
                            serde_json::json!({
                                "results": [],
                                "query": text,
                                "limit": limit,
                                "error": format!("Search failed: {}", e),
                            })
                        }
                    }
                } else {
                    serde_json::json!({
                        "results": [],
                        "query": text,
                        "limit": limit,
                        "error": "Search index not available",
                    })
                };
                Some(search_results)
            }
        }
        "get_slice" => {
            let file_path = request.params.get("file_path").and_then(|v| v.as_str()).unwrap_or("");
            let start_line = request.params.get("start_line").and_then(|v| v.as_u64()).unwrap_or(1) as usize;
            let end_line = request.params.get("end_line").and_then(|v| v.as_u64()).unwrap_or(1) as usize;
            
            if file_path.is_empty() {
                Some(serde_json::json!({
                    "error": "Missing file_path parameter",
                }))
            } else {
                let content = match std::fs::read_to_string(file_path) {
                    Ok(c) => c,
                    Err(e) => {
                        return Ok(JsonRpcResponse {
                            jsonrpc: "2.0".to_string(),
                            id: request.id,
                            result: Some(serde_json::json!({
                                "error": format!("File not found: {}", e),
                                "file_path": file_path,
                            })),
                            error: None,
                        });
                    }
                };
                let lines: Vec<&str> = content.lines().collect();
                let start = start_line.saturating_sub(1);
                let end = end_line.min(lines.len());
                
                if start < lines.len() {
                    let slice = lines[start..end].join("\n");
                    Some(serde_json::json!({
                        "file_path": file_path,
                        "start_line": start_line,
                        "end_line": end_line,
                        "content": slice,
                        "total_lines": lines.len(),
                    }))
                } else {
                    Some(serde_json::json!({
                        "error": "Invalid line range",
                        "file_path": file_path,
                        "requested_start": start_line,
                        "total_lines": lines.len(),
                    }))
                }
            }
        }
        "get_symbol" => {
            let symbol_name = request.params.get("symbol_name").and_then(|v| v.as_str()).unwrap_or("");
            
            if symbol_name.is_empty() {
                Some(serde_json::json!({
                    "error": "Missing symbol_name parameter",
                }))
            } else {
                let db = Database::open(&paths.metadata_db())?;
                let symbols = db.find_symbols_by_name(symbol_name)?;
                
                let json_symbols: Vec<_> = symbols.iter().map(|s| {
                    serde_json::json!({
                        "symbol_name": s.symbol_name,
                        "file_path": s.file_path.to_string_lossy(),
                        "line_number": s.line_number,
                        "symbol_type": s.symbol_type.to_string(),
                    })
                }).collect();
                
                Some(serde_json::json!({
                    "symbol_name": symbol_name,
                    "symbols": json_symbols,
                    "total": symbols.len(),
                }))
            }
        }
        "list_files" => {
            let db = Database::open(&paths.metadata_db())?;
            let files = db.get_all_files()?;
            
            let file_strings: Vec<String> = files.iter()
                .map(|p| p.to_string_lossy().to_string())
                .collect();
            
            Some(serde_json::json!({
                "files": file_strings,
                "total": files.len(),
            }))
        }
        "stats" => {
            let db = Database::open(&paths.metadata_db())?;
            let stats = db.get_stats()?;
            
            Some(serde_json::json!({
                "total_files": stats.total_files,
                "total_chunks": stats.total_chunks,
                "total_symbols": stats.total_symbols,
                "index_size_bytes": stats.index_size_bytes,
                "index_size_mb": stats.index_size_bytes / 1024 / 1024,
                "last_update": stats.last_update,
            }))
        }
        _ => {
            return Ok(JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                id: request.id,
                result: None,
                error: Some(JsonRpcError {
                    code: -32601,
                    message: format!("Method not found: {}", request.method),
                    data: None,
                }),
            });
        }
    };
    
    Ok(JsonRpcResponse {
        jsonrpc: "2.0".to_string(),
        id: request.id,
        result,
        error: None,
    })
}

#[derive(Debug, Deserialize)]
struct JsonRpcRequest {
    #[allow(dead_code)]
    jsonrpc: String,
    method: String,
    params: serde_json::Value,
    id: Option<u64>,
}

#[derive(Debug, Serialize)]
struct JsonRpcResponse {
    jsonrpc: String,
    id: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    result: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<JsonRpcError>,
}

#[derive(Debug, Serialize)]
struct JsonRpcError {
    code: i32,
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    data: Option<serde_json::Value>,
}
