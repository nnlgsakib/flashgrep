//! MCP tools implementation

use crate::mcp::bootstrap::BOOTSTRAP_TOOL_ALIASES;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolDefinition {
    pub name: String,
    pub description: String,
    pub parameters: serde_json::Value,
    pub returns: serde_json::Value,
}

pub fn create_tools() -> Vec<ToolDefinition> {
    vec![
        create_glob_tool(),
        create_search_tool(),
        create_search_in_directory_tool(),
        create_search_with_context_tool(),
        create_search_by_regex_tool(),
    ]
}

fn create_glob_tool() -> ToolDefinition {
    ToolDefinition {
        name: "glob".to_string(),
        description: "Advanced glob file discovery with include/exclude filters".to_string(),
        parameters: serde_json::json!({
            "type": "object",
            "properties": {
                "pattern": {"type": "string"},
                "path": {"type": "string"},
                "include": {"type": "array", "items": {"type": "string"}},
                "exclude": {"type": "array", "items": {"type": "string"}},
                "extensions": {"type": "array", "items": {"type": "string"}},
                "max_depth": {"type": "integer", "minimum": 0},
                "recursive": {"type": "boolean"},
                "include_hidden": {"type": "boolean"},
                "follow_symlinks": {"type": "boolean"},
                "case_sensitive": {"type": "boolean"},
                "sort_by": {"type": "string", "enum": ["path", "name", "modified", "size"]},
                "sort_order": {"type": "string", "enum": ["asc", "desc"]},
                "offset": {"type": "integer", "minimum": 0},
                "limit": {"type": "integer", "minimum": 1}
            }
        }),
        returns: serde_json::json!({
            "type": "object",
            "properties": {
                "results": {"type": "array"},
                "total": {"type": "integer"}
            }
        }),
    }
}

pub fn create_bootstrap_tools() -> Vec<ToolDefinition> {
    BOOTSTRAP_TOOL_ALIASES
        .iter()
        .map(|name| create_bootstrap_alias_tool(name))
        .collect()
}

fn create_bootstrap_alias_tool(name: &str) -> ToolDefinition {
    let description = if name == "bootstrap_skill" {
        "Canonical MCP bootstrap method for Flashgrep skill injection"
    } else {
        "Alias of bootstrap_skill"
    };

    ToolDefinition {
        name: (*name).to_string(),
        description: description.to_string(),
        parameters: serde_json::json!({
            "type": "object",
            "properties": {
                "trigger": {
                    "type": "string",
                    "description": "Bootstrap trigger alias. Accepted: flashgrep-init, fgrep-boot"
                },
                "compact": {
                    "type": "boolean",
                    "description": "Return compact policy guidance without full markdown"
                },
                "force": {
                    "type": "boolean",
                    "description": "Force reinjection even if already injected in this session"
                }
            }
        }),
        returns: serde_json::json!({
            "type": "object",
            "properties": {
                "status": { "type": "string" },
                "canonical_trigger": { "type": "string" },
                "skill_hash": { "type": "string" },
                "skill_version": { "type": "string" },
                "policy": { "type": "array", "items": { "type": "string" } },
                "policy_metadata": {
                    "type": "object",
                    "properties": {
                        "policy_version": { "type": "string" },
                        "policy_strength": { "type": "string" },
                        "preferred_tools": { "type": "object" },
                        "fallback_rules": { "type": "array" },
                        "compliance_checks": { "type": "object" }
                    }
                }
            },
            "required": ["status", "canonical_trigger"]
        }),
    }
}

fn create_search_tool() -> ToolDefinition {
    ToolDefinition {
        name: "search".to_string(),
        description: "Perform a basic grep search".to_string(),
        parameters: serde_json::json!({
            "type": "object",
            "properties": {
                "pattern": {
                    "type": "string",
                    "description": "Search pattern"
                },
                "files": {
                    "type": "array",
                    "items": { "type": "string" },
                    "description": "List of files to search"
                },
                "case_sensitive": {
                    "type": "boolean",
                    "description": "Case sensitive search"
                }
            },
            "required": ["pattern", "files"]
        }),
        returns: serde_json::json!({
            "type": "array",
            "items": {
                "type": "object",
                "properties": {
                    "file": { "type": "string" },
                    "line": { "type": "integer" },
                    "content": { "type": "string" }
                },
                "required": ["file", "line", "content"]
            }
        }),
    }
}

fn create_search_in_directory_tool() -> ToolDefinition {
    ToolDefinition {
        name: "search-in-directory".to_string(),
        description: "Search recursively in a directory".to_string(),
        parameters: serde_json::json!({
            "type": "object",
            "properties": {
                "pattern": {
                    "type": "string",
                    "description": "Search pattern"
                },
                "directory": {
                    "type": "string",
                    "description": "Directory to search"
                },
                "extensions": {
                    "type": "array",
                    "items": { "type": "string" },
                    "description": "File extensions to filter (e.g., [\"rs\", \"txt\"])"
                },
                "case_sensitive": {
                    "type": "boolean",
                    "description": "Case sensitive search"
                }
            },
            "required": ["pattern", "directory"]
        }),
        returns: serde_json::json!({
            "type": "array",
            "items": {
                "type": "object",
                "properties": {
                    "file": { "type": "string" },
                    "line": { "type": "integer" },
                    "content": { "type": "string" }
                },
                "required": ["file", "line", "content"]
            }
        }),
    }
}

fn create_search_with_context_tool() -> ToolDefinition {
    ToolDefinition {
        name: "search-with-context".to_string(),
        description: "Search with context lines".to_string(),
        parameters: serde_json::json!({
            "type": "object",
            "properties": {
                "pattern": {
                    "type": "string",
                    "description": "Search pattern"
                },
                "files": {
                    "type": "array",
                    "items": { "type": "string" },
                    "description": "List of files to search"
                },
                "context": {
                    "type": "integer",
                    "description": "Number of context lines before and after"
                },
                "case_sensitive": {
                    "type": "boolean",
                    "description": "Case sensitive search"
                }
            },
            "required": ["pattern", "files"]
        }),
        returns: serde_json::json!({
            "type": "array",
            "items": {
                "type": "object",
                "properties": {
                    "file": { "type": "string" },
                    "line": { "type": "integer" },
                    "content": { "type": "string" },
                    "context": {
                        "type": "object",
                        "properties": {
                            "before": { "type": "array", "items": { "type": "string" } },
                            "after": { "type": "array", "items": { "type": "string" } }
                        }
                    }
                },
                "required": ["file", "line", "content"]
            }
        }),
    }
}

fn create_search_by_regex_tool() -> ToolDefinition {
    ToolDefinition {
        name: "search-by-regex".to_string(),
        description: "Search using regular expressions".to_string(),
        parameters: serde_json::json!({
            "type": "object",
            "properties": {
                "pattern": {
                    "type": "string",
                    "description": "Regular expression pattern"
                },
                "files": {
                    "type": "array",
                    "items": { "type": "string" },
                    "description": "List of files to search"
                },
                "flags": {
                    "type": "string",
                    "description": "Regex flags (e.g., 'i' for case-insensitive, 'm' for multiline)"
                }
            },
            "required": ["pattern", "files"]
        }),
        returns: serde_json::json!({
            "type": "array",
            "items": {
                "type": "object",
                "properties": {
                    "file": { "type": "string" },
                    "line": { "type": "integer" },
                    "content": { "type": "string" }
                },
                "required": ["file", "line", "content"]
            }
        }),
    }
}
