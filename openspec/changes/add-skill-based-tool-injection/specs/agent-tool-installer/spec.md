## ADDED Requirements

### Requirement: Install flashgrep in OpenCode
The system SHALL provide a command to install flashgrep as an MCP tool in OpenCode.

#### Scenario: Install in OpenCode with MCP configuration
- **WHEN** user runs `flashgrep install-opencode`
- **THEN** it SHALL create the tool definition file at `~/.config/opencode/tool/flashgrep.ts`
- **AND** it SHALL add MCP server entry to `~/.config/opencode/opencode.json`
- **AND** it SHALL enable the MCP server

#### Scenario: OpenCode already configured
- **WHEN** user runs `flashgrep install-opencode` and OpenCode config already exists
- **THEN** it SHALL append flashgrep configuration to existing config
- **AND** it SHALL not overwrite other MCP servers

### Requirement: Install flashgrep in Claude Code
The system SHALL provide a command to install flashgrep as a plugin in Claude Code.

#### Scenario: Install in Claude Code via marketplace
- **WHEN** user runs `flashgrep install-claude`
- **THEN** it SHALL add flashgrep to Claude Code marketplace
- **AND** it SHALL install the flashgrep plugin
- **AND** it SHALL return success confirmation

#### Scenario: Claude Code not installed
- **WHEN** user runs `flashgrep install-claude` but Claude Code is not found
- **THEN** it SHALL display clear error message
- **AND** it SHALL provide installation instructions

### Requirement: Install flashgrep in Codex
The system SHALL provide a command to install flashgrep in Codex.

#### Scenario: Install in Codex with AGENTS.md
- **WHEN** user runs `flashgrep install-codex`
- **THEN** it SHALL add flashgrep MCP server configuration
- **AND** it SHALL append skill definition to `~/.codex/AGENTS.md`

#### Scenario: AGENTS.md already contains flashgrep skill
- **WHEN** user runs `flashgrep install-codex` and skill already exists
- **THEN** it SHALL skip adding the skill
- **AND** it SHALL report that skill is already installed

### Requirement: Uninstall flashgrep from agents
The system SHALL provide commands to uninstall flashgrep from each supported agent.

#### Scenario: Uninstall from OpenCode
- **WHEN** user runs `flashgrep uninstall-opencode`
- **THEN** it SHALL remove the tool definition file
- **AND** it SHALL remove MCP server entry from config
- **AND** it SHALL clean up empty directories if applicable

#### Scenario: Uninstall from Claude Code
- **WHEN** user runs `flashgrep uninstall-claude`
- **THEN** it SHALL uninstall the flashgrep plugin
- **AND** it SHALL remove from marketplace

#### Scenario: Uninstall from Codex
- **WHEN** user runs `flashgrep uninstall-codex`
- **THEN** it SHALL remove MCP server configuration
- **AND** it SHALL remove skill definition from AGENTS.md

### Requirement: Detect installed agents
The system SHALL detect which AI agents are installed on the system.

#### Scenario: Auto-detect all agents
- **WHEN** user runs `flashgrep install` without specifying agent
- **THEN** it SHALL detect which agents are installed
- **AND** it SHALL install flashgrep in all detected agents
- **AND** it SHALL report which agents were configured

#### Scenario: No agents detected
- **WHEN** user runs `flashgrep install` and no supported agents are found
- **THEN** it SHALL display message listing supported agents
- **AND** it SHALL provide installation instructions for each
