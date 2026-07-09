# Environment Parameters

### Scope

- **Purpose**: Document CLR_* environment variable fallbacks, runtime configuration overrides, and CLAUDE_CODE_* subprocess variables.
- **Responsibility**: Specify env var names, corresponding CLI parameters, precedence rules, and type handling.
- **In Scope**: CLR_* input vars for run/isolated/refresh, CLR_* runtime config overrides (`CLR_GATE_DIR`, `CLR_GATE_POLL_SECS`, `CLR_GATE_MAX_ATTEMPTS`, `CLR_GATE_STALE_SECS`, `CLR_CONFIG_DIR`), and the 5 `CLAUDE_CODE_*` subprocess variables `clr` injects by default (`CLAUDE_CODE_MAX_OUTPUT_TOKENS`, `CLAUDE_CODE_AUTO_COMPACT_WINDOW`, `CLAUDE_CODE_BASH_TIMEOUT`, `CLAUDE_CODE_BASH_MAX_TIMEOUT`, `CLAUDE_CODE_PRINT_BG_WAIT_CEILING_MS`), precedence, bool/parsed type semantics.
- **Out of Scope**: CLI parameter descriptions (→ param/), subprocess behavior beyond env injection, config-file TOML key reference (→ [config_param.md](config_param.md)).

### All Env Parameters (89 total)

| Category | Count | Purpose |
|----------|-------|---------|
| Input (CLR_*) — `run` subcommand | 64 | Caller env fallbacks for `run` parameters |
| Input (CLR_*) — `isolated` and `refresh` subcommands | 13 | Caller env fallbacks for credential operation parameters |
| Input (CLR_*) — `ps` subcommand | 5 | Caller env fallbacks for session listing display and flag thresholds |
| Runtime config (CLR_*) | 5 | Runtime configuration overrides (not CLI parameter fallbacks) |
| Subprocess (CLAUDE_CODE_*) — injected | 5 | Set by `clr` before spawning the `claude` subprocess |

**Total:** 92 environment variables

---

### Env Param 1: CLR_* Input Parameters — `run` Subcommand

Environment variable fallbacks for all 64 `run` subcommand parameters (62 standard + `CLR_ARGS_FILE` + `CLR_NO_COMPACT_WINDOW`).
`apply_env_vars()` in `src/cli/env.rs` reads these immediately after CLI parsing, before command
dispatch. Each variable is applied **only when the corresponding CLI field is still at its
zero/absent value** — the CLI flag always wins when both are present.

**Bool variables** accept `"1"` or `"true"` (case-insensitive) as truthy.
Any other value — including `"yes"`, `"0"`, `"false"`, empty, or absent — resolves to `false`.

**Parsed variables** (`CLR_MAX_TOKENS`, `CLR_EFFORT`, `CLR_TIMEOUT`, and all `CLR_RETRY_*` / `CLR_*_DELAY` variables) silently ignore
invalid values (parse failure → field stays at default). Exception: `CLR_RETRY_ON_VALIDATION` rejects invalid values at parse time.

| # | Variable | CLI Parameter | Type | JSON Key | Notes |
|---|----------|---------------|------|----------|-------|
| 1 | `CLR_MESSAGE` | [`[MESSAGE]`](param/001_message.md) | string | `"message"` | Escape hatch for messages containing shell-special characters (`(`, `)`, `&`, `;`, `\|`, etc.) — bypasses bash tokenization |
| 2 | `CLR_PRINT` | [`--print`](param/002_print.md) | bool | `"print"` | |
| 3 | `CLR_MODEL` | [`--model`](param/003_model.md) | string | `"model"` | |
| 4 | `CLR_VERBOSE` | [`--verbose`](param/004_verbose.md) | bool | `"verbose"` | |
| 5 | `CLR_NO_SKIP_PERMISSIONS` | [`--no-skip-permissions`](param/005_no_skip_permissions.md) | bool | `"no-skip-permissions"` | |
| 6 | `CLR_INTERACTIVE` | [`--interactive`](param/006_interactive.md) | bool | `"interactive"` | |
| 7 | `CLR_NEW_SESSION` | [`--new-session`](param/007_new_session.md) | bool | `"new-session"` | |
| 8 | `CLR_DIR` | [`--dir`](param/008_dir.md) | string | `"dir"` | |
| 9 | `CLR_MAX_TOKENS` | [`--max-tokens`](param/009_max_tokens.md) | u32 | `"max-tokens"` | Invalid values silently ignored |
| 10 | `CLR_SESSION_DIR` | [`--session-dir`](param/010_session_dir.md) | string | `"session-dir"` | |
| 11 | `CLR_DRY_RUN` | [`--dry-run`](param/011_dry_run.md) | bool | `"dry-run"` | |
| 12 | `CLR_QUIET` | [`--quiet`](param/074_quiet.md) | bool | `"quiet"` | Applied only when `--quiet` is absent from CLI |
| 13 | `CLR_TRACE` | [`--trace`](param/013_trace.md) | bool | `"trace"` | |
| 14 | `CLR_NO_ULTRATHINK` | [`--no-ultrathink`](param/014_no_ultrathink.md) | bool | `"no-ultrathink"` | |
| 15 | `CLR_SYSTEM_PROMPT` | [`--system-prompt`](param/015_system_prompt.md) | string | `"system-prompt"` | |
| 16 | `CLR_APPEND_SYSTEM_PROMPT` | [`--append-system-prompt`](param/016_append_system_prompt.md) | string | `"append-system-prompt"` | |
| 17 | `CLR_EFFORT` | [`--effort`](param/017_effort.md) | string | `"effort"` | Parsed as `EffortLevel` (`low`/`medium`/`high`/`max`); invalid values silently ignored |
| 18 | `CLR_NO_EFFORT_MAX` | [`--no-effort-max`](param/018_no_effort_max.md) | bool | `"no-effort-max"` | |
| 19 | `CLR_NO_CHROME` | [`--no-chrome`](param/021_no_chrome.md) | bool | `"no-chrome"` | |
| 20 | `CLR_NO_PERSIST` | [`--no-persist`](param/022_no_persist.md) | bool | `"no-persist"` | |
| 21 | `CLR_JSON_SCHEMA` | [`--json-schema`](param/023_json_schema.md) | string | `"json-schema"` | |
| 22 | `CLR_MCP_CONFIG` | [`--mcp-config`](param/024_mcp_config.md) | string | `"mcp-config"` | Only one value via env var; multiple configs require CLI repeats (`--mcp-config A --mcp-config B`) |
| 23 | `CLR_FILE` | [`--file`](param/025_file.md) | string | `"file"` | Applied when `--file` absent; value is the file path |
| 24 | `CLR_STRIP_FENCES` | [`--strip-fences`](param/026_strip_fences.md) | bool | `"strip-fences"` | |
| 25 | `CLR_KEEP_CLAUDECODE` | [`--keep-claudecode`](param/027_keep_claudecode.md) | bool | `"keep-claudecode"` | |
| 26 | `CLR_SUBDIR` | [`--subdir`](param/028_subdir.md) | string | `"subdir"` | Applied when `--subdir` absent and `CLR_SUBDIR` non-empty; `.` = identity; values containing `/` silently ignored (Fix: BUG-233) |
| 27 | `CLR_OUTPUT_FILE` | [`--output-file`](param/029_output_file.md) | string | `"output-file"` | Applied when `--output-file` absent; value is the output file path |
| 28 | `CLR_EXPECT` | [`--expect`](param/030_expect.md) | string | `"expect"` | Applied when `--expect` absent; same `val1\|val2\|…` syntax |
| 29 | `CLR_EXPECT_STRATEGY` | [`--expect-strategy`](param/031_expect_strategy.md) | string | `"expect-strategy"` | Applied when `--expect-strategy` absent; accepts `fail`, `retry`, or `default:<V>` |
| 30 | `CLR_MAX_SESSIONS` | [`--max-sessions`](param/033_max_sessions.md) | u32 | `"max-sessions"` | Applied when `--max-sessions` absent; invalid values silently ignored (parse failure → field stays at default 6) |
| 31 | `CLR_RETRY_ON_TRANSIENT` | [`--retry-on-transient`](param/034_retry_on_transient.md) | u8 | `"retry-on-transient"` | Transient class retry count (Tier 2); default auto → fallback |
| 32 | `CLR_TRANSIENT_DELAY` | [`--transient-delay`](param/035_transient_delay.md) | u32 | `"transient-delay"` | Transient class delay (Tier 2); default auto → fallback |
| 33 | `CLR_TIMEOUT` | [`--timeout`](param/036_timeout.md) | u32 | `"timeout"` | Applied when `--timeout` absent; `0` = unlimited (no watchdog); invalid values silently ignored. **Cross-command:** also applies to `isolated`/`refresh` via Section 2 (same semantics: `0` = unlimited) |
| 34 | `CLR_RETRY_ON_ACCOUNT` | [`--retry-on-account`](param/040_retry_on_account.md) | u8 | `"retry-on-account"` | Account class retry count (Tier 2); default auto → fallback |
| 35 | `CLR_ACCOUNT_DELAY` | [`--account-delay`](param/041_account_delay.md) | u32 | `"account-delay"` | Account class delay (Tier 2); default auto → fallback |
| 36 | `CLR_RETRY_ON_AUTH` | [`--retry-on-auth`](param/042_retry_on_auth.md) | u8 | `"retry-on-auth"` | Auth class retry count (Tier 2); default auto → fallback |
| 37 | `CLR_AUTH_DELAY` | [`--auth-delay`](param/043_auth_delay.md) | u32 | `"auth-delay"` | Auth class delay (Tier 2); default auto → fallback |
| 38 | `CLR_RETRY_ON_SERVICE` | [`--retry-on-service`](param/044_retry_on_service.md) | u8 | `"retry-on-service"` | Service class retry count (Tier 2); default auto → fallback |
| 39 | `CLR_SERVICE_DELAY` | [`--service-delay`](param/045_service_delay.md) | u32 | `"service-delay"` | Service class delay (Tier 2); default auto → fallback |
| 40 | `CLR_RETRY_ON_PROCESS` | [`--retry-on-process`](param/046_retry_on_process.md) | u8 | `"retry-on-process"` | Process class retry count (Tier 2); default auto → fallback |
| 41 | `CLR_PROCESS_DELAY` | [`--process-delay`](param/047_process_delay.md) | u32 | `"process-delay"` | Process class delay (Tier 2); default auto → fallback |
| 42 | `CLR_RETRY_ON_VALIDATION` | [`--retry-on-validation`](param/048_retry_on_validation.md) | u8 | `"retry-on-validation"` | Validation class retry count (Tier 2); invalid values rejected at parse time |
| 43 | `CLR_VALIDATION_DELAY` | [`--validation-delay`](param/049_validation_delay.md) | u32 | `"validation-delay"` | Validation class delay (Tier 2); default auto → fallback |
| 44 | `CLR_RETRY_ON_RUNNER` | [`--retry-on-runner`](param/050_retry_on_runner.md) | u8 | `"retry-on-runner"` | Runner class retry count (Tier 2); default auto → fallback |
| 45 | `CLR_RUNNER_DELAY` | [`--runner-delay`](param/051_runner_delay.md) | u32 | `"runner-delay"` | Runner class delay (Tier 2); default auto → fallback |
| 46 | `CLR_RETRY_ON_UNKNOWN` | [`--retry-on-unknown`](param/052_retry_on_unknown.md) | u8 | `"retry-on-unknown"` | Unknown class retry count (Tier 2); default auto → fallback |
| 47 | `CLR_UNKNOWN_DELAY` | [`--unknown-delay`](param/053_unknown_delay.md) | u32 | `"unknown-delay"` | Unknown class delay (Tier 2); default auto → fallback |
| 48 | `CLR_RETRY_OVERRIDE` | [`--retry-override`](param/054_retry_override.md) | u8 | `"retry-override"` | Tier 1: forces retry count for all error classes; default auto |
| 49 | `CLR_RETRY_OVERRIDE_DELAY` | [`--retry-override-delay`](param/055_retry_override_delay.md) | u32 | `"retry-override-delay"` | Tier 1: forces delay for all error classes; default auto |
| 50 | `CLR_RETRY_DEFAULT` | [`--retry-default`](param/056_retry_default.md) | u8 | `"retry-default"` | Tier 3: fallback retry count for all unset classes; default 2 |
| 51 | `CLR_RETRY_DEFAULT_DELAY` | [`--retry-default-delay`](param/057_retry_default_delay.md) | u32 | `"retry-default-delay"` | Tier 3: fallback delay for all unset classes; default 30 |
| 52 | `CLR_OUTPUT_FORMAT` | [`--output-format`](param/061_output_format.md) | string | `"output-format"` | Parsed as enum (`text`/`json`/`stream-json`); any string accepted (forwarded as-is to claude) |
| 53 | `CLR_MAX_TURNS` | [`--max-turns`](param/062_max_turns.md) | string | `"max-turns"` | Forwarded as-is to claude; no parse validation |
| 54 | `CLR_ALLOWED_TOOLS` | [`--allowed-tools`](param/063_allowed_tools.md) | string | `"allowed-tools"` | Forwarded as-is to claude |
| 55 | `CLR_DISALLOWED_TOOLS` | [`--disallowed-tools`](param/064_disallowed_tools.md) | string | `"disallowed-tools"` | Forwarded as-is to claude |
| 56 | `CLR_MAX_BUDGET_USD` | [`--max-budget-usd`](param/065_max_budget_usd.md) | string | `"max-budget-usd"` | Forwarded as-is to claude; no parse validation |
| 57 | `CLR_ADD_DIR` | [`--add-dir`](param/066_add_dir.md) | string | `"add-dir"` | Forwarded as-is to claude |
| 58 | `CLR_FALLBACK_MODEL` | [`--fallback-model`](param/067_fallback_model.md) | string | `"fallback-model"` | Forwarded as-is to claude |
| 59 | `CLR_OUTPUT_STYLE` | [`--output-style`](param/070_output_style.md) | string | `"output-style"` | Parsed as enum (`summary`/`raw`); invalid values exit 1 (not silently ignored) |
| 60 | `CLR_SUMMARY_FIELDS` | [`--summary-fields`](param/071_summary_fields.md) | string | `"summary-fields"` | Preset name (`minimal`/`standard`/`full`) or comma-separated field whitelist; invalid values exit 1 |
| 61 | `CLR_JOURNAL` | [`--journal`](param/072_journal.md) | string | `"journal"` | Parsed as enum (`full`/`meta`/`off`); invalid values exit 1; default `full` |
| 62 | `CLR_JOURNAL_DIR` | [`--journal-dir`](param/073_journal_dir.md) | string | `"journal-dir"` | Applied when `--journal-dir` absent; path to journal JSONL directory; default `~/.clr/journal/` |
| 63 | `CLR_ARGS_FILE` | [`--args-file`](param/075_args_file.md) | string | `"args-file"` | Path to a JSON config file; applied when `--args-file` absent from CLI; JSON source overrides all other CLR_* vars but is overridden by explicit CLI flags. **Cross-command:** applies to `run`, `ask`, `isolated`, and `refresh` subcommands |
| 64 | `CLR_NO_COMPACT_WINDOW` | `--no-compact-window` | bool | `"no-compact-window"` | Suppresses `CLAUDE_CODE_AUTO_COMPACT_WINDOW` injection — subprocess inherits caller env or uses model native window |

**Precedence (current — 5 levels):**

1. CLI flag (wins unconditionally when provided)
2. JSON config (from `--args-file`, `CLR_ARGS_FILE`, or stdin JSON pipe — see [feature/004_json_config.md](../feature/004_json_config.md))
3. `CLR_*` env var (applied when CLI field absent and no JSON source)
4. Config file — project `.clr.toml` (cwd), then user `~/.clr/config.toml` (or `$CLR_CONFIG_DIR/config.toml`); applied only when CLI, JSON, and env are all absent — see [config_param.md](config_param.md)
5. Built-in default

**Discovery:** Use `--dry-run` or `--trace` to see effective values after env var and config-file application.

```sh
CLR_MODEL=sonnet clr --dry-run "task"           # shows: claude --model sonnet ...
CLR_MODEL=sonnet clr --model opus --dry-run "task"  # CLI wins; CLR_MODEL ignored
```

---

### Env Param 2: CLR_* Input Parameters — `isolated` and `refresh` Subcommands

Environment variable fallbacks for the 13 credential operation parameters.
`apply_isolated_env_vars()` and `apply_refresh_env_vars()` in `src/cli/cred_parse.rs` read these
after subcommand argument parsing.

| # | Variable | CLI Parameter | Type | JSON Key | Notes |
|---|----------|---------------|------|----------|-------|
| 1 | `CLR_CREDS` | [`--creds`](param/019_creds.md) | string | `"creds"` | Applied when `--creds` absent (`creds_path` is empty string) |
| 2 | `CLR_TIMEOUT` | [`--timeout`](param/020_timeout.md) | u64 | `"timeout"` | Applied when CLI timeout equals its command default (30 for `isolated`, 45 for `refresh`); `0` = unlimited (no watchdog), matching `run`/`ask` semantics. Also applies to `run`/`ask` via Section 1 row 34 |
| 3 | `CLR_TRACE` | [`--trace`](param/013_trace.md) | bool | `"trace"` | Applied when `--trace` absent; also applies to `run` via Section 1 |
| 4 | `CLR_DIR` | [`--dir`](param/008_dir.md) | string | `"dir"` | Applied when `--dir` absent; isolated only (refresh has no `--dir`) |
| 5 | `CLR_ADD_DIR` | [`--add-dir`](param/066_add_dir.md) | string | `"add-dir"` | Applied when `--add-dir` absent; single value (not comma-split); isolated only |
| 6 | `CLR_JOURNAL` | [`--journal`](param/072_journal.md) | string | `"journal"` | Applied when `--journal` absent; invalid values exit 1 |
| 7 | `CLR_JOURNAL_DIR` | [`--journal-dir`](param/073_journal_dir.md) | string | `"journal-dir"` | Applied when `--journal-dir` absent |
| 8 | `CLR_OUTPUT_FILE` | [`--output-file`](param/029_output_file.md) | string | `"output-file"` | Applied when `--output-file` absent; isolated only |
| 9 | `CLR_STRIP_FENCES` | [`--strip-fences`](param/026_strip_fences.md) | bool | `"strip-fences"` | Applied when `--strip-fences` absent; isolated only |
| 10 | `CLR_OUTPUT_STYLE` | [`--output-style`](param/070_output_style.md) | string | `"output-style"` | Applied when `--output-style` absent; isolated only |
| 11 | `CLR_SUMMARY_FIELDS` | [`--summary-fields`](param/071_summary_fields.md) | string | `"summary-fields"` | Applied when `--summary-fields` absent; isolated only |
| 12 | `CLR_NO_COMPACT_WINDOW` | `--no-compact-window` | bool | `"no-compact-window"` | Suppresses `CLAUDE_CODE_AUTO_COMPACT_WINDOW` injection; applies to both `isolated` and `refresh`. Cross-command: also applies to `run` via Section 1 row 64 |
| 13 | `CLR_DRY_RUN` | [`--dry-run`](param/011_dry_run.md) | bool | `"dry-run"` | Applied when `--dry-run` absent; `refresh` only (isolated uses CLI flag; run via Section 1 row 11) |

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

### Env Param 3: CLR_* Input Parameters — `ps` Subcommand

Environment variable fallbacks for the 5 `ps` session listing display and flag-threshold parameters.
`dispatch_ps()` in `src/cli/ps.rs` reads these before table rendering.
Each variable is applied **only when the corresponding CLI flag is absent** — the
CLI flag always wins when both are present.

`ps` subcommand parameters are not configurable via `--args-file` / JSON config; `CLR_PS_*` env vars apply only to the `ps` subcommand.

| # | Variable | CLI Parameter | Type | Notes |
|---|----------|---------------|------|-------|
| 1 | `CLR_PS_MODE` | [`--mode`](param/058_mode.md) | string | Parsed as enum (`all`/`interactive`/`print`); invalid values → exit 1 |
| 2 | `CLR_PS_COLUMNS` | [`--columns`](param/059_columns.md) | string | Comma-separated column keys; applied when `--columns` absent |
| 3 | `CLR_PS_PID` | [`--pid`](param/068_pid.md) | string | Comma-separated numeric PIDs; non-numeric entries silently ignored; applied when `--pid` absent |
| 4 | `CLR_PS_ANCIENT_SECS` | _(no CLI flag)_ | u64 | Elapsed-seconds threshold above which the 🕰 (Ancient) flag is shown; default `28800` (8 h); invalid values silently ignored |
| 5 | `CLR_PS_HIGH_RAM_MB` | _(no CLI flag)_ | u64 | RSS megabyte threshold above which the 🐘 (High RAM) flag is shown; default `400`; invalid values silently ignored |

**Precedence:**

1. CLI flag (wins unconditionally when provided)
2. `CLR_PS_*` env var (applied when CLI field absent)
3. Built-in default (`all` for mode, 9 default columns for columns, no PID filter, 28800 for ancient threshold, 400 for high-RAM threshold)

**Note:** `--wide` has no env var equivalent — it is a convenience shorthand only.
When both `CLR_PS_COLUMNS` and `--wide` are active, `CLR_PS_COLUMNS` wins (explicit
column selection overrides the convenience flag).

**Note (`CLR_PS_ANCIENT_SECS` / `CLR_PS_HIGH_RAM_MB`):** These variables have no corresponding
CLI flags. They are the only mechanism to override the flag thresholds. Invalid values (non-numeric,
overflow) are silently ignored — the built-in defaults are used instead.

---

### Env Param 4: `CLAUDE_CODE_MAX_OUTPUT_TOKENS`

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

---

### Env Param 5: Gate Runtime Configuration

Runtime configuration overrides for the `--max-sessions` concurrency gate (`gate.rs`). None
of the four variables has a corresponding CLI flag or `--args-file` JSON key — env-var-only,
matching the `CLR_PS_ANCIENT_SECS`/`CLR_PS_HIGH_RAM_MB` precedent (Env Param 3).

| Variable | Default | Type | Notes |
|----------|---------|------|-------|
| `CLR_GATE_DIR` | `/tmp/clr-gate` | path | Gate state directory; read by `gate_dir()` in `gate.rs`, called directly by `ps.rs` |
| `CLR_GATE_POLL_SECS` | `30` | u64 | Poll interval between gate attempts; read by `gate_poll_secs()` in `gate.rs`; invalid values silently fall back to `30` |
| `CLR_GATE_MAX_ATTEMPTS` | `1000` | u32 | Attempt limit before gate exhaustion; read by `gate_max_attempts()` in `gate.rs`; invalid values silently fall back to `1000` |
| `CLR_GATE_STALE_SECS` | unset (`None`) | u64, optional | Staleness threshold for reclaiming a live-but-stalled slot owner; read by `gate_stale_secs()` in `gate.rs`; unset or invalid values resolve to `None` (feature off) — a live owner denies unconditionally, exactly as before this variable existed; never a numeric fallback (BUG-400) |

**`CLR_GATE_DIR`:** Overrides the default gate state directory used by `gate.rs` (write) and
`ps.rs` (read). When a `clr` process is blocked at the `--max-sessions` concurrency gate,
`gate.rs` writes a JSON state file to `$CLR_GATE_DIR/{pid}.json`. `clr ps` reads those files
to populate the queued CLR processes table. Primary use: test isolation — override in tests
to point at a temp dir, preventing cross-test contamination from real gate files in
`/tmp/clr-gate/`.

**`CLR_GATE_POLL_SECS` / `CLR_GATE_MAX_ATTEMPTS`:** Override the gate's poll interval and
attempt limit (production default: 30s x 1000 attempts). `clr` sleeps `poll_secs` between
attempts but **not after the final attempt** — an `N`-attempt sequence elapses `(N-1) *
poll_secs` seconds before the gate-exhaustion path fires, since there is no reason to sleep
immediately before giving up. Exhaustion is then subject to further Runner-class retry via
`--retry-on-runner`/`--retry-override` (see [param/033_max_sessions.md](param/033_max_sessions.md)
and [param/054_retry_override.md](param/054_retry_override.md)) before `clr` actually exits.
Primary use: automation pipelines that want the gate to fail fast instead of waiting up to
~500 minutes (999 x 30s) for the production defaults.

```sh
CLR_GATE_POLL_SECS=5 CLR_GATE_MAX_ATTEMPTS=12 clr --max-sessions 1 --retry-override 0 "task"
# gate exhausts after ~55s (11 sleeps x 5s) instead of ~29970s (999 x 30s); --retry-override 0
# disables the runner-retry wrapper so exhaustion surfaces on the first pass
```

**`CLR_GATE_STALE_SECS`:** Opt-in staleness threshold for reclaiming a slot from a live-but-stalled
owner (hung, deadlocked, or `SIGSTOP`ped — a process that never releases its slot but is not dead
either), checked in `acquire_slot()` alongside `pid_alive(owner)`. Unset by default: `is_stale` is
always `false`, so a live owner denies reclaim unconditionally, exactly as before this variable
existed. When set, a slot whose recorded owner is still alive but whose `since` timestamp exceeds
this many seconds becomes reclaim-eligible — it falls through into the same ticket-arbitrated
handoff a dead owner's slot already uses (no separate mechanism), closing the gap where a
hung-but-alive session could hold its slot forever with no escape hatch (see
`docs/invariant/012_gate_slot_atomicity.md` Provenance : BUG-400). Unlike `CLR_GATE_POLL_SECS` /
`CLR_GATE_MAX_ATTEMPTS`, an invalid value does **not** fall back to a numeric default — it resolves
to `None`, the same as unset, because there is no safe universal staleness threshold that would not
risk reclaiming a legitimately long-running session.

```sh
CLR_GATE_STALE_SECS=1800 clr --max-sessions 3 "task"
# a slot owner still alive after 30 minutes becomes reclaim-eligible instead of blocking
# indefinitely; a fresher owner (elapsed <= 1800s) is unaffected and still denies normally
```

**Commands affected:** `run` / `ask` only (`gate.rs` is invoked only from those two commands)
— `CLR_GATE_DIR` is the sole exception, additionally read by `ps` for display.

**No precedence rule** — all four variables are always applied; there is no corresponding
CLI flag or JSON key for any of them. `CLR_GATE_STALE_SECS` differs only in that its
"always applied" default is the disabled state (`None`), not a numeric fallback.

---

### Env Param 6: `CLAUDE_CODE_AUTO_COMPACT_WINDOW` — Injected Default

Set by `clr` before spawning any `claude` subprocess to cap the auto-compaction threshold at
300 000 tokens. This prevents extended-context models from accumulating a 1M-token context
window and triggering expensive compaction late in long sessions.

- **Type:** integer (tokens)
- **Injected value:** `300 000` (all 4 running commands: `run`, `ask`, `isolated`, `refresh`)
- **Available since:** Claude Code v2.1.75 (2026-03-13)
- **Mechanism:** injected via `ClaudeCommand::new()` default in `claude_runner_core/src/command/mod.rs`
- **Scope:** `claude` subprocess only; not read by `clr` itself
- **Opt-out:** `--no-compact-window` / `CLR_NO_COMPACT_WINDOW=1` — suppresses injection; subprocess inherits caller env or uses model native window

**Effect:** When the subprocess conversation approaches this token count (as a percentage governed
by `CLAUDE_AUTOCOMPACT_PCT_OVERRIDE`), Claude Code compacts the context automatically. The
injected 300k cap prevents runaway context growth on extended-context models (e.g., 1M-window
Opus), without affecting the hard per-turn output limit set by `CLAUDE_CODE_MAX_OUTPUT_TOKENS`.

**Limitation:** Only meaningful for multi-turn interactive sessions that accumulate context.
Single-turn `--print` mode runs never trigger compaction regardless of this setting, because
context does not persist between invocations.

| Variable | Injected value | Opt-out | Notes |
|----------|----------------|---------|-------|
| `CLAUDE_CODE_AUTO_COMPACT_WINDOW` | `300 000` | `--no-compact-window` / `CLR_NO_COMPACT_WINDOW` | Caps compaction threshold, not hard context |

**Cross-reference:** `contract/claude_code/docs/param/074_auto_compact_window.md` — canonical
parameter spec (type, default, since, description, behavioral contract).

---

### Env Param 7: `CLR_CONFIG_DIR` — Config File Discovery

Overrides the default user-level config directory used by `config.rs` when discovering
`config.toml`. Mirrors the `CLR_GATE_DIR` pattern (Env Param 5): a test-injection point,
not a user-facing feature in its own right.

- **Type:** directory path (string)
- **Default:** `$HOME/.clr`
- **Commands affected:** `run` / `ask` (`load_config()` in `config.rs` is called only from
  `dispatch_run()`); project-level `.clr.toml` discovery in the current directory is
  unaffected by this variable
- **Mechanism:** read by `user_config_dir()` in `config.rs`; an unset or empty value falls
  back to `$HOME/.clr`
- **Primary use:** test isolation — override in tests to point user-level config discovery
  at a temp dir, preventing cross-test contamination from a real `~/.clr/config.toml`

| Variable | Default | Type | Notes |
|----------|---------|------|-------|
| `CLR_CONFIG_DIR` | `$HOME/.clr` | path | Overrides user-level `config.toml` directory for `config.rs`; project `.clr.toml` discovery is separate and unaffected |

**No precedence rule** — this variable is always applied (there is no corresponding CLI flag).

**Full parameter list, TOML key reference, and the config-file precedence tier itself:**
see [config_param.md](config_param.md).

---

### Env Param 8: `CLAUDE_CODE_BASH_TIMEOUT` — Injected Default

Set by `clr` before spawning any `claude` subprocess. Raises the default
per-command Bash tool timeout from the binary's own 2-minute default to 1 hour.

- **Type:** integer (milliseconds)
- **Injected value:** `3 600 000` (1 hour) — field `bash_default_timeout_ms` in `ClaudeCommand::new()`
- **Binary default (unset):** `120 000` (2 minutes)
- **Mechanism:** injected via `env_pairs()` in `claude_runner_core/src/command/mod.rs`
- **Scope:** subprocess-only; not read by `clr` itself
- **Rationale (source comment):** standard 2-minute default causes premature
  timeout in real automation workflows

| Variable | Injected value | Binary default | Opt-out |
|----------|-----------------|-----------------|---------|
| `CLAUDE_CODE_BASH_TIMEOUT` | `3 600 000` | `120 000` | none exposed — always injected |

**Cross-reference:** `contract/claude_code/docs/param/013_bash_timeout.md` — canonical parameter spec.

---

### Env Param 9: `CLAUDE_CODE_BASH_MAX_TIMEOUT` — Injected Default

Set by `clr` before spawning any `claude` subprocess. Raises the ceiling on
model-requested Bash timeouts from the binary's own 10-minute default to 2 hours.

- **Type:** integer (milliseconds)
- **Injected value:** `7 200 000` (2 hours) — field `bash_max_timeout_ms` in `ClaudeCommand::new()`
- **Binary default (unset):** `600 000` (10 minutes)
- **Mechanism:** injected via `env_pairs()` in `claude_runner_core/src/command/mod.rs`
- **Scope:** subprocess-only; not read by `clr` itself

| Variable | Injected value | Binary default | Opt-out |
|----------|-----------------|-----------------|---------|
| `CLAUDE_CODE_BASH_MAX_TIMEOUT` | `7 200 000` | `600 000` | none exposed — always injected |

**Cross-reference:** `contract/claude_code/docs/param/012_bash_max_timeout.md` — canonical parameter spec.

---

### Env Param 10: `CLAUDE_CODE_PRINT_BG_WAIT_CEILING_MS` — Injected Default

Set by `clr` before spawning any `claude` subprocess in print mode. Disables
the binary's own internal ceiling-exceeded sweep for outstanding background
tasks.

- **Type:** integer (milliseconds)
- **Injected value:** `0` — field `print_bg_wait_ceiling_ms` in `ClaudeCommand::new()`
- **Binary default (unset):** `600 000` (10 minutes)
- **Mechanism:** injected via `env_pairs()` in `claude_runner_core/src/command/mod.rs`
- **Scope:** subprocess-only; not read by `clr` itself

**What `0` actually does (do not assume "exit immediately"):** the binary's
internal `ceilingExceeded` computation requires the ceiling to be `> 0` as a
precondition — `0` fails that guard permanently, which disables the
ceiling-based forced sweep rather than triggering it instantly. See
`contract/claude_code/docs/param/131_print_bg_wait_ceiling_ms.md` for the
full decompiled evidence, and `docs/claude_code_background_task_env_vars.md`
(repo root) for independent, empirically-tested (live-process) confirmation
that `0` behaves as "wait indefinitely" for this path. A separate, fixed
~5-second grace period governs plain background Bash tasks regardless of
this setting.

**Relationship to clr's own timeout layer:** this is an *inner* layer —
claude's own internal wait logic for background work during its print-mode
wind-down. It is independent of clr's *outer* layer:
[`run_print_mode()`'s own 1-hour watchdog](../invariant/007_print_mode_timeout.md)
(`DEFAULT_PRINT_TIMEOUT_SECS`), which unconditionally kills the entire
`claude` subprocess after `--timeout`/`CLR_TIMEOUT` (default 3600s) regardless
of what the inner ceiling is doing. Setting
`CLAUDE_CODE_PRINT_BG_WAIT_CEILING_MS=0` does not remove clr's outer bound —
a genuinely long-running background agent can still be killed by the outer
watchdog even though the inner ceiling has been neutralized. For a session
expected to run background work past 1 hour, `--timeout 0` /
`CLR_TIMEOUT=0` (unlimited) is the relevant lever, not this variable.

| Variable | Injected value | Binary default | Opt-out |
|----------|-----------------|-----------------|---------|
| `CLAUDE_CODE_PRINT_BG_WAIT_CEILING_MS` | `0` | `600 000` | none exposed — override the outer bound instead via `--timeout 0` / `CLR_TIMEOUT=0` |

**Cross-reference:** `contract/claude_code/docs/param/131_print_bg_wait_ceiling_ms.md`
— canonical parameter spec (includes the corrected `ra>0` guard evidence).
[`invariant/007_print_mode_timeout.md`](../invariant/007_print_mode_timeout.md)
— clr's own outer watchdog, the more likely cause of "background work
dropped before finishing" for genuinely long-running tasks.

### Provenance

| File | Notes |
|------|-------|
| [../env_param.md](env_param.md) | Original un-migrated source; retained as reference |
