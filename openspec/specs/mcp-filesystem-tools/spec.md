## Purpose

Define MCP-native filesystem lifecycle operations so agents can perform deterministic file and directory workflows without falling back to host-native tools.

## ADDED Requirements

### Requirement: MCP filesystem lifecycle toolset
The MCP server MUST expose filesystem lifecycle tools for create, read, write, list, stat, copy, move, and remove operations with deterministic machine-readable responses.

#### Scenario: Create file or directory via MCP
- **WHEN** a caller invokes filesystem create with valid target parameters
- **THEN** the server MUST create the target and return structured success metadata including target path and target kind

#### Scenario: List and stat paths via MCP
- **WHEN** a caller invokes filesystem list or stat
- **THEN** the server MUST return deterministic metadata fields for path, type, size, and modification information

### Requirement: Deterministic mutation controls
Mutating MCP filesystem tools MUST implement explicit overwrite, recursive, dry-run, and force controls with deterministic behavior suitable for automation.

#### Scenario: Copy/move conflict without overwrite
- **WHEN** copy or move target exists and overwrite is not enabled
- **THEN** the server MUST return deterministic conflict diagnostics and MUST NOT mutate destination content

#### Scenario: Dry-run mutation returns plan only
- **WHEN** a mutating filesystem tool is called with dry-run enabled
- **THEN** the server MUST return intended operation metadata and MUST NOT change filesystem state

#### Scenario: Remove requires explicit recursive behavior
- **WHEN** remove targets a non-empty directory without recursive enabled
- **THEN** the server MUST return deterministic validation diagnostics and MUST NOT remove the directory

### Requirement: Consistent not-found semantics across filesystem tools
Filesystem MCP tools MUST use the shared typed not-found contract for missing source or target paths.

#### Scenario: Missing source path returns typed not-found
- **WHEN** a filesystem read/copy/move/remove operation references a non-existent source path
- **THEN** the response MUST include typed not-found fields (`error`, `reason_code`, `target_kind`, `target_path`)
