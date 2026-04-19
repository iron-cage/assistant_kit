# Data Structure: Command Types

### Scope

- **Purpose**: Document the type-safe enum types used to configure Claude Code command parameters.
- **Responsibility**: Specify enum variants, their CLI string mappings, and valid values for all configuration enums in claude_runner_core.
- **In Scope**: ActionMode, LogLevel, OutputFormat, InputFormat, PermissionMode, EffortLevel — variants and CLI string mappings. ExecutionOutput struct fields.
- **Out of Scope**: Builder methods that accept these types (→ `api/`, `pattern/`), execution flow (→ `feature/`).

### Abstract

`src/types.rs` defines the type-safe configuration enums and the `ExecutionOutput` result struct. Enums prevent invalid string values at compile time. Several enums use non-obvious CLI string values (camelCase, hyphens) that must match the Claude CLI exactly.

### Structure

#### `ActionMode`

Controls tool approval behavior. Passed via `with_action_mode()`. Default: `Ask`.

| Variant | CLI / Env Value | Description |
|---------|-----------------|-------------|
| `Ask` | `"ask"` | Prompt user before executing tools |
| `Allow` | `"allow"` | Auto-approve all tool use |
| `Deny` | `"deny"` | Block all tool use |

#### `LogLevel`

Controls logging verbosity. Passed via `with_log_level()`. Default: `Info`.

| Variant | CLI / Env Value | Description |
|---------|-----------------|-------------|
| `Error` | `"error"` | Errors only |
| `Warn` | `"warn"` | Warnings and errors |
| `Info` | `"info"` | Standard information (default) |
| `Debug` | `"debug"` | Debug output |
| `Trace` | `"trace"` | Full trace output |

#### `OutputFormat`

Controls `--output-format` CLI flag. Passed via `with_output_format()`.

| Variant | CLI String | Note |
|---------|-----------|------|
| `Text` | `"text"` | Plain text output |
| `Json` | `"json"` | JSON output |
| `StreamJson` | `"stream-json"` | Streaming JSON (hyphen required — not underscore) |

#### `InputFormat`

Controls `--input-format` CLI flag. Passed via `with_input_format()`.

| Variant | CLI String | Note |
|---------|-----------|------|
| `Text` | `"text"` | Plain text input |
| `StreamJson` | `"stream-json"` | Streaming JSON input (hyphen required) |

#### `PermissionMode`

Controls `--permission-mode` CLI flag. Passed via `with_permission_mode()`. Default: `Default`.

| Variant | CLI String | Note |
|---------|-----------|------|
| `Default` | `"default"` | Standard permission prompting |
| `AcceptEdits` | `"acceptEdits"` | camelCase — required by Claude CLI |
| `BypassPermissions` | `"bypassPermissions"` | camelCase — required by Claude CLI |

The camelCase strings for `AcceptEdits` and `BypassPermissions` are required by the Claude CLI and must not be lowercased.

#### `EffortLevel`

Controls `--effort` CLI flag. Passed via `with_effort()`. Default: `Medium`.

| Variant | CLI String | Note |
|---------|-----------|------|
| `Low` | `"low"` | Lower reasoning effort |
| `Medium` | `"medium"` | Standard effort (default) |
| `High` | `"high"` | Higher reasoning effort |
| `Max` | `"max"` | Maximum effort (not `"maximum"`) |

#### `ExecutionOutput`

Returned by `execute()`. Carries all process result data.

| Field | Type | Description |
|-------|------|-------------|
| `stdout` | `String` | Full captured standard output |
| `stderr` | `String` | Full captured standard error |
| `exit_code` | `i32` | Process exit code (0 = success) |

### Operations

All enums implement `Display` (or equivalent) to produce their CLI string values. The `ExecutionOutput` struct has no methods — it is a plain data carrier.

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| doc | [api/001_execution_api.md](../api/001_execution_api.md) | ExecutionOutput usage in execute() contract |
| doc | [pattern/001_command_builder.md](../pattern/001_command_builder.md) | Builder methods that accept these enum types |
| source | `../../src/types.rs` | Enum and struct definitions |

### Sources

| File | Notes |
|------|-------|
| `spec.md` (deleted — migrated here) | Vocabulary section (ActionMode, LogLevel, OutputFormat, etc.), FR-14 through FR-16, FR-25 through FR-32, Types component description |
