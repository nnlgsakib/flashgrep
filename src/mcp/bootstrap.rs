use crate::config::paths::FlashgrepPaths;
use crate::mcp::skill::{
    bootstrap_policy, bootstrap_policy_metadata, get_skill_documentation, get_skill_info,
};
use crate::{FlashgrepError, FlashgrepResult};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};

pub const CANONICAL_BOOTSTRAP_TRIGGER: &str = "flashgrep-init";
pub const BOOTSTRAP_TOOL_ALIASES: [&str; 5] = [
    "bootstrap_skill",
    "flashgrep-init",
    "fgrep-boot",
    "flashgrep_init",
    "fgrep_boot",
];

const EMBEDDED_SKILL_MARKDOWN: &str = include_str!("../../skills/SKILL.md");

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
    let payload_resolution = resolve_skill_payload(paths, arguments)?;

    if injected_state.load(Ordering::SeqCst) && !force {
        let mut policy_metadata = bootstrap_policy_metadata();
        annotate_policy_metadata(
            &mut policy_metadata,
            "already_injected",
            &payload_resolution,
        );
        return Ok(json!({
            "ok": true,
            "status": "already_injected",
            "canonical_trigger": canonical_trigger,
            "payload_source": payload_resolution.source,
            "source_path": payload_resolution.source_path,
            "override_requested": payload_resolution.override_requested,
            "fallback_gate": payload_resolution.fallback_gate,
            "fallback_reason_code": payload_resolution.fallback_reason_code,
            "policy": bootstrap_policy(),
            "policy_metadata": policy_metadata,
        }));
    }

    injected_state.store(true, Ordering::SeqCst);
    let mut hasher = Sha256::new();
    hasher.update(payload_resolution.skill_text.as_bytes());
    let skill_hash = hex::encode(hasher.finalize());
    let info = get_skill_info();
    let skill_version = info.version.clone();
    let docs = get_skill_documentation();
    let policy = bootstrap_policy();
    let mut policy_metadata = bootstrap_policy_metadata();
    annotate_policy_metadata(&mut policy_metadata, "injected", &payload_resolution);

    if compact {
        Ok(json!({
            "ok": true,
            "status": "injected",
            "canonical_trigger": canonical_trigger,
            "payload_source": payload_resolution.source,
            "source_path": payload_resolution.source_path,
            "override_requested": payload_resolution.override_requested,
            "fallback_gate": payload_resolution.fallback_gate,
            "fallback_reason_code": payload_resolution.fallback_reason_code,
            "skill_hash": skill_hash,
            "skill_version": skill_version,
            "skill_info": info,
            "policy": policy,
            "policy_metadata": policy_metadata,
        }))
    } else {
        Ok(json!({
            "ok": true,
            "status": "injected",
            "canonical_trigger": canonical_trigger,
            "payload_source": payload_resolution.source,
            "source_path": payload_resolution.source_path,
            "override_requested": payload_resolution.override_requested,
            "fallback_gate": payload_resolution.fallback_gate,
            "fallback_reason_code": payload_resolution.fallback_reason_code,
            "skill_hash": skill_hash,
            "skill_version": skill_version,
            "skill_info": info,
            "skill_overview": docs.overview,
            "policy": policy,
            "policy_metadata": policy_metadata,
            "skill_markdown": payload_resolution.skill_text,
        }))
    }
}

#[derive(Debug)]
struct SkillPayloadResolution {
    skill_text: String,
    source: &'static str,
    source_path: Option<String>,
    override_requested: bool,
    fallback_gate: Option<&'static str>,
    fallback_reason_code: Option<&'static str>,
}

fn resolve_skill_payload(
    paths: &FlashgrepPaths,
    arguments: &Value,
) -> FlashgrepResult<SkillPayloadResolution> {
    let override_requested = arguments
        .get("allow_repo_override")
        .and_then(Value::as_bool)
        .unwrap_or(false);

    if !override_requested {
        return Ok(SkillPayloadResolution {
            skill_text: EMBEDDED_SKILL_MARKDOWN.to_string(),
            source: "embedded",
            source_path: None,
            override_requested,
            fallback_gate: None,
            fallback_reason_code: None,
        });
    }

    let repo_root = paths
        .root()
        .parent()
        .map(|p| p.to_path_buf())
        .ok_or_else(|| FlashgrepError::Config("Unable to resolve repository root".to_string()))?;
    let override_path = arguments
        .get("repo_override_path")
        .and_then(Value::as_str)
        .map(PathBuf::from)
        .unwrap_or_else(|| repo_root.join("skills").join("SKILL.md"));

    match std::fs::read_to_string(&override_path) {
        Ok(text) => Ok(SkillPayloadResolution {
            skill_text: text,
            source: "repo_override",
            source_path: Some(override_path.to_string_lossy().to_string()),
            override_requested,
            fallback_gate: None,
            fallback_reason_code: None,
        }),
        Err(_) => Ok(SkillPayloadResolution {
            skill_text: EMBEDDED_SKILL_MARKDOWN.to_string(),
            source: "embedded",
            source_path: Some(override_path.to_string_lossy().to_string()),
            override_requested,
            fallback_gate: Some("repo_override_read_failed"),
            fallback_reason_code: Some("repo_override_unavailable"),
        }),
    }
}

fn annotate_policy_metadata(
    policy_metadata: &mut Value,
    bootstrap_state: &str,
    resolution: &SkillPayloadResolution,
) {
    if let Some(obj) = policy_metadata.as_object_mut() {
        obj.insert(
            "bootstrap_state".to_string(),
            Value::String(bootstrap_state.to_string()),
        );
        obj.insert(
            "payload_source".to_string(),
            Value::String(resolution.source.to_string()),
        );
        obj.insert(
            "override_requested".to_string(),
            Value::Bool(resolution.override_requested),
        );
        if let Some(path) = resolution.source_path.as_ref() {
            obj.insert("source_path".to_string(), Value::String(path.clone()));
        }
        if let Some(gate) = resolution.fallback_gate {
            obj.insert("fallback_gate".to_string(), Value::String(gate.to_string()));
        }
        if let Some(reason) = resolution.fallback_reason_code {
            obj.insert(
                "fallback_reason_code".to_string(),
                Value::String(reason.to_string()),
            );
        }
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
        assert_eq!(
            second["policy_metadata"]["policy_strength"],
            Value::String("strict".to_string())
        );
    }

    #[test]
    fn bootstrap_includes_policy_metadata_and_legacy_fields() {
        let (_temp, paths, injected) = setup_paths_with_skill(None);
        let payload = build_bootstrap_payload(
            &paths,
            "flashgrep-init",
            &json!({"compact": true, "force": true}),
            &injected,
        )
        .expect("payload");

        assert!(payload["policy"].is_array());
        assert!(payload["policy_metadata"].is_object());
        assert_eq!(
            payload["policy_metadata"]["policy_strength"],
            Value::String("strict".to_string())
        );
        assert!(payload["policy_metadata"]["prohibited_native_tools"].is_object());
        assert!(payload["status"].as_str().is_some());
        assert!(payload["canonical_trigger"].as_str().is_some());
        assert!(payload["skill_hash"].as_str().is_some());
        assert!(payload["skill_version"].as_str().is_some());
        assert_eq!(
            payload["payload_source"],
            Value::String("embedded".to_string())
        );
        assert_eq!(
            payload["policy_metadata"]["bootstrap_state"],
            Value::String("injected".to_string())
        );
    }

    #[test]
    fn bootstrap_defaults_to_embedded_when_skill_file_missing() {
        let temp = TempDir::new().expect("temp dir");
        let paths = FlashgrepPaths::new(&temp.path().to_path_buf());
        let state = AtomicBool::new(false);
        let payload =
            build_bootstrap_payload(&paths, "flashgrep-init", &json!({"compact": true}), &state)
                .expect("payload");
        assert_eq!(payload["ok"], Value::Bool(true));
        assert_eq!(
            payload["payload_source"],
            Value::String("embedded".to_string())
        );
    }

    #[test]
    fn repo_override_is_opt_in_and_reports_fallback_diagnostics() {
        let temp = TempDir::new().expect("temp dir");
        let paths = FlashgrepPaths::new(&temp.path().to_path_buf());
        let state = AtomicBool::new(false);
        let payload = build_bootstrap_payload(
            &paths,
            "flashgrep-init",
            &json!({"compact": true, "allow_repo_override": true}),
            &state,
        )
        .expect("payload");
        assert_eq!(
            payload["payload_source"],
            Value::String("embedded".to_string())
        );
        assert_eq!(
            payload["fallback_gate"],
            Value::String("repo_override_read_failed".to_string())
        );
        assert_eq!(
            payload["fallback_reason_code"],
            Value::String("repo_override_unavailable".to_string())
        );
    }

    #[test]
    fn repo_override_can_provide_payload_when_enabled() {
        let (temp, paths, state) = setup_paths_with_skill(Some("# repo skill override"));
        let payload = build_bootstrap_payload(
            &paths,
            "flashgrep-init",
            &json!({"compact": false, "allow_repo_override": true}),
            &state,
        )
        .expect("payload");
        assert_eq!(
            payload["payload_source"],
            Value::String("repo_override".to_string())
        );
        assert!(payload["skill_markdown"]
            .as_str()
            .unwrap_or_default()
            .contains("repo skill override"));

        drop(temp);
    }
}
