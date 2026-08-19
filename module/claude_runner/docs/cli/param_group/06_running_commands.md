# CLI Parameter Group: Running Commands

**Pattern:** All five subcommands that invoke a `claude` subprocess — `run`, `ask`, `topic`, `isolated`, `refresh` — share a common execution model but differ in what they accept and how they inject defaults.

**Purpose:** Cross-command reference showing which parameters apply to each running command.

### Semantic Coherence Test

"Does this subcommand spawn a `claude` subprocess to do work?" — YES for all 5 (`run`, `ask`, `topic`, `isolated`, `refresh`). NO for `ps`, `kill`, `tools` (excluded from this group).

### Why NOT X

- `ps`: does not invoke claude; reads session metadata only
- `kill`: does not invoke claude; sends SIGTERM to running claude processes
- `tools`: does not invoke claude; lists available tool definitions

### Running Commands: Command Comparison

Key: ✅ = supported, ⬜ = not injected/not applicable, ➖ = not accepted, `*` = hardcoded/injected by runner

| Parameter | `run` | `ask` | `topic` | `isolated` | `refresh` | Notes |
|-----------|-------|-------|---------|------------|-----------|-------|
| **Input** | | | | | | |
| `[MESSAGE]` | ✅ optional | ✅ optional | ✅ optional | ✅ optional | `"."` * | refresh hardcodes message |
| `--file` | ✅ | ✅ | ✅ | ✅ | ➖ | stdin from file |
| passthrough (`--`) | ➖ | ➖ | ➖ | ✅ | ➖ | verbatim args forwarded to claude |
| **Credentials** | | | | | | |
| `--creds` | ➖ | ➖ | ➖ | ✅ | ✅ | credentials JSON path |
| **Execution control** | | | | | | |
| `--timeout` | ✅ 0 = unlimited (TSK-503) | ✅ 0 = unlimited | ✅ 0 = unlimited | ✅ 30s | ✅ 45s | different defaults per command |
| `--dry-run` | ✅ | ✅ | ✅ | ✅ | ✅ | preview without spawning |
| `--trace` | ✅ | ✅ | ✅ | ✅ | ✅ | emit env+command to stderr then execute |
| `--no-compact-window` | ✅ | ✅ | ✅ | ✅ | ✅ | suppress `CLAUDE_CODE_AUTO_COMPACT_WINDOW` injection |
| **Model and effort** | | | | | | |
| `--model` | ✅ | ✅ | ✅ | ✅ | default: `"sonnet"` | isolated default falls back to config tiers then `opus` alias; refresh uses `"sonnet"` constant |
| `--effort` | ✅ user sets | ✅ user sets | ✅ user sets | ✅ user sets (default: `max`) | `low` * | refresh injects `low`; cannot override via flag |
| `--no-effort-max` | ✅ | ✅ | ✅ | ✅ | ➖ | suppresses default `--effort max`; not available for refresh (fixed by design) |
| `--no-chrome` | ✅ | ✅ | ✅ | ✅ | ➖ | suppresses `--chrome` injection; not available for refresh |
| **Output** | | | | | | |
| `--output-style` | ✅ | ✅ | ✅ | ✅ | ➖ | |
| `--output-file` | ✅ | ✅ | ✅ | ✅ | ➖ | |
| `--strip-fences` | ✅ | ✅ | ✅ | ✅ | ➖ | |
| `--output-format` | ✅ | ✅ | ✅ | ➖ | ➖ | forwarded as-is to claude |
| `--summary-fields` | ✅ | ✅ | ✅ | ✅ | ➖ | |
| **Working directory** | | | | | | |
| `--dir` | ✅ | ✅ | ✅ | ✅ | ➖ | subprocess working directory |
| `--add-dir` | ✅ | ✅ | ✅ | ✅ | ➖ | additional allowed directory |
| `--subdir` | ✅ | ✅ | ✅ default: auto-slug | ➖ | ➖ | named subdirectory under `--dir`; `topic`'s default diverges (auto-generated slug, not `.`) |
| **Session** | | | | | | |
| `--new-session` | ✅ | ✅ | ✅ | ➖ | ➖ | isolated always uses fresh temp HOME |
| `--session-dir` | ✅ | ✅ | ✅ | ➖ | ➖ | |
| `--no-persist` | ✅ | ✅ | ✅ | always * | always * | isolated/refresh always inject `--no-session-persistence` |
| **Validation** | | | | | | |
| `--expect` | ✅ | ✅ | ✅ | ✅ | ➖ | |
| `--expect-strategy` | ✅ | ✅ | ✅ | ✅ | ➖ | |
| **System prompt** | | | | | | |
| `--system-prompt` | ✅ | ✅ | ✅ | ✅ | ➖ | forwarded to claude subprocess |
| `--append-system-prompt` | ✅ | ✅ | ✅ | ✅ | ➖ | forwarded to claude subprocess |
| **Claude-native forwarded** | | | | | | |
| `--json-schema` | ✅ | ✅ | ✅ | ✅ | ➖ | forwarded to claude subprocess |
| `--mcp-config` | ✅ | ✅ | ✅ | ✅ | ➖ | repeatable; forwarded to claude subprocess |
| `--allowed-tools` | ✅ | ✅ | ✅ | ✅ | ➖ | forwarded to claude subprocess |
| `--disallowed-tools` | ✅ | ✅ | ✅ | ✅ | ➖ | forwarded to claude subprocess |
| `--max-budget-usd` | ✅ | ✅ | ✅ | ✅ | ➖ | forwarded to claude subprocess |
| `--max-turns` | ✅ | ✅ | ✅ | ✅ | ➖ | forwarded to claude subprocess |
| **Journaling** | | | | | | |
| `--journal` | ✅ | ✅ | ✅ | ✅ | ✅ | |
| `--journal-dir` | ✅ | ✅ | ✅ | ✅ | ✅ | |
| **Retries** | | | | | | |
| `--retry-on-transient` / `--transient-delay` | ✅ | ✅ | ✅ | ➖ | ➖ | run/ask/topic only |
| `--retry-on-auth` / `--auth-delay` | ✅ | ✅ | ✅ | ➖ | ➖ | run/ask/topic only |
| `--max-sessions` | ✅ | ✅ | ✅ | ✅ | ➖ | concurrency gate; `isolated` uses 3-tier (CLI flag + `"max-sessions"` JSON key + `CLR_MAX_SESSIONS` env var; no config-file tier) |
| **Injected subprocess env vars** | | | | | | |
| `CLAUDE_CODE_MAX_OUTPUT_TOKENS` | `128,000` | `128,000` | `128,000` | `128,000` | `128,000` | always injected; `--max-tokens` overrides |
| `CLAUDE_CODE_AUTO_COMPACT_WINDOW` | `300,000` | `300,000` | `300,000` | `300,000` | `300,000` | always injected; `--no-compact-window` suppresses |
| `CLAUDE_CODE_AUTO_CONTINUE` | `true` | `true` | `true` | `true` | `true` | always injected |
| `CLAUDE_CODE_TELEMETRY` | `false` | `false` | `false` | `false` | `false` | always injected |
| `CLAUDE_CODE_BASH_TIMEOUT` | `3,600,000 ms` | `3,600,000 ms` | `3,600,000 ms` | `3,600,000 ms` | `3,600,000 ms` | always injected |

### Universal Params (all 5 running commands)

These parameters apply identically across all 5 running commands:

| Parameter | Effect |
|-----------|--------|
| `--timeout` | Max subprocess wait time (default differs per command) |
| `--trace` | Emit resolved env vars + command line to stderr before executing |
| `--dry-run` | Emit resolved env vars + command line to stderr; do not spawn subprocess |
| `--no-compact-window` | Suppress `CLAUDE_CODE_AUTO_COMPACT_WINDOW=300000` injection |
| `--journal` | Enable journaling (`full`/`meta`/`off`) |
| `--journal-dir` | Override journal output directory |

### Exclusive Parameters (asymmetric coverage)

Complements Universal Params above by summarizing the opposite extreme — parameters confined to one command or one proper subset, rather than shared by all 5. See the Command Comparison matrix above for every partial-overlap case (e.g. params shared by `run`/`ask`/`topic`/`isolated` but not `refresh`, such as `--file`, `--dir`, `--expect`).

| Scope | Parameters | Notes |
|-------|-----------|-------|
| `isolated`-only | passthrough (`--`) | Sole route to `--output-format` on `isolated` — the only remaining native-flag gap after TSK-443 (all other formerly-passthrough-only params now have native flags); ergonomic gap only (last-wins arg order means passthrough already reaches it), not a functional one — see [`../parity/001_run_ask_isolated.md`](../parity/001_run_ask_isolated.md) Exclusion Rationale |
| `isolated` + `refresh` only | `--creds` | Credential-isolated execution config; see [`04_credential_operations.md`](04_credential_operations.md) |
| `run` + `ask` + `topic` only | `--output-format`, `--subdir`, `--new-session`, `--session-dir`, `--retry-on-transient`/`--transient-delay`, `--retry-on-auth`/`--auth-delay` | Session control, retries, and format negotiation — no passthrough equivalent exists for `isolated`/`refresh` since these configure the runner itself, not the `claude` subprocess (`--session-dir` itself is deprecated and inert — BUG-493 — retained here only because it remains an exclusively run/ask/topic-parseable flag, not because it still configures anything) |

### Invariants

1. All 5 running commands inject `CLAUDE_CODE_MAX_OUTPUT_TOKENS=128000` and `CLAUDE_CODE_AUTO_COMPACT_WINDOW=300000` (opt-out via `--no-compact-window`).
2. `--dry-run` and `--trace` use the same code path for all 5 commands — `emit_credential_trace` for `isolated`/`refresh`, `handle_dry_run` for `run`/`ask`/`topic`. Both emit WYSIWYG output matching actual subprocess arguments.
3. `run`, `ask`, and `topic` are functionally identical except for `--subdir`'s default — `ask` is a pure alias for `run`; `topic` diverges only in auto-generating `--subdir`'s value. Formalized as a strict command_group (identical handler, identical parameter set save the one stated divergence) in [`command_group/01_run_ask.md`](../command_group/01_run_ask.md) — see that file for the Representation Absorption Test and default-divergence table backing this claim.
4. `isolated` and `refresh` run in an isolated temp HOME; session persistence is always suppressed.

### Notes

`--timeout 0` means "no timeout" (unlimited) for all 5 commands.

`clr ask` is a pure alias for `clr run` — it accepts all the same parameters, routing them through `dispatch_run` unchanged. Only `--help` output differs.

`clr topic` also routes through `dispatch_run` — its only divergence is `--subdir`'s default (auto-generated slug instead of `.`); see [`11_topic.md`](../command/11_topic.md).

### Referenced Commands

| # | Command | Membership | Notes |
|---|---------|------------|-------|
| 1 | [`run`](../command/01_run.md) | Full — all running params apply | Default command |
| 5 | [`ask`](../command/05_ask.md) | Full — identical to run | Pure alias for run |
| 11 | [`topic`](../command/11_topic.md) | Full — identical to run/ask except `--subdir` default | Auto-naming alias |
| 2 | [`isolated`](../command/03_isolated.md) | Subset — no retries, no session control | Credential-isolated execution |
| 3 | [`refresh`](../command/04_refresh.md) | Minimal — creds + timeout + trace/dry-run | OAuth token refresh only |

### Cross-References

| Type | Path | Responsibility |
|------|------|----------------|
| doc | [`002_command_defaults.md`](../002_command_defaults.md) | Injected env var defaults and behavior matrix |
| doc | [`003_env_param.md`](../003_env_param.md) | All `CLR_*` environment variable fallbacks |
| group | [`04_credential_operations.md`](04_credential_operations.md) | Params exclusive to `isolated`/`refresh` |
| group | [`02_runner_control.md`](02_runner_control.md) | Params consumed by the runner before subprocess launch |
| command_group | [`../command_group/01_run_ask.md`](../command_group/01_run_ask.md) | Strict `run`/`ask` identity — identical handler and parameter set, not just similar defaults |
