//! MCP skill implementation

use crate::mcp::bootstrap::BOOTSTRAP_TOOL_ALIASES;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize)]
pub struct SkillInfo {
    pub name: String,
    pub version: String,
    pub description: String,
    pub author: String,
    pub repository: String,
}

impl Default for SkillInfo {
    fn default() -> Self {
        Self {
            name: "flashgrep".to_string(),
            version: "1.3.0".to_string(),
            description: "High-performance local code indexing engine".to_string(),
            author: "Flashgrep Contributors".to_string(),
            repository: "https://github.com/nnlgsakib/flashgrep".to_string(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SkillDocumentation {
    pub overview: String,
    pub commands: HashMap<String, CommandDocumentation>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CommandDocumentation {
    pub description: String,
    pub parameters: Vec<ParameterDocumentation>,
    pub examples: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ParameterDocumentation {
    pub name: String,
    pub type_: String,
    pub description: String,
    pub required: bool,
}

pub fn get_skill_info() -> SkillInfo {
    SkillInfo::default()
}

pub fn get_skill_documentation() -> SkillDocumentation {
    let mut commands = HashMap::new();

    // Search tool
    commands.insert(
        "search".to_string(),
        CommandDocumentation {
            description: "Perform a basic grep search".to_string(),
            parameters: vec![
                ParameterDocumentation {
                    name: "pattern".to_string(),
                    type_: "string".to_string(),
                    description: "Search pattern".to_string(),
                    required: true,
                },
                ParameterDocumentation {
                    name: "files".to_string(),
                    type_: "array".to_string(),
                    description: "List of files to search".to_string(),
                    required: true,
                },
                ParameterDocumentation {
                    name: "case_sensitive".to_string(),
                    type_: "boolean".to_string(),
                    description: "Case sensitive search".to_string(),
                    required: false,
                },
            ],
            examples: vec![
                r#"{"pattern": "fn main", "files": ["src/main.rs"]}"#.to_string(),
                r#"{"pattern": "struct", "files": ["src/**/*.rs"], "case_sensitive": false}"#
                    .to_string(),
            ],
        },
    );

    // Search in directory tool
    commands.insert(
        "search-in-directory".to_string(),
        CommandDocumentation {
            description: "Search recursively in a directory".to_string(),
            parameters: vec![
                ParameterDocumentation {
                    name: "pattern".to_string(),
                    type_: "string".to_string(),
                    description: "Search pattern".to_string(),
                    required: true,
                },
                ParameterDocumentation {
                    name: "directory".to_string(),
                    type_: "string".to_string(),
                    description: "Directory to search".to_string(),
                    required: true,
                },
                ParameterDocumentation {
                    name: "extensions".to_string(),
                    type_: "array".to_string(),
                    description: "File extensions to filter".to_string(),
                    required: false,
                },
                ParameterDocumentation {
                    name: "case_sensitive".to_string(),
                    type_: "boolean".to_string(),
                    description: "Case sensitive search".to_string(),
                    required: false,
                },
            ],
            examples: vec![
                r#"{"pattern": "fn main", "directory": "src"}"#.to_string(),
                r#"{"pattern": "struct", "directory": "src", "extensions": ["rs"], "case_sensitive": false}"#.to_string(),
            ],
        },
    );

    commands.insert(
        "glob".to_string(),
        CommandDocumentation {
            description:
                "Advanced glob discovery with include/exclude, extension filters, depth bounds, sorting, and limits"
                    .to_string(),
            parameters: vec![
                ParameterDocumentation {
                    name: "pattern".to_string(),
                    type_: "string".to_string(),
                    description: "Primary glob include pattern".to_string(),
                    required: false,
                },
                ParameterDocumentation {
                    name: "path".to_string(),
                    type_: "string".to_string(),
                    description: "Root directory to search".to_string(),
                    required: false,
                },
                ParameterDocumentation {
                    name: "include/exclude/extensions".to_string(),
                    type_: "array".to_string(),
                    description: "Composable filters for one-pass discovery".to_string(),
                    required: false,
                },
                ParameterDocumentation {
                    name: "max_depth/limit/sort_by/sort_order".to_string(),
                    type_: "mixed".to_string(),
                    description: "Bounded deterministic traversal and output ordering"
                        .to_string(),
                    required: false,
                },
            ],
            examples: vec![
                r#"{"pattern":"**/*.rs","exclude":["target/**"],"limit":100}"#.to_string(),
                r#"{"path":"src","extensions":[".rs"],"max_depth":2,"sort_by":"name","sort_order":"asc"}"#.to_string(),
            ],
        },
    );

    // Search with context tool
    commands.insert(
        "search-with-context".to_string(),
        CommandDocumentation {
            description: "Search with context lines".to_string(),
            parameters: vec![
                ParameterDocumentation {
                    name: "pattern".to_string(),
                    type_: "string".to_string(),
                    description: "Search pattern".to_string(),
                    required: true,
                },
                ParameterDocumentation {
                    name: "files".to_string(),
                    type_: "array".to_string(),
                    description: "List of files to search".to_string(),
                    required: true,
                },
                ParameterDocumentation {
                    name: "context".to_string(),
                    type_: "integer".to_string(),
                    description: "Number of context lines before and after".to_string(),
                    required: false,
                },
                ParameterDocumentation {
                    name: "case_sensitive".to_string(),
                    type_: "boolean".to_string(),
                    description: "Case sensitive search".to_string(),
                    required: false,
                },
            ],
            examples: vec![
                r#"{"pattern": "fn main", "files": ["src/main.rs"], "context": 2}"#.to_string(),
            ],
        },
    );

    // Search by regex tool
    commands.insert(
        "search-by-regex".to_string(),
        CommandDocumentation {
            description: "Search using regular expressions".to_string(),
            parameters: vec![
                ParameterDocumentation {
                    name: "pattern".to_string(),
                    type_: "string".to_string(),
                    description: "Regular expression pattern".to_string(),
                    required: true,
                },
                ParameterDocumentation {
                    name: "files".to_string(),
                    type_: "array".to_string(),
                    description: "List of files to search".to_string(),
                    required: true,
                },
                ParameterDocumentation {
                    name: "flags".to_string(),
                    type_: "string".to_string(),
                    description: "Regex flags (e.g., 'i' for case-insensitive, 'm' for multiline)"
                        .to_string(),
                    required: false,
                },
            ],
            examples: vec![
                r#"{"pattern": "fn\\s+\\w+", "files": ["src/**/*.rs"]}"#.to_string(),
                r#"{"pattern": "struct\\s+\\w+", "files": ["src/**/*.rs"], "flags": "i"}"#
                    .to_string(),
            ],
        },
    );

    commands.insert(
        "ask".to_string(),
        CommandDocumentation {
            description:
                "Answer a natural-language codebase question with evidence (neural-first, lexical fallback)"
                    .to_string(),
            parameters: vec![
                ParameterDocumentation {
                    name: "question".to_string(),
                    type_: "string".to_string(),
                    description: "Natural-language question about the codebase".to_string(),
                    required: true,
                },
                ParameterDocumentation {
                    name: "retrieval_mode".to_string(),
                    type_: "string".to_string(),
                    description: "Retrieval mode (neural or lexical; default neural)".to_string(),
                    required: false,
                },
                ParameterDocumentation {
                    name: "ai_mode/budget_profile/prompt_version".to_string(),
                    type_: "mixed".to_string(),
                    description:
                        "AI governance controls for deterministic route and prompt budgeting"
                            .to_string(),
                    required: false,
                },
                ParameterDocumentation {
                    name: "include/exclude/context/limit".to_string(),
                    type_: "mixed".to_string(),
                    description: "Scope and bound evidence snippets".to_string(),
                    required: false,
                },
            ],
            examples: vec![
                r#"{"question":"where is rpc query handled?","retrieval_mode":"neural","include":["src/**/*.rs"],"limit":8}"#.to_string(),
                r#"{"question":"how is policy_denied returned","retrieval_mode":"lexical","limit":10}"#.to_string(),
            ],
        },
    );

    for alias in BOOTSTRAP_TOOL_ALIASES {
        let doc = if alias == "bootstrap_skill" {
            CommandDocumentation {
                description: "Canonical MCP bootstrap method for Flashgrep skill injection"
                    .to_string(),
                parameters: vec![
                    ParameterDocumentation {
                        name: "trigger".to_string(),
                        type_: "string".to_string(),
                        description:
                            "Optional trigger alias (flashgrep-init, fgrep-boot, flashgrep_init, fgrep_boot)"
                                .to_string(),
                        required: false,
                    },
                    ParameterDocumentation {
                        name: "compact".to_string(),
                        type_: "boolean".to_string(),
                        description: "Return compact policy payload without full skill markdown"
                            .to_string(),
                        required: false,
                    },
                    ParameterDocumentation {
                        name: "force".to_string(),
                        type_: "boolean".to_string(),
                        description: "Force reinjection even when already injected in session"
                            .to_string(),
                        required: false,
                    },
                    ParameterDocumentation {
                        name: "allow_repo_override".to_string(),
                        type_: "boolean".to_string(),
                        description:
                            "Optional opt-in to load skills/SKILL.md from repository; default is embedded payload"
                                .to_string(),
                        required: false,
                    },
                    ParameterDocumentation {
                        name: "repo_override_path".to_string(),
                        type_: "string".to_string(),
                        description:
                            "Optional custom path for repo skill override when allow_repo_override is true"
                                .to_string(),
                        required: false,
                    },
                ],
                examples: vec![
                    r#"{"trigger": "flashgrep-init", "compact": true}"#.to_string(),
                    r#"{"force": true}"#.to_string(),
                    r#"{"allow_repo_override": true}"#.to_string(),
                ],
            }
        } else {
            CommandDocumentation {
                description: format!("Alias for {}", BOOTSTRAP_TOOL_ALIASES[0]),
                parameters: vec![ParameterDocumentation {
                    name: "compact".to_string(),
                    type_: "boolean".to_string(),
                    description: "Return compact policy payload without full skill markdown"
                        .to_string(),
                    required: false,
                }],
                examples: vec![r#"{}"#.to_string()],
            }
        };

        commands.insert(alias.to_string(), doc);
    }

    SkillDocumentation {
        overview: "Flashgrep MCP bootstrap injects HYBRID policy directives (compact DSL + strict guard rules) to enforce agent behavior. Discovery is neural-first, lexical fallback is deterministic, and native tools are blocked unless a typed fallback gate is active. For CLI-native natural-language Q&A, prefer the `ask` command with neural-first retrieval and lexical fallback.".to_string(),
        commands,
    }
}

pub fn bootstrap_policy() -> Vec<String> {
    vec![
        "FORMAT v1 HYBRID_ENFORCED".to_string(),
        "ENFORCE bootstrap_required=true reason_code_required=true".to_string(),
        "ROUTE discovery primary=ask(neural) fallback=ask(lexical)".to_string(),
        "ROUTE nl_discovery mcp=ask(neural->lexical) legacy=query(neural->lexical) cli=ask(neural->lexical)".to_string(),
        "ROUTE ai_scopes discovery|synthesis|planning explicit_mode_required=true".to_string(),
        "ROUTE files primary=glob|files symbols=get_symbol reads=read_code writes=write_code|batch_write_code".to_string(),
        "RULE native_tools_banned=true unless=fallback_gate_active".to_string(),
        "RULE prompt_policy_checks=pre_execution typed_denial=policy_denied".to_string(),
        "RULE budget_profiles=fast|balanced|deep token_budget_enforced=true".to_string(),
        "RULE no_guessing=true empty_results_valid=true".to_string(),
        "FALLBACK neural_mode_disabled neural_provider_failure neural_no_relevant_matches".to_string(),
        "FALLBACK exact_match_required query_parse_constraints flashgrep_index_unavailable".to_string(),
        "FALLBACK flashgrep_operation_not_supported flashgrep_tool_runtime_failure repo_override_unavailable".to_string(),
        "WORKFLOW discovery ask(neural)->ask(lexical_on_fail_or_no_match)->get_symbol->read_code".to_string(),
        "WORKFLOW ask_nl cli:ask(neural)->ask(lexical_on_no_match) mcp:ask(neural)->ask(lexical_on_no_match)".to_string(),
        "WORKFLOW edit read_code->write_code(precondition)->read_code".to_string(),
        "WORKFLOW batch_edit read_code->batch_write_code(mode+precondition)->read_code".to_string(),
        "WORKFLOW recovery bootstrap(force=true,compact=true)->verify(policy_metadata)->resume(route_order)".to_string(),
    ]
}

pub fn bootstrap_policy_metadata() -> Value {
    json!({
        "policy_version": "1.1",
        "policy_strength": "strict",
        "enforcement_mode": "strict",
        "search_routing": {
            "default_strategy": "neural_first",
            "discovery_order": ["neural_assisted", "lexical"],
            "nl_discovery": {
                "mcp_primary": "ask:neural",
                "mcp_fallback": "ask:lexical",
                "legacy_query_primary": "query:neural",
                "legacy_query_fallback": "query:lexical",
                "cli_primary": "ask:neural",
                "cli_fallback": "ask:lexical"
            },
            "programmatic_priority": "fallback",
            "routing_mode": "hybrid_enforced",
            "fallback_required": true,
            "ai_mode_required_for_neural": true,
            "fallback_reason_codes": [
                "neural_mode_disabled",
                "neural_provider_failure",
                "neural_no_relevant_matches",
                "exact_match_required",
                "query_parse_constraints",
                "flashgrep_index_unavailable",
                "flashgrep_operation_not_supported",
                "flashgrep_tool_runtime_failure",
                "repo_override_unavailable"
            ]
        },
        "preferred_tool_families": {
            "query": ["ask", "query"],
            "files_glob": ["files", "glob"],
            "symbol": ["symbol", "get_symbol"],
            "read": ["read_code", "get_slice"],
            "write": ["write_code", "batch_write_code"]
        },
        "preferred_tools": {
            "search": ["ask", "query", "files", "glob", "symbol", "get_symbol"],
            "read": ["read_code", "get_slice"],
            "write": ["write_code", "batch_write_code"]
        },
        "fallback_gate_ids": [
            "neural_mode_disabled",
            "neural_provider_failure",
            "neural_no_relevant_matches",
            "exact_match_required",
            "query_parse_constraints",
            "index_unavailable",
            "unsupported_operation",
            "tool_runtime_failure",
            "repo_override_read_failed"
        ],
        "agent_enforcement": {
            "mode": "strict_hybrid",
            "bootstrap_required": true,
            "require_reason_code_on_fallback": true,
            "block_native_tools_without_gate": true,
            "prompt_policy_precheck": true,
            "supported_prompt_versions": ["1.0"],
            "discovery_route_sequence": ["ask:neural", "ask:lexical", "query:neural", "query:lexical"],
            "no_guessing": true
        },
        "ai_budget_profiles": {
            "fast": {
                "system_pct": 15,
                "context_pct": 45,
                "memory_pct": 10,
                "response_pct": 30
            },
            "balanced": {
                "system_pct": 20,
                "context_pct": 50,
                "memory_pct": 20,
                "response_pct": 10
            },
            "deep": {
                "system_pct": 20,
                "context_pct": 60,
                "memory_pct": 10,
                "response_pct": 10
            }
        },
        "prompt_governance": {
            "prompt_id": "flashgrep-core",
            "default_prompt_version": "1.0",
            "typed_denial_error": "policy_denied",
            "required_fields": ["prompt_id", "prompt_version", "prompt_hash", "policy_rule_hits"]
        },
        "fallback_rules": [
            {
                "gate_id": "neural_mode_disabled",
                "condition": "neural_mode_not_enabled_or_not_configured",
                "allowed_tools": ["ask", "query"],
                "reason_code": "neural_mode_disabled"
            },
            {
                "gate_id": "neural_provider_failure",
                "condition": "neural_provider_request_failed_or_timed_out",
                "allowed_tools": ["ask", "query"],
                "reason_code": "neural_provider_failure"
            },
            {
                "gate_id": "neural_no_relevant_matches",
                "condition": "neural_rerank_returns_no_relevant_candidates",
                "allowed_tools": ["ask", "query", "get_symbol"],
                "reason_code": "neural_no_relevant_matches"
            },
            {
                "gate_id": "exact_match_required",
                "condition": "request_requires_literal_or_regex_exactness",
                "allowed_tools": ["ask", "query", "symbol", "get_symbol"],
                "reason_code": "exact_match_required"
            },
            {
                "gate_id": "query_parse_constraints",
                "condition": "query_parse_or_syntax_constraints_prevent_primary_path",
                "allowed_tools": ["ask", "query"],
                "reason_code": "query_parse_constraints"
            },
            {
                "gate_id": "index_unavailable",
                "condition": "index_not_found_or_unreadable",
                "allowed_tools": ["search", "search-in-directory", "search-with-context", "search-by-regex"],
                "reason_code": "flashgrep_index_unavailable"
            },
            {
                "gate_id": "unsupported_operation",
                "condition": "flashgrep_tool_contract_missing_required_operation",
                "allowed_tools": ["search", "search-in-directory", "search-by-regex"],
                "reason_code": "flashgrep_operation_not_supported"
            },
            {
                "gate_id": "tool_runtime_failure",
                "condition": "flashgrep_tool_returns_error_after_valid_retry",
                "allowed_tools": ["search", "search-in-directory", "search-with-context", "search-by-regex"],
                "reason_code": "flashgrep_tool_runtime_failure"
            },
            {
                "gate_id": "repo_override_read_failed",
                "condition": "repository_skill_override_requested_but_not_readable",
                "allowed_tools": ["bootstrap_skill", "flashgrep-init", "fgrep-boot", "flashgrep_init", "fgrep_boot"],
                "reason_code": "repo_override_unavailable"
            }
        ],
        "compliance_checks": {
            "requires_bootstrap_injected": true,
            "requires_gated_fallback_reason": true,
            "requires_payload_source_metadata": true,
            "recommended_preferred_tool_hit_rate": ">=0.9"
        },
        "prohibited_native_tools": {
            "search": ["grep", "rg", "find", "Grep"],
            "discovery": ["Glob", "shell_glob_expansion"],
            "file_io": ["Read", "Write", "cat", "sed", "awk"]
        }
    })
}
