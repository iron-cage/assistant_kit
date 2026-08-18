# CLI Command Group: run / ask / topic

**Pattern:** Three subcommand names dispatch through the exact same handler function — `run` and `ask` with the exact same parameter set and no CLI-facing behavior difference; `topic` through the same handler too, but with one stated divergence: when `--subdir` is not explicitly given, `topic` computes an auto-generated slug default instead of `run`/`ask`'s identity default (`.`). The group exists so operators can name intent ("run a task" vs. "ask a question" vs. "start/continue a named topic") in scripts and shell history — `run` and `ask` because the runner never branches on the name, `topic` because its one divergence is a default-value change, not new behavior (see Representation Absorption Test below).

**Purpose:** Formalize that `run`, `ask`, and `topic` are the same command under three names — two with zero divergence, one with a single documented default divergence — so a future change to the shared handler is never accidentally applied to only some of them.
**Order:** 1

### Shared Handler

`dispatch_run()` (`src/cli/mod.rs:331`). `dispatch_ask()` (`src/cli/mod.rs:487`) intercepts only `--help`/`-h`/bare `help` to print ask-specific help text, then calls `dispatch_run(&tokens[1..])` directly — the identical function, not a duplicate. `dispatch_topic()` is designed to intercept `--help`/`-h`/bare `help` to print topic-specific help text; when `--subdir` is absent from the token stream, it would compute an auto-generated slug (see [`11_topic.md`](../command/11_topic.md) Algorithm) and inject `--subdir <slug>` before calling `dispatch_run(&tokens[1..])` — otherwise forwarding tokens unchanged; dispatch wiring is pending implementation (task 521). Clone-vs-continue behavior requires no new session logic: it falls entirely out of the pre-existing `execute_session_transplant()` mechanism (`src/cli/builder.rs:165`) once `--subdir` resolves to a session-isolated directory — the first invocation of a new subdir clones (no same-named session file exists yet at the destination), a later invocation of the same subdir continues (a diverged file already exists there).

### Representation Absorption Test

"Would the proposed new command be achievable by changing default values of an existing command's parameters?" —

- For `ask` relative to `run`: trivially yes, since `ask` changes zero defaults; it is `run` under a second name. This is the maximal case of Representation Absorption — an alias with no divergence at all — kept as a separate command name rather than a `--flag` because the distinction it signals is operator intent, never runner behavior.
- For `topic` relative to `run`/`ask`: also yes, and non-trivially — this is the case the test was designed to catch. `topic` is fully reachable by changing exactly one existing parameter's default: `--subdir`'s identity default (`.`) becomes an auto-generated slug. No new flag, no new session-handling code, no new execution path is introduced. `topic` is kept as a separate command name (rather than, say, a `--topic` flag on `run`) for the same operator-intent reason as `ask`: naming the invocation communicates "start or continue a named line of work," which a flag buried in a longer invocation would not.

### Why NOT Merged

`run`, `ask`, and `topic` are kept as three command names, not folded into one name behind a flag, because the distinction is a human-facing intent signal (readable in shell history and scripts), not a behavioral branch the runner ever inspects beyond the one stated `--subdir` default. Collapsing them into one name would remove that signal for zero implementation simplification — there is no logic to delete; `dispatch_ask` is already a thin wrapper, and `dispatch_topic`'s only added logic is the slug generator it must have regardless of whether it's exposed under its own name or a flag.

### Default Divergence Table

| # | Command | Parameter | Canonical Default (`run`) | This Command's Default | Rationale |
|---|---------|-----------|---------------------------|--------------------------|-----------|
| — | `ask` | *(none)* | — | — | No parameter default differs from `run`; `ask` forwards tokens to `dispatch_run` unchanged |
| 1 | `topic` | `--subdir` | `.` (identity — no subdirectory appended) | Auto-generated slug from `MESSAGE` (lowercase, hyphenated, truncated, disambiguated via counter suffix on collision) | The entire point of the command: each topic gets its own session-isolated subdirectory without the operator having to name one explicitly; explicit `--subdir NAME` still overrides |

### Invariants

1. Every parameter accepted by `run` is accepted by `ask` and `topic` with an identical default — save `topic`'s one stated `--subdir` divergence — enforced structurally by `dispatch_ask`/`dispatch_topic` delegating to `dispatch_run`, not by parallel maintenance of separate parameter tables.
2. The only observable difference between `clr ask --help` and `clr run --help` is the help text body. `clr topic --help` differs in help text body AND in `--subdir`'s documented default.
3. A future parameter added to `run` (Claude-Native Flags, Runner Control, or System Prompt groups) is automatically available under `ask` and `topic` with no additional wiring — see `command/readme.md`'s maintenance note: "`ask` inherits all `run` params automatically... no separate table update needed for `05_ask.md`." The same inheritance applies to `topic`, minus the one already-documented `--subdir` divergence.
4. `topic` is the only member of this group with a stated default divergence — see Default Divergence Table above; a second divergent parameter would need its own row and its own update to this invariant.

### Referenced Commands

| # | Command | Shared Handler? | Notes |
|---|---------|:---:|-------|
| 1 | [`run`](../command/01_run.md) | Yes — canonical | Default command; delegate target |
| 2 | [`ask`](../command/05_ask.md) | Yes — delegates | Calls `dispatch_run` directly; only `--help` text differs |
| 3 | [`topic`](../command/11_topic.md) | Yes — delegates | Calls `dispatch_run` after computing `--subdir`'s auto-generated slug default; only member with a stated default divergence |

### Referenced Parameters

All parameters in the [Claude-Native Flags](../param_group/01_claude_native_flags.md), [Runner Control](../param_group/02_runner_control.md), and [System Prompt](../param_group/03_system_prompt.md) groups apply identically to all three commands — save `topic`'s stated `--subdir` default divergence — see those group files for the full parameter list rather than duplicating it here.

### Referenced Tests

| # | Test Spec | Scope |
|---|-----------|-------|
| 1 | [command/05_ask.md](../../../tests/docs/cli/command/05_ask.md) (IT-1, IT-2) | Dry-run output equivalence between `ask` and `run` |
| 2 | [command/11_topic.md](../../../tests/docs/cli/command/11_topic.md) (IT-1 through IT-8) | Slug generation, disambiguation, clone/continue, and parameter passthrough equivalence for `topic` |
| 3 | [command_group/01_run_ask.md](../../../tests/docs/cli/command_group/01_run_ask.md) (CG-1 through CG-3) | Group-level structural equivalence and divergence tests |

### Referenced User Stories

| # | User Story | Persona |
|---|------------|---------|
| 15 | [015_ask_mode.md](../user_story/015_ask_mode.md) | Developer |
| 30 | [030_topic_creation.md](../user_story/030_topic_creation.md) | Developer |

### Cross-References

| Type | Path | Responsibility |
|------|------|-----------------|
| group | [`param_group/06_running_commands.md`](../param_group/06_running_commands.md) | Broader 5-command (`run`/`ask`/`topic`/`isolated`/`refresh`) parameter comparison — looser relationship; does not meet this entity's identical-parameter-set bar |
| parity | [`parity/001_run_ask_isolated.md`](../parity/001_run_ask_isolated.md) | Full 3-way behavioral comparison including `isolated` |
