# CLI: Environment Parameters

### Scope

- **Purpose**: Document CLR_* environment variable fallbacks and the CLAUDE_CODE_MAX_OUTPUT_TOKENS subprocess variable.
- **Responsibility**: Specify env var names, corresponding CLI parameters, precedence rules, and type handling.
- **In Scope**: CLR_* input vars for run/isolated/refresh, CLAUDE_CODE_MAX_OUTPUT_TOKENS injection, precedence, bool/parsed type semantics.
- **Out of Scope**: CLI parameter descriptions (-> param/), subprocess behavior beyond env injection.

### All Env Parameters (38 total)

| Category | Count | Purpose |
|----------|-------|---------|
| Input (CLR_*) — `run` subcommand | 34 | Caller env fallbacks for `run` parameters |
| Input (CLR_*) — `isolated` and `refresh` subcommands | 3 | Caller env fallbacks for credential operation parameters |
| Subprocess (CLAUDE_CODE_*) | 1 | Set by `clr` before spawning the `claude` subprocess |

**Total:** 38 environment variables

---

### Env Param 1: CLR_* Input Parameters — `run` Subcommand

Environment variable fallbacks for all 34 `run` subcommand parameters.
`apply_env_vars()` in `src/cli/parse.rs` reads these immediately after CLI parsing, before command
dispatch. Each variable is applied **only when the corresponding CLI field is still at its
zero/absent value** — the CLI flag always wins when both are present.

**Bool variables** accept `"1"` or `"true"` (case-insensitive) as truthy.
Any other value — including `"yes"`, `"0"`, `"false"`, empty, or absent — resolves to `false`.

**Parsed variables** (`CLR_MAX_TOKENS`, `CLR_VERBOSITY`, `CLR_EFFORT`, `CLR_RETRY_ON_RATE_LIMIT`, `CLR_RETRY_DELAY`, `CLR_TIMEOUT`) silently ignore
invalid values (parse failure → field stays at default).

| # | Variable | CLI Parameter | Type | Notes |
|---|----------|---------------|------|-------|
| 1 | `CLR_MESSAGE` | [`[MESSAGE]`](param/001_message.md) | string | Escape hatch for messages containing shell-special characters (`(`, `)`, `&`, `;`, `\|`, etc.) — bypasses bash tokenization |
| 2 | `CLR_PRINT` | [`--print`](param/002_print.md) | bool | |
| 3 | `CLR_MODEL` | [`--model`](param/003_model.md) | string | |
| 4 | `CLR_VERBOSE` | [`--verbose`](param/004_verbose.md) | bool | |
| 5 | `CLR_NO_SKIP_PERMISSIONS` | [`--no-skip-permissions`](param/005_no_skip_permissions.md) | bool | |
| 6 | `CLR_INTERACTIVE` | [`--interactive`](param/006_interactive.md) | bool | |
| 7 | `CLR_NEW_SESSION` | [`--new-session`](param/007_new_session.md) | bool | |
| 8 | `CLR_DIR` | [`--dir`](param/008_dir.md) | string | |
| 9 | `CLR_MAX_TOKENS` | [`--max-tokens`](param/009_max_tokens.md) | u32 | Invalid values silently ignored |
| 10 | `CLR_SESSION_DIR` | [`--session-dir`](param/010_session_dir.md) | string | |
| 11 | `CLR_DRY_RUN` | [`--dry-run`](param/011_dry_run.md) | bool | |
| 12 | `CLR_VERBOSITY` | [`--verbosity`](param/012_verbosity.md) | 0–5 | Applied only when `--verbosity` is absent from CLI |
| 13 | `CLR_TRACE` | [`--trace`](param/013_trace.md) | bool | |
| 14 | `CLR_NO_ULTRATHINK` | [`--no-ultrathink`](param/014_no_ultrathink.md) | bool | |
| 15 | `CLR_SYSTEM_PROMPT` | [`--system-prompt`](param/015_system_prompt.md) | string | |
| 16 | `CLR_APPEND_SYSTEM_PROMPT` | [`--append-system-prompt`](param/016_append_system_prompt.md) | string | |
| 17 | `CLR_EFFORT` | [`--effort`](param/017_effort.md) | string | Parsed as `EffortLevel` (`low`/`medium`/`high`/`max`); invalid values silently ignored |
| 18 | `CLR_NO_EFFORT_MAX` | [`--no-effort-max`](param/018_no_effort_max.md) | bool | |
| 19 | `CLR_NO_CHROME` | [`--no-chrome`](param/021_no_chrome.md) | bool | |
| 20 | `CLR_NO_PERSIST` | [`--no-persist`](param/022_no_persist.md) | bool | |
| 21 | `CLR_JSON_SCHEMA` | [`--json-schema`](param/023_json_schema.md) | string | |
| 22 | `CLR_MCP_CONFIG` | [`--mcp-config`](param/024_mcp_config.md) | string | Only one value via env var; multiple configs require CLI repeats (`--mcp-config A --mcp-config B`) |
| 23 | `CLR_FILE` | [`--file`](param/025_file.md) | string | Applied when `--file` absent; value is the file path |
| 24 | `CLR_STRIP_FENCES` | [`--strip-fences`](param/026_strip_fences.md) | bool | |
| 25 | `CLR_KEEP_CLAUDECODE` | [`--keep-claudecode`](param/027_keep_claudecode.md) | bool | |
| 26 | `CLR_SUBDIR` | [`--subdir`](param/028_subdir.md) | string | Applied when `--subdir` absent and `CLR_SUBDIR` non-empty; `.` = identity; values containing `/` silently ignored (Fix: BUG-233) |
| 27 | `CLR_OUTPUT_FILE` | [`--output-file`](param/029_output_file.md) | string | Applied when `--output-file` absent; value is the output file path |
| 28 | `CLR_EXPECT` | [`--expect`](param/030_expect.md) | string | Applied when `--expect` absent; same `val1\|val2\|…` syntax |
| 29 | `CLR_EXPECT_STRATEGY` | [`--expect-strategy`](param/031_expect_strategy.md) | string | Applied when `--expect-strategy` absent; accepts `fail`, `retry`, or `default:<V>` |
| 30 | `CLR_EXPECT_RETRIES` | [`--expect-retries`](param/032_expect_retries.md) | u8 | Applied when `--expect-retries` absent; invalid values rejected at parse time |
| 31 | `CLR_MAX_SESSIONS` | [`--max-sessions`](param/033_max_sessions.md) | u32 | Applied when `--max-sessions` absent; invalid values silently ignored (parse failure → field stays at default 10) |
| 32 | `CLR_RETRY_ON_RATE_LIMIT` | [`--retry-on-rate-limit`](param/034_retry_on_rate_limit.md) | u8 | Applied when `--retry-on-rate-limit` absent; invalid values silently ignored (parse failure → field stays at default 0) |
| 33 | `CLR_RETRY_DELAY` | [`--retry-delay`](param/035_retry_delay.md) | u32 | Applied when `--retry-delay` absent; invalid values silently ignored (parse failure → field stays at default 60) |
| 34 | `CLR_TIMEOUT` | [`--timeout`](param/036_timeout.md) | u32 | Applied when `--timeout` absent; `0` = unlimited (no watchdog); invalid values silently ignored. **Cross-command:** also applies to `isolated`/`refresh` via Section 2 (same semantics: `0` = unlimited) |

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

### Env Param 2: CLR_* Input Parameters — `isolated` and `refresh` Subcommands

Environment variable fallbacks for the 3 credential operation parameters.
`apply_isolated_env_vars()` and `apply_refresh_env_vars()` in `src/cli/parse.rs` read these
after subcommand argument parsing.

| # | Variable | CLI Parameter | Type | Notes |
|---|----------|---------------|------|-------|
| 1 | `CLR_CREDS` | [`--creds`](param/019_creds.md) | string | Applied when `--creds` absent (`creds_path` is empty string) |
| 2 | `CLR_TIMEOUT` | [`--timeout`](param/020_timeout.md) | u64 | Applied when CLI timeout equals its command default (30 for `isolated`, 45 for `refresh`); `0` = unlimited (no watchdog), matching `run`/`ask` semantics. Also applies to `run`/`ask` via Section 1 row 34 |
| 3 | `CLR_TRACE` | [`--trace`](param/013_trace.md) | bool | Applied when `--trace` absent; also applies to `run` via Section 1 |

**Precedence (`--creds` only):**

1. `--creds` CLI flag (wins unconditionally when provided)
2. `CLR_CREDS` env var (applied when `--creds` absent)
3. `$HOME/.claude/.credentials.json` default (used when both `--creds` and `CLR_CREDS` are absent; exits 1 if `HOME` unset or file missing)

**Precedence (`--timeout`, `--trace`):**

1. `--timeout` / `--trace` CLI flag (wins)
2. `CLR_TIMEOUT` / `CLR_TRACE` env var (applied when CLI field absent/default)

**Limitation (`CLR_TIMEOUT`):** The env var check uses equality with the command's default
timeout as the sentinel, so an explicit `--timeout 30` on `isolated` (or `--timeout 45`
on `refresh`) is indistinguishable from the default — `CLR_TIMEOUT` will still override it.

**Note (`CLR_TRACE`):** `CLR_TRACE` is also listed in Section 1 (row 13) for the `run`
subcommand. It is a cross-command env var that applies to all three executing commands.

---

### Env Param 3: `CLAUDE_CODE_MAX_OUTPUT_TOKENS`

Set by the `clr` runner immediately before spawning the `claude` subprocess.
Controls the maximum number of output tokens the Claude Code subprocess may
generate in a single turn.

- **Source parameter:** [`--max-tokens`](param/009_max_tokens.md)
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
