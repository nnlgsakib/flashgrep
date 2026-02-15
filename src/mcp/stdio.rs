//! MCP Protocol Implementation
//!
//! Implements the Model Context Protocol (MCP) using stdio transport.
//! This is the standard transport method used by most MCP clients.

use crate::config::paths::FlashgrepPaths;
use crate::db::Database;
use crate::mcp::bootstrap::{build_bootstrap_payload, is_bootstrap_tool};
use crate::mcp::code_io::{read_code, read_code_input_schema, write_code, write_code_input_schema};
use crate::mcp::glob_tool::{glob_input_schema, run_glob};
use crate::mcp::safety::{
    check_arguments_size, chunking_guidance, payload_too_large_error, MAX_MCP_GET_SLICE_BYTES,
    MAX_MCP_REQUEST_BYTES, MAX_MCP_RESPONSE_BYTES,
};
use crate::mcp::tools::{create_bootstrap_tools, create_tools};
use crate::search::{QueryOptions, Searcher};
use crate::FlashgrepResult;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::io::{self, BufRead, Write};
use std::path::PathBuf;
use std::sync::atomic::AtomicBool;
use tracing::{debug, error, info, warn};

/// MCP Server using stdio transport
pub struct McpStdioServer {
    paths: FlashgrepPaths,
    skill_injected: AtomicBool,
}

impl McpStdioServer {
    /// Create a new MCP stdio server
    pub fn new(repo_root: PathBuf) -> FlashgrepResult<Self> {
        let paths = FlashgrepPaths::new(&repo_root);
        Ok(Self {
            paths,
            skill_injected: AtomicBool::new(false),
        })
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

            if trimmed_line.as_bytes().len() > MAX_MCP_REQUEST_BYTES {
                let error_response = JsonRpcResponse {
                    jsonrpc: "2.0".to_string(),
                    id: None,
                    result: Some(payload_too_large_error(
                        "request",
                        trimmed_line.as_bytes().len(),
                        MAX_MCP_REQUEST_BYTES,
                        &chunking_guidance(MAX_MCP_REQUEST_BYTES),
                    )),
                    error: None,
                };
                write_response_line(&mut stdout_lock, &error_response)?;
                continue;
            }

            debug!("Received: {}", trimmed_line);

            match serde_json::from_str::<JsonRpcRequest>(&line) {
                Ok(request) => {
                    let response = match self.handle_request(request, tantivy_index.as_ref()) {
                        Ok(r) => r,
                        Err(e) => JsonRpcResponse {
                            jsonrpc: "2.0".to_string(),
                            id: None,
                            result: Some(json!({
                                "error": "invalid_params",
                                "message": format!("request_failed: {}", e),
                            })),
                            error: None,
                        },
                    };
                    write_response_line(&mut stdout_lock, &response)?;
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
                    write_response_line(&mut stdout_lock, &error_response)?;
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
                let mut tools = vec![
                    json!({
                        "name": "query",
                        "description": "Search for text in the indexed codebase",
                        "inputSchema": {
                            "type": "object",
                            "properties": {
                                "text": {"type": "string", "description": "Search text"},
                                "limit": {"type": "integer", "description": "Maximum results", "default": 10},
                                "mode": {"type": "string", "enum": ["smart", "literal", "regex"], "default": "smart"},
                                "case_sensitive": {"type": "boolean", "default": true},
                                "regex_flags": {"type": "string", "description": "Regex flags (e.g. i for case-insensitive)"},
                                "include": {"type": "array", "items": {"type": "string"}},
                                "exclude": {"type": "array", "items": {"type": "string"}},
                                "context": {"type": "integer", "minimum": 0, "default": 0},
                                "offset": {"type": "integer", "minimum": 0, "default": 0}
                            },
                            "required": ["text"]
                        }
                    }),
                    json!({
                        "name": "get_slice",
                        "description": "Get specific lines from a file",
                        "inputSchema": {
                            "type": "object",
                            "properties": {
                                "file_path": {"type": "string"},
                                "start_line": {"type": "integer"},
                                "end_line": {"type": "integer"},
                                "continuation_start_line": {"type": "integer"},
                                "chunk_index": {"type": "integer", "minimum": 0}
                            },
                            "required": ["file_path", "start_line", "end_line"]
                        }
                    }),
                    json!({
                        "name": "read_code",
                        "description": "Token-efficient code read with deterministic budgets and continuation",
                        "inputSchema": read_code_input_schema()
                    }),
                    json!({
                        "name": "write_code",
                        "description": "Minimal-diff line range write with optional precondition checks",
                        "inputSchema": write_code_input_schema()
                    }),
                    json!({
                        "name": "glob",
                        "description": "Advanced glob discovery with filtering, sorting, and limits",
                        "inputSchema": glob_input_schema()
                    }),
                    json!({
                        "name": "get_symbol",
                        "description": "Find symbol definitions",
                        "inputSchema": {
                            "type": "object",
                            "properties": {"symbol_name": {"type": "string"}},
                            "required": ["symbol_name"]
                        }
                    }),
                    json!({
                        "name": "list_files",
                        "description": "List all indexed files",
                        "inputSchema": {"type": "object", "properties": {}}
                    }),
                    json!({
                        "name": "stats",
                        "description": "Get index statistics",
                        "inputSchema": {"type": "object", "properties": {}}
                    }),
                ];

                for def in create_tools()
                    .into_iter()
                    .chain(create_bootstrap_tools().into_iter())
                {
                    tools.push(json!({
                        "name": def.name,
                        "description": def.description,
                        "inputSchema": def.parameters,
                    }));
                }

                Some(json!({ "tools": tools }))
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
                    "read_code" => self.handle_read_code_tool(&arguments)?,
                    "write_code" => self.handle_write_code_tool(&arguments)?,
                    "glob" => self.handle_glob_tool(&arguments)?,
                    "get_symbol" => self.handle_get_symbol_tool(&arguments)?,
                    "list_files" => self.handle_list_files_tool()?,
                    "stats" => self.handle_stats_tool()?,
                    "search" => self.handle_search_tool(&arguments)?,
                    "search-in-directory" => self.handle_search_in_directory_tool(&arguments)?,
                    "search-with-context" => self.handle_search_with_context_tool(&arguments)?,
                    "search-by-regex" => self.handle_search_by_regex_tool(&arguments)?,
                    tool if is_bootstrap_tool(tool) => {
                        self.handle_skill_bootstrap_tool(tool_name, &arguments)?
                    }
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
        let options = match QueryOptions::from_mcp_args(arguments) {
            Ok(opts) => opts,
            Err(e) => {
                return Ok(Some(serde_json::json!({
                    "content": [{"type": "text", "text": serde_json::to_string(&json!({"error": "invalid_params", "message": e.to_string()}))?}],
                    "isError": true
                })))
            }
        };

        if options.text.is_empty() {
            return Ok(Some(serde_json::json!({
                "content": [{"type": "text", "text": "Error: Empty query"}],
                "isError": true
            })));
        }

        if let Some(index) = tantivy_index {
            let searcher = Searcher::new(index, &self.paths.metadata_db())?;
            match searcher.query_with_options(&options) {
                Ok(response) => {
                    let text_results: Vec<String> = response
                        .results
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

                    let payload = json!({
                        "results": text_results,
                        "total": response.results.len(),
                        "truncated": response.truncated,
                        "scanned_files": response.scanned_files,
                        "next_offset": response.next_offset,
                        "mode": format!("{:?}", options.mode).to_lowercase(),
                        "case_sensitive": options.case_sensitive,
                    });

                    Ok(Some(serde_json::json!({
                        "content": [{"type": "text", "text": serde_json::to_string(&payload)?}]
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
        if let Err(e) = check_arguments_size(arguments, MAX_MCP_REQUEST_BYTES) {
            return Ok(Some(json!({
                "content": [{"type": "text", "text": serde_json::to_string(&json!({"error": "invalid_params", "message": e.to_string()}))?}],
                "isError": true
            })));
        }

        let file_path = arguments
            .get("file_path")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let start_line = arguments
            .get("start_line")
            .and_then(|v| v.as_u64())
            .unwrap_or(1);
        let end_line = arguments
            .get("end_line")
            .and_then(|v| v.as_u64())
            .unwrap_or(1);

        if file_path.is_empty() {
            return Ok(Some(serde_json::json!({
                "content": [{"type": "text", "text": "Error: Missing file_path"}],
                "isError": true
            })));
        }

        let mut args = json!({
            "file_path": file_path,
            "start_line": start_line,
            "end_line": end_line,
            "max_bytes": MAX_MCP_GET_SLICE_BYTES,
            "metadata_level": "standard"
        });
        if let Some(c) = arguments.get("continuation_start_line") {
            args["continuation_start_line"] = c.clone();
        }
        if let Some(c) = arguments.get("chunk_index") {
            args["chunk_index"] = c.clone();
        }

        match read_code(&self.paths, &args) {
            Ok(payload) => Ok(Some(serde_json::json!({
                "content": [{"type": "text", "text": serde_json::to_string(&json!({
                    "file_path": payload["file_path"],
                    "start_line": payload["start_line"],
                    "end_line": payload["end_line"],
                    "content": payload["content"],
                    "truncated": payload["truncated"],
                    "continuation_start_line": payload["continuation_start_line"],
                    "continuation": payload["continuation"],
                    "applied_limits": payload["applied_limits"]
                }))?}]
            }))),
            Err(e) => Ok(Some(serde_json::json!({
                "content": [{"type": "text", "text": format!("Error: {}", e)}],
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

    fn handle_glob_tool(&self, arguments: &Value) -> FlashgrepResult<Option<Value>> {
        let payload = match run_glob(arguments) {
            Ok(payload) => payload,
            Err(e) => {
                return Ok(Some(json!({
                    "content": [{"type": "text", "text": serde_json::to_string(&json!({"error": "invalid_params", "message": e.to_string()}))?}],
                    "isError": true
                })))
            }
        };
        Ok(Some(json!({
            "content": [{"type": "text", "text": serde_json::to_string(&payload)?}]
        })))
    }

    fn handle_read_code_tool(&self, arguments: &Value) -> FlashgrepResult<Option<Value>> {
        if let Err(e) = check_arguments_size(arguments, MAX_MCP_REQUEST_BYTES) {
            return Ok(Some(json!({
                "content": [{"type": "text", "text": serde_json::to_string(&json!({"error": "invalid_params", "message": e.to_string()}))?}],
                "isError": true
            })));
        }

        let payload = match read_code(&self.paths, arguments) {
            Ok(v) => v,
            Err(e) => {
                return Ok(Some(json!({
                    "content": [{"type": "text", "text": serde_json::to_string(&json!({"error": "invalid_params", "message": e.to_string()}))?}],
                    "isError": true
                })));
            }
        };
        Ok(Some(serde_json::json!({
            "content": [{"type": "text", "text": serde_json::to_string(&payload)?}]
        })))
    }

    fn handle_write_code_tool(&self, arguments: &Value) -> FlashgrepResult<Option<Value>> {
        if let Err(e) = check_arguments_size(arguments, MAX_MCP_REQUEST_BYTES) {
            return Ok(Some(json!({
                "content": [{"type": "text", "text": serde_json::to_string(&json!({"error": "invalid_params", "message": e.to_string()}))?}],
                "isError": true
            })));
        }

        let payload = match write_code(arguments) {
            Ok(v) => v,
            Err(e) => {
                return Ok(Some(json!({
                    "content": [{"type": "text", "text": serde_json::to_string(&json!({"error": "invalid_params", "message": e.to_string()}))?}],
                    "isError": true
                })));
            }
        };
        let is_error = payload
            .get("ok")
            .and_then(|v| v.as_bool())
            .map(|ok| !ok)
            .unwrap_or(false);

        Ok(Some(serde_json::json!({
            "content": [{"type": "text", "text": serde_json::to_string(&payload)?}],
            "isError": is_error
        })))
    }

    fn handle_search_tool(&self, arguments: &Value) -> FlashgrepResult<Option<Value>> {
        let pattern = arguments
            .get("pattern")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let files = arguments
            .get("files")
            .and_then(|v| v.as_array())
            .cloned()
            .unwrap_or_default();
        let case_sensitive = arguments
            .get("case_sensitive")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);

        if pattern.is_empty() {
            return Ok(Some(json!({
                "content": [{"type": "text", "text": "Error: Empty pattern"}],
                "isError": true
            })));
        }

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
                            results.push(json!({
                                "file": file_path,
                                "line": line_num + 1,
                                "content": line,
                            }));
                        }
                    }
                }
            }
        }

        Ok(Some(json!({
            "content": [{"type": "text", "text": serde_json::to_string(&json!({"results": results}))?}]
        })))
    }

    fn handle_search_in_directory_tool(&self, arguments: &Value) -> FlashgrepResult<Option<Value>> {
        let pattern = arguments
            .get("pattern")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let directory = arguments
            .get("directory")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let extensions = arguments
            .get("extensions")
            .and_then(|v| v.as_array())
            .cloned()
            .unwrap_or_default();
        let case_sensitive = arguments
            .get("case_sensitive")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);

        if pattern.is_empty() || directory.is_empty() {
            return Ok(Some(json!({
                "content": [{"type": "text", "text": "Error: Missing pattern or directory"}],
                "isError": true
            })));
        }

        let mut results = Vec::new();
        if let Ok(dir_entries) = std::fs::read_dir(directory) {
            for entry in dir_entries.flatten() {
                if entry.file_type().map(|ft| ft.is_file()).unwrap_or(false) {
                    let file_path = entry.path();
                    let file_name = file_path.to_string_lossy().to_string();
                    let matches_extension = if extensions.is_empty() {
                        true
                    } else {
                        extensions.iter().any(|ext| {
                            ext.as_str()
                                .and_then(|ext_str| file_path.extension().map(|e| e == ext_str))
                                .unwrap_or(false)
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
                                    results.push(json!({
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

        Ok(Some(json!({
            "content": [{"type": "text", "text": serde_json::to_string(&json!({"results": results}))?}]
        })))
    }

    fn handle_search_with_context_tool(&self, arguments: &Value) -> FlashgrepResult<Option<Value>> {
        let pattern = arguments
            .get("pattern")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let files = arguments
            .get("files")
            .and_then(|v| v.as_array())
            .cloned()
            .unwrap_or_default();
        let context = arguments
            .get("context")
            .and_then(|v| v.as_u64())
            .unwrap_or(1) as usize;
        let case_sensitive = arguments
            .get("case_sensitive")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);

        if pattern.is_empty() {
            return Ok(Some(json!({
                "content": [{"type": "text", "text": "Error: Empty pattern"}],
                "isError": true
            })));
        }

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
                            (*line).to_string()
                        } else {
                            line.to_lowercase()
                        };
                        if line_to_check.contains(&search_pattern) {
                            let start = line_num.saturating_sub(context);
                            let end = (line_num + context + 1).min(lines.len());
                            let before: Vec<&str> = lines[start..line_num].to_vec();
                            let after: Vec<&str> = lines[line_num + 1..end].to_vec();
                            results.push(json!({
                                "file": file_path,
                                "line": line_num + 1,
                                "content": line,
                                "context": {"before": before, "after": after}
                            }));
                        }
                    }
                }
            }
        }

        Ok(Some(json!({
            "content": [{"type": "text", "text": serde_json::to_string(&json!({"results": results}))?}]
        })))
    }

    fn handle_search_by_regex_tool(&self, arguments: &Value) -> FlashgrepResult<Option<Value>> {
        let pattern = arguments
            .get("pattern")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let files = arguments
            .get("files")
            .and_then(|v| v.as_array())
            .cloned()
            .unwrap_or_default();
        let flags = arguments
            .get("flags")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        if pattern.is_empty() {
            return Ok(Some(json!({
                "content": [{"type": "text", "text": "Error: Empty pattern"}],
                "isError": true
            })));
        }

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
        let regex = match regex_builder.build() {
            Ok(r) => r,
            Err(e) => {
                return Ok(Some(json!({
                    "content": [{"type": "text", "text": format!("Error: Invalid regex: {}", e)}],
                    "isError": true
                })));
            }
        };

        let mut results = Vec::new();
        for file in files {
            if let Some(file_path) = file.as_str() {
                if let Ok(content) = std::fs::read_to_string(file_path) {
                    for (line_num, line) in content.lines().enumerate() {
                        if regex.is_match(line) {
                            results.push(json!({
                                "file": file_path,
                                "line": line_num + 1,
                                "content": line,
                            }));
                        }
                    }
                }
            }
        }

        Ok(Some(json!({
            "content": [{"type": "text", "text": serde_json::to_string(&json!({"results": results}))?}]
        })))
    }

    fn handle_skill_bootstrap_tool(
        &self,
        requested_tool: &str,
        arguments: &Value,
    ) -> FlashgrepResult<Option<Value>> {
        let payload =
            build_bootstrap_payload(&self.paths, requested_tool, arguments, &self.skill_injected)?;
        let is_error = payload
            .get("ok")
            .and_then(Value::as_bool)
            .map(|ok| !ok)
            .unwrap_or(false);
        Ok(Some(json!({
            "content": [{"type": "text", "text": serde_json::to_string(&payload)?}],
            "isError": is_error
        })))
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

fn write_response_line<W: Write>(
    writer: &mut W,
    response: &JsonRpcResponse,
) -> FlashgrepResult<()> {
    let mut response_json = serde_json::to_string(response)?;
    if response_json.as_bytes().len() > MAX_MCP_RESPONSE_BYTES {
        let fallback = JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id: response.id,
            result: Some(payload_too_large_error(
                "response",
                response_json.as_bytes().len(),
                MAX_MCP_RESPONSE_BYTES,
                &chunking_guidance(MAX_MCP_RESPONSE_BYTES),
            )),
            error: None,
        };
        response_json = serde_json::to_string(&fallback)?;
    }

    writeln!(writer, "{}", response_json)?;
    writer.flush()?;
    Ok(())
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mcp::safety::MAX_MCP_WRITE_REPLACEMENT_BYTES;
    use serde_json::json;
    use std::fs;
    use tempfile::TempDir;

    fn setup_server_with_skill(skill_text: Option<&str>) -> (TempDir, McpStdioServer) {
        let temp = TempDir::new().expect("temp dir");
        let repo_root = temp.path().to_path_buf();
        let skills_dir = repo_root.join("skills");
        fs::create_dir_all(&skills_dir).expect("skills dir");
        if let Some(text) = skill_text {
            fs::write(skills_dir.join("SKILL.md"), text).expect("write skill file");
        }
        let server = McpStdioServer::new(repo_root).expect("create server");
        (temp, server)
    }

    fn payload_text(result: Option<Value>) -> Value {
        let envelope = result.expect("tool envelope");
        let text = envelope["content"][0]["text"]
            .as_str()
            .expect("text content");
        serde_json::from_str(text).expect("json payload")
    }

    #[test]
    fn bootstrap_success_includes_metadata_and_policy() {
        let (_temp, server) = setup_server_with_skill(Some("# skill"));
        let payload = payload_text(
            server
                .handle_skill_bootstrap_tool("flashgrep_init", &json!({"compact": true}))
                .expect("bootstrap result"),
        );

        assert_eq!(payload["ok"], Value::Bool(true));
        assert_eq!(payload["status"], Value::String("injected".to_string()));
        assert_eq!(
            payload["canonical_trigger"],
            Value::String("flashgrep-init".to_string())
        );
        assert!(payload["skill_hash"].as_str().is_some());
        assert!(payload["skill_version"].as_str().is_some());
        assert!(payload["policy"].is_array());
    }

    #[test]
    fn bootstrap_aliases_and_invalid_trigger_behavior() {
        let (_temp, server) = setup_server_with_skill(Some("# skill"));

        let alias_payload = payload_text(
            server
                .handle_skill_bootstrap_tool("fgrep-boot", &json!({"compact": true, "force": true}))
                .expect("alias bootstrap result"),
        );
        let canonical_payload = payload_text(
            server
                .handle_skill_bootstrap_tool(
                    "bootstrap_skill",
                    &json!({"compact": true, "force": true}),
                )
                .expect("canonical bootstrap result"),
        );
        assert_eq!(
            alias_payload["canonical_trigger"],
            Value::String("flashgrep-init".to_string())
        );
        assert_eq!(
            alias_payload["policy_metadata"]["policy_strength"],
            Value::String("strict".to_string())
        );
        assert_eq!(
            alias_payload["policy_metadata"],
            canonical_payload["policy_metadata"]
        );

        let invalid_payload = payload_text(
            server
                .handle_skill_bootstrap_tool("bootstrap_skill", &json!({"trigger": "bad-trigger"}))
                .expect("invalid bootstrap response"),
        );
        assert_eq!(
            invalid_payload["error"],
            Value::String("invalid_trigger".to_string())
        );
    }

    #[test]
    fn bootstrap_repeated_call_returns_already_injected() {
        let (_temp, server) = setup_server_with_skill(Some("# skill"));

        let _ = server
            .handle_skill_bootstrap_tool("flashgrep-init", &json!({"compact": true}))
            .expect("first bootstrap");

        let second = payload_text(
            server
                .handle_skill_bootstrap_tool("flashgrep-init", &json!({"compact": true}))
                .expect("second bootstrap"),
        );
        assert_eq!(
            second["status"],
            Value::String("already_injected".to_string())
        );
    }

    #[test]
    fn bootstrap_missing_or_unreadable_skill_errors() {
        let temp_missing = TempDir::new().expect("temp dir");
        let server_missing =
            McpStdioServer::new(temp_missing.path().to_path_buf()).expect("create server");
        let missing = payload_text(
            server_missing
                .handle_skill_bootstrap_tool("flashgrep-init", &json!({"compact": true}))
                .expect("missing skill response"),
        );
        assert_eq!(
            missing["error"],
            Value::String("skill_not_found".to_string())
        );

        let temp_unreadable = TempDir::new().expect("temp dir");
        let skills_dir = temp_unreadable.path().join("skills");
        fs::create_dir_all(skills_dir.join("SKILL.md"))
            .expect("create directory in place of skill");
        let server_unreadable =
            McpStdioServer::new(temp_unreadable.path().to_path_buf()).expect("create server");
        let unreadable = payload_text(
            server_unreadable
                .handle_skill_bootstrap_tool("flashgrep-init", &json!({"compact": true}))
                .expect("unreadable skill response"),
        );
        assert_eq!(
            unreadable["error"],
            Value::String("skill_unreadable".to_string())
        );
    }

    #[test]
    fn glob_tool_works_in_stdio_handler() {
        let temp = TempDir::new().expect("temp dir");
        let root = temp.path().to_path_buf();
        std::fs::create_dir_all(root.join("src")).expect("src dir");
        std::fs::write(root.join("src/lib.rs"), "pub fn x() {}\n").expect("write file");

        let server = McpStdioServer::new(root.clone()).expect("server");
        let envelope = server
            .handle_glob_tool(&json!({
                "path": root,
                "pattern": "**/*.rs",
                "limit": 5
            }))
            .expect("glob result")
            .expect("glob envelope");
        let payload_text = envelope["content"][0]["text"]
            .as_str()
            .expect("content text");
        let payload: Value = serde_json::from_str(payload_text).expect("payload json");
        assert!(payload["total"].as_u64().unwrap_or(0) >= 1);
    }

    #[test]
    fn oversized_write_error_does_not_break_followup_tool_calls() {
        let temp = TempDir::new().expect("temp dir");
        let root = temp.path().to_path_buf();
        std::fs::create_dir_all(root.join("src")).expect("src dir");
        std::fs::write(root.join("src/lib.rs"), "pub fn x() {}\n").expect("write file");

        let server = McpStdioServer::new(root.clone()).expect("server");
        let oversize = "x".repeat(MAX_MCP_WRITE_REPLACEMENT_BYTES + 1);
        let write = server
            .handle_write_code_tool(&json!({
                "file_path": root.join("src/lib.rs").to_string_lossy(),
                "start_line": 1,
                "end_line": 1,
                "replacement": oversize
            }))
            .expect("write response")
            .expect("write envelope");
        let write_payload: Value = serde_json::from_str(
            write["content"][0]["text"]
                .as_str()
                .expect("write payload text"),
        )
        .expect("write payload");
        assert_eq!(
            write_payload["error"],
            Value::String("payload_too_large".to_string())
        );

        let follow = server
            .handle_get_slice_tool(&json!({
                "file_path": root.join("src/lib.rs").to_string_lossy(),
                "start_line": 1,
                "end_line": 1
            }))
            .expect("follow response")
            .expect("follow envelope");
        assert!(follow["content"][0]["text"].as_str().is_some());
    }
}
