# Send a single-turn question using ask as a semantic alias for run

**Persona:** Developer who wants to signal intent — the invocation is a question, not a task — without any behavioral difference from `run`. Scripts and shell history benefit from the semantic distinction even when the mechanics are identical.
**Goal:** Use `clr ask` to send a single-turn question to Claude and get a clean answer on stdout, with the same parameter set, same defaults, and same execution path as `clr run`. The distinction is documentation only: `ask` communicates that the invocation is a question.
**Benefit:** Improves script readability and intent clarity with no behavioral overhead.
**Priority:** Low

### Acceptance Criteria

- `clr ask "What does this function do?"` behaves identically to `clr run "What does this function do?"`
- `clr ask --dry-run "X"` and `clr run --dry-run "X"` produce identical assembled commands
- No flags are forced or overridden by the `ask` subcommand — `--new-session`, `--no-chrome`,
  `--no-persist`, `--no-ultrathink`, `--effort`, `--max-tokens` are all at their `run` defaults
- All 34 parameters from `run` are accepted by `ask` with identical defaults and behavior
- `clr ask --effort high "..."`, `clr ask --max-tokens 200000 "..."`,
  `clr ask --new-session "..."` all work exactly as they do under `run`

### Referenced Commands

| # | Command | Role |
|---|---------|------|
| 5 | [`ask`](../command/05_ask.md) | Pure semantic alias for run with identical parameter set |

### Referenced Parameter Groups

| # | Parameter Group | Role |
|---|-----------------|------|
| 1 | [Claude-Native Flags](../param_group/01_claude_native_flags.md) | All members accepted; no defaults differ from run |
| 2 | [Runner Control](../param_group/02_runner_control.md) | All members accepted; no defaults differ from run |

### Referenced Parameters

| # | Parameter | Role |
|---|-----------|------|
| 1 | [`--message`](../param/001_message.md) | Question text for Claude |
| 2 | [`--print`](../param/002_print.md) | Auto-enabled when message given (same as run) |
| 17 | [`--effort`](../param/017_effort.md) | Same default as run (max); overridable |
| 9 | [`--max-tokens`](../param/009_max_tokens.md) | Same default as run (200000); overridable |
| 28 | [`--subdir`](../param/028_subdir.md) | Named workspace isolation within ask |

### Workflow Steps

1. `clr ask "What does this function do?"` — send a question; identical to `clr run "..."`
2. `clr ask --dry-run "What is the return type?"` — preview the assembled command without executing
3. `clr ask --new-session "Explain this error"` — ask a question in a fresh conversation

### Related User Stories

| # | User Story | Relationship |
|---|------------|--------------|
| 2 | [Print Mode Capture](002_print_mode_capture.md) | `ask` uses print mode when message is given (same as run) |
| 17 | [Model Selection](017_model_selection.md) | `ask` accepts `--model` to select the model |
| 22 | [Session Isolation via Subdirectory](022_session_isolation_subdir.md) | `ask --subdir work` isolates session within a named workspace |
