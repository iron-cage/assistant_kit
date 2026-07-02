# CLI Parameter Group: Claude-Native Flags

**Pattern:** Forwarded as-is to the `claude` subprocess via `ClaudeCommand` builder calls; runner does not interpret or transform these values.

**Purpose:** Pass selected `claude` binary flags through without runner modification.
**Order:** 1

Session continuation (`-c`) is applied automatically and is not exposed as a user flag.
Use `--new-session` (Runner Control) to disable it.

`--dangerously-skip-permissions` is injected automatically by `clr` (default-on).
Use `--no-skip-permissions` (Runner Control) to disable the automatic bypass.

`--effort max` is injected automatically by `clr` (default-on). Use `--effort <level>`
to override or `--no-effort-max` (Runner Control) to suppress entirely.

### Semantic Coherence Test

"Is this flag consumed by the claude subprocess?" — YES for all 13.

### Why NOT X

- `--dir`: sets runner working directory, not a claude flag
- `--max-tokens`: set via env var by runner, not a claude CLI flag
- `--dry-run`: prevents execution entirely, runner-only concern
- `--new-session`: controls runner session behavior, not forwarded to claude
- `--no-skip-permissions`: controls whether runner injects `--dangerously-skip-permissions`; consumed by runner, not forwarded to claude
- `--no-effort-max`: controls whether runner injects `--effort max`; consumed by runner, not forwarded to claude
- `--no-chrome`: controls whether runner injects `--chrome`; consumed by runner, not forwarded to claude
- `--no-persist`: controls whether runner injects `--no-session-persistence`; consumed by runner, not forwarded to claude

### Invariants

All parameters are forwarded to the subprocess as-is. The runner applies no transformation to their values.

### Notes

—

### Typical Patterns

```sh
clr -p "Fix bug" --model sonnet --verbose
```

### Referenced Commands

| # | Command | Membership | Excluded Params | Notes |
|---|---------|------------|-----------------|-------|
| 1 | [`run`](../command/01_run.md) | Full | — | All 13 params apply; default command |
| 5 | [`ask`](../command/05_ask.md) | Full | — | All 13 params apply; only defaults differ |

### Referenced Parameters

| Parameter | Type | Default | Role in Group | Description |
|-----------|------|---------|---------------|-------------|
| [`-p`/`--print`](../param/002_print.md) | bool | auto | Print mode selector | Print mode (default when message given) |
| [`--model`](../param/003_model.md) | [`ModelName`](../type/04_model_name.md) | — | Model selection | Model to use |
| [`--verbose`](../param/004_verbose.md) | bool | false | Verbosity toggle | Enable Claude verbose output |
| [`--effort`](../param/017_effort.md) | [`EffortLevel`](../type/07_effort_level.md) | max | Effort override | Reasoning effort level (default: max) |
| [`--json-schema`](../param/023_json_schema.md) | [`JsonSchemaText`](../type/10_json_schema_text.md) | — | Output structure constraint | JSON Schema for structured output |
| [`--mcp-config`](../param/024_mcp_config.md) | [`McpConfigPath`](../type/11_mcp_config_path.md) | — | Tool server config | MCP server config (repeatable) |
| [`--output-format`](../param/061_output_format.md) | enum | — | Output format selector | Output format (`text`/`json`/`stream-json`) |
| [`--max-turns`](../param/062_max_turns.md) | u32 | — | Turn limiter | Max agentic turns |
| [`--allowed-tools`](../param/063_allowed_tools.md) | string | — | Tool whitelist | Restrict to specified tools |
| [`--disallowed-tools`](../param/064_disallowed_tools.md) | string | — | Tool blacklist | Block specified tools |
| [`--max-budget-usd`](../param/065_max_budget_usd.md) | f64 | — | Budget cap | Max dollar budget for session |
| [`--add-dir`](../param/066_add_dir.md) | path | — | Directory expansion | Additional directory to access |
| [`--fallback-model`](../param/067_fallback_model.md) | string | — | Model fallback | Fallback model when primary unavailable |

### Referenced Tests

| # | Test Spec | Scope |
|---|-----------|-------|
| 1 | [01_claude_native_flags.md](../../../tests/docs/cli/param_group/01_claude_native_flags.md) | Claude-Native Flags group behavior |

### Referenced User Stories

| # | User Story | Persona |
|---|------------|---------|
| 2 | [002_print_mode_capture.md](../user_story/002_print_mode_capture.md) | Developer |
| 12 | [012_code_block_extraction.md](../user_story/012_code_block_extraction.md) | Developer |
| 13 | [013_structured_json_pipeline.md](../user_story/013_structured_json_pipeline.md) | Developer |
| 15 | [015_ask_mode.md](../user_story/015_ask_mode.md) | Developer |
| 17 | [017_model_selection.md](../user_story/017_model_selection.md) | Developer |
| 18 | [018_env_var_configuration.md](../user_story/018_env_var_configuration.md) | Developer |
| 19 | [019_mcp_config_injection.md](../user_story/019_mcp_config_injection.md) | Developer |
