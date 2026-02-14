//! MCP Protocol Implementation
//!
//! Implements the Model Context Protocol (MCP) using stdio transport.
//! This is the standard transport method used by most MCP clients.

use crate::config::paths::FlashgrepPaths;
use crate::db::Database;
use crate::search::Searcher;
use crate::FlashgrepResult;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::io::{self, BufRead, Write};
use std::path::PathBuf;
use tracing::{debug, error, info, warn};

/// MCP Server using stdio transport
pub struct McpStdioServer {
    paths: FlashgrepPaths,
}

impl McpStdioServer {
    /// Create a new MCP stdio server
    pub fn new(repo_root: PathBuf) -> FlashgrepResult<Self> {
        let paths = FlashgrepPaths::new(&repo_root);
        Ok(Self { paths })
    }

    /// Start the MCP server on stdio
    pub fn start(&self) -> FlashgrepResult<()> {
        info!("Starting MCP server on stdio");
        eprintln!("MCP server started on stdio");

        let stdin = io::stdin();
        let stdout = io::stdout();
        let mut stdout_lock = stdout.lock();
        let reader = stdin.lock();

        // Open Tantivy index for searching
        let tantivy_index = match tantivy::Index::open_in_dir(self.paths.text_index_dir()) {
            Ok(idx) => Some(idx),
            Err(e) => {
                warn!("Failed to open Tantivy index: {}", e);
                None
            }
        };

        for line in reader.lines() {
            let line = match line {
                Ok(l) => l,
                Err(e) => {
                    error!("Error reading line: {}", e);
                    continue;
                }
            };

            let trimmed_line = line.trim();
            if trimmed_line.is_empty() {
                continue;
            }

            debug!("Received: {}", trimmed_line);

            match serde_json::from_str::<JsonRpcRequest>(&line) {
                Ok(request) => {
                    let response = self.handle_request(request, tantivy_index.as_ref())?;
                    let response_json = serde_json::to_string(&response)?;
                    writeln!(stdout_lock, "{}", response_json)?;
                    stdout_lock.flush()?;
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
                    writeln!(stdout_lock, "{}", response_json)?;
                    stdout_lock.flush()?;
                }
            }
        }

        Ok(())
    }

    fn handle_request(
        &self,
        request: JsonRpcRequest,
        tantivy_index: Option<&tantivy::Index>,
    ) -> FlashgrepResult<JsonRpcResponse> {
        let result = match request.method.as_str() {
            "initialize" => {
                info!(
                    "MCP client connected: {:?}",
                    request.params.get("clientInfo")
                );

                Some(serde_json::json!({
                    "protocolVersion": "2024-11-05",
                    "serverInfo": {
                        "name": "flashgrep",
                        "version": env!("CARGO_PKG_VERSION"),
                    },
                    "capabilities": {
                        "tools": {
                            "listChanged": false,
                        },
                        "resources": {
                            "subscribe": false,
                            "listChanged": false,
                        },
                    },
                }))
            }
            "tools/list" => {
                // Return list of available tools
                Some(serde_json::json!({
                    "tools": [
                        {
                            "name": "query",
                            "description": "Search for text in the indexed codebase",
                            "inputSchema": {
                                "type": "object",
                                "properties": {
                                    "text": {
                                        "type": "string",
                                        "description": "Search text"
                                    },
                                    "limit": {
                                        "type": "integer",
                                        "description": "Maximum results",
                                        "default": 10
                                    }
                                },
                                "required": ["text"]
                            }
                        },
                        {
                            "name": "get_slice",
                            "description": "Get specific lines from a file",
                            "inputSchema": {
                                "type": "object",
                                "properties": {
                                    "file_path": {"type": "string"},
                                    "start_line": {"type": "integer"},
                                    "end_line": {"type": "integer"}
                                },
                                "required": ["file_path", "start_line", "end_line"]
                            }
                        },
                        {
                            "name": "get_symbol",
                            "description": "Find symbol definitions",
                            "inputSchema": {
                                "type": "object",
                                "properties": {
                                    "symbol_name": {"type": "string"}
                                },
                                "required": ["symbol_name"]
                            }
                        },
                        {
                            "name": "list_files",
                            "description": "List all indexed files",
                            "inputSchema": {
                                "type": "object",
                                "properties": {}
                            }
                        },
                        {
                            "name": "stats",
                            "description": "Get index statistics",
                            "inputSchema": {
                                "type": "object",
                                "properties": {}
                            }
                        }
                    ]
                }))
            }
            "tools/call" => {
                // Handle tool calls
                let tool_name = request
                    .params
                    .get("name")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                let arguments = request
                    .params
                    .get("arguments")
                    .cloned()
                    .unwrap_or(serde_json::json!({}));

                match tool_name {
                    "query" => self.handle_query_tool(&arguments, tantivy_index)?,
                    "get_slice" => self.handle_get_slice_tool(&arguments)?,
                    "get_symbol" => self.handle_get_symbol_tool(&arguments)?,
                    "list_files" => self.handle_list_files_tool()?,
                    "stats" => self.handle_stats_tool()?,
                    _ => {
                        return Ok(JsonRpcResponse {
                            jsonrpc: "2.0".to_string(),
                            id: request.id,
                            result: None,
                            error: Some(JsonRpcError {
                                code: -32601,
                                message: format!("Tool not found: {}", tool_name),
                                data: None,
                            }),
                        });
                    }
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

    fn handle_query_tool(
        &self,
        arguments: &Value,
        tantivy_index: Option<&tantivy::Index>,
    ) -> FlashgrepResult<Option<Value>> {
        let text = arguments.get("text").and_then(|v| v.as_str()).unwrap_or("");
        let limit = arguments
            .get("limit")
            .and_then(|v| v.as_u64())
            .unwrap_or(10) as usize;

        if text.is_empty() {
            return Ok(Some(serde_json::json!({
                "content": [{"type": "text", "text": "Error: Empty query"}],
                "isError": true
            })));
        }

        if let Some(index) = tantivy_index {
            let searcher = Searcher::new(index, &self.paths.metadata_db())?;
            match searcher.query(text, limit) {
                Ok(results) => {
                    let text_results: Vec<String> = results
                        .iter()
                        .map(|r| {
                            format!(
                                "{}:{}-{} (score: {:.2})\n{}",
                                r.file_path.display(),
                                r.start_line,
                                r.end_line,
                                r.relevance_score,
                                r.preview
                            )
                        })
                        .collect();

                    Ok(Some(serde_json::json!({
                        "content": [{"type": "text", "text": text_results.join("\n---\n")}]
                    })))
                }
                Err(e) => Ok(Some(serde_json::json!({
                    "content": [{"type": "text", "text": format!("Search error: {}", e)}],
                    "isError": true
                }))),
            }
        } else {
            Ok(Some(serde_json::json!({
                "content": [{"type": "text", "text": "Error: Search index not available"}],
                "isError": true
            })))
        }
    }

    fn handle_get_slice_tool(&self, arguments: &Value) -> FlashgrepResult<Option<Value>> {
        let file_path = arguments
            .get("file_path")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let start_line = arguments
            .get("start_line")
            .and_then(|v| v.as_u64())
            .unwrap_or(1) as usize;
        let end_line = arguments
            .get("end_line")
            .and_then(|v| v.as_u64())
            .unwrap_or(1) as usize;

        if file_path.is_empty() {
            return Ok(Some(serde_json::json!({
                "content": [{"type": "text", "text": "Error: Missing file_path"}],
                "isError": true
            })));
        }

        match std::fs::read_to_string(file_path) {
            Ok(content) => {
                let lines: Vec<&str> = content.lines().collect();
                let start = start_line.saturating_sub(1);
                let end = end_line.min(lines.len());

                if start < lines.len() {
                    let slice = lines[start..end].join("\n");
                    Ok(Some(serde_json::json!({
                        "content": [{"type": "text", "text": slice}]
                    })))
                } else {
                    Ok(Some(serde_json::json!({
                        "content": [{"type": "text", "text": "Error: Invalid line range"}],
                        "isError": true
                    })))
                }
            }
            Err(e) => Ok(Some(serde_json::json!({
                "content": [{"type": "text", "text": format!("Error reading file: {}", e)}],
                "isError": true
            }))),
        }
    }

    fn handle_get_symbol_tool(&self, arguments: &Value) -> FlashgrepResult<Option<Value>> {
        let symbol_name = arguments
            .get("symbol_name")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        if symbol_name.is_empty() {
            return Ok(Some(serde_json::json!({
                "content": [{"type": "text", "text": "Error: Missing symbol_name"}],
                "isError": true
            })));
        }

        let db = Database::open(&self.paths.metadata_db())?;
        match db.find_symbols_by_name(symbol_name) {
            Ok(symbols) => {
                let text: Vec<String> = symbols
                    .iter()
                    .map(|s| {
                        format!(
                            "{} {} ({}): {}:{}",
                            s.symbol_type,
                            s.symbol_name,
                            s.symbol_type,
                            s.file_path.display(),
                            s.line_number
                        )
                    })
                    .collect();

                Ok(Some(serde_json::json!({
                    "content": [{"type": "text", "text": text.join("\n")}]
                })))
            }
            Err(e) => Ok(Some(serde_json::json!({
                "content": [{"type": "text", "text": format!("Error: {}", e)}],
                "isError": true
            }))),
        }
    }

    fn handle_list_files_tool(&self) -> FlashgrepResult<Option<Value>> {
        let db = Database::open(&self.paths.metadata_db())?;
        match db.get_all_files() {
            Ok(files) => {
                let file_list: Vec<String> = files
                    .iter()
                    .map(|p| p.to_string_lossy().to_string())
                    .collect();
                Ok(Some(serde_json::json!({
                    "content": [{"type": "text", "text": file_list.join("\n")}]
                })))
            }
            Err(e) => Ok(Some(serde_json::json!({
                "content": [{"type": "text", "text": format!("Error: {}", e)}],
                "isError": true
            }))),
        }
    }

    fn handle_stats_tool(&self) -> FlashgrepResult<Option<Value>> {
        let db = Database::open(&self.paths.metadata_db())?;
        match db.get_stats() {
            Ok(stats) => {
                let text = format!(
                    "Files: {}\nChunks: {}\nSymbols: {}\nIndex size: {} MB\n",
                    stats.total_files,
                    stats.total_chunks,
                    stats.total_symbols,
                    stats.index_size_bytes / 1024 / 1024
                );
                Ok(Some(serde_json::json!({
                    "content": [{"type": "text", "text": text}]
                })))
            }
            Err(e) => Ok(Some(serde_json::json!({
                "content": [{"type": "text", "text": format!("Error: {}", e)}],
                "isError": true
            }))),
        }
    }
}

#[derive(Debug, Deserialize)]
struct JsonRpcRequest {
    #[allow(dead_code)]
    jsonrpc: String,
    method: String,
    #[serde(default)]
    params: Value,
    id: Option<u64>,
}

#[derive(Debug, Serialize)]
struct JsonRpcResponse {
    jsonrpc: String,
    id: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<JsonRpcError>,
}

#[derive(Debug, Serialize)]
struct JsonRpcError {
    code: i32,
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    data: Option<Value>,
}
