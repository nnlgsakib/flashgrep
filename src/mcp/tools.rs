//! MCP tools implementation

use mcp_protocol::types::ToolDefinition;

pub fn create_tools() -> Vec<ToolDefinition> {
    vec![
        create_search_tool(),
        create_search_in_directory_tool(),
        create_search_with_context_tool(),
        create_search_by_regex_tool(),
    ]
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
