# CLI Parameter Group: Running Commands

**Pattern:** All four subcommands that invoke a `claude` subprocess — `run`, `ask`, `isolated`, `refresh` — share a common execution model but differ in what they accept and how they inject defaults.

**Purpose:** Cross-command reference showing which parameters apply to each running command.

### Semantic Coherence Test

"Does this subcommand spawn a `claude` subprocess to do work?" — YES for all 4 (`run`, `ask`, `isolated`, `refresh`). NO for `ps`, `kill`, `tools` (excluded from this group).

### Why NOT X

- `ps`: does not invoke claude; reads session metadata only
- `kill`: does not invoke claude; sends SIGTERM to running claude processes
- `tools`: does not invoke claude; lists available tool definitions

### Running Commands: Command Comparison

Key: ✅ = supported, ⬜ = not injected/not applicable, ➖ = not accepted, `*` = hardcoded/injected by runner

| Parameter | `run` | `ask` | `isolated` | `refresh` | Notes |
|-----------|-------|-------|------------|-----------|-------|
| **Input** | | | | | |
| `[MESSAGE]` | ✅ optional | ✅ optional | ✅ optional | `"."` * | refresh hardcodes message |
| `--file` | ✅ | ✅ | ✅ | ➖ | stdin from file |
| passthrough (`--`) | ➖ | ➖ | ✅ | ➖ | verbatim args forwarded to claude |
| **Credentials** | | | | | |
| `--creds` | ➖ | ➖ | ✅ | ✅ | credentials JSON path |
| **Execution control** | | | | | |
| `--timeout` | ✅ 3600s | ✅ 3600s | ✅ 30s | ✅ 45s | different defaults per command |
| `--dry-run` | ✅ | ✅ | ✅ | ✅ | preview without spawning |
| `--trace` | ✅ | ✅ | ✅ | ✅ | emit env+command to stderr then execute |
| `--no-compact-window` | ✅ | ✅ | ✅ | ✅ | suppress `CLAUDE_CODE_AUTO_COMPACT_WINDOW` injection |
| **Model and effort** | | | | | |
| `--model` | ✅ | ✅ | default: `"opus"` | default: `"sonnet"` | isolated/refresh use constants as default |
| `--effort` | ✅ user sets | ✅ user sets | `max` * | `low` * | isolated/refresh inject effort; cannot override via flag |
| `--no-effort-max` | ✅ | ✅ | ➖ | ➖ | suppresses default `--effort max` in run/ask |
| **Output** | | | | | |
| `--output-style` | ✅ | ✅ | ✅ | ➖ | |
| `--output-file` | ✅ | ✅ | ✅ | ➖ | |
| `--strip-fences` | ✅ | ✅ | ✅ | ➖ | |
| `--output-format` | ✅ | ✅ | ➖ | ➖ | forwarded as-is to claude |
| `--summary-fields` | ✅ | ✅ | ✅ | ➖ | |
| **Working directory** | | | | | |
| `--dir` | ✅ | ✅ | ✅ | ➖ | subprocess working directory |
| `--add-dir` | ✅ | ✅ | ✅ | ➖ | additional allowed directory |
| `--subdir` | ✅ | ✅ | ➖ | ➖ | named subdirectory under `--dir` |
| **Session** | | | | | |
| `--new-session` | ✅ | ✅ | ➖ | ➖ | isolated always uses fresh temp HOME |
| `--session-dir` | ✅ | ✅ | ➖ | ➖ | |
| `--no-persist` | ✅ | ✅ | always * | always * | isolated/refresh always inject `--no-session-persistence` |
| **Validation** | | | | | |
| `--expect` | ✅ | ✅ | ✅ | ➖ | |
| `--expect-strategy` | ✅ | ✅ | ✅ | ➖ | |
| **Journaling** | | | | | |
| `--journal` | ✅ | ✅ | ✅ | ✅ | |
| `--journal-dir` | ✅ | ✅ | ✅ | ✅ | |
| **Retries** | | | | | |
| `--retry-on-transient` / `--transient-delay` | ✅ | ✅ | ➖ | ➖ | run/ask only |
| `--retry-on-auth` / `--auth-delay` | ✅ | ✅ | ➖ | ➖ | run/ask only |
| `--max-sessions` | ✅ | ✅ | ➖ | ➖ | concurrency gate; run/ask only |
| **Injected subprocess env vars** | | | | | |
| `CLAUDE_CODE_MAX_OUTPUT_TOKENS` | `200,000` | `200,000` | `200,000` | `200,000` | always injected; `--max-tokens` overrides |
| `CLAUDE_CODE_AUTO_COMPACT_WINDOW` | `200,000` | `200,000` | `200,000` | `200,000` | always injected; `--no-compact-window` suppresses |
| `CLAUDE_CODE_AUTO_CONTINUE` | `true` | `true` | `true` | `true` | always injected |
| `CLAUDE_CODE_TELEMETRY` | `false` | `false` | `false` | `false` | always injected |
| `CLAUDE_CODE_BASH_TIMEOUT` | `3,600,000 ms` | `3,600,000 ms` | `3,600,000 ms` | `3,600,000 ms` | always injected |

### Universal Params (all 4 running commands)

These parameters apply identically across all 4 running commands:

| Parameter | Effect |
|-----------|--------|
| `--timeout` | Max subprocess wait time (default differs per command) |
| `--trace` | Emit resolved env vars + command line to stderr before executing |
| `--dry-run` | Emit resolved env vars + command line to stderr; do not spawn subprocess |
| `--no-compact-window` | Suppress `CLAUDE_CODE_AUTO_COMPACT_WINDOW=200000` injection |
| `--journal` | Enable journaling (`full`/`meta`/`off`) |
| `--journal-dir` | Override journal output directory |

### Invariants

1. All 4 running commands inject `CLAUDE_CODE_MAX_OUTPUT_TOKENS=200000` and `CLAUDE_CODE_AUTO_COMPACT_WINDOW=200000` (opt-out via `--no-compact-window`).
2. `--dry-run` and `--trace` use the same code path for all 4 commands — `emit_credential_trace` for `isolated`/`refresh`, `handle_dry_run` for `run`/`ask`. Both emit WYSIWYG output matching actual subprocess arguments.
3. `run` and `ask` are functionally identical — `ask` is an alias for `run` with distinct help text.
4. `isolated` and `refresh` run in an isolated temp HOME; session persistence is always suppressed.

### Notes

`--timeout 0` means "no timeout" (unlimited) for all 4 commands.

`clr ask` is a pure alias for `clr run` — it accepts all the same parameters, routing them through `dispatch_run` unchanged. Only `--help` output differs.

### Referenced Commands

| # | Command | Membership | Notes |
|---|---------|------------|-------|
| 1 | [`run`](../command/01_run.md) | Full — all running params apply | Default command |
| 5 | [`ask`](../command/05_ask.md) | Full — identical to run | Pure alias for run |
| 2 | [`isolated`](../command/02_isolated.md) | Subset — no retries, no session control | Credential-isolated execution |
| 3 | [`refresh`](../command/03_refresh.md) | Minimal — creds + timeout + trace/dry-run | OAuth token refresh only |

### Cross-References

| Type | Path | Responsibility |
|------|------|----------------|
| doc | [`command_defaults.md`](../command_defaults.md) | Injected env var defaults and behavior matrix |
| doc | [`env_param.md`](../env_param.md) | All `CLR_*` environment variable fallbacks |
| group | [`04_credential_operations.md`](04_credential_operations.md) | Params exclusive to `isolated`/`refresh` |
| group | [`02_runner_control.md`](02_runner_control.md) | Params consumed by the runner before subprocess launch |
