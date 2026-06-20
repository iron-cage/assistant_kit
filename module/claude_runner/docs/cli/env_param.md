# CLI: Environment Parameters

### Scope

- **Purpose**: Document CLR_* environment variable fallbacks, runtime configuration overrides, and the CLAUDE_CODE_MAX_OUTPUT_TOKENS subprocess variable.
- **Responsibility**: Specify env var names, corresponding CLI parameters, precedence rules, and type handling.
- **In Scope**: CLR_* input vars for run/isolated/refresh, CLR_* runtime config overrides (`CLR_GATE_DIR`), CLAUDE_CODE_MAX_OUTPUT_TOKENS injection, precedence, bool/parsed type semantics.
- **Out of Scope**: CLI parameter descriptions (-> param/), subprocess behavior beyond env injection.

### All Env Parameters (68 total)

| Category | Count | Purpose |
|----------|-------|---------|
| Input (CLR_*) — `run` subcommand | 58 | Caller env fallbacks for `run` parameters |
| Input (CLR_*) — `isolated` and `refresh` subcommands | 3 | Caller env fallbacks for credential operation parameters |
| Input (CLR_*) — `ps` subcommand | 5 | Caller env fallbacks for session listing display and flag thresholds |
| Runtime config (CLR_*) | 1 | Runtime configuration overrides (not CLI parameter fallbacks) |
| Subprocess (CLAUDE_CODE_*) | 1 | Set by `clr` before spawning the `claude` subprocess |

**Total:** 68 environment variables

---

### Env Param 1: CLR_* Input Parameters — `run` Subcommand

Environment variable fallbacks for all 58 `run` subcommand parameters.
`apply_env_vars()` in `src/cli/env.rs` reads these immediately after CLI parsing, before command
dispatch. Each variable is applied **only when the corresponding CLI field is still at its
zero/absent value** — the CLI flag always wins when both are present.

**Bool variables** accept `"1"` or `"true"` (case-insensitive) as truthy.
Any other value — including `"yes"`, `"0"`, `"false"`, empty, or absent — resolves to `false`.

**Parsed variables** (`CLR_MAX_TOKENS`, `CLR_VERBOSITY`, `CLR_EFFORT`, `CLR_TIMEOUT`, and all `CLR_RETRY_*` / `CLR_*_DELAY` variables) silently ignore
invalid values (parse failure → field stays at default). Exception: `CLR_RETRY_ON_VALIDATION` rejects invalid values at parse time.

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
| 30 | `CLR_MAX_SESSIONS` | [`--max-sessions`](param/033_max_sessions.md) | u32 | Applied when `--max-sessions` absent; invalid values silently ignored (parse failure → field stays at default 30) |
| 31 | `CLR_RETRY_ON_TRANSIENT` | [`--retry-on-transient`](param/034_retry_on_transient.md) | u8 | Transient class retry count (Tier 2); default auto → fallback |
| 32 | `CLR_TRANSIENT_DELAY` | [`--transient-delay`](param/035_transient_delay.md) | u32 | Transient class delay (Tier 2); default auto → fallback |
| 33 | `CLR_TIMEOUT` | [`--timeout`](param/036_timeout.md) | u32 | Applied when `--timeout` absent; `0` = unlimited (no watchdog); invalid values silently ignored. **Cross-command:** also applies to `isolated`/`refresh` via Section 2 (same semantics: `0` = unlimited) |
| 34 | `CLR_RETRY_ON_ACCOUNT` | [`--retry-on-account`](param/040_retry_on_account.md) | u8 | Account class retry count (class default = 0; opt-in only) |
| 35 | `CLR_ACCOUNT_DELAY` | [`--account-delay`](param/041_account_delay.md) | u32 | Account class delay (Tier 2); default auto → fallback |
| 36 | `CLR_RETRY_ON_AUTH` | [`--retry-on-auth`](param/042_retry_on_auth.md) | u8 | Auth class retry count (Tier 2); default auto → fallback |
| 37 | `CLR_AUTH_DELAY` | [`--auth-delay`](param/043_auth_delay.md) | u32 | Auth class delay (Tier 2); default auto → fallback |
| 38 | `CLR_RETRY_ON_SERVICE` | [`--retry-on-service`](param/044_retry_on_service.md) | u8 | Service class retry count (Tier 2); default auto → fallback |
| 39 | `CLR_SERVICE_DELAY` | [`--service-delay`](param/045_service_delay.md) | u32 | Service class delay (Tier 2); default auto → fallback |
| 40 | `CLR_RETRY_ON_PROCESS` | [`--retry-on-process`](param/046_retry_on_process.md) | u8 | Process class retry count (Tier 2); default auto → fallback |
| 41 | `CLR_PROCESS_DELAY` | [`--process-delay`](param/047_process_delay.md) | u32 | Process class delay (Tier 2); default auto → fallback |
| 42 | `CLR_RETRY_ON_VALIDATION` | [`--retry-on-validation`](param/048_retry_on_validation.md) | u8 | Validation class retry count (Tier 2); invalid values rejected at parse time |
| 43 | `CLR_VALIDATION_DELAY` | [`--validation-delay`](param/049_validation_delay.md) | u32 | Validation class delay (Tier 2); default auto → fallback |
| 44 | `CLR_RETRY_ON_RUNNER` | [`--retry-on-runner`](param/050_retry_on_runner.md) | u8 | Runner class retry count (Tier 2); default auto → fallback |
| 45 | `CLR_RUNNER_DELAY` | [`--runner-delay`](param/051_runner_delay.md) | u32 | Runner class delay (Tier 2); default auto → fallback |
| 46 | `CLR_RETRY_ON_UNKNOWN` | [`--retry-on-unknown`](param/052_retry_on_unknown.md) | u8 | Unknown class retry count (Tier 2); default auto → fallback |
| 47 | `CLR_UNKNOWN_DELAY` | [`--unknown-delay`](param/053_unknown_delay.md) | u32 | Unknown class delay (Tier 2); default auto → fallback |
| 48 | `CLR_RETRY_OVERRIDE` | [`--retry-override`](param/054_retry_override.md) | u8 | Tier 1: forces retry count for all error classes; default auto |
| 49 | `CLR_RETRY_OVERRIDE_DELAY` | [`--retry-override-delay`](param/055_retry_override_delay.md) | u32 | Tier 1: forces delay for all error classes; default auto |
| 50 | `CLR_RETRY_DEFAULT` | [`--retry-default`](param/056_retry_default.md) | u8 | Tier 3: fallback retry count for all unset classes; default 2 |
| 51 | `CLR_RETRY_DEFAULT_DELAY` | [`--retry-default-delay`](param/057_retry_default_delay.md) | u32 | Tier 3: fallback delay for all unset classes; default 30 |
| 52 | `CLR_OUTPUT_FORMAT` | [`--output-format`](param/061_output_format.md) | string | Parsed as enum (`text`/`json`/`stream-json`); any string accepted (forwarded as-is to claude) |
| 53 | `CLR_MAX_TURNS` | [`--max-turns`](param/062_max_turns.md) | string | Forwarded as-is to claude; no parse validation |
| 54 | `CLR_ALLOWED_TOOLS` | [`--allowed-tools`](param/063_allowed_tools.md) | string | Forwarded as-is to claude |
| 55 | `CLR_DISALLOWED_TOOLS` | [`--disallowed-tools`](param/064_disallowed_tools.md) | string | Forwarded as-is to claude |
| 56 | `CLR_MAX_BUDGET_USD` | [`--max-budget-usd`](param/065_max_budget_usd.md) | string | Forwarded as-is to claude; no parse validation |
| 57 | `CLR_ADD_DIR` | [`--add-dir`](param/066_add_dir.md) | string | Forwarded as-is to claude |
| 58 | `CLR_FALLBACK_MODEL` | [`--fallback-model`](param/067_fallback_model.md) | string | Forwarded as-is to claude |

**Precedence (current — 3 tiers):**

1. CLI flag (wins unconditionally when provided)
2. `CLR_*` env var (applied when CLI field is absent/at default)
3. Built-in default

**Precedence (target — 4 tiers):**

1. CLI flag (highest — wins unconditionally)
2. `CLR_*` env var (applied when env var set and CLI field absent)
3. Config file (applied when env var absent) — **not yet implemented**
4. Built-in default (lowest)

Config file tier design: keys use `snake_case` matching CLI `--kebab-case` names (e.g., `retry_on_transient = 2`). File path TBD — candidates: `~/.config/clr/config.toml`, `$CLR_CONFIG` override, `.clr.toml` (project-local). All parameters should be configurable at the config file tier. See [`type/14_error_class.md`](type/14_error_class.md) § Configuration Tiers for the full gap analysis.

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

### Env Param 3: CLR_* Input Parameters — `ps` Subcommand

Environment variable fallbacks for the 5 `ps` session listing display and flag-threshold parameters.
`dispatch_ps()` in `src/cli/ps.rs` reads these before table rendering.
Each variable is applied **only when the corresponding CLI flag is absent** — the
CLI flag always wins when both are present.

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

### Env Param 5: `CLR_GATE_DIR` — Runtime Configuration

Overrides the default gate state directory used by `gate.rs` (write) and `ps.rs` (read).

When a `clr` process is blocked at the `--max-sessions` concurrency gate, `gate.rs` writes
a JSON state file to `$CLR_GATE_DIR/{pid}.json`. `clr ps` reads those files to populate the
queued CLR processes table.

- **Type:** directory path (string)
- **Default:** `/tmp/clr-gate`
- **Commands affected:** `run` / `ask` (writes gate files via `gate.rs`), `ps` (reads gate files)
- **Mechanism:** read by `gate_dir()` in `gate.rs` and `gate_dir_ps()` in `ps.rs` at runtime
- **Primary use:** test isolation — override in tests to point at a temp dir, preventing
  cross-test contamination from real gate files in `/tmp/clr-gate/`

| Variable | Default | Type | Notes |
|----------|---------|------|-------|
| `CLR_GATE_DIR` | `/tmp/clr-gate` | path | Override gate state directory for `gate.rs` and `ps.rs` |

**No precedence rule** — this variable is always applied (there is no corresponding CLI flag).
