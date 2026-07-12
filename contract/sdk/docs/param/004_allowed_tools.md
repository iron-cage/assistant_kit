# allowedTools

Allowlist of tool names (including `mcp__server__tool`-form entries) pre-approved without a permission prompt.

### Forms

| | Value |
|-|-------|
| TS Field | `allowedTools?: string[]` |
| Python Field | `allowed_tools` (confirmed via official quickstart: `allowed_tools=["Read", "Edit", "Bash"]`) |
| CLI Equivalent | `--allowed-tools` — [`../../../claude_code/docs/param/006_allowed_tools.md`](../../../claude_code/docs/param/006_allowed_tools.md) |

### Type

string array

### Default

`[]`

### Since

SDK GA

### Description

Field-for-field equivalent of the CLI's `--allowed-tools` flag. Entries are either built-in tool names (`"Read"`, `"Bash"`, `"Glob"`, ...) or `mcp__{server}__{tool}`-form entries addressing SDK-registered or external MCP tools (see [S5](../behavior/005_s5_mcp_tool_naming.md)). Every official code example that uses custom or MCP tools sets this explicitly — e.g. the Subagents example includes `"Agent"` in `allowedTools` specifically so subagent invocations don't require separate approval.

### Cross-References

| Type | File | Responsibility |
|------|------|-----------------|
| doc | [readme.md](readme.md) | Master curated parameter table |
| doc | [005_disallowed_tools.md](005_disallowed_tools.md) | Denylist counterpart |
| behavior | [../behavior/005_s5_mcp_tool_naming.md](../behavior/005_s5_mcp_tool_naming.md) | `mcp__server__tool` naming this list consumes |
| doc | [../../../claude_code/docs/param/006_allowed_tools.md](../../../claude_code/docs/param/006_allowed_tools.md) | CLI-level equivalent flag |
