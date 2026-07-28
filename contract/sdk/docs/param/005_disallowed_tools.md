# disallowedTools

Denylist of tool names, supporting `mcp__*`-style wildcard forms.

### Forms

| | Value |
|-|-------|
| TS Field | `disallowedTools?: string[]` |
| Python Field | `disallowed_tools` (inferred snake_case-identical) |
| CLI Equivalent | `--disallowed-tools` — [`../../../claude_code/docs/param/022_disallowed_tools.md`](../../../claude_code/docs/param/022_disallowed_tools.md) |

### Type

string array

### Default

`[]`

### Since

SDK GA

### Description

Field-for-field equivalent of the CLI's `--disallowed-tools` flag, with three documented MCP-specific wildcard forms available in addition to plain tool names: `mcp__server` (all tools from one server), `mcp__server__*` (pattern match, documented as functionally identical to the plain form), `mcp__*` (every MCP tool, SDK-registered or external, from every server). See [S5](../behavior/005_s5_mcp_tool_naming.md) for the full naming convention these wildcards match against.

### Cross-References

| Type | File | Responsibility |
|------|------|-----------------|
| doc | [readme.md](readme.md) | Master curated parameter table |
| doc | [004_allowed_tools.md](004_allowed_tools.md) | Allowlist counterpart |
| behavior | [../behavior/005_s5_mcp_tool_naming.md](../behavior/005_s5_mcp_tool_naming.md) | Naming convention the wildcard forms match against |
| doc | [../../../claude_code/docs/param/022_disallowed_tools.md](../../../claude_code/docs/param/022_disallowed_tools.md) | CLI-level equivalent flag |
