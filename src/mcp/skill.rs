//! MCP skill implementation

use serde::{Deserialize, Serialize};
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
            repository: "https://github.com/yourusername/flashgrep".to_string(),
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

    SkillDocumentation {
        overview: "Flashgrep is a high-performance local code indexing engine that supports fast searching with grep-like capabilities via the Model Context Protocol (MCP).".to_string(),
        commands,
    }
}
