# CLI Parameter: --no-chrome

Suppress the automatic `--chrome` injection. When set, no `--chrome` flag
is forwarded to the `claude` subprocess.

- **Type:** bool (standalone flag)
- **Default:** false (chrome injection is **ON** in interactive mode; **suppressed** in print mode — BUG-304)
- **Command:** [`run`](../command/01_run.md)
- **Group:** [Runner Control](../param_group/02_runner_control.md)

```sh
clr --interactive "Fix bug" # sends: --chrome (interactive default)
clr "Fix bug"               # no --chrome (print mode — BUG-304 suppression)
clr --no-chrome "Fix bug"   # no --chrome (explicit opt-out)
```

**Note:** `--chrome` is automatically suppressed in print mode (message given without
`--interactive`) to prevent permanent session hang (BUG-304). Use `--no-chrome` explicitly
in interactive/headless/CI environments where no Chrome instance is available.

### Referenced Type

| Type | Kind | Fundamental | Key Constraint |
|------|------|-------------|----------------|
| bool | Primitive | bool | present/absent |

### Referenced Parameter Groups

| # | Group | Membership | Co-members |
|---|-------|------------|------------|
| 2 | [Runner Control](../param_group/02_runner_control.md) | Full | 16 other params |

### Referenced Commands

| # | Command | Default | Notes |
|---|---------|---------|-------|
| 1 | [`run`](../command/01_run.md) | false | Chrome suppressed in print mode via `use_print` guard (BUG-304) |
| 5 | [`ask`](../command/05_ask.md) | false | Chrome suppressed — ask is always print mode (BUG-304) |

### Referenced User Stories

*None — no user story directly exercises `--no-chrome`.*
