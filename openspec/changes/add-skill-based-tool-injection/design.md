## Context

Flashgrep currently operates as a standalone tool where users must manually invoke specific tools (query, files, symbol, read_code, write_code) for each operation. Modern AI agents like OpenCode, Claude Code, and Codex support a skill-based approach where tools can inject themselves into the agent with contextual guidance on when to use them.

The mgrep tool demonstrates this pattern effectively - it installs itself as an MCP tool and injects a skill document that tells the AI:
- When to use mgrep ("Whenever you need to search your local files")
- How to use it (with concrete examples)
- What not to do (vague queries, unnecessary filters)
- Keywords for context matching

Flashgrep needs to adopt this pattern to provide a more seamless user experience where the AI automatically selects the appropriate flashgrep tool based on context.

## Goals / Non-Goals

**Goals:**
- Provide CLI commands to install flashgrep in supported AI agents (OpenCode, Claude Code, Codex)
- Generate skill definitions that describe when and how to use each flashgrep tool
- Register flashgrep as an MCP server in agent configurations
- Support uninstallation to clean up agent configurations
- Enable AI agents to automatically prefer flashgrep tools over generic alternatives
- Ensure skill definitions include concrete examples of tool usage

**Non-Goals:**
- Support for AI agents other than OpenCode, Claude Code, and Codex (can be added later)
- Automatic installation on flashgrep first run (explicit install command required)
- Modification of agent core behavior (only configuration changes)
- Runtime skill injection (skills are loaded at agent startup)
- Support for non-MCP based agent integration

## Decisions

**Decision 1: Skill Document Format**
- **Choice**: Use the standard skill markdown format with frontmatter (name, description, license) and sections for usage guidance, examples, and keywords
- **Rationale**: This format is widely supported across AI agents and provides clear structure for both humans and machines
- **Alternative**: JSON-based skill definitions - rejected as less readable for human editing

**Decision 2: Per-Agent Installation Strategy**
- **Choice**: Create separate install commands for each agent platform (install-opencode, install-claude, install-codex) that handle platform-specific configuration
- **Rationale**: Each agent has different configuration mechanisms and paths
- **Alternative**: Single generic install command - rejected as it would be complex and fragile

**Decision 3: MCP Server Registration**
- **Choice**: Register flashgrep as an MCP server with command `flashgrep mcp` that implements the MCP protocol
- **Rationale**: Standard MCP protocol allows seamless integration with any MCP-compatible agent
- **Alternative**: Direct tool registration - rejected as MCP is the emerging standard

**Decision 4: Skill Content Structure**
- **Choice**: One comprehensive skill document covering all flashgrep tools (query, files, symbol, read_code, write_code) with sections for each
- **Rationale**: Users typically need multiple flashgrep tools in a session; having them in one skill document provides better context
- **Alternative**: Separate skill per tool - rejected as would fragment guidance

**Decision 5: Bootstrap Enhancement**
- **Choice**: Enhance the existing bootstrap mechanism to include skill-based tool priority policies alongside existing policy metadata
- **Rationale**: Keeps tool selection logic in one place; agents that understand skills can use the enhanced guidance
- **Alternative**: Separate skill injection endpoint - rejected as would complicate agent integration

## Risks / Trade-offs

**Risk 1: Agent Configuration Conflicts**
- **Concern**: Installing flashgrep might conflict with existing tool configurations in the agent
- **Mitigation**: Check for existing configurations before modification, provide backup option, support clean uninstallation

**Risk 2: Agent Version Compatibility**
- **Concern**: Different versions of agents may have different configuration formats
- **Mitigation**: Detect agent version and adapt configuration format accordingly; document minimum supported versions

**Risk 3: Permission Issues**
- **Concern**: Writing to agent configuration directories may require elevated permissions
- **Mitigation**: Clear error messages when permissions are denied; suggest running with appropriate permissions; support user-specified config paths

**Risk 4: Skill Override Conflicts**
- **Concern**: User may have custom skills that conflict with flashgrep's skill guidance
- **Mitigation**: Append flashgrep skill rather than replace; include unique identifiers so users can manually manage; document how to customize

**Risk 5: Over-reliance on Flashgrep**
- **Concern**: AI might use flashgrep for inappropriate tasks where native tools are better
- **Mitigation**: Skill document includes clear "don't" examples; bootstrap includes fallback gating rules for edge cases

## Migration Plan

**Phase 1: Core Infrastructure**
1. Implement MCP server endpoint (`flashgrep mcp`)
2. Create skill document template with all tool descriptions
3. Implement OpenCode installer (config file modification)

**Phase 2: Multi-Agent Support**
1. Implement Claude Code installer (plugin marketplace integration)
2. Implement Codex installer (MCP + AGENTS.md injection)
3. Add uninstall commands for all platforms

**Phase 3: Integration & Polish**
1. Enhance bootstrap to include skill-based policies
2. Add comprehensive error handling and user feedback
3. Create documentation and usage examples

**Rollback Strategy:**
- Each install command has a corresponding uninstall command
- Installers create backups of modified config files where possible
- Configuration changes are additive (append, don't replace)
- Users can manually remove flashgrep from agent configs if needed
