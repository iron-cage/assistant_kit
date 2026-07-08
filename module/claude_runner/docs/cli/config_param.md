# CLI: Config File Parameters

### Scope

- **Purpose**: Document the config-file parameter tier (`~/.clr/config.toml` user-level, `.clr.toml` project-level) — the 4th of 5 levels in the CLI parameter resolution chain.
- **Responsibility**: Specify the eligible parameter set, TOML key names, file discovery rules, precedence, and error handling.
- **In Scope**: `ConfigDefaults` eligible-parameter enumeration, TOML key reference, discovery order (project > user), `CLR_CONFIG_DIR` test-injection override, example `config.toml`, error handling (malformed TOML, missing file, unknown key).
- **Out of Scope**: CLI parameter semantics (→ `param/`), `CLR_*` env var fallbacks (→ [003_env_param.md](003_env_param.md)), JSON `--args-file` config (→ [../feature/004_json_config.md](../feature/004_json_config.md)), `isolated`/`refresh`/`ps` config support (not implemented — separate dispatch paths, revisit only if a concrete need arises).

### Discovery & Precedence

`clr run` / `clr ask` read config defaults from up to two TOML files, in `load_config()` (`src/cli/config.rs`), called from `dispatch_run()` immediately after `apply_env_vars()`:

1. **Project-level**: `.clr.toml` in the current working directory
2. **User-level**: `config.toml` under `$CLR_CONFIG_DIR` (if set and non-empty), else `$HOME/.clr`

Both files are optional. A missing file at either location is silently treated as empty — no error. When both files set the same key, **the project file wins**. Merging happens on raw TOML tables before the typed deserialize step, so an absent key is never confused with an explicit `false` for `bool` fields.

**Full 5-level precedence chain:**

1. CLI flag (wins unconditionally when provided)
2. `--args-file` / `CLR_ARGS_FILE` JSON config (see [../feature/004_json_config.md](../feature/004_json_config.md))
3. `CLR_*` env var (see [003_env_param.md](003_env_param.md))
4. **Config file** — project `.clr.toml`, then user `config.toml` (this document)
5. Built-in default

A config-file value is applied only when the corresponding field is still unset after levels 1–3 — the exact same fill-only-if-unset guard used by `apply_env_vars()` and JSON config application.

**Discovery:** Use `--dry-run` or `--trace` to see effective values after config-file application — no `--dry-run`-specific code exists for this tier; it is a natural consequence of `apply_config_defaults()` running before the existing dry-run print step.

```sh
echo 'model = "claude-opus-4-8"' > .clr.toml
clr --dry-run "task"                 # shows: claude --model claude-opus-4-8 ...
clr --model sonnet --dry-run "task"  # CLI wins; config value ignored
```

### Eligibility Rule

A parameter qualifies for the config-file tier if its value is a stable, repeatable-choice default — the kind of setting a user or project would want to set once and reuse across invocations. A parameter is excluded if its value is inherently specific to a single invocation (see [Not Configurable](#not-configurable) below). The eligible set is a closed, explicitly enumerated list (this document) — not automatically every `CliArgs` field.

### Eligible Parameters (38 total)

TOML keys are **snake_case**, matching `ConfigDefaults` struct field names exactly (unlike JSON `--args-file` keys, which are kebab-case matching CLI flag names).

**Model and Effort**

| TOML Key | CLI Flag | CLR_* Env Var | Type | Notes |
|----------|----------|---------------|------|-------|
| `model` | [`--model`](param/003_model.md) | `CLR_MODEL` | string | |
| `max_tokens` | [`--max-tokens`](param/009_max_tokens.md) | `CLR_MAX_TOKENS` | u32 | |
| `effort` | [`--effort`](param/017_effort.md) | `CLR_EFFORT` | string | Parsed as `EffortLevel` (`low`/`medium`/`high`/`max`); invalid values silently ignored |
| `no_effort_max` | [`--no-effort-max`](param/018_no_effort_max.md) | `CLR_NO_EFFORT_MAX` | bool | |
| `fallback_model` | [`--fallback-model`](param/067_fallback_model.md) | `CLR_FALLBACK_MODEL` | string | |

**Concurrency**

| TOML Key | CLI Flag | CLR_* Env Var | Type | Notes |
|----------|----------|---------------|------|-------|
| `max_sessions` | [`--max-sessions`](param/033_max_sessions.md) | `CLR_MAX_SESSIONS` | u32 | |

**Retry — Tier 2 (per-class)**

| TOML Key | CLI Flag | CLR_* Env Var | Type |
|----------|----------|---------------|------|
| `retry_on_transient` | [`--retry-on-transient`](param/034_retry_on_transient.md) | `CLR_RETRY_ON_TRANSIENT` | u8 |
| `transient_delay` | [`--transient-delay`](param/035_transient_delay.md) | `CLR_TRANSIENT_DELAY` | u32 |
| `retry_on_account` | [`--retry-on-account`](param/040_retry_on_account.md) | `CLR_RETRY_ON_ACCOUNT` | u8 |
| `account_delay` | [`--account-delay`](param/041_account_delay.md) | `CLR_ACCOUNT_DELAY` | u32 |
| `retry_on_auth` | [`--retry-on-auth`](param/042_retry_on_auth.md) | `CLR_RETRY_ON_AUTH` | u8 |
| `auth_delay` | [`--auth-delay`](param/043_auth_delay.md) | `CLR_AUTH_DELAY` | u32 |
| `retry_on_service` | [`--retry-on-service`](param/044_retry_on_service.md) | `CLR_RETRY_ON_SERVICE` | u8 |
| `service_delay` | [`--service-delay`](param/045_service_delay.md) | `CLR_SERVICE_DELAY` | u32 |
| `retry_on_process` | [`--retry-on-process`](param/046_retry_on_process.md) | `CLR_RETRY_ON_PROCESS` | u8 |
| `process_delay` | [`--process-delay`](param/047_process_delay.md) | `CLR_PROCESS_DELAY` | u32 |
| `retry_on_validation` | [`--retry-on-validation`](param/048_retry_on_validation.md) | `CLR_RETRY_ON_VALIDATION` | u8 |
| `validation_delay` | [`--validation-delay`](param/049_validation_delay.md) | `CLR_VALIDATION_DELAY` | u32 |
| `retry_on_runner` | [`--retry-on-runner`](param/050_retry_on_runner.md) | `CLR_RETRY_ON_RUNNER` | u8 |
| `runner_delay` | [`--runner-delay`](param/051_runner_delay.md) | `CLR_RUNNER_DELAY` | u32 |
| `retry_on_unknown` | [`--retry-on-unknown`](param/052_retry_on_unknown.md) | `CLR_RETRY_ON_UNKNOWN` | u8 |
| `unknown_delay` | [`--unknown-delay`](param/053_unknown_delay.md) | `CLR_UNKNOWN_DELAY` | u32 |

**Retry — Tier 1 (override) and Tier 3 (default)**

| TOML Key | CLI Flag | CLR_* Env Var | Type | Notes |
|----------|----------|---------------|------|-------|
| `retry_override` | [`--retry-override`](param/054_retry_override.md) | `CLR_RETRY_OVERRIDE` | u8 | Forces retry count for all error classes |
| `retry_override_delay` | [`--retry-override-delay`](param/055_retry_override_delay.md) | `CLR_RETRY_OVERRIDE_DELAY` | u32 | Forces delay for all error classes |
| `retry_default` | [`--retry-default`](param/056_retry_default.md) | `CLR_RETRY_DEFAULT` | u8 | Fallback retry count for all unset classes |
| `retry_default_delay` | [`--retry-default-delay`](param/057_retry_default_delay.md) | `CLR_RETRY_DEFAULT_DELAY` | u32 | Fallback delay for all unset classes |

**Timeout**

| TOML Key | CLI Flag | CLR_* Env Var | Type |
|----------|----------|---------------|------|
| `timeout` | [`--timeout`](param/036_timeout.md) | `CLR_TIMEOUT` | u32 |

**Output**

| TOML Key | CLI Flag | CLR_* Env Var | Type | Notes |
|----------|----------|---------------|------|-------|
| `output_style` | [`--output-style`](param/070_output_style.md) | `CLR_OUTPUT_STYLE` | string | `summary`/`raw`; invalid values exit 1 |
| `summary_fields` | [`--summary-fields`](param/071_summary_fields.md) | `CLR_SUMMARY_FIELDS` | string | Preset name or comma-separated field whitelist; invalid values exit 1 |
| `quiet` | [`--quiet`](param/074_quiet.md) | `CLR_QUIET` | bool | |

**Journal**

| TOML Key | CLI Flag | CLR_* Env Var | Type | Notes |
|----------|----------|---------------|------|-------|
| `journal` | [`--journal`](param/072_journal.md) | `CLR_JOURNAL` | string | `full`/`meta`/`off`; invalid values exit 1 |
| `journal_dir` | [`--journal-dir`](param/073_journal_dir.md) | `CLR_JOURNAL_DIR` | string | |

**Session Behavior**

| TOML Key | CLI Flag | CLR_* Env Var | Type | Notes |
|----------|----------|---------------|------|-------|
| `no_chrome` | [`--no-chrome`](param/021_no_chrome.md) | `CLR_NO_CHROME` | bool | |
| `no_persist` | [`--no-persist`](param/022_no_persist.md) | `CLR_NO_PERSIST` | bool | |
| `no_compact_window` | [`--no-compact-window`](param/077_no_compact_window.md) | `CLR_NO_COMPACT_WINDOW` | bool | Suppresses `CLAUDE_CODE_AUTO_COMPACT_WINDOW` injection |

**Tools and Budget**

| TOML Key | CLI Flag | CLR_* Env Var | Type | Notes |
|----------|----------|---------------|------|-------|
| `allowed_tools` | [`--allowed-tools`](param/063_allowed_tools.md) | `CLR_ALLOWED_TOOLS` | string | Forwarded as-is to claude |
| `disallowed_tools` | [`--disallowed-tools`](param/064_disallowed_tools.md) | `CLR_DISALLOWED_TOOLS` | string | Forwarded as-is to claude |
| `max_budget_usd` | [`--max-budget-usd`](param/065_max_budget_usd.md) | `CLR_MAX_BUDGET_USD` | string | Forwarded as-is to claude; no parse validation |

### Not Configurable

The following categories of `CliArgs` fields are excluded by the Eligibility Rule — they are inherently specific to a single invocation, not stable repeatable defaults:

| Category | Examples | Reason |
|----------|----------|--------|
| Positional message | `[MESSAGE]` | The task text itself; never a default |
| File I/O paths | `--file`, `--output-file`, `--mcp-config`, `--json-schema` | Path is specific to one invocation |
| One-shot behavior flags | `--dry-run`, `--new-session`, `--interactive`, `--trace` | Meaningless as a persistent default |
| Call-specific values | `--expect`, `--expect-strategy`, `--subdir`, `--session-dir`, `--session-from`, `--add-dir`, `--dir` | Tied to one task's working context |
| Meta and self-referential | `--help`, `--args-file` | `--help` exits before resolution; `--args-file` chaining is not supported |

All other `CliArgs` fields not listed in [Eligible Parameters](#eligible-parameters-38-total) above (e.g. `--verbose`, `--no-skip-permissions`, `--no-ultrathink`, `--keep-claudecode`, `--strip-fences`, `--system-prompt`, `--append-system-prompt`, `--output-format`, `--max-turns`) are simply not part of this task's scope — extending config-file coverage to additional parameters is a separate follow-up if a concrete need arises.

`isolated`, `refresh`, and `ps` subcommands do not read config files at all — separate `CliArgs`-shaped structs and dispatch paths.

### Example `config.toml`

```toml
# ~/.clr/config.toml (user-level) or .clr.toml (project-level, cwd)
model = "claude-opus-4-8"
max_sessions = 4
effort = "high"
timeout = 600
quiet = true
retry_on_transient = 3
transient_delay = 5
journal = "meta"
```

Project `.clr.toml` values win over user `config.toml` values on the same key; unrecognized keys (e.g. a typo, or a key from a newer `clr` version) are silently ignored.

### `CLR_CONFIG_DIR`

Test-injection override for user-level discovery only — mirrors the existing `CLR_GATE_DIR` pattern ([003_env_param.md](003_env_param.md), Env Param 7). When set to a non-empty value, `config.toml` is looked up at `$CLR_CONFIG_DIR/config.toml` instead of `$HOME/.clr/config.toml`. Project-level `.clr.toml` discovery in the current directory is unaffected.

### Error Handling

| Scenario | Behavior |
|----------|----------|
| File absent (either location) | Silently treated as empty — no error |
| Malformed TOML (present file) | `clr` exits 1; stderr names the offending file path |
| Unknown key (present, well-formed file) | Silently ignored; other recognized keys in the same file still apply |

### Sources

| File | Relationship |
|------|--------------|
| `../../src/cli/config.rs` | `ConfigDefaults`, `discover_config_paths()`, `load_config()`, `apply_config_defaults()` |
| `../../src/cli/mod.rs` | Call site — `dispatch_run()`, immediately after `apply_env_vars()` |

### Tests

| File | Relationship |
|------|--------------|
| `../../tests/config_file_test.rs` | T01–T15: precedence (CLI/JSON/env/config/default), project-over-user, `CLR_CONFIG_DIR` scope, malformed TOML, unknown key, dry-run reflection, invalid `output_style`/`journal`/`summary_fields` rejection |
