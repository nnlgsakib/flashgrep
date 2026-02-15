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
            version: "0.1.0".to_string(),
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
                ],
                examples: vec![
                    r#"{"trigger": "flashgrep-init", "compact": true}"#.to_string(),
                    r#"{"force": true}"#.to_string(),
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
        overview: "Flashgrep is a high-performance local code indexing engine with native MCP bootstrap support. Use flashgrep-init/fgrep-boot to inject Flashgrep-first guidance and prefer indexed tools (query/files/symbol/read_code/write_code) over generic grep/glob flows.".to_string(),
        commands,
    }
}

pub fn bootstrap_policy() -> Vec<String> {
    vec![
        "Prefer Flashgrep tools before generic grep/glob when searching code.".to_string(),
        "Use query/files/symbol for indexed discovery and navigation.".to_string(),
        "Use read_code with budgets for token-efficient reads.".to_string(),
        "Use write_code for targeted, precondition-safe edits.".to_string(),
    ]
}

pub fn bootstrap_policy_metadata() -> Value {
    json!({
        "policy_version": "1.0",
        "policy_strength": "strict",
        "preferred_tools": {
            "search": ["query", "glob", "files", "get_symbol"],
            "read": ["read_code", "get_slice"],
            "write": ["write_code"]
        },
        "fallback_rules": [
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
            }
        ],
        "compliance_checks": {
            "requires_bootstrap_injected": true,
            "requires_gated_fallback_reason": true,
            "recommended_preferred_tool_hit_rate": ">=0.9"
        }
    })
}
