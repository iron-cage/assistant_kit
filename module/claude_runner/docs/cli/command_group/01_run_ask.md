# CLI Command Group: run / ask

**Pattern:** Two subcommand names dispatch through the exact same handler function with the exact same parameter set — no CLI-facing behavior differs. The group exists so operators can name intent ("run a task" vs. "ask a question") in scripts and shell history, not because the runner branches on it.

**Purpose:** Formalize that `run` and `ask` are the same command under two names, so a future change to one is never accidentally applied without the other.
**Order:** 1

### Shared Handler

`dispatch_run()` (`src/cli/mod.rs:247`). `dispatch_ask()` (`src/cli/mod.rs:376`) intercepts only `--help`/`-h`/bare `help` to print ask-specific help text, then calls `dispatch_run(&tokens[1..])` directly — the identical function, not a duplicate.

### Representation Absorption Test

"Would the proposed new command be achievable by changing default values of an existing command's parameters?" — for `ask` relative to `run`: trivially yes, since `ask` changes zero defaults; it is `run` under a second name. This is the maximal case of Representation Absorption — an alias with no divergence at all — kept as a separate command name rather than a `--flag` because the distinction it signals is operator intent, never runner behavior.

### Why NOT Merged

`run` and `ask` are kept as two command names, not folded into one name behind a flag, because the distinction is a human-facing intent signal (readable in shell history and scripts), not a behavioral branch the runner ever inspects. Collapsing them into one name would remove that signal for zero implementation simplification — there is no logic to delete; `dispatch_ask` is already a thin wrapper.

### Default Divergence Table

| # | Command | Parameter | Canonical Default (`run`) | This Command's Default | Rationale |
|---|---------|-----------|---------------------------|--------------------------|-----------|
| — | `ask` | *(none)* | — | — | No parameter default differs from `run`; `ask` forwards tokens to `dispatch_run` unchanged |

### Invariants

1. Every parameter accepted by `run` is accepted by `ask` with an identical default — enforced structurally by `dispatch_ask` delegating to `dispatch_run`, not by parallel maintenance of two parameter tables.
2. The only observable difference between `clr ask --help` and `clr run --help` is the help text body.
3. A future parameter added to `run` (Claude-Native Flags, Runner Control, or System Prompt groups) is automatically available under `ask` with no additional wiring — see `command/readme.md`'s maintenance note: "`ask` inherits all `run` params automatically... no separate table update needed for `05_ask.md`."

### Referenced Commands

| # | Command | Shared Handler? | Notes |
|---|---------|:---:|-------|
| 1 | [`run`](../command/01_run.md) | Yes — canonical | Default command; delegate target |
| 2 | [`ask`](../command/05_ask.md) | Yes — delegates | Calls `dispatch_run` directly; only `--help` text differs |

### Referenced Parameters

All parameters in the [Claude-Native Flags](../param_group/01_claude_native_flags.md), [Runner Control](../param_group/02_runner_control.md), and [System Prompt](../param_group/03_system_prompt.md) groups apply identically to both commands — see those group files for the full parameter list rather than duplicating it here.

### Referenced Tests

| # | Test Spec | Scope |
|---|-----------|-------|
| 1 | [command/05_ask.md](../../../tests/docs/cli/command/05_ask.md) (IT-1, IT-2) | Dry-run output equivalence between `ask` and `run` |

### Referenced User Stories

| # | User Story | Persona |
|---|------------|---------|
| 15 | [015_ask_mode.md](../user_story/015_ask_mode.md) | Developer |

### Cross-References

| Type | Path | Responsibility |
|------|------|-----------------|
| group | [`param_group/06_running_commands.md`](../param_group/06_running_commands.md) | Broader 4-command (`run`/`ask`/`isolated`/`refresh`) parameter comparison — looser relationship; does not meet this entity's identical-parameter-set bar |
| parity | [`parity/001_run_ask_isolated.md`](../parity/001_run_ask_isolated.md) | Full 3-way behavioral comparison including `isolated` |
