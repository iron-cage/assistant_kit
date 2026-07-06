# Feature: JSON Config Loading

### Scope

- **Purpose**: Document loading clr parameters from a JSON file (`--args-file`) and from stdin JSON pipe for all executing subcommands.
- **Responsibility**: Define the JSON parameter source, precedence rules, JSON key format, stdin detection, and the covering env var `CLR_ARGS_FILE`.
- **In Scope**: `--args-file` flag, `CLR_ARGS_FILE` env var, stdin JSON pipe detection, full parameter coverage, precedence ordering, JSON key-to-parameter mapping, error handling for invalid/missing files.
- **Out of Scope**: Individual parameter semantics (→ `cli/param/`), CLR_* env var semantics, Claude Code subprocess flag forwarding.

### Design

`clr` supports loading any subset of its parameters from a JSON file, enabling repeatable automation configurations without long flag lists on the command line.

**JSON parameter source:** `--args-file <PATH>` reads a JSON object from the specified file. Each key in the object corresponds to a clr parameter name (without the leading `--`) and the value sets that parameter. The file is consumed before normal argument parsing; its contents are injected into the argument list at the position of `--args-file`, preserving left-to-right CLI override semantics. Example config file:

```json
{
  "model": "claude-haiku-4-5-20251001",
  "max-sessions": 5,
  "dry-run": false,
  "system-prompt": "You are a helpful assistant."
}
```

**Stdin JSON pipe:** When standard input is not a TTY and the first non-whitespace byte is `{`, `clr` treats stdin as a JSON parameter source equivalent to `--args-file`. This allows pipeline integration:

```sh
cat fast.json | clr -p "Fix the bug"
echo '{"model":"claude-haiku-4-5-20251001","max-sessions":3}' | clr "Summarize this"
```

Stdin JSON is only consumed when `--file` is not also specified; `--file` takes priority over stdin JSON detection. When stdin is a TTY, no JSON detection occurs.

**Precedence (highest to lowest):**

1. **CLI flags** — explicit `--flag value` on the command line; always win
2. **JSON source** — from `--args-file <PATH>`, `CLR_ARGS_FILE`, or stdin JSON pipe
3. **CLR_* env vars** — the existing per-parameter environment variable fallbacks
4. **Built-in defaults** — hard-coded parameter defaults in `parse.rs`

When both `--args-file` and a CLR_* env var cover the same parameter, the JSON source wins.

**CLR_ARGS_FILE env var:** `CLR_ARGS_FILE=/path/to/config.json` is equivalent to passing `--args-file /path/to/config.json`. The CLI flag takes precedence over `CLR_ARGS_FILE` when both are provided.

**JSON key format:** Keys are the long form of the parameter without the leading `--`. Multi-word flags use hyphens (`max-sessions`, `retry-on-transient`, `system-prompt`). The positional `[MESSAGE]` parameter is set with key `"message"`.

**Boolean flags:** A JSON boolean `true` activates the flag (equivalent to its presence on the command line); `false` is a no-op (same as absence). String booleans (`"true"`, `"1"`) are rejected — only JSON boolean literals are accepted.

**Unknown keys:** Unrecognized JSON keys are ignored without error, allowing forward-compatible config files that reference parameters not yet known to the installed binary.

**Error handling:** A non-existent or unreadable `--args-file` path causes `clr` to exit 1 with a file-not-found error on stderr before any subprocess is spawned. Invalid JSON (malformed, non-object root value) also causes exit 1 with a parse error message on stderr. These errors occur before any CLR_* env var or built-in default resolution.

**Subcommand coverage:** JSON config loading applies to all four executing subcommands: `run`, `ask`, `isolated`, `refresh`. Parameters that are only valid for a specific subcommand are ignored when the active subcommand does not support them (consistent with the existing unknown-flag handling behavior).

**Dry-run inspection:** `--dry-run` combined with `--args-file` prints the merged parameter set (CLI + JSON) in the command preview, making JSON config inspection transparent.

### Acceptance Criteria

| # | Criterion |
|---|-----------|
| AC-001 | `clr --args-file fast.json "task"` loads fast.json and applies its parameters as defaults below CLI flags |
| AC-002 | `echo '{"model":"claude-haiku-4-5-20251001"}' \| clr -p "task"` consumes stdin as JSON param source |
| AC-003 | CLI flag `--model X` wins over `"model": "Y"` in JSON config |
| AC-004 | JSON config `"model"` wins over `CLR_MODEL` env var when both are set |
| AC-005 | `CLR_ARGS_FILE=/path/to/config.json clr "task"` is equivalent to `--args-file /path/to/config.json` |
| AC-006 | Invalid JSON in config file causes exit 1 with parse error on stderr; subprocess not spawned |
| AC-007 | Non-existent `--args-file` path causes exit 1 with file-not-found error on stderr |
| AC-008 | Boolean flag `"dry-run": true` in JSON is treated as flag presence; `"dry-run": false` is ignored |
| AC-009 | Unknown JSON key is silently ignored; no error; other keys are applied normally |
| AC-010 | JSON config applies to `run`, `ask`, `isolated`, and `refresh` subcommands |

### Features

| File | Relationship |
|------|--------------|
| [feature/001_runner_tool.md](001_runner_tool.md) | Parent feature — CLR binary that hosts JSON config loading |

### Parameters

| File | Relationship |
|------|--------------|
| [cli/param/075_args_file.md](../cli/param/075_args_file.md) | `--args-file` flag that activates JSON config loading |

## Parameter Parity Reference

Every clr parameter configurable via JSON config has three synonymous forms: CLI flag, CLR_* env var, and JSON key. The JSON key is always the long-form CLI flag name without the leading `--` (e.g., `--max-sessions` → `"max-sessions"`).

### Key Format Rules

- **Value params**: `{"max-sessions": 5}`, `{"model": "claude-haiku-4-5-20251001"}` — the value must match the CLI parameter type
- **Boolean flags**: `{"dry-run": true}` activates the flag; `{"dry-run": false}` is a no-op (same as omitting the key)
- **Boolean negation flags**: `{"no-ultrathink": true}` suppresses ultrathink suffix; `{"no-ultrathink": false}` is a no-op
- **Positional message**: `{"message": "Fix the bug"}` — the only positional parameter available as a JSON key
- **Repeatable flags** (`--mcp-config`, `--add-dir`): provide a single string value; multiple values require CLI flags
- **Self-referential**: `"args-file"` key in a loaded JSON config is ignored — chaining is not performed

### Complete Parity Table (74 active parameters)

Columns: JSON Key | CLI Flag | CLR_* Env Var | Type | Supported By

**Message and Mode**

| JSON Key | CLI Flag | CLR_* Env Var | Type | Supported By |
|----------|----------|---------------|------|--------------|
| `"message"` | `[MESSAGE]` | `CLR_MESSAGE` | string | run, ask |
| `"print"` | `--print` / `-p` | `CLR_PRINT` | bool | run, ask |
| `"interactive"` | `--interactive` | `CLR_INTERACTIVE` | bool | run, ask |
| `"new-session"` | `--new-session` | `CLR_NEW_SESSION` | bool | run, ask |
| `"dry-run"` | `--dry-run` | `CLR_DRY_RUN` | bool | run, ask |

**Model and Effort**

| JSON Key | CLI Flag | CLR_* Env Var | Type | Supported By |
|----------|----------|---------------|------|--------------|
| `"model"` | `--model` | `CLR_MODEL` | string | run, ask |
| `"effort"` | `--effort` | `CLR_EFFORT` | EffortLevel (low/medium/high/max) | run, ask |
| `"no-effort-max"` | `--no-effort-max` | `CLR_NO_EFFORT_MAX` | bool | run, ask |
| `"fallback-model"` | `--fallback-model` | `CLR_FALLBACK_MODEL` | string | run, ask |
| `"verbose"` | `--verbose` | `CLR_VERBOSE` | bool | run, ask |

**Session and Directory**

| JSON Key | CLI Flag | CLR_* Env Var | Type | Supported By |
|----------|----------|---------------|------|--------------|
| `"session-dir"` | `--session-dir` | `CLR_SESSION_DIR` | string | run, ask |
| `"no-persist"` | `--no-persist` | `CLR_NO_PERSIST` | bool | run, ask |
| `"dir"` | `--dir` | `CLR_DIR` | string | run, ask, isolated |
| `"subdir"` | `--subdir` | `CLR_SUBDIR` | string | run, ask |
| `"add-dir"` | `--add-dir` | `CLR_ADD_DIR` | string | run, ask, isolated |

**Permissions and Chrome**

| JSON Key | CLI Flag | CLR_* Env Var | Type | Supported By |
|----------|----------|---------------|------|--------------|
| `"no-skip-permissions"` | `--no-skip-permissions` | `CLR_NO_SKIP_PERMISSIONS` | bool | run, ask |
| `"no-chrome"` | `--no-chrome` | `CLR_NO_CHROME` | bool | run, ask |
| `"keep-claudecode"` | `--keep-claudecode` | `CLR_KEEP_CLAUDECODE` | bool | run, ask |

**System Prompt**

| JSON Key | CLI Flag | CLR_* Env Var | Type | Supported By |
|----------|----------|---------------|------|--------------|
| `"system-prompt"` | `--system-prompt` | `CLR_SYSTEM_PROMPT` | string | run, ask |
| `"append-system-prompt"` | `--append-system-prompt` | `CLR_APPEND_SYSTEM_PROMPT` | string | run, ask |
| `"no-ultrathink"` | `--no-ultrathink` | `CLR_NO_ULTRATHINK` | bool | run, ask |

**Tools and Input**

| JSON Key | CLI Flag | CLR_* Env Var | Type | Supported By |
|----------|----------|---------------|------|--------------|
| `"json-schema"` | `--json-schema` | `CLR_JSON_SCHEMA` | string | run, ask |
| `"mcp-config"` | `--mcp-config` | `CLR_MCP_CONFIG` | string (single; use CLI for multiple) | run, ask |
| `"file"` | `--file` | `CLR_FILE` | string | run, ask, isolated |
| `"allowed-tools"` | `--allowed-tools` | `CLR_ALLOWED_TOOLS` | string | run, ask |
| `"disallowed-tools"` | `--disallowed-tools` | `CLR_DISALLOWED_TOOLS` | string | run, ask |
| `"max-turns"` | `--max-turns` | `CLR_MAX_TURNS` | string | run, ask |
| `"max-budget-usd"` | `--max-budget-usd` | `CLR_MAX_BUDGET_USD` | string | run, ask |

**Output**

| JSON Key | CLI Flag | CLR_* Env Var | Type | Supported By |
|----------|----------|---------------|------|--------------|
| `"max-tokens"` | `--max-tokens` | `CLR_MAX_TOKENS` | u32 | run, ask |
| `"output-format"` | `--output-format` | `CLR_OUTPUT_FORMAT` | string | run, ask |
| `"output-file"` | `--output-file` | `CLR_OUTPUT_FILE` | string | run, ask |
| `"output-style"` | `--output-style` | `CLR_OUTPUT_STYLE` | string (summary/raw) | run, ask |
| `"summary-fields"` | `--summary-fields` | `CLR_SUMMARY_FIELDS` | string | run, ask |
| `"quiet"` | `--quiet` | `CLR_QUIET` | bool | run, ask |
| `"strip-fences"` | `--strip-fences` | `CLR_STRIP_FENCES` | bool | run, ask |

**Observability and Tracing**

| JSON Key | CLI Flag | CLR_* Env Var | Type | Supported By |
|----------|----------|---------------|------|--------------|
| `"trace"` | `--trace` | `CLR_TRACE` | bool | run, ask, isolated, refresh |

**Validation**

| JSON Key | CLI Flag | CLR_* Env Var | Type | Supported By |
|----------|----------|---------------|------|--------------|
| `"expect"` | `--expect` | `CLR_EXPECT` | string (val1\|val2\|…) | run, ask |
| `"expect-strategy"` | `--expect-strategy` | `CLR_EXPECT_STRATEGY` | string (fail/retry/default:V) | run, ask |

**Concurrency**

| JSON Key | CLI Flag | CLR_* Env Var | Type | Supported By |
|----------|----------|---------------|------|--------------|
| `"max-sessions"` | `--max-sessions` | `CLR_MAX_SESSIONS` | u32 (0 = unlimited) | run, ask |

**Timeout**

| JSON Key | CLI Flag | CLR_* Env Var | Type | Supported By |
|----------|----------|---------------|------|--------------|
| `"timeout"` | `--timeout` | `CLR_TIMEOUT` | u32 seconds (0 = unlimited) | run, ask, isolated, refresh |

**Retry — Tier 2 (per-class)**

| JSON Key | CLI Flag | CLR_* Env Var | Type | Supported By |
|----------|----------|---------------|------|--------------|
| `"retry-on-transient"` | `--retry-on-transient` | `CLR_RETRY_ON_TRANSIENT` | u8 | run, ask |
| `"transient-delay"` | `--transient-delay` | `CLR_TRANSIENT_DELAY` | u32 seconds | run, ask |
| `"retry-on-account"` | `--retry-on-account` | `CLR_RETRY_ON_ACCOUNT` | u8 | run, ask |
| `"account-delay"` | `--account-delay` | `CLR_ACCOUNT_DELAY` | u32 seconds | run, ask |
| `"retry-on-auth"` | `--retry-on-auth` | `CLR_RETRY_ON_AUTH` | u8 | run, ask |
| `"auth-delay"` | `--auth-delay` | `CLR_AUTH_DELAY` | u32 seconds | run, ask |
| `"retry-on-service"` | `--retry-on-service` | `CLR_RETRY_ON_SERVICE` | u8 | run, ask |
| `"service-delay"` | `--service-delay` | `CLR_SERVICE_DELAY` | u32 seconds | run, ask |
| `"retry-on-process"` | `--retry-on-process` | `CLR_RETRY_ON_PROCESS` | u8 | run, ask |
| `"process-delay"` | `--process-delay` | `CLR_PROCESS_DELAY` | u32 seconds | run, ask |
| `"retry-on-validation"` | `--retry-on-validation` | `CLR_RETRY_ON_VALIDATION` | u8 | run, ask |
| `"validation-delay"` | `--validation-delay` | `CLR_VALIDATION_DELAY` | u32 seconds | run, ask |
| `"retry-on-runner"` | `--retry-on-runner` | `CLR_RETRY_ON_RUNNER` | u8 | run, ask |
| `"runner-delay"` | `--runner-delay` | `CLR_RUNNER_DELAY` | u32 seconds | run, ask |
| `"retry-on-unknown"` | `--retry-on-unknown` | `CLR_RETRY_ON_UNKNOWN` | u8 | run, ask |
| `"unknown-delay"` | `--unknown-delay` | `CLR_UNKNOWN_DELAY` | u32 seconds | run, ask |

**Retry — Tier 1 (override) and Tier 3 (default)**

| JSON Key | CLI Flag | CLR_* Env Var | Type | Supported By |
|----------|----------|---------------|------|--------------|
| `"retry-override"` | `--retry-override` | `CLR_RETRY_OVERRIDE` | u8 | run, ask |
| `"retry-override-delay"` | `--retry-override-delay` | `CLR_RETRY_OVERRIDE_DELAY` | u32 seconds | run, ask |
| `"retry-default"` | `--retry-default` | `CLR_RETRY_DEFAULT` | u8 (default: 2) | run, ask |
| `"retry-default-delay"` | `--retry-default-delay` | `CLR_RETRY_DEFAULT_DELAY` | u32 seconds (default: 30) | run, ask |

**Journal**

| JSON Key | CLI Flag | CLR_* Env Var | Type | Supported By |
|----------|----------|---------------|------|--------------|
| `"journal"` | `--journal` | `CLR_JOURNAL` | string (full/meta/off) | run, ask, isolated, refresh |
| `"journal-dir"` | `--journal-dir` | `CLR_JOURNAL_DIR` | string | run, ask, isolated, refresh |

**Isolated and Refresh Subcommands Only**

| JSON Key | CLI Flag | CLR_* Env Var | Type | Supported By |
|----------|----------|---------------|------|--------------|
| `"creds"` | `--creds` | `CLR_CREDS` | string (file path) | isolated, refresh |

**Config Source (self-referential)**

| JSON Key | CLI Flag | CLR_* Env Var | Type | Supported By |
|----------|----------|---------------|------|--------------|
| `"args-file"` | `--args-file` | `CLR_ARGS_FILE` | string (file path) | run, ask, isolated, refresh — **not re-processed when encountered inside a JSON source; chaining not supported** |

### Not Configurable via JSON

The following parameters cannot appear in a JSON config file:

| Parameter | Reason |
|-----------|--------|
| `--help` / `-h` | Meta-flag; exits before JSON loading |
| `--mode`, `--columns`, `--pid`, `--wide`, `--inspect` | `ps` subcommand — not supported by `--args-file` |
| `CLR_PS_*` env vars | `ps` subcommand only |
| `CLR_GATE_DIR`, `CLR_GATE_POLL_SECS`, `CLR_GATE_MAX_ATTEMPTS` | Runtime config; no CLI flag equivalent |
| `CLAUDE_CODE_*` subprocess vars | Subprocess env vars, not clr params |

### Sources

| File | Relationship |
|------|--------------|
| `../../src/cli/mod.rs` | Stdin JSON pipe detection; early arg injection before subcommand dispatch |
| `../../src/cli/env.rs` | `CLR_ARGS_FILE` loading; JSON source injected at env-var resolution point |
| `../../src/cli/parse.rs` | JSON key-to-`CliArgs` field mapping; boolean flag handling |

### Tests

| File | Relationship |
|------|--------------|
| `../../tests/json_config_test.rs` | EC tests for JSON file loading, stdin pipe, precedence, error cases |
