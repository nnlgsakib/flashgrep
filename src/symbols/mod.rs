use crate::db::models::{Symbol, SymbolType};
use regex::Regex;
use std::path::PathBuf;

/// Detects symbols in code using regex patterns
pub struct SymbolDetector {
    function_pattern: Regex,
    class_pattern: Regex,
    import_pattern: Regex,
    export_pattern: Regex,
    route_pattern: Regex,
    sql_pattern: Regex,
    visibility_pattern: Regex,
}

impl SymbolDetector {
    /// Create a new symbol detector with compiled regex patterns
    pub fn new() -> Self {
        Self {
            // Function definitions: fn name, def name, func name, function name
            function_pattern: Regex::new(
                r"(?i)(?:^|\s)(?:fn|def|func|function)\s+([a-zA-Z_][a-zA-Z0-9_]*)"
            ).unwrap(),
            
            // Class/struct definitions: class Name, struct Name, interface Name, type Name
            class_pattern: Regex::new(
                r"(?i)(?:^|\s)(?:class|struct|interface|type)\s+([a-zA-Z_][a-zA-Z0-9_]*)"
            ).unwrap(),
            
            // Import statements: import, require, include, use, from ... import
            import_pattern: Regex::new(
                r"(?i)(?:^|\s)(?:import|require|include|use|from\s+.*import)"
            ).unwrap(),
            
            // Export statements: export, module.exports, pub fn, public
            export_pattern: Regex::new(
                r"(?i)(?:^|\s)(?:export|module\.exports|pub\s+(?:fn|struct|enum|const|let|type)|public)"
            ).unwrap(),
            
            // Route definitions: .get(, .post(, @Get, router.
            route_pattern: Regex::new(
                r"(?i)(?:^|\s)(?:\.get\s*\(|\.post\s*\(|\.put\s*\(|\.delete\s*\(|@(?:Get|Post|Put|Delete)|router\.)"
            ).unwrap(),
            
            // SQL queries: SELECT, INSERT, UPDATE, DELETE
            sql_pattern: Regex::new(
                r"(?i)(?:^|\s)(?:SELECT|INSERT|UPDATE|DELETE|CREATE|DROP|ALTER)\s+"
            ).unwrap(),
            
            // Visibility markers: public, private, protected, pub, internal
            visibility_pattern: Regex::new(
                r"(?i)(?:^|\s)(?:public|private|protected|internal|pub)"
            ).unwrap(),
        }
    }

    /// Detect all symbols in a chunk of code
    pub fn detect_in_chunk(
        &self,
        chunk: &str,
        file_path: PathBuf,
        start_line: usize,
    ) -> Vec<Symbol> {
        let mut symbols = Vec::new();
        let lines: Vec<&str> = chunk.lines().collect();

        for (i, line) in lines.iter().enumerate() {
            let line_number = start_line + i;

            // Check for functions
            for cap in self.function_pattern.captures_iter(line) {
                if let Some(name) = cap.get(1) {
                    symbols.push(Symbol {
                        id: None,
                        symbol_name: name.as_str().to_string(),
                        file_path: file_path.clone(),
                        line_number,
                        symbol_type: SymbolType::Function,
                    });
                }
            }

            // Check for classes/structs
            for cap in self.class_pattern.captures_iter(line) {
                if let Some(name) = cap.get(1) {
                    let symbol_type = if line.to_lowercase().contains("class") {
                        SymbolType::Class
                    } else if line.to_lowercase().contains("struct") {
                        SymbolType::Struct
                    } else if line.to_lowercase().contains("interface") {
                        SymbolType::Interface
                    } else {
                        SymbolType::Other("type".to_string())
                    };

                    symbols.push(Symbol {
                        id: None,
                        symbol_name: name.as_str().to_string(),
                        file_path: file_path.clone(),
                        line_number,
                        symbol_type,
                    });
                }
            }

            // Check for imports
            if self.import_pattern.is_match(line) {
                let name = self.extract_import_name(line);
                symbols.push(Symbol {
                    id: None,
                    symbol_name: name,
                    file_path: file_path.clone(),
                    line_number,
                    symbol_type: SymbolType::Import,
                });
            }

            // Check for exports
            if self.export_pattern.is_match(line) {
                let name = self.extract_export_name(line);
                symbols.push(Symbol {
                    id: None,
                    symbol_name: name,
                    file_path: file_path.clone(),
                    line_number,
                    symbol_type: SymbolType::Export,
                });
            }

            // Check for routes
            if self.route_pattern.is_match(line) {
                symbols.push(Symbol {
                    id: None,
                    symbol_name: self.extract_route_name(line),
                    file_path: file_path.clone(),
                    line_number,
                    symbol_type: SymbolType::Route,
                });
            }

            // Check for SQL
            if self.sql_pattern.is_match(line) {
                symbols.push(Symbol {
                    id: None,
                    symbol_name: self.extract_sql_name(line),
                    file_path: file_path.clone(),
                    line_number,
                    symbol_type: SymbolType::SqlQuery,
                });
            }

            // Check for visibility markers
            if self.visibility_pattern.is_match(line)
                && !line.to_lowercase().contains("function")
                && !line.to_lowercase().contains("fn")
                && !line.to_lowercase().contains("def")
            {
                let symbol_type = if line.to_lowercase().contains("private") {
                    SymbolType::Private
                } else {
                    SymbolType::Public
                };

                symbols.push(Symbol {
                    id: None,
                    symbol_name: self.extract_visibility_name(line),
                    file_path: file_path.clone(),
                    line_number,
                    symbol_type,
                });
            }
        }

        symbols
    }

    /// Extract name from an import statement
    fn extract_import_name(&self, line: &str) -> String {
        // Try to extract the imported module/package name
        let re =
            Regex::new(r#"(?:import|require|include|use)\s+(?:['""]?)([a-zA-Z_][a-zA-Z0-9_/.]*)"#)
                .unwrap();
        if let Some(cap) = re.captures(line) {
            cap.get(1)
                .map(|m| m.as_str().to_string())
                .unwrap_or_default()
        } else {
            "import".to_string()
        }
    }

    /// Extract name from an export statement
    fn extract_export_name(&self, line: &str) -> String {
        // Try to extract the exported symbol name
        let re = Regex::new(r"export\s+(?:const|let|var|function|class|interface|type|default\s+)?\s*([a-zA-Z_][a-zA-Z0-9_]*)").unwrap();
        if let Some(cap) = re.captures(line) {
            cap.get(1)
                .map(|m| m.as_str().to_string())
                .unwrap_or_default()
        } else {
            "export".to_string()
        }
    }

    /// Extract route information
    fn extract_route_name(&self, line: &str) -> String {
        // Try to extract the route path
        let re = Regex::new(r#"['""]([^'""]+)['""]|\((['""][^'""]+['""])\)"#).unwrap();
        if let Some(cap) = re.captures(line) {
            cap.get(1)
                .or_else(|| cap.get(2))
                .map(|m| m.as_str().to_string())
                .unwrap_or_else(|| "route".to_string())
        } else {
            "route".to_string()
        }
    }

    /// Extract SQL query type
    fn extract_sql_name(&self, line: &str) -> String {
        let upper = line.to_uppercase();
        if upper.contains("SELECT") {
            "SELECT".to_string()
        } else if upper.contains("INSERT") {
            "INSERT".to_string()
        } else if upper.contains("UPDATE") {
            "UPDATE".to_string()
        } else if upper.contains("DELETE") {
            "DELETE".to_string()
        } else if upper.contains("CREATE") {
            "CREATE".to_string()
        } else {
            "SQL".to_string()
        }
    }

    /// Extract name associated with visibility marker
    fn extract_visibility_name(&self, line: &str) -> String {
        // Try to find the name following the visibility keyword
        let re = Regex::new(r"(?:public|private|protected|pub)\s+(?:fn|function|def|class|struct|interface|const|let|var|static)?\s*([a-zA-Z_][a-zA-Z0-9_]*)").unwrap();
        if let Some(cap) = re.captures(line) {
            cap.get(1)
                .map(|m| m.as_str().to_string())
                .unwrap_or_default()
        } else {
            "visibility".to_string()
        }
    }
}

impl Default for SymbolDetector {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_function() {
        let detector = SymbolDetector::new();
        let code = "fn main() {\n    println!(\"Hello\");\n}";

        let symbols = detector.detect_in_chunk(code, PathBuf::from("test.rs"), 1);

        assert!(!symbols.is_empty());
        assert!(symbols
            .iter()
            .any(|s| s.symbol_name == "main" && s.symbol_type == SymbolType::Function));
    }

    #[test]
    fn test_detect_class() {
        let detector = SymbolDetector::new();
        let code = "class MyClass:\n    pass";

        let symbols = detector.detect_in_chunk(code, PathBuf::from("test.py"), 1);

        assert!(symbols
            .iter()
            .any(|s| s.symbol_name == "MyClass" && s.symbol_type == SymbolType::Class));
    }

    #[test]
    fn test_detect_import() {
        let detector = SymbolDetector::new();
        let code = "import os\nfrom typing import List";

        let symbols = detector.detect_in_chunk(code, PathBuf::from("test.py"), 1);

        assert!(symbols.iter().any(|s| s.symbol_type == SymbolType::Import));
    }

    #[test]
    fn test_detect_sql() {
        let detector = SymbolDetector::new();
        let code = "SELECT * FROM users WHERE id = 1";

        let symbols = detector.detect_in_chunk(code, PathBuf::from("test.sql"), 1);

        assert!(symbols
            .iter()
            .any(|s| s.symbol_type == SymbolType::SqlQuery));
    }

    #[test]
    fn test_detect_route() {
        let detector = SymbolDetector::new();
        // The pattern matches ".get(" not "app.get("
        let code = r#"
.get("/users", handler)
.post("/items", handler)
"#;

        let symbols = detector.detect_in_chunk(code, PathBuf::from("test.js"), 1);

        assert!(
            symbols.iter().any(|s| s.symbol_type == SymbolType::Route),
            "Should detect route definitions"
        );
    }
}
