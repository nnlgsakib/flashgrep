use crate::{FlashgrepError, FlashgrepResult};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

pub const MAX_MCP_REQUEST_BYTES: usize = 512 * 1024;
pub const MAX_MCP_RESPONSE_BYTES: usize = 512 * 1024;
pub const MAX_MCP_READ_BYTES: usize = 192 * 1024;
pub const MAX_MCP_GET_SLICE_BYTES: usize = 192 * 1024;
pub const MAX_MCP_WRITE_REPLACEMENT_BYTES: usize = 128 * 1024;

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
