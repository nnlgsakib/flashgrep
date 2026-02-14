## Context

Flashgrep now has a working MCP (Model Context Protocol) server that supports both TCP and stdio transports. AI agents can connect to it and use tools like `query`, `get_slice`, `get_symbol`, `list_files`, and `stats`. However, AI agents don't have comprehensive documentation on:

- What each tool does and when to use it
- How to structure effective search queries
- Common workflows for code analysis
- Best practices for using flashgrep effectively

Without this documentation, AI agents may use flashgrep suboptimally or not know about its capabilities.

## Goals / Non-Goals

**Goals:**
- Create comprehensive SKILL.md documentation for AI agents
- Document all MCP tools with clear use cases and examples
- Provide common search patterns and workflows
- Include troubleshooting and error handling guidance
- Make the skill discoverable in the OpenCode skill system

**Non-Goals:**
- Modify flashgrep MCP server code
- Add new MCP tools
- Create user-facing documentation (this is for AI agents only)
- Support other AI platforms (focus on OpenCode/MCP protocol)

## Decisions

### 1. Documentation Location
**Decision**: Place skill at `.opencode/skills/flashgrep-mcp/SKILL.md`
**Rationale**: OpenCode's skill system looks for markdown files in this directory structure. This makes the skill automatically discoverable.

**Alternatives considered**:
- Root README.md - too user-focused, not AI-agent focused
- docs/ folder - not integrated with OpenCode skill system
- Inline code comments - not easily discoverable by AI

### 2. Documentation Format
**Decision**: Use OpenCode's skill format with clear sections, examples, and patterns
**Rationale**: Structured format helps AI agents parse and understand the content better

### 3. Tool Documentation Approach
**Decision**: Document each tool with:
- Purpose and use cases
- Required and optional parameters
- Example queries for common scenarios
- Expected response format

**Rationale**: AI agents need concrete examples to understand how to use tools effectively

## Risks / Trade-offs

**[Risk] Documentation becomes outdated as MCP evolves**
→ Mitigation: Keep documentation close to code, update with each MCP change

**[Risk] AI agents don't discover or use the skill**
→ Mitigation: Reference the skill in prompts and system messages

**[Trade-off] Comprehensive vs. concise documentation**
→ Balance: Provide enough detail for effective use without overwhelming the AI context window
