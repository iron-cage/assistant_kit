# CLI Parameter: --verbosity

Control how much diagnostic output the runner itself emits. Does not
affect Claude Code subprocess output.

- **Type:** [`VerbosityLevel`](../type/05_verbosity_level.md)
- **Default:** 3 (normal)
- **Command:** [`run`](../command/01_run.md)
- **Group:** [Runner Control](../param_group/02_runner_control.md)
- **Validation:** must be integer 0–5; out of range → error

```sh
clr --verbosity 0 "Silent run"    # suppress runner output
clr --verbosity 4 "Debug"         # verbose command preview
```

### Referenced Type

| Type | Kind | Fundamental | Key Constraint |
|------|------|-------------|----------------|
| [`VerbosityLevel`](../type/05_verbosity_level.md) | Semantic | unsigned 8-bit integer | 0 to 5 |

### Referenced Parameter Groups

| # | Group | Membership | Co-members |
|---|-------|------------|------------|
| 2 | [Runner Control](../param_group/02_runner_control.md) | Full | 16 other params |

### Referenced Commands

| # | Command | Default | Notes |
|---|---------|---------|-------|
| 1 | [`run`](../command/01_run.md) | 3 | — |
| 5 | [`ask`](../command/05_ask.md) | 3 | — |

### Referenced User Stories

| # | User Story | Persona |
|---|------------|---------|
| 4 | [004_dry_run_preview.md](../user_story/004_dry_run_preview.md) | Developer |
| 6 | [006_verbose_debugging.md](../user_story/006_verbose_debugging.md) | Developer |
| 8 | [008_trace_execution.md](../user_story/008_trace_execution.md) | Developer |
