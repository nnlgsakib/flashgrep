# Flashgrep MCP Server - AI Agent Skill Guide

## Overview

Flashgrep is a high-performance code search MCP server optimized for large codebases. It provides:

- **Indexed search**: Pre-built indexes for instant full-text and symbol search
- **Symbol navigation**: Jump to definitions for functions, classes, variables
- **Code slicing**: Extract specific line ranges from files
- **Fast queries**: Near-instant search across millions of lines of code

Use flashgrep when you need to:
- Find code patterns, functions, or symbols in large projects
- Navigate to specific code locations
- Analyze code structure and relationships
- Extract code snippets for analysis or modification

## Available Tools

### 1. `flashgrep_query` - Full-text code search

Search the indexed codebase for text patterns with fast fuzzy matching.

**Parameters:**
- `text` (required): Search query text
- `limit` (optional): Maximum results (default: 10)

**Usage:**
```
flashgrep_query:0
{
  "text": "function handleError",
  "limit": 20
}
```

**Returns:** Array of matches with file paths, line numbers, and content.

**Best for:**
- Finding function definitions or calls
- Searching for specific code patterns
- Locating error handling code
- Finding imports and exports
- Searching for TODOs or comments

---

### 2. `flashgrep_get_slice` - Extract code range

Get specific line ranges from a file for detailed analysis.

**Parameters:**
- `file_path` (required): Absolute path to the file
- `start_line` (required): Starting line number (1-indexed)
- `end_line` (required): Ending line number (1-indexed)

**Usage:**
```
flashgrep_get_slice:1
{
  "file_path": "/home/user/project/src/utils.js",
  "start_line": 45,
  "end_line": 60
}
```

**Returns:** Lines of code with line numbers as `45: function name() {`.

**Best for:**
- Reading function implementations
- Viewing class definitions
- Analyzing specific code blocks
- Understanding context around search results

---

### 3. `flashgrep_get_symbol` - Find symbol definition

Locate the definition of a specific function, class, variable, or type.

**Parameters:**
- `symbol_name` (required): Name of the symbol to find

**Usage:**
```
flashgrep_get_symbol:2
{
  "symbol_name": "handleError"
}
```

**Returns:** File path, line number, and content of the symbol definition.

**Best for:**
- Jumping to function definitions
- Finding class declarations
- Locating type definitions
- Understanding where symbols are declared

---

### 4. `flashgrep_list_files` - List indexed files

Get a complete list of all files indexed by flashgrep.

**Parameters:** None

**Usage:**
```
flashgrep_list_files:3
{}
```

**Returns:** Array of all indexed file paths.

**Best for:**
- Understanding project structure
- Verifying files are indexed
- Finding file paths for slicing
- Initial project exploration

---

### 5. `flashgrep_stats` - Index statistics

Get statistics about the indexed codebase.

**Parameters:** None

**Usage:**
```
flashgrep_stats:4
{}
```

**Returns:** Count of indexed files, total lines of code, index size, etc.

**Best for:**
- Understanding codebase scale
- Verifying indexing status
- Before/after comparison when adding files

## Common Search Patterns

### Finding Functions
```
# JavaScript/TypeScript
flashgrep_query:5
{
  "text": "function handleError"
}

# Python
flashgrep_query:6
{
  "text": "def process_data"
}

# Rust
flashgrep_query:7
{
  "text": "fn calculate_total"
}

# Arrow functions / methods
flashgrep_query:8
{
  "text": "const processData = "
}
```

### Finding Classes
```
# JavaScript/TypeScript classes
flashgrep_query:9
{
  "text": "class UserController"
}

# Python classes
flashgrep_query:10
{
  "text": "class DataProcessor"
}

# Rust structs
flashgrep_query:11
{
  "text": "struct Config"
}
```

### Finding Imports/Exports
```
# JavaScript imports
flashgrep_query:12
{
  "text": "import { useState }"
}

# Python imports
flashgrep_query:13
{
  "text": "from flask import Flask"
}

# Exports
flashgrep_query:14
{
  "text": "export default"
}
flashgrep_query:15
{
  "text": "module.exports"
}
```

### Finding Error Handling
```
flashgrep_query:16
{
  "text": "try {"   
}
flashgrep_query:17
{
  "text": "catch (error)"
}
flashgrep_query:18
{
  "text": "throw new Error"
}
```

### Finding Tests
```
flashgrep_query:19
{
  "text": "describe('"
}
flashgrep_query:20
{
  "text": "it('should"
}
flashgrep_query:21
{
  "text": "test('"
}
flashgrep_query:22
{
  "text": "def test_"
}
```

### Finding TODOs and Comments
```
flashgrep_query:23
{
  "text": "TODO:"
}
flashgrep_query:24
{
  "text": "FIXME:"
}
flashgrep_query:25
{
  "text": "NOTE:"
}
```

### Finding Configuration
```
flashgrep_query:26
{
  "text": "module.exports = {"
}
flashgrep_query:27
{
  "text": "export default {"
}
flashgrep_query:28
{
  "text": "const config = {"
}
```

## Example Workflows

### Workflow 1: Understanding a Function

**Goal:** Find and understand the `processPayment` function.

**Steps:**
1. **Find the definition:**
   ```
   flashgrep_get_symbol:29
   {
     "symbol_name": "processPayment"
   }
   ```

2. **Read the full function:**
   (Using file path and line numbers from step 1)
   ```
   flashgrep_get_slice:30
   {
     "file_path": "/home/user/project/src/payment.js",
     "start_line": 45,
     "end_line": 80
   }
   ```

3. **Find where it's called:**
   ```
   flashgrep_query:31
   {
     "text": "processPayment("
   }
   ```

### Workflow 2: Tracing Error Handling

**Goal:** Find all error handling related to database connections.

**Steps:**
1. **Search for database error patterns:**
   ```
   flashgrep_query:32
   {
     "text": "DatabaseError"
   }
   ```

2. **Look for related catch blocks:**
   ```
   flashgrep_query:33
   {
     "text": "catch.*database"
   }
   ```

3. **Read a specific error handler:**
   ```
   flashgrep_get_slice:34
   {
     "file_path": "/home/user/project/src/db.js",
     "start_line": 120,
     "end_line": 140
   }
   ```

### Workflow 3: Finding All API Endpoints

**Goal:** Discover all API endpoints in an Express.js application.

**Steps:**
1. **Search for route definitions:**
   ```
   flashgrep_query:35
   {
     "text": "app.get("
   }
   ```

2. **Find other HTTP methods:**
   ```
   flashgrep_query:36
   {
     "text": "app.post("
   }
   ```

3. **Also check router methods:**
   ```
   flashgrep_query:37
   {
     "text": "router.get("
   }
   ```

### Workflow 4: Analyzing a Class

**Goal:** Understand the `UserService` class structure.

**Steps:**
1. **Find the class definition:**
   ```
   flashgrep_query:38
   {
     "text": "class UserService"
   }
   ```

2. **Get the full class:**
   (Assuming class is ~50 lines)
   ```
   flashgrep_get_slice:39
   {
     "file_path": "/home/user/project/src/services/user.js",
     "start_line": 1,
     "end_line": 100
   }
   ```

3. **Find class methods:**
   ```
   flashgrep_query:40
   {
     "text": "async.*UserService|UserService.prototype"
   }
   ```

### Workflow 5: Project Structure Overview

**Goal:** Get a high-level understanding of a project.

**Steps:**
1. **Check indexing status:**
   ```
   flashgrep_stats:41
   {}
   ```

2. **List all files:**
   ```
   flashgrep_list_files:42
   {}
   ```

3. **Find main entry points:**
   ```
   flashgrep_query:43
   {
     "text": "index.js"
   }
   ```

4. **Find package/configuration files:**
   ```
   flashgrep_query:44
   {
     "text": "package.json"
   }
   ```

## Error Handling

### Common Errors and Solutions

**Error: "File not found" in get_slice**
- **Cause:** File path doesn't exist or isn't indexed
- **Solution:** Use `flashgrep_list_files` to verify the correct path
- **Example:**
  ```
  # Wrong
  "file_path": "src/utils.js"
  
  # Correct
  "file_path": "/home/user/project/src/utils.js"
  ```

**Error: "Symbol not found" in get_symbol**
- **Cause:** Symbol name doesn't exist in the index
- **Solution:** 
  1. Search for partial matches with `flashgrep_query`
  2. Check spelling and case sensitivity
  3. Verify the symbol is defined, not just used

**Error: Empty query results**
- **Cause:** No matches found
- **Solution:**
  1. Try broader search terms
  2. Use partial words instead of full phrases
  3. Check if the codebase is indexed

**Error: Line numbers out of range**
- **Cause:** Requested lines beyond file length
- **Solution:** Use search results to find valid line ranges, then slice

### Defensive Programming

Always verify results before using them:

```javascript
// Good: Check results exist before slicing
const results = await flashgrep_query({ text: "function processData" });
if (results && results.length > 0) {
  const firstMatch = results[0];
  await flashgrep_get_slice({
    file_path: firstMatch.file_path,
    start_line: firstMatch.line_number,
    end_line: firstMatch.line_number + 20
  });
}
```

## Best Practices

### 1. Start Broad, Then Narrow
- Begin with broad searches (`flashgrep_query`)
- Use `get_symbol` for precise navigation
- Extract context with `get_slice`

### 2. Use Absolute Paths
- Always use absolute file paths
- Get paths from search results or `list_files`
- Don't assume relative paths work

### 3. Batch Related Searches
- Run multiple independent queries in parallel
- Combine results for comprehensive analysis

**Example:**
```
# Parallel searches for different HTTP methods
flashgrep_query:45
{ "text": "app.get(" }
flashgrep_query:46
{ "text": "app.post(" }
flashgrep_query:47
{ "text": "app.put(" }
flashgrep_query:48
{ "text": "app.delete(" }
```

### 4. Extract Meaningful Context
- When using `get_slice`, include surrounding lines
- Show 5-10 lines before and after for context
- Don't extract single lines in isolation

**Example:**
```javascript
// Good: Include context around the target
{
  "start_line": targetLine - 5,
  "end_line": targetLine + 15
}

// Bad: Too narrow
{
  "start_line": targetLine,
  "end_line": targetLine
}
```

### 5. Handle Multiple Matches
- Functions may be defined in multiple files
- Classes may have the same name in different modules
- Always check file paths to disambiguate

**Example:**
```javascript
// Results might show:
// /src/api/user.js:45 - function processData
// /src/db/models.js:120 - function processData
// Choose the right one based on context
```

### 6. Use Fuzzy Matching Wisely
- Flashgrep uses fuzzy search
- Results are ranked by relevance
- Review top 5-10 results for best matches

### 7. Verify Before Acting
- Read code before modifying it
- Understand the full context
- Check for dependencies and side effects

### 8. Keep Search Terms Focused
- Avoid overly broad searches (e.g., just "function")
- Use distinctive identifiers
- Include file extensions when relevant

### 9. Document Your Findings
- Note file paths and line numbers
- Quote relevant code snippets
- Explain the purpose of found code

### 10. Index Status Awareness
- Run `flashgrep_stats` periodically
- Be aware of what's indexed
- Note that uncommitted changes may not be indexed

## Quick Reference

| Task | Primary Tool | Example Query |
|------|--------------|---------------|
| Find function definition | `get_symbol` | `{"symbol_name": "processData"}` |
| Find all function calls | `query` | `{"text": "processData("}` |
| Read function body | `get_slice` | `{"file_path": "...", "start_line": 10, "end_line": 50}` |
| Find imports | `query` | `{"text": "import React"}` |
| List all files | `list_files` | `{}` |
| Check index status | `stats` | `{}` |
| Find TODOs | `query` | `{"text": "TODO:"}` |
| Find tests | `query` | `{"text": "it('should"}` |
| Find error handling | `query` | `{"text": "catch (error)"}` |
| Find classes | `query` | `{"text": "class User"}` |

## Tips for Specific Languages

### JavaScript/TypeScript
- Search for `const`, `let`, `var` for variable declarations
- Use `=>` to find arrow functions
- Look for `export`/`import` for module boundaries

### Python
- Search for `def` for functions, `class` for classes
- Use `self.` for instance methods
- Look for `__init__` for constructors

### Rust
- Search for `fn` for functions, `struct`/`enum` for types
- Use `impl` for implementations
- Look for `use` for imports

### Go
- Search for `func` for functions
- Use `type` for type definitions
- Look for `package` for file organization

## Summary

Flashgrep provides fast, indexed code search ideal for:
- **Navigation**: Jump to definitions with `get_symbol`
- **Discovery**: Find patterns with `query`
- **Analysis**: Extract and read code with `get_slice`
- **Exploration**: Understand structure with `list_files` and `stats`

**Remember:** Always use absolute paths, include context when slicing, and verify results before acting on them.
