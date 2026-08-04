# CLI Command: isolated

### Description

Run Claude in a credential-isolated subprocess with a temporary HOME containing only the provided credentials file. Use `clr isolated` when running Claude with alternate accounts, test tokens, or deployment-specific credentials without exposing the caller's real HOME, settings, or session history.

-- **Parameters:** `--creds`, `--model`, `--timeout`, `--max-sessions`, `--trace`, `--dry-run`, `--effort`, `--no-effort-max`, `--no-chrome`, `--no-compact-window`, `--dir`, `--add-dir`, `--file`, `--expect`, `--expect-strategy`, `--journal`, `--journal-dir`, `--output-file`, `--strip-fences`, `--output-style`, `--summary-fields`, `--system-prompt`, `--append-system-prompt`, `--json-schema`, `--mcp-config`, `--allowed-tools`, `--disallowed-tools`, `--max-budget-usd`, `--max-turns`, `--args-file`
-- **Exit Codes:** 0 (success) | 1 (error) | 2 (timeout) | 3 (expect mismatch) | N (subprocess passthrough) | 128+signal (signal)

### Syntax

```sh
clr isolated [--creds <FILE>] [--timeout <SECS>] [OPTIONS] [MESSAGE] [-- PASSTHROUGH...]
```

### Parameters

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| [`[MESSAGE]`](../param/001_message.md) | [`MessageText`](../type/01_message_text.md) | — | Prompt forwarded to Claude |
| [`--creds`](../param/019_creds.md) | [`CredentialsFilePath`](../type/08_credentials_file_path.md) | `~/.claude/.credentials.json` | Credentials JSON file path (optional; defaults to current account credentials) |
| [`--model`](../param/003_model.md) | [`ModelName`](../type/04_model_name.md) | — | Model override; when absent falls back to project `.clr.toml` → user `~/.clr/config.toml` → `opus` alias; env: `CLR_MODEL` |
| [`--timeout`](../param/020_timeout.md) | [`TimeoutSecs`](../type/09_timeout_secs.md) | 30 | Max seconds to wait for subprocess |
| [`--max-sessions`](../param/033_max_sessions.md) | u32 | 6 | Max concurrent non-interactive sessions before blocking; `0` = unlimited (gate disabled); JSON key: `"max-sessions"`; env: `CLR_MAX_SESSIONS`; no config-file tier |
| [`--trace`](../param/013_trace.md) | bool | false | Print underlying call details to stderr then execute |
| [`--dry-run`](../param/011_dry_run.md) | bool | false | Print subprocess env+command to stderr (same path as `--trace`); exit 0 without spawning |
| [`--effort`](../param/017_effort.md) | [`EffortLevel`](../type/07_effort_level.md) | max | Reasoning effort: `low`, `medium`, `high`, `max`; default `max` injected when absent; env: `CLR_EFFORT` |
| [`--no-effort-max`](../param/018_no_effort_max.md) | bool | false | Suppress automatic `--effort max` injection entirely; env: `CLR_NO_EFFORT_MAX` |
| [`--no-chrome`](../param/021_no_chrome.md) | bool | false | Suppress automatic `--chrome` injection; env: `CLR_NO_CHROME` |
| [`--no-compact-window`](../param/077_no_compact_window.md) | bool | false | Suppress `CLAUDE_CODE_AUTO_COMPACT_WINDOW=300000` injection; env: `CLR_NO_COMPACT_WINDOW` |
| [`--dir`](../param/008_dir.md) | path | — | Working directory injected into subprocess command; validated to exist before spawn; env: `CLR_DIR` |
| [`--add-dir`](../param/066_add_dir.md) | path (repeatable) | — | Additional directory Claude may access; injected per entry into subprocess command; env: `CLR_ADD_DIR` |
| [`--file`](../param/025_file.md) | path | — | File piped as stdin to the subprocess; validated to exist before spawn |
| [`--expect`](../param/030_expect.md) | string | — | Pipe-separated expected values; mismatch triggers `--expect-strategy` (case-insensitive, trimmed) |
| [`--expect-strategy`](../param/031_expect_strategy.md) | enum | `fail` | Mismatch strategy: `fail` → exit 3; `default:<V>` → print `<V>`, exit 0; `retry` → exit 1 (unsupported for isolated) |
| [`--journal`](../param/072_journal.md) | enum | `full` | Journal level: `full` (stdout+stderr ≤1MB), `meta` (metadata only), `off` (disabled) |
| [`--journal-dir`](../param/073_journal_dir.md) | path | `~/.clr/journal/` | Directory for journal JSONL files; overrides `CLR_JOURNAL_DIR` |
| [`--output-file`](../param/029_output_file.md) | path | — | Write output to file (also prints to stdout); env: `CLR_OUTPUT_FILE` |
| [`--strip-fences`](../param/026_strip_fences.md) | bool | false | Strip outermost markdown code fences from output; env: `CLR_STRIP_FENCES` |
| [`--output-style`](../param/070_output_style.md) | enum | `raw` | Output rendering: `raw` (default), `summary`; env: `CLR_OUTPUT_STYLE` |
| [`--summary-fields`](../param/071_summary_fields.md) | string | — | Summary field selection: `full`, `standard`, `minimal`, or comma-separated; env: `CLR_SUMMARY_FIELDS` |
| [`--system-prompt`](../param/015_system_prompt.md) | [`SystemPromptText`](../type/06_system_prompt_text.md) | — | Replace the default system prompt; forwarded to claude subprocess; env: `CLR_SYSTEM_PROMPT` |
| [`--append-system-prompt`](../param/016_append_system_prompt.md) | [`SystemPromptText`](../type/06_system_prompt_text.md) | — | Append text to the default system prompt; forwarded to claude subprocess; env: `CLR_APPEND_SYSTEM_PROMPT` |
| [`--json-schema`](../param/023_json_schema.md) | [`JsonSchemaText`](../type/10_json_schema_text.md) | — | JSON schema for structured output; forwarded to claude subprocess; env: `CLR_JSON_SCHEMA` |
| [`--mcp-config`](../param/024_mcp_config.md) | [`McpConfigPath`](../type/11_mcp_config_path.md) | — | MCP server config file (repeatable); forwarded to claude subprocess; env: `CLR_MCP_CONFIG` |
| [`--allowed-tools`](../param/063_allowed_tools.md) | string | — | Comma-separated tool whitelist; forwarded to claude subprocess; env: `CLR_ALLOWED_TOOLS` |
| [`--disallowed-tools`](../param/064_disallowed_tools.md) | string | — | Comma-separated tool blacklist; forwarded to claude subprocess; env: `CLR_DISALLOWED_TOOLS` |
| [`--max-budget-usd`](../param/065_max_budget_usd.md) | string | — | Max API spend in USD; forwarded to claude subprocess; env: `CLR_MAX_BUDGET_USD` |
| [`--max-turns`](../param/062_max_turns.md) | string | — | Max agentic turns; forwarded to claude subprocess; env: `CLR_MAX_TURNS` |
| [`--args-file`](../param/075_args_file.md) | [`FilePath`](../type/12_file_path.md) | — | Load clr params from JSON config file; stdin JSON auto-detected when no TTY; env: `CLR_ARGS_FILE` |
| `-h`/`--help` | — | — | Print isolated subcommand help and exit 0 |

**Algorithm (7 steps):**
1. Wait for a concurrency-gate slot per `--max-sessions` (default 6; same mechanism as `run`/`ask`); `--dry-run` bypasses this step entirely.
2. Resolve credentials path: `--creds` if given, else `$HOME/.claude/.credentials.json`; exit 1 if file not found.
3. Create temporary HOME directory; write `.claude/.credentials.json` from resolved credentials.
4. Write minimal `~/.claude/CLAUDE.md` to temp HOME to suppress interactive prompts.
5. Build subprocess command: `--model` from native flag (if given) else project `.clr.toml` → user `~/.clr/config.toml` → `"opus"` alias; `--effort` from native flag else `max` unless `--no-effort-max` suppresses it entirely; `--no-session-persistence` always; `--dangerously-skip-permissions` when message present; `--chrome` unless `--no-chrome` is set; push `--system-prompt`, `--append-system-prompt`, `--json-schema`, `--mcp-config` (each entry), `--allowed-tools`, `--disallowed-tools`, `--max-budget-usd`, `--max-turns` when given; prepend all before `--print` and message; passthrough args appended last for last-wins override.
6. Spawn `claude` with `HOME=<temp>`; wait up to `--timeout` seconds (0 = unlimited).
7. If credentials were refreshed at startup, write updated file back to `--creds`; delete temp HOME unconditionally; propagate subprocess exit code (or exit 2 on timeout without refresh).

### Exit Codes

| Code | Meaning |
|------|---------|
| 0 | Claude exited successfully (may have refreshed creds in-place) |
| 1 | Error (creds file not found, claude not in PATH, I/O failure, unsupported `--expect-strategy retry`, session gate timed out or unavailable) |
| 2 | Timeout — subprocess did not finish within `--timeout` seconds; any partial stdout accumulated before the timeout is preserved in the error output |
| 3 | `--expect` mismatch with `fail` strategy |
| N | Passthrough from claude subprocess (non-zero) |
| 128+signal | POSIX signal termination — subprocess killed by signal (e.g., 130 = SIGINT, 143 = SIGTERM); passes through from subprocess identically to any other non-zero `N` |

### Examples

```sh
# Quick prompt with isolated credentials
clr isolated --creds ~/.claude/.credentials.json "What is 2+2?"

# Custom timeout for long-running tasks
clr isolated --creds /path/to/creds.json --timeout 120 "Refactor this module"

# Verify credentials work (--version exits fast)
clr isolated --creds /path/to/creds.json -- --version

# Interactive isolated session (no message — REPL mode)
clr isolated --creds /path/to/creds.json
```

### Notes

The isolated subprocess has no access to the caller's real `$HOME` — no `~/.claude/settings.json`, no previous conversation state. A minimal `~/.claude/CLAUDE.md` is written to the temp HOME before spawn instructing the subprocess to execute immediately without asking clarifying questions or requesting confirmation.

Subprocess injected defaults (see [`invariant/005_isolated_subprocess_defaults.md`](../../invariant/005_isolated_subprocess_defaults.md)):
- `--model` — resolved across 2 tiers: project `.clr.toml`'s `model` key, then user `~/.clr/config.toml`'s `model` key (set via `clr .model.select`); first tier with a value wins. Falls back to `"opus"` (`ISOLATED_DEFAULT_MODEL` — Opus alias; binary resolves to latest Opus) when neither tier sets a value. See [`parity/001_run_ask_isolated.md`](../parity/001_run_ask_isolated.md) for the full comparison against `run`/`ask`'s equivalent cascade.
- `--effort max` (maximum reasoning effort)
- `--no-session-persistence` (temp HOME is discarded after every run; session writes are waste)
- `--dangerously-skip-permissions` — injected when `[MESSAGE]` is present; omitted in interactive mode (no message)
- `--chrome` injected by default (isolated tasks may use browser tools); suppress with `--no-chrome`

Injected flags are prepended before `--print` and message so passthrough args override via last-wins:

```sh
# Override effort for a lighter task:
clr isolated "summarize this file" -- --effort medium
# Opt out of skip-permissions for a read-only task:
clr isolated "what is 2+2?" -- --no-skip-permissions
```

If the subprocess times out but already wrote refreshed credentials, `clr isolated` exits 0 and writes updated credentials back to `--creds` instead of returning exit 2. This matches the `IsolatedRunResult { exit_code: -1, credentials: Some(…) }` path in `claude_runner_core::run_isolated_ext()`.

`--timeout 0` disables the watchdog entirely (unlimited runtime), matching `run`/`ask` semantics.

`--max-sessions` gates `isolated` through the same concurrency mechanism as `run`/`ask` (see
[user_story/025_concurrency_gate.md](../user_story/025_concurrency_gate.md)) as a 3-tier chain:
CLI flag + `"max-sessions"` JSON key (via `--args-file`) + `CLR_MAX_SESSIONS` env var —
no config-file tier (consistent with `isolated` having no config-file tier for any parameter). The 3 gate-tuning knobs (`CLR_GATE_POLL_SECS`, `CLR_GATE_MAX_ATTEMPTS`,
`CLR_GATE_STALE_SECS`) also apply to `isolated` env-var-only — no `--gate-poll-secs`/
`--gate-max-attempts`/`--gate-stale-secs` CLI flags exist for `isolated` (contrast `run`/`ask`,
which have full 5-tier parity for these 3 — see
[003_env_param.md](../003_env_param.md#env-param-5-gate-runtime-configuration)). If the process
scanner cannot read the process list (e.g. `/proc` unavailable), `isolated` fails loudly with the
same `GateUnavailable` Runner-class error as `run`/`ask`, rather than silently proceeding as if
the gate were disabled — see [param/033_max_sessions.md](../param/033_max_sessions.md).

### Related Commands

| # | Command | Relationship |
|---|---------|--------------|
| 1 | [`refresh`](04_refresh.md) | Both use `run_isolated_ext()`; `refresh` sends a trivial ping to trigger token refresh only |

### Referenced Parameter Groups

| # | Group | Membership | Excluded Params |
|---|-------|------------|-----------------|
| 4 | [Credential Operations](../param_group/04_credential_operations.md) | Full | — |
| 6 | [Running Commands](../param_group/06_running_commands.md) | Subset — `--timeout`, `--trace`, `--dry-run`, `--no-compact-window`, `--journal`, `--journal-dir` | `--creds` is Group 4 |

### Referenced User Stories

| # | User Story | Persona |
|---|------------|---------|
| 8 | [008_trace_execution.md](../user_story/008_trace_execution.md) | Developer |
| 10 | [010_credential_isolated_execution.md](../user_story/010_credential_isolated_execution.md) | Developer |
| 25 | [025_concurrency_gate.md](../user_story/025_concurrency_gate.md) | Developer |

---

**Category:** Credential management
**Complexity:** 15
**API Requirement:** Write
**Idempotent:** No
**Risk Level:** Low
