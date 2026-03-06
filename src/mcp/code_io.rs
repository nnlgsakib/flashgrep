use crate::config::paths::FlashgrepPaths;
use crate::db::Database;
use crate::mcp::safety::{
    chunking_guidance, continuation_meta, payload_too_large_error, MAX_MCP_READ_BYTES,
    MAX_MCP_WRITE_REPLACEMENT_BYTES, REASON_BATCH_DUPLICATE_OPERATION_ID,
    REASON_BATCH_DUPLICATE_TARGET, REASON_BATCH_OVERLAPPING_OPERATIONS, REASON_FILE_NOT_FOUND,
    REASON_INTERNAL_ERROR, REASON_INVALID_RANGE, REASON_PAYLOAD_TOO_LARGE,
    REASON_PRECONDITION_FAILED,
};
use crate::{FlashgrepError, FlashgrepResult};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

const DEFAULT_SYMBOL_CONTEXT_LINES: usize = 20;

pub fn read_code_input_schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            "file_path": {"type": "string", "description": "File to read (slice mode)"},
            "symbol_name": {"type": "string", "description": "Symbol name to resolve (symbol mode)"},
            "start_line": {"type": "integer", "minimum": 1, "description": "1-indexed start line for slice mode"},
            "end_line": {"type": "integer", "minimum": 1, "description": "1-indexed end line for slice mode"},
            "continuation_start_line": {"type": "integer", "minimum": 1, "description": "Start line for continuation reads"},
            "symbol_context_lines": {"type": "integer", "minimum": 0, "description": "Context lines around resolved symbol"},
            "max_tokens": {"type": "integer", "minimum": 1, "description": "Approximate token budget"},
            "max_bytes": {"type": "integer", "minimum": 1, "description": "Byte budget"},
            "max_lines": {"type": "integer", "minimum": 1, "description": "Line budget"},
            "chunk_index": {"type": "integer", "minimum": 0, "description": "Continuation chunk index"},
            "metadata_level": {
                "type": "string",
                "enum": ["minimal", "standard"],
                "default": "standard",
                "description": "Response verbosity"
            }
        }
    })
}

pub fn write_code_input_schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            "file_path": {"type": "string", "description": "Target file path"},
            "start_line": {"type": "integer", "minimum": 1, "description": "1-indexed start line (inclusive)"},
            "end_line": {"type": "integer", "minimum": 1, "description": "1-indexed end line (inclusive)"},
            "replacement": {"type": "string", "description": "Replacement text for the line range"},
            "continuation_id": {"type": "string", "description": "Write continuation session identifier"},
            "chunk_index": {"type": "integer", "minimum": 0, "description": "Chunk index for continuation writes"},
            "is_final_chunk": {"type": "boolean", "description": "Whether this chunk finalizes the write"},
            "precondition": {
                "type": "object",
                "properties": {
                    "expected_file_hash": {"type": "string"},
                    "expected_start_line_text": {"type": "string"},
                    "expected_end_line_text": {"type": "string"}
                }
            }
        },
        "required": ["file_path", "start_line", "end_line", "replacement"]
    })
}

pub fn batch_write_code_input_schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            "mode": {
                "type": "string",
                "enum": ["atomic", "best_effort"],
                "default": "atomic",
                "description": "Batch mode: atomic(all-or-nothing) or best_effort"
            },
            "dry_run": {
                "type": "boolean",
                "description": "Validate/preflight only without writing"
            },
            "operations": {
                "type": "array",
                "minItems": 1,
                "items": {
                    "type": "object",
                    "properties": {
                        "id": {"type": "string"},
                        "file_path": {"type": "string"},
                        "start_line": {"type": "integer", "minimum": 1},
                        "end_line": {"type": "integer", "minimum": 1},
                        "replacement": {"type": "string"},
                        "precondition": {
                            "type": "object",
                            "properties": {
                                "expected_file_hash": {"type": "string"},
                                "expected_start_line_text": {"type": "string"},
                                "expected_end_line_text": {"type": "string"}
                            }
                        }
                    },
                    "required": ["id", "file_path", "start_line", "end_line", "replacement"]
                }
            }
        },
        "required": ["operations"]
    })
}

pub fn read_code(paths: &FlashgrepPaths, arguments: &Value) -> FlashgrepResult<Value> {
    let metadata_level = parse_metadata_level(arguments)?;
    let mode = parse_read_mode(arguments)?;
    let limits = parse_limits(arguments)?;

    let read_target = match mode {
        ReadMode::FileSlice { file_path } => {
            let start_line = arguments
                .get("continuation_start_line")
                .and_then(Value::as_u64)
                .map(|n| n as usize)
                .or_else(|| {
                    arguments
                        .get("start_line")
                        .and_then(Value::as_u64)
                        .map(|n| n as usize)
                })
                .unwrap_or(1);
            let requested_end_line = arguments
                .get("end_line")
                .and_then(Value::as_u64)
                .map(|n| n as usize);
            read_file_slice(file_path, start_line, requested_end_line, None)?
        }
        ReadMode::Symbol { symbol_name } => {
            let context_lines = arguments
                .get("symbol_context_lines")
                .and_then(Value::as_u64)
                .map(|n| n as usize)
                .unwrap_or(DEFAULT_SYMBOL_CONTEXT_LINES);
            read_symbol_slice(paths, symbol_name, context_lines)?
        }
    };

    let bounded = match apply_budgets(&read_target.lines, &limits) {
        Some(value) => value,
        None => {
            let observed_bytes = read_target.lines.first().map(|(_, l)| l.len()).unwrap_or(0);
            return Ok(payload_too_large_error(
                "read_code",
                observed_bytes,
                limits.max_bytes.unwrap_or(MAX_MCP_READ_BYTES),
                &chunking_guidance(limits.max_bytes.unwrap_or(MAX_MCP_READ_BYTES)),
            ));
        }
    };

    let content = bounded
        .included_lines
        .iter()
        .map(|(_, line)| line.as_str())
        .collect::<Vec<_>>()
        .join("\n");

    let mut response = json!({
        "file_path": read_target.file_path,
        "content": content,
        "start_line": bounded.first_line,
        "end_line": bounded.last_line,
        "truncated": bounded.truncated,
        "continuation_start_line": bounded.next_start_line,
        "applied_limits": {
            "max_lines": limits.max_lines,
            "max_bytes": limits.max_bytes,
            "max_tokens": limits.max_tokens,
            "server_max_bytes": MAX_MCP_READ_BYTES,
            "consumed_lines": bounded.consumed_lines,
            "consumed_bytes": bounded.consumed_bytes,
            "consumed_tokens": bounded.consumed_tokens
        }
    });

    let chunk_index = arguments
        .get("chunk_index")
        .and_then(Value::as_u64)
        .unwrap_or(0) as usize;
    response["continuation"] = continuation_meta(
        json!({
            "continuation_start_line": bounded.next_start_line,
            "file_path": read_target.file_path,
        }),
        chunk_index,
        !bounded.truncated,
    );

    if metadata_level == MetadataLevel::Standard {
        response["mode"] = Value::String(read_target.mode_name.to_string());
        response["total_lines_available"] = Value::Number((read_target.lines.len() as u64).into());
        if let Some(symbol_name) = read_target.symbol_name {
            response["symbol_name"] = Value::String(symbol_name);
        }
    }

    Ok(response)
}

pub fn write_code(arguments: &Value) -> FlashgrepResult<Value> {
    let file_path = arguments
        .get("file_path")
        .and_then(Value::as_str)
        .ok_or_else(|| {
            FlashgrepError::Config("Missing required parameter: file_path".to_string())
        })?;
    let start_line = get_required_usize(arguments, "start_line")?;
    let end_line = get_required_usize(arguments, "end_line")?;
    let replacement = arguments
        .get("replacement")
        .and_then(Value::as_str)
        .ok_or_else(|| {
            FlashgrepError::Config("Missing required parameter: replacement".to_string())
        })?;

    if let Some(id) = arguments.get("continuation_id").and_then(Value::as_str) {
        return write_code_chunked(arguments, id, file_path, start_line, end_line, replacement);
    }

    let replacement_size = replacement.len();
    if replacement_size > MAX_MCP_WRITE_REPLACEMENT_BYTES {
        let mut payload = payload_too_large_error(
            "write_code",
            replacement_size,
            MAX_MCP_WRITE_REPLACEMENT_BYTES,
            &chunking_guidance(MAX_MCP_WRITE_REPLACEMENT_BYTES),
        );
        payload["ok"] = Value::Bool(false);
        payload["file_path"] = Value::String(file_path.to_string());
        return Ok(payload);
    }

    if start_line == 0 || end_line == 0 || start_line > end_line {
        return Err(FlashgrepError::Config(
            "Invalid range: start_line and end_line must be >= 1 and start_line <= end_line"
                .to_string(),
        ));
    }

    let path = PathBuf::from(file_path);
    let original_content = std::fs::read_to_string(&path)?;
    let original_hash = calculate_sha256(&original_content);
    let had_trailing_newline = original_content.ends_with('\n');

    let original_lines: Vec<String> = original_content.lines().map(ToString::to_string).collect();
    if original_lines.is_empty() {
        return Err(FlashgrepError::Config(
            "Cannot apply line-range write to empty file".to_string(),
        ));
    }

    if end_line > original_lines.len() {
        return Err(FlashgrepError::Config(format!(
            "Invalid range: end_line {} exceeds file line count {}",
            end_line,
            original_lines.len()
        )));
    }

    let conflict = check_preconditions(
        arguments.get("precondition"),
        &original_lines,
        &original_hash,
        start_line,
        end_line,
    );
    if let Some(conflict_payload) = conflict {
        return Ok(json!({
            "ok": false,
            "error": "precondition_failed",
            "file_path": file_path,
            "conflict": conflict_payload
        }));
    }

    let replacement_lines: Vec<String> = if replacement.is_empty() {
        Vec::new()
    } else {
        replacement.split('\n').map(ToString::to_string).collect()
    };

    let mut new_lines = Vec::new();
    new_lines.extend_from_slice(&original_lines[..start_line - 1]);
    new_lines.extend(replacement_lines.iter().cloned());
    new_lines.extend_from_slice(&original_lines[end_line..]);

    let mut new_content = new_lines.join("\n");
    if had_trailing_newline {
        new_content.push('\n');
    }

    std::fs::write(&path, &new_content)?;
    let new_hash = calculate_sha256(&new_content);

    Ok(json!({
        "ok": true,
        "file_path": file_path,
        "start_line": start_line,
        "end_line": end_line,
        "replaced_line_count": end_line - start_line + 1,
        "new_line_count": replacement_lines.len(),
        "file_hash_before": original_hash,
        "file_hash_after": new_hash
    }))
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum BatchMode {
    Atomic,
    BestEffort,
}

#[derive(Clone)]
struct BatchOperation {
    id: String,
    file_path: String,
    start_line: usize,
    end_line: usize,
    replacement: String,
    precondition: Option<Value>,
}

#[derive(Clone)]
struct BatchState {
    operation: BatchOperation,
    status: String,
    reason_code: Option<String>,
    message: Option<String>,
    file_hash_before: Option<String>,
    file_hash_after: Option<String>,
}

#[derive(Clone)]
struct FileSnapshot {
    original_content: String,
    lines: Vec<String>,
    file_hash_before: String,
}

pub fn batch_write_code(arguments: &Value) -> FlashgrepResult<Value> {
    let mode = match arguments.get("mode").and_then(Value::as_str) {
        None | Some("atomic") => BatchMode::Atomic,
        Some("best_effort") => BatchMode::BestEffort,
        Some(other) => {
            return Err(FlashgrepError::Config(format!(
                "Invalid mode '{}'. Expected 'atomic' or 'best_effort'",
                other
            )))
        }
    };
    let dry_run = arguments
        .get("dry_run")
        .and_then(Value::as_bool)
        .unwrap_or(false);

    let operations = parse_batch_operations(arguments)?;
    let mut states = operations
        .into_iter()
        .map(|operation| BatchState {
            operation,
            status: "pending".to_string(),
            reason_code: None,
            message: None,
            file_hash_before: None,
            file_hash_after: None,
        })
        .collect::<Vec<_>>();

    let mut file_snapshots: HashMap<String, FileSnapshot> = HashMap::new();
    let mut seen_ids = HashSet::new();
    let mut seen_targets = HashSet::new();
    let mut seen_ranges: HashMap<String, Vec<(usize, usize)>> = HashMap::new();

    for state in &mut states {
        let op = &state.operation;

        if !seen_ids.insert(op.id.clone()) {
            state.status = "failed".to_string();
            state.reason_code = Some(REASON_BATCH_DUPLICATE_OPERATION_ID.to_string());
            state.message = Some(format!("Duplicate operation id '{}'", op.id));
            continue;
        }

        let target_key = format!("{}:{}:{}", op.file_path, op.start_line, op.end_line);
        if !seen_targets.insert(target_key) {
            state.status = "failed".to_string();
            state.reason_code = Some(REASON_BATCH_DUPLICATE_TARGET.to_string());
            state.message = Some("Duplicate batch target".to_string());
            continue;
        }

        if op.replacement.len() > MAX_MCP_WRITE_REPLACEMENT_BYTES {
            state.status = "failed".to_string();
            state.reason_code = Some(REASON_PAYLOAD_TOO_LARGE.to_string());
            state.message = Some(format!(
                "Replacement exceeds {} bytes",
                MAX_MCP_WRITE_REPLACEMENT_BYTES
            ));
            continue;
        }

        if op.start_line == 0 || op.end_line == 0 || op.start_line > op.end_line {
            state.status = "failed".to_string();
            state.reason_code = Some(REASON_INVALID_RANGE.to_string());
            state.message = Some("Invalid line range".to_string());
            continue;
        }

        let ranges = seen_ranges.entry(op.file_path.clone()).or_default();
        if ranges
            .iter()
            .any(|(s, e)| op.start_line <= *e && *s <= op.end_line)
        {
            state.status = "failed".to_string();
            state.reason_code = Some(REASON_BATCH_OVERLAPPING_OPERATIONS.to_string());
            state.message = Some("Overlapping operation range in same file".to_string());
            continue;
        }
        ranges.push((op.start_line, op.end_line));

        if !file_snapshots.contains_key(&op.file_path) {
            let path = PathBuf::from(&op.file_path);
            let original_content = match std::fs::read_to_string(&path) {
                Ok(v) => v,
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                    state.status = "failed".to_string();
                    state.reason_code = Some(REASON_FILE_NOT_FOUND.to_string());
                    state.message = Some("Target file not found".to_string());
                    continue;
                }
                Err(e) => return Err(FlashgrepError::Io(e)),
            };

            let lines: Vec<String> = original_content.lines().map(ToString::to_string).collect();
            let snapshot = FileSnapshot {
                file_hash_before: calculate_sha256(&original_content),
                original_content,
                lines,
            };
            file_snapshots.insert(op.file_path.clone(), snapshot);
        }

        if let Some(snapshot) = file_snapshots.get(&op.file_path) {
            state.file_hash_before = Some(snapshot.file_hash_before.clone());
            if snapshot.lines.is_empty() || op.end_line > snapshot.lines.len() {
                state.status = "failed".to_string();
                state.reason_code = Some(REASON_INVALID_RANGE.to_string());
                state.message = Some(format!(
                    "end_line {} exceeds line count {}",
                    op.end_line,
                    snapshot.lines.len()
                ));
                continue;
            }

            if let Some(conflict_payload) = check_preconditions(
                op.precondition.as_ref(),
                &snapshot.lines,
                &snapshot.file_hash_before,
                op.start_line,
                op.end_line,
            ) {
                state.status = "conflict".to_string();
                state.reason_code = Some(REASON_PRECONDITION_FAILED.to_string());
                state.message = Some(conflict_payload.to_string());
                continue;
            }
        }
    }

    let has_blocking = states
        .iter()
        .any(|s| s.status == "failed" || s.status == "conflict");
    if mode == BatchMode::Atomic && has_blocking {
        for state in &mut states {
            if state.status == "pending" {
                state.status = "skipped".to_string();
                state.reason_code = Some("atomic_aborted_preflight".to_string());
                state.message = Some("Skipped due to atomic preflight failure".to_string());
            }
        }
        return Ok(batch_result_payload(mode, states));
    }

    if dry_run {
        for state in &mut states {
            if state.status == "pending" {
                state.status = "skipped".to_string();
                state.reason_code = Some("dry_run".to_string());
                state.message = Some("Validated in dry_run mode".to_string());
            }
        }
        return Ok(batch_result_payload(mode, states));
    }

    let mut touched_files = HashSet::new();
    for idx in 0..states.len() {
        if states[idx].status != "pending" {
            continue;
        }

        let op = states[idx].operation.clone();
        let args = json!({
            "file_path": op.file_path,
            "start_line": op.start_line,
            "end_line": op.end_line,
            "replacement": op.replacement,
            "precondition": op.precondition,
        });

        match write_code(&args) {
            Ok(payload) => {
                if payload.get("ok").and_then(Value::as_bool).unwrap_or(false) {
                    touched_files.insert(states[idx].operation.file_path.clone());
                    states[idx].status = "applied".to_string();
                    states[idx].file_hash_before = payload
                        .get("file_hash_before")
                        .and_then(Value::as_str)
                        .map(ToString::to_string)
                        .or_else(|| states[idx].file_hash_before.clone());
                    states[idx].file_hash_after = payload
                        .get("file_hash_after")
                        .and_then(Value::as_str)
                        .map(ToString::to_string);
                } else {
                    let reason = payload
                        .get("error")
                        .and_then(Value::as_str)
                        .unwrap_or(REASON_INTERNAL_ERROR);
                    states[idx].status = if reason == REASON_PRECONDITION_FAILED {
                        "conflict".to_string()
                    } else {
                        "failed".to_string()
                    };
                    states[idx].reason_code = Some(reason.to_string());
                    states[idx].message = payload
                        .get("message")
                        .and_then(Value::as_str)
                        .map(ToString::to_string)
                        .or_else(|| payload.get("conflict").map(ToString::to_string));

                    if mode == BatchMode::Atomic {
                        rollback_atomic_files(&touched_files, &file_snapshots)?;
                        for state in &mut states {
                            if state.status == "applied" {
                                state.status = "skipped".to_string();
                                state.reason_code = Some("atomic_rolled_back".to_string());
                                state.message =
                                    Some("Rolled back due to atomic failure".to_string());
                                state.file_hash_after = None;
                            } else if state.status == "pending" {
                                state.status = "skipped".to_string();
                                state.reason_code = Some("atomic_aborted".to_string());
                                state.message =
                                    Some("Skipped due to earlier atomic failure".to_string());
                            }
                        }
                        return Ok(batch_result_payload(mode, states));
                    }
                }
            }
            Err(e) => {
                states[idx].status = "failed".to_string();
                states[idx].reason_code = Some(match &e {
                    FlashgrepError::Io(ioe) if ioe.kind() == std::io::ErrorKind::NotFound => {
                        REASON_FILE_NOT_FOUND.to_string()
                    }
                    _ => REASON_INTERNAL_ERROR.to_string(),
                });
                states[idx].message = Some(e.to_string());

                if mode == BatchMode::Atomic {
                    rollback_atomic_files(&touched_files, &file_snapshots)?;
                    for state in &mut states {
                        if state.status == "applied" {
                            state.status = "skipped".to_string();
                            state.reason_code = Some("atomic_rolled_back".to_string());
                            state.message = Some("Rolled back due to atomic failure".to_string());
                            state.file_hash_after = None;
                        } else if state.status == "pending" {
                            state.status = "skipped".to_string();
                            state.reason_code = Some("atomic_aborted".to_string());
                            state.message =
                                Some("Skipped due to earlier atomic failure".to_string());
                        }
                    }
                    return Ok(batch_result_payload(mode, states));
                }
            }
        }
    }

    Ok(batch_result_payload(mode, states))
}

fn parse_batch_operations(arguments: &Value) -> FlashgrepResult<Vec<BatchOperation>> {
    let operations = arguments
        .get("operations")
        .and_then(Value::as_array)
        .ok_or_else(|| {
            FlashgrepError::Config("Missing required parameter: operations".to_string())
        })?;

    if operations.is_empty() {
        return Err(FlashgrepError::Config(
            "operations must contain at least one item".to_string(),
        ));
    }

    operations
        .iter()
        .map(|op| {
            let id = op
                .get("id")
                .and_then(Value::as_str)
                .ok_or_else(|| FlashgrepError::Config("Batch operation missing id".to_string()))?
                .to_string();
            let file_path = op
                .get("file_path")
                .and_then(Value::as_str)
                .ok_or_else(|| {
                    FlashgrepError::Config(format!("Operation '{}' missing file_path", id))
                })?
                .to_string();
            let start_line = op
                .get("start_line")
                .and_then(Value::as_u64)
                .ok_or_else(|| {
                    FlashgrepError::Config(format!("Operation '{}' missing start_line", id))
                })? as usize;
            let end_line = op.get("end_line").and_then(Value::as_u64).ok_or_else(|| {
                FlashgrepError::Config(format!("Operation '{}' missing end_line", id))
            })? as usize;
            let replacement = op
                .get("replacement")
                .and_then(Value::as_str)
                .ok_or_else(|| {
                    FlashgrepError::Config(format!("Operation '{}' missing replacement", id))
                })?
                .to_string();

            Ok(BatchOperation {
                id,
                file_path,
                start_line,
                end_line,
                replacement,
                precondition: op.get("precondition").cloned(),
            })
        })
        .collect()
}

fn rollback_atomic_files(
    touched_files: &HashSet<String>,
    snapshots: &HashMap<String, FileSnapshot>,
) -> FlashgrepResult<()> {
    for path in touched_files {
        if let Some(snapshot) = snapshots.get(path) {
            std::fs::write(path, &snapshot.original_content)?;
        }
    }
    Ok(())
}

fn batch_result_payload(mode: BatchMode, states: Vec<BatchState>) -> Value {
    let mut applied_count = 0usize;
    let mut failed_count = 0usize;
    let mut conflict_count = 0usize;
    let mut skipped_count = 0usize;

    let results = states
        .into_iter()
        .map(|state| {
            match state.status.as_str() {
                "applied" => applied_count += 1,
                "failed" => failed_count += 1,
                "conflict" => conflict_count += 1,
                _ => skipped_count += 1,
            }
            json!({
                "id": state.operation.id,
                "status": state.status,
                "reason_code": state.reason_code,
                "message": state.message,
                "file_path": state.operation.file_path,
                "start_line": state.operation.start_line,
                "end_line": state.operation.end_line,
                "file_hash_before": state.file_hash_before,
                "file_hash_after": state.file_hash_after,
            })
        })
        .collect::<Vec<_>>();

    let ok = failed_count == 0 && conflict_count == 0;
    json!({
        "ok": ok,
        "mode": match mode {
            BatchMode::Atomic => "atomic",
            BatchMode::BestEffort => "best_effort",
        },
        "results": results,
        "applied_count": applied_count,
        "failed_count": failed_count,
        "conflict_count": conflict_count,
        "skipped_count": skipped_count,
    })
}

#[derive(Debug, Serialize, Deserialize)]
struct WriteSession {
    continuation_id: String,
    file_path: String,
    start_line: usize,
    end_line: usize,
    file_hash_before: String,
    had_trailing_newline: bool,
    replacement_accumulated: String,
    next_chunk_index: usize,
}

fn write_code_chunked(
    arguments: &Value,
    continuation_id: &str,
    file_path: &str,
    start_line: usize,
    end_line: usize,
    replacement_chunk: &str,
) -> FlashgrepResult<Value> {
    let chunk_index = arguments
        .get("chunk_index")
        .and_then(Value::as_u64)
        .unwrap_or(0) as usize;
    let is_final_chunk = arguments
        .get("is_final_chunk")
        .and_then(Value::as_bool)
        .unwrap_or(false);

    let replacement_size = replacement_chunk.len();
    if replacement_size > MAX_MCP_WRITE_REPLACEMENT_BYTES {
        let mut payload = payload_too_large_error(
            "write_code",
            replacement_size,
            MAX_MCP_WRITE_REPLACEMENT_BYTES,
            &chunking_guidance(MAX_MCP_WRITE_REPLACEMENT_BYTES),
        );
        payload["ok"] = Value::Bool(false);
        payload["file_path"] = Value::String(file_path.to_string());
        return Ok(payload);
    }

    let session_path = write_session_path(continuation_id);
    let mut session = if chunk_index == 0 {
        let path = PathBuf::from(file_path);
        let original_content = std::fs::read_to_string(&path)?;
        let original_hash = calculate_sha256(&original_content);
        let had_trailing_newline = original_content.ends_with('\n');
        let original_lines: Vec<String> =
            original_content.lines().map(ToString::to_string).collect();
        if original_lines.is_empty() {
            return Err(FlashgrepError::Config(
                "Cannot apply line-range write to empty file".to_string(),
            ));
        }
        if end_line > original_lines.len() {
            return Err(FlashgrepError::Config(format!(
                "Invalid range: end_line {} exceeds file line count {}",
                end_line,
                original_lines.len()
            )));
        }

        let conflict = check_preconditions(
            arguments.get("precondition"),
            &original_lines,
            &original_hash,
            start_line,
            end_line,
        );
        if let Some(conflict_payload) = conflict {
            return Ok(json!({
                "ok": false,
                "error": "precondition_failed",
                "file_path": file_path,
                "conflict": conflict_payload
            }));
        }

        WriteSession {
            continuation_id: continuation_id.to_string(),
            file_path: file_path.to_string(),
            start_line,
            end_line,
            file_hash_before: original_hash,
            had_trailing_newline,
            replacement_accumulated: String::new(),
            next_chunk_index: 0,
        }
    } else {
        let loaded = load_write_session(&session_path)?;
        if loaded.file_path != file_path
            || loaded.start_line != start_line
            || loaded.end_line != end_line
            || loaded.next_chunk_index != chunk_index
        {
            return Ok(json!({
                "ok": false,
                "error": "invalid_continuation_state",
                "expected": {
                    "file_path": loaded.file_path,
                    "start_line": loaded.start_line,
                    "end_line": loaded.end_line,
                    "next_chunk_index": loaded.next_chunk_index
                },
                "received": {
                    "file_path": file_path,
                    "start_line": start_line,
                    "end_line": end_line,
                    "chunk_index": chunk_index
                }
            }));
        }
        loaded
    };

    session.replacement_accumulated.push_str(replacement_chunk);
    session.next_chunk_index = chunk_index.saturating_add(1);

    if !is_final_chunk {
        save_write_session(&session_path, &session)?;
        return Ok(json!({
            "ok": true,
            "continuation": continuation_meta(
                json!({"continuation_id": continuation_id, "next_chunk_index": session.next_chunk_index}),
                chunk_index,
                false
            ),
            "received_bytes": replacement_size,
            "file_path": file_path
        }));
    }

    let path = PathBuf::from(file_path);
    let original_content = std::fs::read_to_string(&path)?;
    let original_lines: Vec<String> = original_content.lines().map(ToString::to_string).collect();

    let replacement_lines: Vec<String> = if session.replacement_accumulated.is_empty() {
        Vec::new()
    } else {
        session
            .replacement_accumulated
            .split('\n')
            .map(ToString::to_string)
            .collect()
    };

    let mut new_lines = Vec::new();
    new_lines.extend_from_slice(&original_lines[..start_line - 1]);
    new_lines.extend(replacement_lines.iter().cloned());
    new_lines.extend_from_slice(&original_lines[end_line..]);

    let mut new_content = new_lines.join("\n");
    if session.had_trailing_newline {
        new_content.push('\n');
    }

    std::fs::write(&path, &new_content)?;
    let new_hash = calculate_sha256(&new_content);

    let _ = std::fs::remove_file(&session_path);

    Ok(json!({
        "ok": true,
        "file_path": file_path,
        "start_line": start_line,
        "end_line": end_line,
        "replaced_line_count": end_line - start_line + 1,
        "new_line_count": replacement_lines.len(),
        "file_hash_before": session.file_hash_before,
        "file_hash_after": new_hash,
        "continuation": continuation_meta(
            json!({"continuation_id": continuation_id, "next_chunk_index": session.next_chunk_index}),
            chunk_index,
            true
        )
    }))
}

fn write_session_path(continuation_id: &str) -> PathBuf {
    let safe_id: String = continuation_id
        .chars()
        .filter(|c| c.is_ascii_alphanumeric() || *c == '-' || *c == '_')
        .collect();
    std::env::temp_dir()
        .join("flashgrep-write-sessions")
        .join(format!("{}.json", safe_id))
}

fn save_write_session(path: &Path, session: &WriteSession) -> FlashgrepResult<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let data = serde_json::to_vec(session)?;
    std::fs::write(path, data)?;
    Ok(())
}

fn load_write_session(path: &Path) -> FlashgrepResult<WriteSession> {
    let data = std::fs::read(path).map_err(|_| {
        FlashgrepError::Config(
            "Missing write continuation session; restart with chunk_index=0".to_string(),
        )
    })?;
    let session: WriteSession = serde_json::from_slice(&data)?;
    Ok(session)
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum MetadataLevel {
    Minimal,
    Standard,
}

enum ReadMode<'a> {
    FileSlice { file_path: &'a str },
    Symbol { symbol_name: &'a str },
}

struct ReadTarget {
    file_path: String,
    lines: Vec<(usize, String)>,
    mode_name: &'static str,
    symbol_name: Option<String>,
}

struct Limits {
    max_lines: Option<usize>,
    max_bytes: Option<usize>,
    max_tokens: Option<usize>,
}

struct BoundedContent {
    included_lines: Vec<(usize, String)>,
    first_line: usize,
    last_line: usize,
    consumed_lines: usize,
    consumed_bytes: usize,
    consumed_tokens: usize,
    truncated: bool,
    next_start_line: Option<usize>,
}

fn parse_metadata_level(arguments: &Value) -> FlashgrepResult<MetadataLevel> {
    match arguments.get("metadata_level").and_then(Value::as_str) {
        None | Some("standard") => Ok(MetadataLevel::Standard),
        Some("minimal") => Ok(MetadataLevel::Minimal),
        Some(other) => Err(FlashgrepError::Config(format!(
            "Invalid metadata_level '{}'. Expected 'minimal' or 'standard'",
            other
        ))),
    }
}

fn parse_read_mode(arguments: &Value) -> FlashgrepResult<ReadMode<'_>> {
    let file_path = arguments.get("file_path").and_then(Value::as_str);
    let symbol_name = arguments.get("symbol_name").and_then(Value::as_str);

    match (file_path, symbol_name) {
        (Some(_), Some(_)) => Err(FlashgrepError::Config(
            "Provide either file_path or symbol_name, not both".to_string(),
        )),
        (None, None) => Err(FlashgrepError::Config(
            "Missing read target: provide file_path (slice mode) or symbol_name (symbol mode)"
                .to_string(),
        )),
        (Some(path), None) if path.trim().is_empty() => Err(FlashgrepError::Config(
            "file_path cannot be empty".to_string(),
        )),
        (None, Some(name)) if name.trim().is_empty() => Err(FlashgrepError::Config(
            "symbol_name cannot be empty".to_string(),
        )),
        (Some(path), None) => Ok(ReadMode::FileSlice { file_path: path }),
        (None, Some(name)) => Ok(ReadMode::Symbol { symbol_name: name }),
    }
}

fn parse_limits(arguments: &Value) -> FlashgrepResult<Limits> {
    let max_lines = get_optional_usize(arguments, "max_lines")?;
    let max_bytes = get_optional_usize(arguments, "max_bytes")?;
    let max_tokens = get_optional_usize(arguments, "max_tokens")?;

    if max_lines == Some(0) || max_bytes == Some(0) || max_tokens == Some(0) {
        return Err(FlashgrepError::Config(
            "Budget limits must be positive integers".to_string(),
        ));
    }

    if let Some(requested) = max_bytes {
        if requested > MAX_MCP_READ_BYTES {
            return Err(FlashgrepError::Config(format!(
                "max_bytes {} exceeds server safety limit {}",
                requested, MAX_MCP_READ_BYTES
            )));
        }
    }

    Ok(Limits {
        max_lines,
        max_bytes: Some(max_bytes.unwrap_or(MAX_MCP_READ_BYTES)),
        max_tokens,
    })
}

fn read_file_slice(
    file_path: &str,
    start_line: usize,
    requested_end_line: Option<usize>,
    symbol_name: Option<String>,
) -> FlashgrepResult<ReadTarget> {
    if start_line == 0 {
        return Err(FlashgrepError::Config(
            "start_line must be greater than 0".to_string(),
        ));
    }

    let content = std::fs::read_to_string(file_path)?;
    let all_lines: Vec<&str> = content.lines().collect();

    if all_lines.is_empty() {
        return Ok(ReadTarget {
            file_path: file_path.to_string(),
            lines: vec![],
            mode_name: if symbol_name.is_some() {
                "symbol"
            } else {
                "slice"
            },
            symbol_name,
        });
    }

    if start_line > all_lines.len() {
        return Err(FlashgrepError::Config(format!(
            "start_line {} exceeds file line count {}",
            start_line,
            all_lines.len()
        )));
    }

    let end_line = requested_end_line
        .unwrap_or(all_lines.len())
        .min(all_lines.len());
    if end_line < start_line {
        return Err(FlashgrepError::Config(format!(
            "Invalid range: end_line {} is less than start_line {}",
            end_line, start_line
        )));
    }

    let lines = all_lines[start_line - 1..end_line]
        .iter()
        .enumerate()
        .map(|(idx, line)| (start_line + idx, (*line).to_string()))
        .collect::<Vec<_>>();

    Ok(ReadTarget {
        file_path: file_path.to_string(),
        lines,
        mode_name: if symbol_name.is_some() {
            "symbol"
        } else {
            "slice"
        },
        symbol_name,
    })
}

fn read_symbol_slice(
    paths: &FlashgrepPaths,
    symbol_name: &str,
    context_lines: usize,
) -> FlashgrepResult<ReadTarget> {
    let db = Database::open(&paths.metadata_db())?;
    let symbols = db.find_symbols_by_name(symbol_name)?;
    let symbol = symbols
        .first()
        .ok_or_else(|| FlashgrepError::Config(format!("Symbol not found: {}", symbol_name)))?;

    let file_path = symbol.file_path.to_string_lossy().to_string();
    let start_line = symbol.line_number.saturating_sub(context_lines).max(1);
    let end_line = symbol.line_number.saturating_add(context_lines);

    read_file_slice(
        &file_path,
        start_line,
        Some(end_line),
        Some(symbol_name.to_string()),
    )
}

fn apply_budgets(lines: &[(usize, String)], limits: &Limits) -> Option<BoundedContent> {
    if lines.is_empty() {
        return Some(BoundedContent {
            included_lines: Vec::new(),
            first_line: 1,
            last_line: 0,
            consumed_lines: 0,
            consumed_bytes: 0,
            consumed_tokens: 0,
            truncated: false,
            next_start_line: None,
        });
    }

    let mut included = Vec::new();
    let mut consumed_bytes = 0usize;
    let mut consumed_tokens = 0usize;

    for (line_no, line) in lines {
        let line_bytes = line.len();
        let line_tokens = estimate_tokens(line);
        let sep_bytes = if included.is_empty() { 0 } else { 1 };
        let next_lines = included.len() + 1;
        let next_bytes = consumed_bytes + line_bytes + sep_bytes;
        let next_tokens = consumed_tokens + line_tokens;

        let lines_ok = limits.max_lines.map(|l| next_lines <= l).unwrap_or(true);
        let bytes_ok = limits.max_bytes.map(|b| next_bytes <= b).unwrap_or(true);
        let tokens_ok = limits.max_tokens.map(|t| next_tokens <= t).unwrap_or(true);

        if lines_ok && bytes_ok && tokens_ok {
            included.push((*line_no, line.clone()));
            consumed_bytes = next_bytes;
            consumed_tokens = next_tokens;
        } else {
            break;
        }
    }

    if included.is_empty() {
        return None;
    }

    let consumed_lines = included.len();
    let first_line = included.first().map(|(n, _)| *n).unwrap_or(1);
    let last_line = included.last().map(|(n, _)| *n).unwrap_or(0);
    let truncated = consumed_lines < lines.len();
    let next_start_line = if truncated {
        lines.get(consumed_lines).map(|(n, _)| *n)
    } else {
        None
    };

    Some(BoundedContent {
        included_lines: included,
        first_line,
        last_line,
        consumed_lines,
        consumed_bytes,
        consumed_tokens,
        truncated,
        next_start_line,
    })
}

fn estimate_tokens(line: &str) -> usize {
    line.split_whitespace().count()
}

fn get_optional_usize(arguments: &Value, key: &str) -> FlashgrepResult<Option<usize>> {
    match arguments.get(key) {
        None => Ok(None),
        Some(value) => {
            let num = value
                .as_u64()
                .ok_or_else(|| FlashgrepError::Config(format!("{} must be an integer", key)))?;
            Ok(Some(num as usize))
        }
    }
}

fn get_required_usize(arguments: &Value, key: &str) -> FlashgrepResult<usize> {
    let num = arguments
        .get(key)
        .and_then(Value::as_u64)
        .ok_or_else(|| FlashgrepError::Config(format!("Missing required parameter: {}", key)))?;
    Ok(num as usize)
}

fn calculate_sha256(content: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(content.as_bytes());
    hex::encode(hasher.finalize())
}

fn check_preconditions(
    precondition: Option<&Value>,
    lines: &[String],
    current_hash: &str,
    start_line: usize,
    end_line: usize,
) -> Option<Value> {
    let precondition = precondition?;
    let mut mismatches = Vec::new();

    if let Some(expected_hash) = precondition
        .get("expected_file_hash")
        .and_then(Value::as_str)
    {
        if expected_hash != current_hash {
            mismatches.push(json!({
                "field": "expected_file_hash",
                "expected": expected_hash,
                "actual": current_hash,
            }));
        }
    }

    if let Some(expected_start) = precondition
        .get("expected_start_line_text")
        .and_then(Value::as_str)
    {
        let actual_start = lines.get(start_line - 1).cloned().unwrap_or_default();
        if expected_start != actual_start {
            mismatches.push(json!({
                "field": "expected_start_line_text",
                "line": start_line,
                "expected": expected_start,
                "actual": actual_start,
            }));
        }
    }

    if let Some(expected_end) = precondition
        .get("expected_end_line_text")
        .and_then(Value::as_str)
    {
        let actual_end = lines.get(end_line - 1).cloned().unwrap_or_default();
        if expected_end != actual_end {
            mismatches.push(json!({
                "field": "expected_end_line_text",
                "line": end_line,
                "expected": expected_end,
                "actual": actual_end,
            }));
        }
    }

    if mismatches.is_empty() {
        None
    } else {
        Some(json!({ "mismatches": mismatches }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::fs;
    use tempfile::TempDir;

    fn setup_file(content: &str) -> (TempDir, PathBuf) {
        let temp = TempDir::new().expect("temp dir");
        let file_path = temp.path().join("sample.rs");
        fs::write(&file_path, content).expect("write sample file");
        (temp, file_path)
    }

    #[test]
    fn read_code_respects_max_lines_and_continuation() {
        let (temp, file_path) = setup_file("a\nb\nc\nd\n");
        let repo_root = temp.path().to_path_buf();
        let paths = FlashgrepPaths::new(&repo_root);

        let first = read_code(
            &paths,
            &json!({
                "file_path": file_path.to_string_lossy(),
                "max_lines": 2,
                "metadata_level": "minimal"
            }),
        )
        .expect("first read");

        assert_eq!(first["content"], Value::String("a\nb".to_string()));
        assert_eq!(first["continuation_start_line"], Value::Number(3u64.into()));

        let second = read_code(
            &paths,
            &json!({
                "file_path": file_path.to_string_lossy(),
                "continuation_start_line": 3,
                "max_lines": 2,
                "metadata_level": "minimal"
            }),
        )
        .expect("second read");

        assert_eq!(second["content"], Value::String("c\nd".to_string()));
        assert!(second["continuation_start_line"].is_null());
    }

    #[test]
    fn read_code_rejects_ambiguous_mode() {
        let temp = TempDir::new().expect("temp dir");
        let repo_root = temp.path().to_path_buf();
        let paths = FlashgrepPaths::new(&repo_root);
        let result = read_code(
            &paths,
            &json!({
                "file_path": "src/lib.rs",
                "symbol_name": "main"
            }),
        );

        assert!(result.is_err());
    }

    #[test]
    fn write_code_applies_minimal_diff_range() {
        let (_temp, file_path) = setup_file("line1\nline2\nline3\n");
        let args = json!({
            "file_path": file_path.to_string_lossy(),
            "start_line": 2,
            "end_line": 2,
            "replacement": "updated"
        });

        let result = write_code(&args).expect("write result");
        assert_eq!(result["ok"], Value::Bool(true));

        let updated = fs::read_to_string(file_path).expect("read updated file");
        assert_eq!(updated, "line1\nupdated\nline3\n");
    }

    #[test]
    fn write_code_reports_precondition_conflict() {
        let (_temp, file_path) = setup_file("line1\nline2\nline3\n");
        let args = json!({
            "file_path": file_path.to_string_lossy(),
            "start_line": 2,
            "end_line": 2,
            "replacement": "updated",
            "precondition": {
                "expected_start_line_text": "different"
            }
        });

        let result = write_code(&args).expect("write result");
        assert_eq!(result["ok"], Value::Bool(false));
        assert_eq!(
            result["error"],
            Value::String("precondition_failed".to_string())
        );
    }

    #[test]
    fn read_code_budget_response_is_smaller_than_full_response() {
        let (temp, file_path) = setup_file("alpha\nbeta\ngamma\ndelta\n");
        let repo_root = temp.path().to_path_buf();
        let paths = FlashgrepPaths::new(&repo_root);

        let full = read_code(
            &paths,
            &json!({
                "file_path": file_path.to_string_lossy(),
                "metadata_level": "standard"
            }),
        )
        .expect("full read");

        let budgeted = read_code(
            &paths,
            &json!({
                "file_path": file_path.to_string_lossy(),
                "max_lines": 1,
                "metadata_level": "minimal"
            }),
        )
        .expect("budgeted read");

        let full_bytes = serde_json::to_vec(&full).expect("encode full").len();
        let budgeted_bytes = serde_json::to_vec(&budgeted)
            .expect("encode budgeted")
            .len();
        assert!(budgeted_bytes < full_bytes);
    }

    #[test]
    fn write_code_rejects_oversized_replacement() {
        let (_temp, file_path) = setup_file("line1\nline2\n");
        let giant = "x".repeat(MAX_MCP_WRITE_REPLACEMENT_BYTES + 1);
        let result = write_code(&json!({
            "file_path": file_path.to_string_lossy(),
            "start_line": 1,
            "end_line": 1,
            "replacement": giant
        }))
        .expect("write payload");

        assert_eq!(result["ok"], Value::Bool(false));
        assert_eq!(
            result["error"],
            Value::String("payload_too_large".to_string())
        );
    }

    #[test]
    fn read_code_rejects_max_bytes_over_server_limit() {
        let (temp, file_path) = setup_file("alpha\nbeta\n");
        let repo_root = temp.path().to_path_buf();
        let paths = FlashgrepPaths::new(&repo_root);
        let result = read_code(
            &paths,
            &json!({
                "file_path": file_path.to_string_lossy(),
                "max_bytes": (MAX_MCP_READ_BYTES + 1)
            }),
        );
        assert!(result.is_err());
    }

    #[test]
    fn read_code_continuation_reconstructs_full_content() {
        let (temp, file_path) = setup_file("l1\nl2\nl3\nl4\nl5\n");
        let repo_root = temp.path().to_path_buf();
        let paths = FlashgrepPaths::new(&repo_root);

        let mut collected = String::new();
        let mut next_line: Option<u64> = None;
        let mut chunk_index = 0u64;

        loop {
            let mut args = json!({
                "file_path": file_path.to_string_lossy(),
                "max_lines": 2,
                "chunk_index": chunk_index,
                "metadata_level": "minimal"
            });
            if let Some(n) = next_line {
                args["continuation_start_line"] = Value::Number(n.into());
            }

            let chunk = read_code(&paths, &args).expect("chunk read");
            if !collected.is_empty() && !chunk["content"].as_str().unwrap_or("").is_empty() {
                collected.push('\n');
            }
            collected.push_str(chunk["content"].as_str().unwrap_or(""));

            next_line = chunk["continuation_start_line"].as_u64();
            if next_line.is_none() {
                break;
            }
            chunk_index += 1;
        }

        assert_eq!(collected, "l1\nl2\nl3\nl4\nl5");
    }

    #[test]
    fn write_code_chunked_sequence_applies_exact_result() {
        let (_temp, file_path) = setup_file("a\nb\nc\n");
        let continuation_id = "test-chunked-write";

        let step1 = write_code(&json!({
            "file_path": file_path.to_string_lossy(),
            "start_line": 2,
            "end_line": 2,
            "replacement": "hello ",
            "continuation_id": continuation_id,
            "chunk_index": 0,
            "is_final_chunk": false
        }))
        .expect("step1");
        assert_eq!(step1["ok"], Value::Bool(true));
        assert_eq!(step1["continuation"]["completed"], Value::Bool(false));

        let step2 = write_code(&json!({
            "file_path": file_path.to_string_lossy(),
            "start_line": 2,
            "end_line": 2,
            "replacement": "world",
            "continuation_id": continuation_id,
            "chunk_index": 1,
            "is_final_chunk": true
        }))
        .expect("step2");
        assert_eq!(step2["ok"], Value::Bool(true));
        assert_eq!(step2["continuation"]["completed"], Value::Bool(true));

        let updated = fs::read_to_string(file_path).expect("updated");
        assert_eq!(updated, "a\nhello world\nc\n");
    }

    #[test]
    fn batch_write_code_atomic_rejects_overlap() {
        let (_temp, file_path) = setup_file("a\nb\nc\n");
        let result = batch_write_code(&json!({
            "mode": "atomic",
            "operations": [
                {
                    "id": "op1",
                    "file_path": file_path.to_string_lossy(),
                    "start_line": 1,
                    "end_line": 2,
                    "replacement": "x\ny"
                },
                {
                    "id": "op2",
                    "file_path": file_path.to_string_lossy(),
                    "start_line": 2,
                    "end_line": 3,
                    "replacement": "z\nw"
                }
            ]
        }))
        .expect("batch result");

        assert_eq!(result["ok"], Value::Bool(false));
        assert_eq!(result["failed_count"], Value::Number(1u64.into()));
        assert_eq!(result["skipped_count"], Value::Number(1u64.into()));
        let content = fs::read_to_string(file_path).expect("file read");
        assert_eq!(content, "a\nb\nc\n");
    }

    #[test]
    fn batch_write_code_best_effort_applies_valid_ops() {
        let (_temp, file_path) = setup_file("a\nb\nc\n");
        let result = batch_write_code(&json!({
            "mode": "best_effort",
            "operations": [
                {
                    "id": "op1",
                    "file_path": file_path.to_string_lossy(),
                    "start_line": 1,
                    "end_line": 1,
                    "replacement": "alpha"
                },
                {
                    "id": "op2",
                    "file_path": file_path.to_string_lossy(),
                    "start_line": 10,
                    "end_line": 10,
                    "replacement": "omega"
                }
            ]
        }))
        .expect("batch result");

        assert_eq!(result["applied_count"], Value::Number(1u64.into()));
        assert_eq!(result["failed_count"], Value::Number(1u64.into()));
        let content = fs::read_to_string(file_path).expect("file read");
        assert_eq!(content, "alpha\nb\nc\n");
    }

    #[test]
    fn batch_write_code_atomic_rolls_back_on_runtime_failure() {
        let (_temp, file_path) = setup_file("a\nb\nc\n");
        let result = batch_write_code(&json!({
            "mode": "atomic",
            "operations": [
                {
                    "id": "op1",
                    "file_path": file_path.to_string_lossy(),
                    "start_line": 1,
                    "end_line": 1,
                    "replacement": "alpha"
                },
                {
                    "id": "op2",
                    "file_path": file_path.to_string_lossy(),
                    "start_line": 2,
                    "end_line": 2,
                    "replacement": "beta",
                    "precondition": {
                        "expected_start_line_text": "does-not-match"
                    }
                }
            ]
        }))
        .expect("batch result");

        assert_eq!(result["ok"], Value::Bool(false));
        let content = fs::read_to_string(file_path).expect("file read");
        assert_eq!(content, "a\nb\nc\n");
    }

    #[test]
    fn batch_write_code_dry_run_does_not_mutate_files() {
        let (_temp, file_path) = setup_file("a\nb\nc\n");
        let result = batch_write_code(&json!({
            "mode": "best_effort",
            "dry_run": true,
            "operations": [
                {
                    "id": "op1",
                    "file_path": file_path.to_string_lossy(),
                    "start_line": 2,
                    "end_line": 2,
                    "replacement": "beta"
                }
            ]
        }))
        .expect("batch result");

        assert_eq!(result["ok"], Value::Bool(true));
        assert_eq!(result["skipped_count"], Value::Number(1u64.into()));
        let content = fs::read_to_string(file_path).expect("file read");
        assert_eq!(content, "a\nb\nc\n");
    }
}
