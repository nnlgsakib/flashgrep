pub mod bootstrap;
pub mod code_io;
pub mod glob_tool;
pub mod safety;
pub mod skill;
pub mod stdio;
pub mod tools;

use crate::config::paths::FlashgrepPaths;
use crate::config::Config;
use crate::db::Database;
use crate::mcp::bootstrap::{build_bootstrap_payload, is_bootstrap_tool};
use crate::mcp::code_io::{read_code, write_code};
use crate::mcp::glob_tool::run_glob;
use crate::mcp::safety::{
    check_arguments_size, chunking_guidance, invalid_params_error, payload_too_large_error,
    MAX_MCP_GET_SLICE_BYTES, MAX_MCP_REQUEST_BYTES, MAX_MCP_RESPONSE_BYTES,
};
use crate::search::{QueryOptions, Searcher};
use crate::FlashgrepResult;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::atomic::AtomicBool;
use tokio::io::{AsyncBufReadExt, AsyncWrite, AsyncWriteExt, BufReader};
use tokio::net::{TcpListener, TcpStream};
use tracing::{debug, error, info};

static SKILL_INJECTED_TCP: AtomicBool = AtomicBool::new(false);

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
        let trimmed_line = line.trim();

        // Skip empty lines which might be keep-alive or protocol noise
        if trimmed_line.is_empty() {
            line.clear();
            continue;
        }

        if trimmed_line.as_bytes().len() > MAX_MCP_REQUEST_BYTES {
            let payload = payload_too_large_error(
                "request",
                trimmed_line.as_bytes().len(),
                MAX_MCP_REQUEST_BYTES,
                &chunking_guidance(MAX_MCP_REQUEST_BYTES),
            );
            let response = JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                id: None,
                result: Some(payload),
                error: None,
            };
            write_response_line(&mut writer, response).await?;
            line.clear();
            continue;
        }

        debug!("Received: {}", trimmed_line);

        match serde_json::from_str::<JsonRpcRequest>(trimmed_line) {
            Ok(request) => {
                let response = match handle_request(request, &paths, tantivy_index.as_ref()).await {
                    Ok(r) => r,
                    Err(e) => JsonRpcResponse {
                        jsonrpc: "2.0".to_string(),
                        id: None,
                        result: Some(invalid_params_error(&format!("request_failed: {}", e))),
                        error: None,
                    },
                };
                write_response_line(&mut writer, response).await?;
            }
            Err(e) => {
                // Log parse errors but don't send back responses for invalid protocol
                debug!(
                    "Failed to parse JSON-RPC request: {} for line: '{}'",
                    e, trimmed_line
                );
                // Skip sending response for invalid requests that aren't valid JSON-RPC
            }
        }

        line.clear();
    }

    Ok(())
}

async fn write_response_line<W>(writer: &mut W, response: JsonRpcResponse) -> FlashgrepResult<()>
where
    W: AsyncWrite + Unpin,
{
    let mut response_json = serde_json::to_string(&response)?;
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
    writer.write_all(response_json.as_bytes()).await?;
    writer.write_all(b"\n").await?;
    writer.flush().await?;
    Ok(())
}

async fn handle_request(
    request: JsonRpcRequest,
    paths: &FlashgrepPaths,
    tantivy_index: Option<&tantivy::Index>,
) -> FlashgrepResult<JsonRpcResponse> {
    let result = match request.method.as_str() {
        // Existing methods
        "query" => {
            let options = match QueryOptions::from_mcp_args(&request.params) {
                Ok(opts) => opts,
                Err(e) => {
                    return Ok(JsonRpcResponse {
                        jsonrpc: "2.0".to_string(),
                        id: request.id,
                        result: Some(serde_json::json!({
                            "results": [],
                            "error": "invalid_params",
                            "message": e.to_string(),
                        })),
                        error: None,
                    })
                }
            };

            if options.text.is_empty() {
                Some(serde_json::json!({
                    "results": [],
                    "query": options.text,
                    "limit": options.limit,
                    "error": "Empty query"
                }))
            } else {
                // Perform actual search using Tantivy
                let search_results = if let Some(index) = tantivy_index {
                    let searcher = Searcher::new(index, &paths.metadata_db())?;
                    match searcher.query_with_options(&options) {
                        Ok(response) => {
                            let json_results: Vec<_> = response
                                .results
                                .iter()
                                .map(|r| {
                                    serde_json::json!({
                                        "file_path": r.file_path.to_string_lossy(),
                                        "start_line": r.start_line,
                                        "end_line": r.end_line,
                                        "symbol_name": r.symbol_name,
                                        "relevance_score": r.relevance_score,
                                        "preview": r.preview,
                                    })
                                })
                                .collect();
                            serde_json::json!({
                                "results": json_results,
                                "query": options.text,
                                "limit": options.limit,
                                "total": response.results.len(),
                                "truncated": response.truncated,
                                "scanned_files": response.scanned_files,
                                "next_offset": response.next_offset,
                                "mode": format!("{:?}", options.mode).to_lowercase(),
                                "case_sensitive": options.case_sensitive,
                            })
                        }
                        Err(e) => {
                            error!("Search error: {}", e);
                            serde_json::json!({
                                "results": [],
                                "query": options.text,
                                "limit": options.limit,
                                "error": format!("Search failed: {}", e),
                            })
                        }
                    }
                } else {
                    serde_json::json!({
                        "results": [],
                        "query": options.text,
                        "limit": options.limit,
                        "error": "Search index not available",
                    })
                };
                Some(search_results)
            }
        }
        "get_slice" => {
            if let Err(e) = check_arguments_size(&request.params, MAX_MCP_REQUEST_BYTES) {
                return Ok(JsonRpcResponse {
                    jsonrpc: "2.0".to_string(),
                    id: request.id,
                    result: Some(invalid_params_error(&e.to_string())),
                    error: None,
                });
            }

            let file_path = request
                .params
                .get("file_path")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            let start_line = request
                .params
                .get("start_line")
                .and_then(|v| v.as_u64())
                .unwrap_or(1);
            let end_line = request
                .params
                .get("end_line")
                .and_then(|v| v.as_u64())
                .unwrap_or(1);

            if file_path.is_empty() {
                Some(serde_json::json!({"error": "Missing file_path parameter"}))
            } else {
                let mut args = serde_json::json!({
                    "file_path": file_path,
                    "start_line": start_line,
                    "end_line": end_line,
                    "max_bytes": MAX_MCP_GET_SLICE_BYTES,
                    "metadata_level": "standard"
                });
                if let Some(c) = request.params.get("continuation_start_line") {
                    args["continuation_start_line"] = c.clone();
                }
                if let Some(c) = request.params.get("chunk_index") {
                    args["chunk_index"] = c.clone();
                }

                match read_code(paths, &args) {
                    Ok(payload) => Some(serde_json::json!({
                        "file_path": payload["file_path"],
                        "start_line": payload["start_line"],
                        "end_line": payload["end_line"],
                        "content": payload["content"],
                        "truncated": payload["truncated"],
                        "continuation_start_line": payload["continuation_start_line"],
                        "continuation": payload["continuation"],
                        "applied_limits": payload["applied_limits"],
                    })),
                    Err(e) => Some(invalid_params_error(&e.to_string())),
                }
            }
        }
        "read_code" => {
            if let Err(e) = check_arguments_size(&request.params, MAX_MCP_REQUEST_BYTES) {
                Some(invalid_params_error(&e.to_string()))
            } else {
                match read_code(paths, &request.params) {
                    Ok(payload) => Some(payload),
                    Err(e) => Some(invalid_params_error(&e.to_string())),
                }
            }
        }
        "write_code" => {
            if let Err(e) = check_arguments_size(&request.params, MAX_MCP_REQUEST_BYTES) {
                Some(invalid_params_error(&e.to_string()))
            } else {
                match write_code(&request.params) {
                    Ok(payload) => Some(payload),
                    Err(e) => Some(invalid_params_error(&e.to_string())),
                }
            }
        }
        "glob" => match run_glob(&request.params) {
            Ok(payload) => Some(payload),
            Err(e) => Some(serde_json::json!({
                "results": [],
                "error": "invalid_params",
                "message": e.to_string(),
            })),
        },
        method if is_bootstrap_tool(method) => Some(handle_skill_bootstrap_payload(
            paths,
            request.method.as_str(),
            &request.params,
        )?),
        "get_symbol" => {
            let symbol_name = request
                .params
                .get("symbol_name")
                .and_then(|v| v.as_str())
                .unwrap_or("");

            if symbol_name.is_empty() {
                Some(serde_json::json!({
                    "error": "Missing symbol_name parameter",
                }))
            } else {
                let db = Database::open(&paths.metadata_db())?;
                let symbols = db.find_symbols_by_name(symbol_name)?;

                let json_symbols: Vec<_> = symbols
                    .iter()
                    .map(|s| {
                        serde_json::json!({
                            "symbol_name": s.symbol_name,
                            "file_path": s.file_path.to_string_lossy(),
                            "line_number": s.line_number,
                            "symbol_type": s.symbol_type.to_string(),
                        })
                    })
                    .collect();

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

            let file_strings: Vec<String> = files
                .iter()
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
            let pattern = request
                .params
                .get("pattern")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            let files = request
                .params
                .get("files")
                .and_then(|v| v.as_array())
                .cloned()
                .unwrap_or_default();
            let case_sensitive = request
                .params
                .get("case_sensitive")
                .and_then(|v| v.as_bool())
                .unwrap_or(true);

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
            let pattern = request
                .params
                .get("pattern")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            let directory = request
                .params
                .get("directory")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            let extensions = request
                .params
                .get("extensions")
                .and_then(|v| v.as_array())
                .cloned()
                .unwrap_or_default();
            let case_sensitive = request
                .params
                .get("case_sensitive")
                .and_then(|v| v.as_bool())
                .unwrap_or(true);

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
            let pattern = request
                .params
                .get("pattern")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            let files = request
                .params
                .get("files")
                .and_then(|v| v.as_array())
                .cloned()
                .unwrap_or_default();
            let context = request
                .params
                .get("context")
                .and_then(|v| v.as_u64())
                .unwrap_or(1) as usize;
            let case_sensitive = request
                .params
                .get("case_sensitive")
                .and_then(|v| v.as_bool())
                .unwrap_or(true);

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
            let pattern = request
                .params
                .get("pattern")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            let files = request
                .params
                .get("files")
                .and_then(|v| v.as_array())
                .cloned()
                .unwrap_or_default();
            let flags = request
                .params
                .get("flags")
                .and_then(|v| v.as_str())
                .unwrap_or("");

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

fn handle_skill_bootstrap_payload(
    paths: &FlashgrepPaths,
    requested_tool: &str,
    arguments: &serde_json::Value,
) -> FlashgrepResult<serde_json::Value> {
    build_bootstrap_payload(paths, requested_tool, arguments, &SKILL_INJECTED_TCP)
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mcp::safety::MAX_MCP_WRITE_REPLACEMENT_BYTES;
    use tempfile::TempDir;

    #[tokio::test]
    async fn glob_method_works_in_tcp_handler() {
        let tmp = TempDir::new().expect("temp dir");
        let root = tmp.path().to_path_buf();
        std::fs::create_dir_all(root.join("src")).expect("src dir");
        std::fs::write(root.join("src/main.rs"), "fn main() {}\n").expect("main file");

        let paths = FlashgrepPaths::new(&root);
        let req = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            method: "glob".to_string(),
            params: serde_json::json!({
                "path": root,
                "pattern": "**/*.rs",
                "limit": 10
            }),
            id: Some(1),
        };

        let response = handle_request(req, &paths, None).await.expect("response");
        let result = response.result.expect("result payload");
        assert!(result["total"].as_u64().unwrap_or(0) >= 1);
    }

    #[tokio::test]
    async fn oversized_write_error_does_not_break_followup_request() {
        let tmp = TempDir::new().expect("temp dir");
        let root = tmp.path().to_path_buf();
        std::fs::create_dir_all(root.join("src")).expect("src dir");
        std::fs::write(root.join("src/main.rs"), "fn main() {}\n").expect("main file");

        let paths = FlashgrepPaths::new(&root);
        let oversize = "x".repeat(MAX_MCP_WRITE_REPLACEMENT_BYTES + 1);

        let write_req = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            method: "write_code".to_string(),
            params: serde_json::json!({
                "file_path": root.join("src/main.rs").to_string_lossy(),
                "start_line": 1,
                "end_line": 1,
                "replacement": oversize,
            }),
            id: Some(1),
        };
        let write_resp = handle_request(write_req, &paths, None)
            .await
            .expect("write response");
        let write_payload = write_resp.result.expect("write result");
        assert_eq!(
            write_payload["error"],
            serde_json::Value::String("payload_too_large".to_string())
        );

        let follow_req = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            method: "get_slice".to_string(),
            params: serde_json::json!({
                "file_path": root.join("src/main.rs").to_string_lossy(),
                "start_line": 1,
                "end_line": 1
            }),
            id: Some(2),
        };
        let follow_resp = handle_request(follow_req, &paths, None)
            .await
            .expect("follow response");
        assert!(follow_resp.result.is_some());
    }
}
