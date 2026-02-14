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
        // Existing methods
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
        // New MCP tool methods
        "search" => {
            let pattern = request.params.get("pattern").and_then(|v| v.as_str()).unwrap_or("");
            let files = request.params.get("files").and_then(|v| v.as_array()).cloned().unwrap_or_default();
            let case_sensitive = request.params.get("case_sensitive").and_then(|v| v.as_bool()).unwrap_or(true);
            
            if pattern.is_empty() {
                Some(serde_json::json!({
                    "results": [],
                    "error": "Empty pattern",
                }))
            } else {
                let mut results = Vec::new();
                
                for file in files {
                    if let Some(file_path) = file.as_str() {
                        if let Ok(content) = std::fs::read_to_string(file_path) {
                            let search_pattern = if case_sensitive {
                                pattern.to_string()
                            } else {
                                pattern.to_lowercase()
                            };
                            
                            for (line_num, line) in content.lines().enumerate() {
                                let line_to_check = if case_sensitive {
                                    line.to_string()
                                } else {
                                    line.to_lowercase()
                                };
                                
                                if line_to_check.contains(&search_pattern) {
                                    results.push(serde_json::json!({
                                        "file": file_path,
                                        "line": line_num + 1,
                                        "content": line,
                                    }));
                                }
                            }
                        }
                    }
                }
                
                Some(serde_json::json!({
                    "results": results,
                }))
            }
        }
        "search-in-directory" => {
            let pattern = request.params.get("pattern").and_then(|v| v.as_str()).unwrap_or("");
            let directory = request.params.get("directory").and_then(|v| v.as_str()).unwrap_or("");
            let extensions = request.params.get("extensions").and_then(|v| v.as_array()).cloned().unwrap_or_default();
            let case_sensitive = request.params.get("case_sensitive").and_then(|v| v.as_bool()).unwrap_or(true);
            
            if pattern.is_empty() || directory.is_empty() {
                Some(serde_json::json!({
                    "results": [],
                    "error": "Missing pattern or directory",
                }))
            } else {
                let mut results = Vec::new();
                
                if let Ok(dir_entries) = std::fs::read_dir(directory) {
                    for entry in dir_entries.flatten() {
                        if entry.file_type().map_or(false, |ft| ft.is_file()) {
                            let file_path = entry.path();
                            let file_name = file_path.to_string_lossy().to_string();
                            
                            // Check if file matches extensions
                            let matches_extension = if extensions.is_empty() {
                                true
                            } else {
                                extensions.iter().any(|ext| {
                                    if let Some(ext_str) = ext.as_str() {
                                        file_path.extension().map_or(false, |e| e == ext_str)
                                    } else {
                                        false
                                    }
                                })
                            };
                            
                            if matches_extension {
                                if let Ok(content) = std::fs::read_to_string(&file_path) {
                                    let search_pattern = if case_sensitive {
                                        pattern.to_string()
                                    } else {
                                        pattern.to_lowercase()
                                    };
                                    
                                    for (line_num, line) in content.lines().enumerate() {
                                        let line_to_check = if case_sensitive {
                                            line.to_string()
                                        } else {
                                            line.to_lowercase()
                                        };
                                        
                                        if line_to_check.contains(&search_pattern) {
                                            results.push(serde_json::json!({
                                                "file": file_name,
                                                "line": line_num + 1,
                                                "content": line,
                                            }));
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                
                Some(serde_json::json!({
                    "results": results,
                }))
            }
        }
        "search-with-context" => {
            let pattern = request.params.get("pattern").and_then(|v| v.as_str()).unwrap_or("");
            let files = request.params.get("files").and_then(|v| v.as_array()).cloned().unwrap_or_default();
            let context = request.params.get("context").and_then(|v| v.as_u64()).unwrap_or(1) as usize;
            let case_sensitive = request.params.get("case_sensitive").and_then(|v| v.as_bool()).unwrap_or(true);
            
            if pattern.is_empty() {
                Some(serde_json::json!({
                    "results": [],
                    "error": "Empty pattern",
                }))
            } else {
                let mut results = Vec::new();
                
                for file in files {
                    if let Some(file_path) = file.as_str() {
                        if let Ok(content) = std::fs::read_to_string(file_path) {
                            let lines: Vec<&str> = content.lines().collect();
                            let search_pattern = if case_sensitive {
                                pattern.to_string()
                            } else {
                                pattern.to_lowercase()
                            };
                            
                            for (line_num, line) in lines.iter().enumerate() {
                                let line_to_check = if case_sensitive {
                                    line.to_string()
                                } else {
                                    line.to_lowercase()
                                };
                                
                                if line_to_check.contains(&search_pattern) {
                                    let start = line_num.saturating_sub(context);
                                    let end = (line_num + context + 1).min(lines.len());
                                    
                                    let before = lines[start..line_num].to_vec();
                                    let after = lines[line_num + 1..end].to_vec();
                                    
                                    results.push(serde_json::json!({
                                        "file": file_path,
                                        "line": line_num + 1,
                                        "content": line,
                                        "context": {
                                            "before": before,
                                            "after": after,
                                        },
                                    }));
                                }
                            }
                        }
                    }
                }
                
                Some(serde_json::json!({
                    "results": results,
                }))
            }
        }
        "search-by-regex" => {
            let pattern = request.params.get("pattern").and_then(|v| v.as_str()).unwrap_or("");
            let files = request.params.get("files").and_then(|v| v.as_array()).cloned().unwrap_or_default();
            let flags = request.params.get("flags").and_then(|v| v.as_str()).unwrap_or("");
            
            if pattern.is_empty() {
                Some(serde_json::json!({
                    "results": [],
                    "error": "Empty pattern",
                }))
            } else {
                let mut results = Vec::new();
                
                // Build regex with flags
                let mut regex_builder = regex::RegexBuilder::new(pattern);
                if flags.contains('i') {
                    regex_builder.case_insensitive(true);
                }
                if flags.contains('m') {
                    regex_builder.multi_line(true);
                }
                if flags.contains('s') {
                    regex_builder.dot_matches_new_line(true);
                }
                
                match regex_builder.build() {
                    Ok(regex) => {
                        for file in files {
                            if let Some(file_path) = file.as_str() {
                                if let Ok(content) = std::fs::read_to_string(file_path) {
                                    for (line_num, line) in content.lines().enumerate() {
                                        if regex.is_match(line) {
                                            results.push(serde_json::json!({
                                                "file": file_path,
                                                "line": line_num + 1,
                                                "content": line,
                                            }));
                                        }
                                    }
                                }
                            }
                        }
                    }
                    Err(e) => {
                        return Ok(JsonRpcResponse {
                            jsonrpc: "2.0".to_string(),
                            id: request.id,
                            result: Some(serde_json::json!({
                                "results": [],
                                "error": format!("Invalid regex: {}", e),
                            })),
                            error: None,
                        });
                    }
                }
                
                Some(serde_json::json!({
                    "results": results,
                }))
            }
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
