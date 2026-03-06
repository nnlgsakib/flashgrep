use crate::config::paths::FlashgrepPaths;
use crate::mcp::safety::{
    REASON_FALLBACK_GATE_MISMATCH, REASON_FALLBACK_GATE_REQUIRED,
    REASON_UNSUPPORTED_FALLBACK_REASON_CODE,
};
use crate::mcp::skill::{
    bootstrap_policy, bootstrap_policy_metadata, get_skill_documentation, get_skill_info,
};
use crate::{FlashgrepError, FlashgrepResult};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::OnceLock;

pub const CANONICAL_BOOTSTRAP_TRIGGER: &str = "flashgrep-init";
pub const BOOTSTRAP_TOOL_ALIASES: [&str; 5] = [
    "bootstrap_skill",
    "flashgrep-init",
    "fgrep-boot",
    "flashgrep_init",
    "fgrep_boot",
];

const FALLBACK_ROUTE_TOOLS: [&str; 4] = [
    "search",
    "search-in-directory",
    "search-with-context",
    "search-by-regex",
];

const NATIVE_ROUTE_TOOLS: [&str; 17] = [
    "query",
    "glob",
    "get_slice",
    "read_code",
    "write_code",
    "batch_write_code",
    "get_symbol",
    "list_files",
    "stats",
    "fs_create",
    "fs_read",
    "fs_write",
    "fs_list",
    "fs_stat",
    "fs_copy",
    "fs_move",
    "fs_remove",
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PolicyRouteState {
    AllowedNative,
    AllowedFallback,
    Denied,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PolicyRouteDecision {
    pub route_state: PolicyRouteState,
    pub reason_code: Option<String>,
    pub fallback_gate_id: Option<String>,
    pub recovery_hint: Option<String>,
}

impl PolicyRouteDecision {
    pub fn as_str(&self) -> &'static str {
        match self.route_state {
            PolicyRouteState::AllowedNative => "allowed_native",
            PolicyRouteState::AllowedFallback => "allowed_fallback",
            PolicyRouteState::Denied => "denied",
        }
    }
}

const EMBEDDED_SKILL_MARKDOWN: &str = include_str!("../../skills/SKILL.md");
static POLICY_HASH_CACHE: OnceLock<String> = OnceLock::new();

pub fn current_policy_hash() -> String {
    POLICY_HASH_CACHE
        .get_or_init(|| {
            let mut hasher = Sha256::new();
            hasher.update(bootstrap_policy().join("\n").as_bytes());
            hasher.update(bootstrap_policy_metadata().to_string().as_bytes());
            hex::encode(hasher.finalize())
        })
        .clone()
}

pub fn policy_denied_payload(tool_name: &str, decision: &PolicyRouteDecision) -> Value {
    json!({
        "ok": false,
        "error": "policy_denied",
        "route_state": decision.as_str(),
        "requested_tool": tool_name,
        "reason_code": decision.reason_code,
        "fallback_gate_id": decision.fallback_gate_id,
        "recovery_hint": decision.recovery_hint,
        "policy_hash": current_policy_hash(),
    })
}

pub fn evaluate_policy_route(tool_name: &str, arguments: &Value) -> PolicyRouteDecision {
    if let Some(observed_hash) = arguments.get("policy_hash").and_then(Value::as_str) {
        let expected_hash = current_policy_hash();
        if observed_hash != expected_hash {
            return PolicyRouteDecision {
                route_state: PolicyRouteState::Denied,
                reason_code: Some("policy_state_mismatch".to_string()),
                fallback_gate_id: None,
                recovery_hint: Some(
                    "Policy hash mismatch. Re-run bootstrap with force=true and retry".to_string(),
                ),
            };
        }
    }

    if let Some(observed_version) = arguments.get("policy_version").and_then(Value::as_str) {
        let metadata = bootstrap_policy_metadata();
        let expected_version = metadata["policy_version"].as_str().unwrap_or("1.1");
        if observed_version != expected_version {
            return PolicyRouteDecision {
                route_state: PolicyRouteState::Denied,
                reason_code: Some("policy_state_mismatch".to_string()),
                fallback_gate_id: None,
                recovery_hint: Some(
                    "Policy version mismatch. Re-run bootstrap with force=true and retry"
                        .to_string(),
                ),
            };
        }
    }

    if is_bootstrap_tool(tool_name) || NATIVE_ROUTE_TOOLS.contains(&tool_name) {
        return PolicyRouteDecision {
            route_state: PolicyRouteState::AllowedNative,
            reason_code: None,
            fallback_gate_id: None,
            recovery_hint: None,
        };
    }

    if FALLBACK_ROUTE_TOOLS.contains(&tool_name) {
        let fallback_reason = arguments
            .get("fallback_reason_code")
            .and_then(Value::as_str);
        let fallback_gate = arguments.get("fallback_gate").and_then(Value::as_str);

        let Some(reason) = fallback_reason else {
            return PolicyRouteDecision {
                route_state: PolicyRouteState::Denied,
                reason_code: Some(REASON_FALLBACK_GATE_REQUIRED.to_string()),
                fallback_gate_id: None,
                recovery_hint: Some(
                    "Provide fallback_reason_code and fallback_gate, or route via native Flashgrep tools"
                        .to_string(),
                ),
            };
        };

        let Some(expected_gate) = reason_to_gate_id(reason) else {
            return PolicyRouteDecision {
                route_state: PolicyRouteState::Denied,
                reason_code: Some(REASON_UNSUPPORTED_FALLBACK_REASON_CODE.to_string()),
                fallback_gate_id: fallback_gate.map(ToString::to_string),
                recovery_hint: Some(
                    "Use a supported fallback_reason_code from policy metadata fallback rules"
                        .to_string(),
                ),
            };
        };

        if let Some(gate) = fallback_gate {
            if gate != expected_gate {
                return PolicyRouteDecision {
                    route_state: PolicyRouteState::Denied,
                    reason_code: Some(REASON_FALLBACK_GATE_MISMATCH.to_string()),
                    fallback_gate_id: Some(gate.to_string()),
                    recovery_hint: Some(format!(
                        "fallback_gate '{}' must match reason_code '{}'",
                        expected_gate, reason
                    )),
                };
            }
        }

        return PolicyRouteDecision {
            route_state: PolicyRouteState::AllowedFallback,
            reason_code: Some(reason.to_string()),
            fallback_gate_id: Some(expected_gate.to_string()),
            recovery_hint: None,
        };
    }

    PolicyRouteDecision {
        route_state: PolicyRouteState::Denied,
        reason_code: Some("flashgrep_operation_not_supported".to_string()),
        fallback_gate_id: None,
        recovery_hint: Some("Use a supported Flashgrep MCP tool".to_string()),
    }
}

fn reason_to_gate_id(reason_code: &str) -> Option<&'static str> {
    match reason_code {
        "neural_mode_disabled" => Some("neural_mode_disabled"),
        "neural_provider_failure" => Some("neural_provider_failure"),
        "neural_no_relevant_matches" => Some("neural_no_relevant_matches"),
        "exact_match_required" => Some("exact_match_required"),
        "query_parse_constraints" => Some("query_parse_constraints"),
        "flashgrep_index_unavailable" => Some("index_unavailable"),
        "flashgrep_operation_not_supported" => Some("unsupported_operation"),
        "flashgrep_tool_runtime_failure" => Some("tool_runtime_failure"),
        "repo_override_unavailable" => Some("repo_override_read_failed"),
        _ => None,
    }
}

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
        obj.insert(
            "policy_hash".to_string(),
            Value::String(current_policy_hash()),
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
        assert_eq!(
            payload["policy_metadata"]["search_routing"]["default_strategy"],
            Value::String("neural_first".to_string())
        );
        assert_eq!(
            payload["policy_metadata"]["search_routing"]["programmatic_priority"],
            Value::String("fallback".to_string())
        );
        assert!(payload["policy_metadata"]["search_routing"]["fallback_reason_codes"].is_array());
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
        let paths = FlashgrepPaths::new(temp.path());
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
        let paths = FlashgrepPaths::new(temp.path());
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

    #[test]
    fn fallback_reason_codes_are_typed_and_present() {
        let (_temp, paths, injected) = setup_paths_with_skill(None);
        let payload = build_bootstrap_payload(
            &paths,
            "flashgrep-init",
            &json!({"compact": true, "force": true}),
            &injected,
        )
        .expect("payload");

        let reasons = payload["policy_metadata"]["search_routing"]["fallback_reason_codes"]
            .as_array()
            .expect("reason codes");
        let as_strings: Vec<&str> = reasons.iter().filter_map(Value::as_str).collect();

        assert!(as_strings.contains(&"exact_match_required"));
        assert!(as_strings.contains(&"query_parse_constraints"));
    }

    #[test]
    fn evaluate_policy_route_denies_fallback_without_reason_code() {
        let decision = evaluate_policy_route("search", &json!({}));
        assert_eq!(decision.route_state, PolicyRouteState::Denied);
        assert_eq!(
            decision.reason_code,
            Some("fallback_gate_required".to_string())
        );
    }

    #[test]
    fn evaluate_policy_route_allows_valid_fallback_gate_and_reason() {
        let decision = evaluate_policy_route(
            "search-by-regex",
            &json!({
                "fallback_gate": "tool_runtime_failure",
                "fallback_reason_code": "flashgrep_tool_runtime_failure"
            }),
        );
        assert_eq!(decision.route_state, PolicyRouteState::AllowedFallback);
        assert_eq!(
            decision.fallback_gate_id,
            Some("tool_runtime_failure".to_string())
        );
    }

    #[test]
    fn evaluate_policy_route_detects_policy_hash_mismatch() {
        let decision = evaluate_policy_route(
            "query",
            &json!({
                "policy_hash": "stale-hash"
            }),
        );
        assert_eq!(decision.route_state, PolicyRouteState::Denied);
        assert_eq!(
            decision.reason_code,
            Some("policy_state_mismatch".to_string())
        );
    }

    #[test]
    fn bootstrap_payload_includes_policy_hash_metadata() {
        let (_temp, paths, injected) = setup_paths_with_skill(None);
        let payload = build_bootstrap_payload(
            &paths,
            "flashgrep-init",
            &json!({"compact": true, "force": true}),
            &injected,
        )
        .expect("payload");

        assert!(payload["policy_metadata"]["policy_hash"].as_str().is_some());
    }

    #[test]
    fn force_reinjection_restores_injected_state() {
        let (_temp, paths, injected) = setup_paths_with_skill(None);

        let first = build_bootstrap_payload(
            &paths,
            "flashgrep-init",
            &json!({"compact": true}),
            &injected,
        )
        .expect("first payload");
        assert_eq!(first["status"], Value::String("injected".to_string()));

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

        let forced = build_bootstrap_payload(
            &paths,
            "flashgrep-init",
            &json!({"compact": true, "force": true}),
            &injected,
        )
        .expect("forced payload");
        assert_eq!(forced["status"], Value::String("injected".to_string()));
    }
}
