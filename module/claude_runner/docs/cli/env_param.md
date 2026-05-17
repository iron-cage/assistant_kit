# Environment Parameters

### All Env Parameters (25 total)

| Category | Count | Purpose |
|----------|-------|---------|
| Input (CLR_*) — `run` subcommand | 22 | Caller env fallbacks for `run` parameters |
| Input (CLR_*) — `isolated` subcommand | 2 | Caller env fallbacks for `isolated` parameters |
| Subprocess (CLAUDE_CODE_*) | 1 | Set by `clr` before spawning the `claude` subprocess |

**Total:** 25 environment variables

---

### Env Param :: 1. CLR_* Input Parameters — `run` Subcommand

Environment variable fallbacks for the 22 `run` subcommand parameters.
`apply_env_vars()` in `src/lib.rs` reads these immediately after CLI parsing, before command
dispatch. Each variable is applied **only when the corresponding CLI field is still at its
zero/absent value** — the CLI flag always wins when both are present.

**Bool variables** accept `"1"` or `"true"` (case-insensitive) as truthy.
Any other value — including `"yes"`, `"0"`, `"false"`, empty, or absent — resolves to `false`.

**Parsed variables** (`CLR_MAX_TOKENS`, `CLR_VERBOSITY`, `CLR_EFFORT`) silently ignore
invalid values (parse failure → field stays at default).

| # | Variable | CLI Parameter | Type | Notes |
|---|----------|---------------|------|-------|
| 1 | `CLR_MESSAGE` | [`[MESSAGE]`](param/01_message.md) | string | |
| 2 | `CLR_PRINT` | [`--print`](param/02_print.md) | bool | |
| 3 | `CLR_MODEL` | [`--model`](param/03_model.md) | string | |
| 4 | `CLR_VERBOSE` | [`--verbose`](param/04_verbose.md) | bool | |
| 5 | `CLR_NO_SKIP_PERMISSIONS` | [`--no-skip-permissions`](param/05_no_skip_permissions.md) | bool | |
| 6 | `CLR_INTERACTIVE` | [`--interactive`](param/06_interactive.md) | bool | |
| 7 | `CLR_NEW_SESSION` | [`--new-session`](param/07_new_session.md) | bool | |
| 8 | `CLR_DIR` | [`--dir`](param/08_dir.md) | string | |
| 9 | `CLR_MAX_TOKENS` | [`--max-tokens`](param/09_max_tokens.md) | u32 | Invalid values silently ignored |
| 10 | `CLR_SESSION_DIR` | [`--session-dir`](param/10_session_dir.md) | string | |
| 11 | `CLR_DRY_RUN` | [`--dry-run`](param/11_dry_run.md) | bool | |
| 12 | `CLR_VERBOSITY` | [`--verbosity`](param/12_verbosity.md) | 0–5 | Applied only when CLI verbosity == default (3); `--verbosity 3` explicit is indistinguishable |
| 13 | `CLR_TRACE` | [`--trace`](param/13_trace.md) | bool | |
| 14 | `CLR_NO_ULTRATHINK` | [`--no-ultrathink`](param/14_no_ultrathink.md) | bool | |
| 15 | `CLR_SYSTEM_PROMPT` | [`--system-prompt`](param/15_system_prompt.md) | string | |
| 16 | `CLR_APPEND_SYSTEM_PROMPT` | [`--append-system-prompt`](param/16_append_system_prompt.md) | string | |
| 17 | `CLR_EFFORT` | [`--effort`](param/17_effort.md) | string | Parsed as `EffortLevel` (`low`/`medium`/`high`/`max`); invalid values silently ignored |
| 18 | `CLR_NO_EFFORT_MAX` | [`--no-effort-max`](param/18_no_effort_max.md) | bool | |
| 19 | `CLR_NO_CHROME` | [`--no-chrome`](param/21_no_chrome.md) | bool | |
| 20 | `CLR_NO_PERSIST` | [`--no-persist`](param/22_no_persist.md) | bool | |
| 21 | `CLR_JSON_SCHEMA` | [`--json-schema`](param/23_json_schema.md) | string | |
| 22 | `CLR_MCP_CONFIG` | [`--mcp-config`](param/24_mcp_config.md) | string | Only one value via env var; multiple configs require CLI repeats (`--mcp-config A --mcp-config B`) |

**Precedence:**

1. CLI flag (wins unconditionally when provided)
2. `CLR_*` env var (applied when CLI field is absent/at default)
3. Built-in default

**Discovery:** Use `--dry-run` or `--trace` to see effective values after env var application.

```sh
CLR_MODEL=sonnet clr --dry-run "task"           # shows: claude --model sonnet ...
CLR_MODEL=sonnet clr --model opus --dry-run "task"  # CLI wins; CLR_MODEL ignored
```

---

### Env Param :: 2. CLR_* Input Parameters — `isolated` Subcommand

Environment variable fallbacks for the 2 `isolated` subcommand parameters.
`apply_isolated_env_vars()` in `src/lib.rs` reads these after `isolated` argument parsing.

| # | Variable | CLI Parameter | Type | Notes |
|---|----------|---------------|------|-------|
| 1 | `CLR_CREDS` | [`--creds`](param/19_creds.md) | string | Applied when `--creds` absent (`creds_path` is empty string) |
| 2 | `CLR_TIMEOUT` | [`--timeout`](param/20_timeout.md) | u64 | Applied when CLI timeout == 30 (default); `--timeout 30` explicit is indistinguishable |

**Precedence:**

1. `--creds` / `--timeout` CLI flag (wins)
2. `CLR_CREDS` / `CLR_TIMEOUT` env var (applied when CLI field absent/default)

**Limitation (`CLR_TIMEOUT`):** `isolated` defaults to `timeout_secs = 30`. The env var
check uses `== 30` as the sentinel, so an explicit `--timeout 30` on the CLI is
indistinguishable from the default — `CLR_TIMEOUT` will still override it.

---

### Env Param :: 3. `CLAUDE_CODE_MAX_OUTPUT_TOKENS`

Set by the `clr` runner immediately before spawning the `claude` subprocess.
Controls the maximum number of output tokens the Claude Code subprocess may
generate in a single turn.

- **Source parameter:** [`--max-tokens`](param/09_max_tokens.md)
- **Type:** u32 (serialized as decimal string)
- **Default:** `200000`
- **Mechanism:** injected via `std::process::Command::env("CLAUDE_CODE_MAX_OUTPUT_TOKENS", value.to_string())`
- **Scope:** subprocess-only; not visible to or read by `clr` itself

**Precedence:**

1. Explicit `--max-tokens <N>` CLI value (overrides default)
2. Built-in default `200000` (when `--max-tokens` is absent)

**Discovery:** Use `--dry-run` or `--trace` to see the current value in the
assembled environment before subprocess invocation.

```sh
clr --dry-run "test"                         # shows: CLAUDE_CODE_MAX_OUTPUT_TOKENS=200000
clr --max-tokens 50000 --dry-run "test"      # shows: CLAUDE_CODE_MAX_OUTPUT_TOKENS=50000
```
