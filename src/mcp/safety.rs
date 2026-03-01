use crate::{FlashgrepError, FlashgrepResult};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

pub const MAX_MCP_REQUEST_BYTES: usize = 512 * 1024;
pub const MAX_MCP_RESPONSE_BYTES: usize = 512 * 1024;
pub const MAX_MCP_READ_BYTES: usize = 192 * 1024;
pub const MAX_MCP_GET_SLICE_BYTES: usize = 192 * 1024;
pub const MAX_MCP_WRITE_REPLACEMENT_BYTES: usize = 128 * 1024;

pub const REASON_FILE_NOT_FOUND: &str = "file_not_found";
pub const REASON_DIRECTORY_NOT_FOUND: &str = "directory_not_found";
pub const REASON_PATH_NOT_FOUND: &str = "path_not_found";

pub fn json_size_bytes(value: &Value) -> FlashgrepResult<usize> {
    Ok(serde_json::to_vec(value)?.len())
}

pub fn check_arguments_size(arguments: &Value, limit: usize) -> FlashgrepResult<()> {
    let observed = json_size_bytes(arguments)?;
    if observed > limit {
        return Err(FlashgrepError::Config(format!(
            "request_arguments_too_large: observed_bytes={}, limit_bytes={}",
            observed, limit
        )));
    }
    Ok(())
}

pub fn payload_too_large_error(
    operation: &str,
    observed_bytes: usize,
    limit_bytes: usize,
    guidance: &str,
) -> Value {
    json!({
        "error": "payload_too_large",
        "operation": operation,
        "observed_bytes": observed_bytes,
        "limit_bytes": limit_bytes,
        "guidance": guidance,
    })
}

pub fn invalid_params_error(message: &str) -> Value {
    json!({
        "error": "invalid_params",
        "message": message,
    })
}

pub fn not_found_error(target_path: &str, target_kind: &str) -> Value {
    let reason_code = match target_kind {
        "file" => REASON_FILE_NOT_FOUND,
        "directory" => REASON_DIRECTORY_NOT_FOUND,
        _ => REASON_PATH_NOT_FOUND,
    };
    json!({
        "ok": false,
        "error": "not_found",
        "reason_code": reason_code,
        "target_path": target_path,
        "target_kind": target_kind,
    })
}

pub fn map_error_with_not_found(
    err: &FlashgrepError,
    target_path: Option<&str>,
    target_kind: Option<&str>,
) -> Value {
    match err {
        FlashgrepError::Io(ioe) if ioe.kind() == std::io::ErrorKind::NotFound => {
            not_found_error(target_path.unwrap_or(""), target_kind.unwrap_or("path"))
        }
        _ => invalid_params_error(&err.to_string()),
    }
}

pub fn chunking_guidance(max_bytes: usize) -> String {
    format!(
        "Retry with smaller chunks. Keep each request/response under {} bytes and use continuation fields for reads.",
        max_bytes
    )
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContinuationMeta {
    pub cursor: Value,
    pub chunk_index: usize,
    pub completed: bool,
}

pub fn continuation_meta(cursor: Value, chunk_index: usize, completed: bool) -> Value {
    json!(ContinuationMeta {
        cursor,
        chunk_index,
        completed,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn not_found_reason_codes_are_stable() {
        let file = not_found_error("a.txt", "file");
        let dir = not_found_error("a", "directory");
        let path = not_found_error("a", "path");
        assert_eq!(
            file["reason_code"],
            Value::String(REASON_FILE_NOT_FOUND.to_string())
        );
        assert_eq!(
            dir["reason_code"],
            Value::String(REASON_DIRECTORY_NOT_FOUND.to_string())
        );
        assert_eq!(
            path["reason_code"],
            Value::String(REASON_PATH_NOT_FOUND.to_string())
        );
    }
}
