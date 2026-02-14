use crate::config::paths::FlashgrepPaths;
use crate::mcp::skill::{bootstrap_policy, get_skill_documentation, get_skill_info};
use crate::{FlashgrepError, FlashgrepResult};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::sync::atomic::{AtomicBool, Ordering};

pub const CANONICAL_BOOTSTRAP_TRIGGER: &str = "flashgrep-init";
pub const BOOTSTRAP_TOOL_ALIASES: [&str; 5] = [
    "bootstrap_skill",
    "flashgrep-init",
    "fgrep-boot",
    "flashgrep_init",
    "fgrep_boot",
];

pub fn is_bootstrap_tool(name: &str) -> bool {
    BOOTSTRAP_TOOL_ALIASES.contains(&name)
}

pub fn build_bootstrap_payload(
    paths: &FlashgrepPaths,
    requested_tool: &str,
    arguments: &Value,
    injected_state: &AtomicBool,
) -> FlashgrepResult<Value> {
    let requested_trigger = arguments
        .get("trigger")
        .and_then(Value::as_str)
        .unwrap_or(requested_tool);

    let canonical_trigger = if is_bootstrap_tool(requested_trigger) {
        CANONICAL_BOOTSTRAP_TRIGGER
    } else {
        return Ok(json!({
            "ok": false,
            "error": "invalid_trigger",
            "requested_trigger": requested_trigger,
            "allowed": BOOTSTRAP_TOOL_ALIASES,
        }));
    };

    let force = arguments
        .get("force")
        .and_then(Value::as_bool)
        .unwrap_or(false);
    let compact = arguments
        .get("compact")
        .and_then(Value::as_bool)
        .unwrap_or(false);

    if injected_state.load(Ordering::SeqCst) && !force {
        return Ok(json!({
            "ok": true,
            "status": "already_injected",
            "canonical_trigger": canonical_trigger,
            "policy": bootstrap_policy(),
        }));
    }

    let repo_root = paths
        .root()
        .parent()
        .map(|p| p.to_path_buf())
        .ok_or_else(|| FlashgrepError::Config("Unable to resolve repository root".to_string()))?;
    let skill_path = repo_root.join("skills").join("SKILL.md");
    let skill_text = match std::fs::read_to_string(&skill_path) {
        Ok(text) => text,
        Err(e) => {
            return Ok(json!({
                "ok": false,
                "error": if e.kind() == std::io::ErrorKind::NotFound {
                    "skill_not_found"
                } else {
                    "skill_unreadable"
                },
                "message": e.to_string(),
                "source_path": skill_path,
            }));
        }
    };

    injected_state.store(true, Ordering::SeqCst);
    let mut hasher = Sha256::new();
    hasher.update(skill_text.as_bytes());
    let skill_hash = hex::encode(hasher.finalize());
    let info = get_skill_info();
    let skill_version = info.version.clone();
    let docs = get_skill_documentation();
    let policy = bootstrap_policy();

    if compact {
        Ok(json!({
            "ok": true,
            "status": "injected",
            "canonical_trigger": canonical_trigger,
            "source_path": skill_path,
            "skill_hash": skill_hash,
            "skill_version": skill_version,
            "skill_info": info,
            "policy": policy,
        }))
    } else {
        Ok(json!({
            "ok": true,
            "status": "injected",
            "canonical_trigger": canonical_trigger,
            "source_path": skill_path,
            "skill_hash": skill_hash,
            "skill_version": skill_version,
            "skill_info": info,
            "skill_overview": docs.overview,
            "policy": policy,
            "skill_markdown": skill_text,
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::paths::FlashgrepPaths;
    use std::fs;
    use tempfile::TempDir;

    fn setup_paths_with_skill(skill_text: Option<&str>) -> (TempDir, FlashgrepPaths, AtomicBool) {
        let temp = TempDir::new().expect("temp dir");
        let repo_root = temp.path().to_path_buf();
        let skill_dir = repo_root.join("skills");
        fs::create_dir_all(&skill_dir).expect("create skill dir");
        if let Some(text) = skill_text {
            fs::write(skill_dir.join("SKILL.md"), text).expect("write skill file");
        }
        let paths = FlashgrepPaths::new(&repo_root);
        (temp, paths, AtomicBool::new(false))
    }

    #[test]
    fn accepts_all_bootstrap_aliases() {
        let (_temp, paths, injected) = setup_paths_with_skill(Some("# skill"));
        for alias in BOOTSTRAP_TOOL_ALIASES {
            let payload = build_bootstrap_payload(
                &paths,
                alias,
                &json!({"compact": true, "force": true}),
                &injected,
            )
            .expect("payload");
            assert_eq!(
                payload["canonical_trigger"],
                Value::String(CANONICAL_BOOTSTRAP_TRIGGER.to_string())
            );
        }
    }

    #[test]
    fn invalid_trigger_returns_typed_error() {
        let (_temp, paths, injected) = setup_paths_with_skill(Some("# skill"));
        let payload = build_bootstrap_payload(
            &paths,
            "bootstrap_skill",
            &json!({"trigger": "unknown"}),
            &injected,
        )
        .expect("payload");
        assert_eq!(
            payload["error"],
            Value::String("invalid_trigger".to_string())
        );
    }

    #[test]
    fn idempotent_behavior_is_preserved() {
        let (_temp, paths, injected) = setup_paths_with_skill(Some("# skill"));
        let _ = build_bootstrap_payload(
            &paths,
            "flashgrep-init",
            &json!({"compact": true}),
            &injected,
        )
        .expect("first payload");

        let second = build_bootstrap_payload(
            &paths,
            "flashgrep-init",
            &json!({"compact": true}),
            &injected,
        )
        .expect("second payload");
        assert_eq!(
            second["status"],
            Value::String("already_injected".to_string())
        );
    }

    #[test]
    fn missing_or_unreadable_skill_is_typed_error() {
        let temp_missing = TempDir::new().expect("temp dir");
        let paths_missing = FlashgrepPaths::new(&temp_missing.path().to_path_buf());
        let state_missing = AtomicBool::new(false);
        let missing = build_bootstrap_payload(
            &paths_missing,
            "flashgrep-init",
            &json!({"compact": true}),
            &state_missing,
        )
        .expect("missing payload");
        assert_eq!(
            missing["error"],
            Value::String("skill_not_found".to_string())
        );

        let temp_unreadable = TempDir::new().expect("temp dir");
        let skill_dir = temp_unreadable.path().join("skills");
        fs::create_dir_all(skill_dir.join("SKILL.md")).expect("create dir instead of file");
        let paths_unreadable = FlashgrepPaths::new(&temp_unreadable.path().to_path_buf());
        let state_unreadable = AtomicBool::new(false);
        let unreadable = build_bootstrap_payload(
            &paths_unreadable,
            "flashgrep-init",
            &json!({"compact": true}),
            &state_unreadable,
        )
        .expect("unreadable payload");
        assert_eq!(
            unreadable["error"],
            Value::String("skill_unreadable".to_string())
        );
    }
}
