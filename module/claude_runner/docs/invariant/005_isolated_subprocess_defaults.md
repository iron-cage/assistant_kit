# Invariant: Isolated Subprocess Defaults

### Scope

- **Purpose**: Define the flag injection and subprocess environment requirements for `isolated` and `refresh` commands.
- **Responsibility**: State which model, effort level, flags, and behaviors must be applied to every isolated subprocess invocation, and the semantics that must hold.
- **In Scope**: Model selection per command, effort level injection, skip-permissions injection, session-persistence suppression, chrome suppression for refresh, CLAUDE.md provisioning, timeout-zero semantics.
- **Out of Scope**: Default flags for `run`/`ask` (→ `001_default_flags.md`); credential file resolution (→ `feature/001_runner_tool.md`); numeric timeout defaults (→ `param/020_timeout.md`).

### Invariant Statement

`clr isolated` and `clr refresh` spawn a subprocess inside a temporary HOME that is unconditionally discarded after each run. They serve different purposes from `run`/`ask` and must apply the following defaults:

| Behavior | isolated | refresh | Rationale |
|----------|----------|---------|-----------|
| Model | `"opus"` (`ISOLATED_DEFAULT_MODEL`) | `"claude-sonnet-5"` (`REFRESH_DEFAULT_MODEL`) | Isolated runs real user tasks requiring maximum capability; refresh executes a trivial `"."` ping to trigger OAuth token exchange only |
| `--effort` | `max` (injected) | `low` (injected) | Real tasks need maximum reasoning; credential ping needs minimal reasoning |
| `--dangerously-skip-permissions` | ON when message present | not injected | Isolated tasks invoke tools; without this flag, every tool call blocks on an interactive permission prompt |
| `--no-session-persistence` | always injected | always injected | Temp HOME is unconditionally discarded after run; writing session files to it is pure I/O waste |
| `--chrome` | ON (ClaudeCommand default) | suppressed (`--no-chrome` injected) | Isolated tasks may use browser context; refresh is HTTP-only OAuth exchange — chrome overhead has no benefit |
| CLAUDE.md provisioning | written to `<temp_home>/.claude/CLAUDE.md` | written to `<temp_home>/.claude/CLAUDE.md` | Without user-level instructions, the subprocess may ask clarifying questions or request interactive confirmation, blocking the subprocess permanently |
| `--timeout 0` semantics | unlimited (no watchdog) | unlimited (no watchdog) | Matches `run`/`ask` semantics: `0` = disabled; passing `0` must never kill the subprocess immediately |

### CLAUDE.md Content

Written to `<temp_home>/.claude/CLAUDE.md` before subprocess spawn. Must contain exactly:

    # Isolated subprocess

    Execute the given task immediately and exit.

    - Do not ask clarifying questions — act on the message as given.
    - Do not request human confirmation for any operation.
    - Do not explain your reasoning or narrate your steps.
    - Output only the direct result of the task; no preamble, no summary.
    - If the input is a single character or whitespace only, reply with a single period.

### Passthrough Override Convention

All injected flags are prepended **before** `--print`, message, and passthrough args. Claude binary last-wins semantics allow callers to override:

```sh
# Override injected --effort max with medium for a lighter isolated task:
clr isolated "summarize this file" -- --effort medium

# Opt out of skip-permissions for a read-only task:
clr isolated "what is 2+2?" -- --no-skip-permissions
```

### Enforcement Mechanism

**In `src/cli/credential.rs::run_isolated_command()`:**
- Effort: prepended as `["--effort", "max"]` for isolated, `["--effort", "low"]` for refresh (via an `effort: EffortLevel` parameter)
- Skip-permissions: prepended as `["--dangerously-skip-permissions"]` when `message.is_some()`
- No-session-persistence: prepended as `["--no-session-persistence"]` unconditionally
- Chrome (refresh only): prepended as `["--no-chrome"]` for refresh before `--print` and message

**In `module/claude_runner_core/src/isolated.rs::run_isolated()`:**
- Model: prepended via `IsolatedModel::Default` → `ISOLATED_DEFAULT_MODEL = "opus"` for isolated; `IsolatedModel::Specific(REFRESH_DEFAULT_MODEL)` for refresh where `REFRESH_DEFAULT_MODEL = "claude-sonnet-5"`
- `CLAUDE_CODE_AUTO_COMPACT_WINDOW`: set to `200000` in subprocess env via `ClaudeCommand::new()` → `compact_window: Some(200_000)`; suppressed by `--no-compact-window` / `CLR_NO_COMPACT_WINDOW` (same as all 4 running commands)
- CLAUDE.md: written to `claude_dir/CLAUDE.md` before subprocess spawn
- Timeout=0: deadline is skipped when `timeout_secs == 0` (no watchdog)

### Violation Consequences

If any default is removed or reverted:
- Isolated tasks stall at every tool call waiting for permission prompt (skip-permissions removed)
- Session files written to discarded temp HOME waste disk I/O (no-session-persistence removed)
- Refresh sets up browser context for a pure HTTP operation (chrome suppression removed)
- Subprocess asks clarifying questions or prompts for confirmation, blocking permanently (CLAUDE.md removed)
- `--timeout 0` kills subprocess immediately, making unlimited timeout impossible (timeout fix reverted)
- Isolated user tasks run on `"sonnet"`/`"claude-sonnet-5"` instead of `"opus"` (model reverted)

### Related Docs

| File | Relationship |
|------|--------------|
| [`001_default_flags.md`](001_default_flags.md) | Default flags for `run`/`ask` — counterpart for the main execution paths |
| [`../cli/002_command_defaults.md`](../cli/002_command_defaults.md) | Design analysis: full parameter matrix, issues I1–I7, changes S1–S7 |
| [`../cli/command/03_isolated.md`](../cli/command/03_isolated.md) | Command reference for `isolated` |
| [`../cli/command/04_refresh.md`](../cli/command/04_refresh.md) | Command reference for `refresh` |

### Sources

| File | Relationship |
|------|--------------|
| `../../src/cli/credential.rs` | `run_isolated_command()`, `run_refresh_command()` — flag injection entry points |
| `../../src/cli/cred_parse.rs` | `IsolatedArgs`, `RefreshArgs` — timeout sentinel values |
| `../../../claude_runner_core/src/isolated.rs` | `run_isolated()` — model prepend, CLAUDE.md write, timeout logic |
| `../../../claude_runner_core/src/command/mod.rs` | `ClaudeCommand::new()` — chrome and env var defaults |

### Tests

| File | Relationship |
|------|--------------|
| `../../tests/isolated_defaults_test.rs` | Verify model, effort, skip-permissions, no-session-persistence, chrome suppression, CLAUDE.md presence, timeout=0 unlimited behavior (ISD-1–ISD-13) |
| `../../tests/isolated_correctness_test.rs` | Verify correctness gaps S2–S6: no-session-persistence, skip-perms condition, no-chrome, timeout-0 unlimited, CLAUDE.md provisioning (CT-1–CT-6) |
