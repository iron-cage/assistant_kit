# Forward MCP server configurations for tool access

**Persona:** Developer working with MCP-enabled workflows who needs to load one or more MCP server configurations for Claude Code to access external tools or data sources.
**Goal:** Forward MCP server configuration files to the `claude` subprocess so that MCP tools are available during execution, with support for multiple configs and env var configuration.
**Benefit:** Activates MCP tool access for a session without modifying global configuration.
**Priority:** Medium

### Acceptance Criteria

- `clr --mcp-config path "Task"` forwards `--mcp-config <path>` to the `claude` subprocess
- Multiple `--mcp-config` flags are each forwarded individually as separate `--mcp-config` arguments
- `CLR_MCP_CONFIG=path clr "Task"` applies a single config path when `--mcp-config` is absent
- Default behavior (no flag, no env var) produces no `--mcp-config` in the assembled command

### Referenced Commands

| # | Command | Role |
|---|---------|------|
| 1 | [`run`](../command/01_run.md) | Default command; `--mcp-config` injects MCP server configs |
| 5 | [`ask`](../command/05_ask.md) | Ask command also accepts `--mcp-config` |

### Referenced Parameter Groups

| # | Parameter Group | Role |
|---|-----------------|------|
| 1 | [Claude-Native Flags](../param_group/01_claude_native_flags.md) | `--mcp-config` is a Claude-native flag |

### Referenced Parameters

| # | Parameter | Role |
|---|-----------|------|
| 24 | [`--mcp-config`](../param/024_mcp_config.md) | Path to MCP server config JSON; repeatable |

### Workflow Steps

1. `clr --mcp-config /path/to/server.json "Task"` — forward a single MCP config to the subprocess
2. `clr --mcp-config /path/to/a.json --mcp-config /path/to/b.json "Task"` — forward multiple MCP configs
3. `CLR_MCP_CONFIG=/path/to/server.json clr "Task"` — set MCP config via environment variable

### Related User Stories

| # | User Story | Relationship |
|---|------------|--------------|
| 18 | [Env-var Configuration](018_env_var_configuration.md) | `CLR_MCP_CONFIG` is one of 25 CLR_* env vars |
