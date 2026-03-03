use crate::mcp::safety::not_found_error;
use crate::{FlashgrepError, FlashgrepResult};
use serde_json::{json, Value};
use std::cmp::Ordering;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::UNIX_EPOCH;

pub fn fs_create(arguments: &Value) -> FlashgrepResult<Value> {
    let path = required_path(arguments, "path")?;
    let target = resolve_path(path);
    let is_dir = arguments
        .get("dir")
        .and_then(Value::as_bool)
        .unwrap_or(false);
    let parents = arguments
        .get("parents")
        .and_then(Value::as_bool)
        .unwrap_or(false);
    let overwrite = arguments
        .get("overwrite")
        .and_then(Value::as_bool)
        .unwrap_or(false);
    let dry_run = arguments
        .get("dry_run")
        .and_then(Value::as_bool)
        .unwrap_or(false);

    if target.exists() && !overwrite {
        return Ok(json!({
            "ok": false,
            "error": "conflict",
            "reason_code": "destination_exists",
            "operation": "fs_create",
            "target_path": target.to_string_lossy(),
        }));
    }

    if !dry_run {
        if is_dir {
            if parents {
                fs::create_dir_all(&target)?;
            } else if !target.exists() {
                fs::create_dir(&target)?;
            }
        } else {
            if parents {
                if let Some(parent) = target.parent() {
                    fs::create_dir_all(parent)?;
                }
            }
            if target.exists() {
                fs::write(&target, "")?;
            } else {
                fs::File::create(&target)?;
            }
        }
    }

    Ok(json!({
        "ok": true,
        "operation": "fs_create",
        "dry_run": dry_run,
        "entry": if dry_run {
            json!({
                "path": target.to_string_lossy(),
                "file_type": if is_dir {"directory"} else {"file"},
            })
        } else {
            metadata_json(&target)?
        }
    }))
}

pub fn fs_read(arguments: &Value) -> FlashgrepResult<Value> {
    let path = required_path(arguments, "path")?;
    let target = resolve_path(path);
    if !target.exists() {
        return Ok(with_operation(
            not_found_error(path, "file"),
            "fs_read",
            false,
        ));
    }
    if !target.is_file() {
        return Ok(json!({
            "ok": false,
            "error": "invalid_params",
            "reason_code": "not_a_file",
            "operation": "fs_read",
            "target_path": target.to_string_lossy(),
        }));
    }

    let content = fs::read_to_string(&target)?;
    Ok(json!({
        "ok": true,
        "operation": "fs_read",
        "entry": metadata_json(&target)?,
        "content": content,
    }))
}

pub fn fs_write(arguments: &Value) -> FlashgrepResult<Value> {
    let path = required_path(arguments, "path")?;
    let target = resolve_path(path);
    let content = arguments
        .get("content")
        .and_then(Value::as_str)
        .unwrap_or("");
    let append = arguments
        .get("append")
        .and_then(Value::as_bool)
        .unwrap_or(false);
    let parents = arguments
        .get("parents")
        .and_then(Value::as_bool)
        .unwrap_or(false);
    let overwrite = arguments
        .get("overwrite")
        .and_then(Value::as_bool)
        .unwrap_or(true);
    let dry_run = arguments
        .get("dry_run")
        .and_then(Value::as_bool)
        .unwrap_or(false);

    if target.exists() && target.is_dir() {
        return Ok(json!({
            "ok": false,
            "error": "invalid_params",
            "reason_code": "target_is_directory",
            "operation": "fs_write",
            "target_path": target.to_string_lossy(),
        }));
    }
    if target.exists() && !append && !overwrite {
        return Ok(json!({
            "ok": false,
            "error": "conflict",
            "reason_code": "destination_exists",
            "operation": "fs_write",
            "target_path": target.to_string_lossy(),
        }));
    }

    if !dry_run {
        if parents {
            if let Some(parent) = target.parent() {
                fs::create_dir_all(parent)?;
            }
        }
        if append {
            use std::io::Write;
            let mut f = fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(&target)?;
            f.write_all(content.as_bytes())?;
        } else {
            fs::write(&target, content)?;
        }
    }

    Ok(json!({
        "ok": true,
        "operation": "fs_write",
        "dry_run": dry_run,
        "bytes": content.len(),
        "entry": if dry_run {
            json!({"path": target.to_string_lossy(), "file_type": "file"})
        } else {
            metadata_json(&target)?
        }
    }))
}

pub fn fs_stat(arguments: &Value) -> FlashgrepResult<Value> {
    let path = required_path(arguments, "path")?;
    let target = resolve_path(path);
    if !target.exists() {
        let kind = if looks_like_directory(path) {
            "directory"
        } else {
            "path"
        };
        return Ok(with_operation(
            not_found_error(path, kind),
            "fs_stat",
            false,
        ));
    }
    Ok(json!({
        "ok": true,
        "operation": "fs_stat",
        "entry": metadata_json(&target)?
    }))
}

pub fn fs_list(arguments: &Value) -> FlashgrepResult<Value> {
    let path = required_path(arguments, "path")?;
    let target = resolve_path(path);
    if !target.exists() {
        return Ok(with_operation(
            not_found_error(path, "directory"),
            "fs_list",
            false,
        ));
    }
    if !target.is_dir() {
        return Ok(json!({
            "ok": false,
            "error": "invalid_params",
            "reason_code": "not_a_directory",
            "operation": "fs_list",
            "target_path": target.to_string_lossy(),
        }));
    }

    let include_hidden = arguments
        .get("include_hidden")
        .and_then(Value::as_bool)
        .unwrap_or(false);
    let sort_by = arguments
        .get("sort_by")
        .and_then(Value::as_str)
        .unwrap_or("path");
    let sort_order = arguments
        .get("sort_order")
        .and_then(Value::as_str)
        .unwrap_or("asc");
    let offset = arguments.get("offset").and_then(Value::as_u64).unwrap_or(0) as usize;
    let limit = arguments
        .get("limit")
        .and_then(Value::as_u64)
        .map(|v| v as usize);

    if sort_order != "asc" && sort_order != "desc" {
        return Ok(json!({
            "ok": false,
            "error": "invalid_params",
            "reason_code": "invalid_sort_order",
            "operation": "fs_list",
            "sort_order": sort_order,
        }));
    }
    if !matches!(sort_by, "path" | "name" | "modified" | "size") {
        return Ok(json!({
            "ok": false,
            "error": "invalid_params",
            "reason_code": "invalid_sort_by",
            "operation": "fs_list",
            "sort_by": sort_by,
        }));
    }

    let mut entries = Vec::new();
    for entry in fs::read_dir(&target)? {
        let entry = entry?;
        let name = entry.file_name().to_string_lossy().to_string();
        if !include_hidden && name.starts_with('.') {
            continue;
        }
        entries.push(metadata_json(&entry.path())?);
    }

    entries.sort_by(|a, b| compare_entries(a, b, sort_by, sort_order));
    let total = entries.len();
    let mut window: Vec<Value> = entries.into_iter().skip(offset).collect();
    if let Some(l) = limit {
        window.truncate(l);
    }
    let next_offset = if offset.saturating_add(window.len()) < total {
        Some(offset.saturating_add(window.len()))
    } else {
        None
    };

    Ok(json!({
        "ok": true,
        "operation": "fs_list",
        "entries": window,
        "total": total,
        "returned": window.len(),
        "next_offset": next_offset,
        "completed": next_offset.is_none(),
    }))
}

pub fn fs_copy(arguments: &Value) -> FlashgrepResult<Value> {
    let src = required_path(arguments, "src")?;
    let dst = required_path(arguments, "dst")?;
    let src_path = resolve_path(src);
    let dst_path = resolve_path(dst);
    let recursive = arguments
        .get("recursive")
        .and_then(Value::as_bool)
        .unwrap_or(false);
    let overwrite = arguments
        .get("overwrite")
        .and_then(Value::as_bool)
        .unwrap_or(false);
    let dry_run = arguments
        .get("dry_run")
        .and_then(Value::as_bool)
        .unwrap_or(false);

    if !src_path.exists() {
        return Ok(with_operation(
            not_found_error(src, infer_kind_from_expected(src, false)),
            "fs_copy",
            dry_run,
        ));
    }
    if dst_path.exists() && !overwrite {
        return Ok(json!({
            "ok": false,
            "error": "conflict",
            "reason_code": "destination_exists",
            "operation": "fs_copy",
            "source_path": src_path.to_string_lossy(),
            "target_path": dst_path.to_string_lossy(),
        }));
    }

    if !dry_run {
        if src_path.is_dir() {
            if !recursive {
                return Ok(json!({
                    "ok": false,
                    "error": "invalid_params",
                    "reason_code": "recursive_required",
                    "operation": "fs_copy",
                    "source_path": src_path.to_string_lossy(),
                }));
            }
            copy_dir_recursive(&src_path, &dst_path, overwrite)?;
        } else {
            if let Some(parent) = dst_path.parent() {
                fs::create_dir_all(parent)?;
            }
            let _ = fs::copy(&src_path, &dst_path)?;
        }
    }

    Ok(json!({
        "ok": true,
        "operation": "fs_copy",
        "dry_run": dry_run,
        "source": metadata_json(&src_path)?,
        "target_path": dst_path.to_string_lossy(),
    }))
}

pub fn fs_move(arguments: &Value) -> FlashgrepResult<Value> {
    let src = required_path(arguments, "src")?;
    let dst = required_path(arguments, "dst")?;
    let src_path = resolve_path(src);
    let dst_path = resolve_path(dst);
    let overwrite = arguments
        .get("overwrite")
        .and_then(Value::as_bool)
        .unwrap_or(false);
    let dry_run = arguments
        .get("dry_run")
        .and_then(Value::as_bool)
        .unwrap_or(false);

    if !src_path.exists() {
        return Ok(with_operation(
            not_found_error(src, infer_kind_from_expected(src, false)),
            "fs_move",
            dry_run,
        ));
    }
    if dst_path.exists() && !overwrite {
        return Ok(json!({
            "ok": false,
            "error": "conflict",
            "reason_code": "destination_exists",
            "operation": "fs_move",
            "source_path": src_path.to_string_lossy(),
            "target_path": dst_path.to_string_lossy(),
        }));
    }

    if !dry_run {
        if dst_path.exists() {
            if dst_path.is_dir() {
                fs::remove_dir_all(&dst_path)?;
            } else {
                fs::remove_file(&dst_path)?;
            }
        }
        if let Some(parent) = dst_path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::rename(&src_path, &dst_path)?;
    }

    Ok(json!({
        "ok": true,
        "operation": "fs_move",
        "dry_run": dry_run,
        "source_path": src_path.to_string_lossy(),
        "target_path": dst_path.to_string_lossy(),
    }))
}

pub fn fs_remove(arguments: &Value) -> FlashgrepResult<Value> {
    let path = required_path(arguments, "path")?;
    let target = resolve_path(path);
    let recursive = arguments
        .get("recursive")
        .and_then(Value::as_bool)
        .unwrap_or(false);
    let force = arguments
        .get("force")
        .and_then(Value::as_bool)
        .unwrap_or(false);
    let dry_run = arguments
        .get("dry_run")
        .and_then(Value::as_bool)
        .unwrap_or(false);

    if !target.exists() {
        if force {
            return Ok(json!({
                "ok": true,
                "operation": "fs_remove",
                "dry_run": dry_run,
                "target_path": target.to_string_lossy(),
                "already_missing": true,
            }));
        }
        return Ok(with_operation(
            not_found_error(path, infer_kind_from_expected(path, true)),
            "fs_remove",
            dry_run,
        ));
    }

    if !dry_run {
        if target.is_dir() {
            if recursive {
                fs::remove_dir_all(&target)?;
            } else {
                fs::remove_dir(&target).map_err(|_| {
                    FlashgrepError::Config(format!(
                        "Directory removal requires recursive=true when non-empty: {}",
                        target.display()
                    ))
                })?;
            }
        } else {
            fs::remove_file(&target)?;
        }
    }

    Ok(json!({
        "ok": true,
        "operation": "fs_remove",
        "dry_run": dry_run,
        "target_path": target.to_string_lossy(),
    }))
}

fn required_path<'a>(arguments: &'a Value, key: &str) -> FlashgrepResult<&'a str> {
    arguments
        .get(key)
        .and_then(Value::as_str)
        .filter(|s| !s.trim().is_empty())
        .ok_or_else(|| FlashgrepError::Config(format!("Missing required parameter: {}", key)))
}

fn resolve_path(path: &str) -> PathBuf {
    let p = PathBuf::from(path);
    if p.is_absolute() {
        p
    } else {
        std::env::current_dir()
            .unwrap_or_else(|_| PathBuf::from("."))
            .join(p)
    }
}

fn metadata_json(path: &Path) -> FlashgrepResult<Value> {
    let md = fs::metadata(path)?;
    let modified = md
        .modified()
        .ok()
        .and_then(|m| m.duration_since(UNIX_EPOCH).ok())
        .map(|d| d.as_secs());
    Ok(json!({
        "path": path.to_string_lossy(),
        "file_type": if md.is_dir() {"directory"} else {"file"},
        "size": md.len(),
        "modified_unix": modified,
        "readonly": md.permissions().readonly(),
    }))
}

fn with_operation(mut payload: Value, operation: &str, dry_run: bool) -> Value {
    if let Some(obj) = payload.as_object_mut() {
        obj.insert(
            "operation".to_string(),
            Value::String(operation.to_string()),
        );
        obj.insert("dry_run".to_string(), Value::Bool(dry_run));
    }
    payload
}

fn compare_entries(a: &Value, b: &Value, sort_by: &str, sort_order: &str) -> Ordering {
    let ord = match sort_by {
        "name" => entry_name(a)
            .cmp(&entry_name(b))
            .then_with(|| entry_path(a).cmp(&entry_path(b))),
        "modified" => entry_u64(a, "modified_unix")
            .cmp(&entry_u64(b, "modified_unix"))
            .then_with(|| entry_path(a).cmp(&entry_path(b))),
        "size" => entry_u64(a, "size")
            .cmp(&entry_u64(b, "size"))
            .then_with(|| entry_path(a).cmp(&entry_path(b))),
        _ => entry_path(a).cmp(&entry_path(b)),
    };
    if sort_order == "desc" {
        ord.reverse()
    } else {
        ord
    }
}

fn entry_path(v: &Value) -> String {
    v.get("path")
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string()
}

fn entry_name(v: &Value) -> String {
    Path::new(&entry_path(v))
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_default()
}

fn entry_u64(v: &Value, key: &str) -> u64 {
    v.get(key).and_then(Value::as_u64).unwrap_or(0)
}

fn copy_dir_recursive(src: &Path, dst: &Path, overwrite: bool) -> FlashgrepResult<()> {
    fs::create_dir_all(dst)?;
    for entry in walkdir::WalkDir::new(src) {
        let entry = entry.map_err(|e| FlashgrepError::Config(e.to_string()))?;
        let rel = entry
            .path()
            .strip_prefix(src)
            .map_err(|e| FlashgrepError::Config(e.to_string()))?;
        if rel.as_os_str().is_empty() {
            continue;
        }
        let target = dst.join(rel);
        if entry.file_type().is_dir() {
            fs::create_dir_all(&target)?;
        } else if entry.file_type().is_file() {
            if target.exists() && !overwrite {
                return Err(FlashgrepError::Config(format!(
                    "Destination already exists: {}",
                    target.display()
                )));
            }
            if let Some(parent) = target.parent() {
                fs::create_dir_all(parent)?;
            }
            let _ = fs::copy(entry.path(), &target)?;
        }
    }
    Ok(())
}

fn infer_kind_from_expected(path: &str, remove_mode: bool) -> &'static str {
    if path.ends_with('/') || path.ends_with('\\') {
        return "directory";
    }
    let lower = path.to_ascii_lowercase();
    if lower.ends_with(".rs")
        || lower.ends_with(".md")
        || lower.ends_with(".txt")
        || lower.contains('.')
    {
        return "file";
    }
    if remove_mode {
        "path"
    } else {
        "directory"
    }
}

fn looks_like_directory(path: &str) -> bool {
    path.ends_with('/') || path.ends_with('\\') || !path.contains('.')
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mcp::safety::{REASON_DIRECTORY_NOT_FOUND, REASON_FILE_NOT_FOUND};
    use tempfile::TempDir;

    #[test]
    fn read_missing_file_returns_typed_not_found() {
        let payload = fs_read(&json!({"path": "__missing__.txt"})).expect("payload");
        assert_eq!(payload["error"], Value::String("not_found".to_string()));
        assert_eq!(
            payload["reason_code"],
            Value::String(REASON_FILE_NOT_FOUND.to_string())
        );
    }

    #[test]
    fn list_missing_dir_returns_typed_not_found() {
        let payload = fs_list(&json!({"path": "__missing__/"})).expect("payload");
        assert_eq!(payload["error"], Value::String("not_found".to_string()));
        assert_eq!(
            payload["reason_code"],
            Value::String(REASON_DIRECTORY_NOT_FOUND.to_string())
        );
    }

    #[test]
    fn create_and_stat_work() {
        let tmp = TempDir::new().expect("tmp");
        let p = tmp.path().join("a.txt");
        let create =
            fs_create(&json!({"path": p.to_string_lossy(), "parents": true})).expect("create");
        assert_eq!(create["ok"], Value::Bool(true));
        let stat = fs_stat(&json!({"path": p.to_string_lossy()})).expect("stat");
        assert_eq!(
            stat["entry"]["file_type"],
            Value::String("file".to_string())
        );
    }
}
