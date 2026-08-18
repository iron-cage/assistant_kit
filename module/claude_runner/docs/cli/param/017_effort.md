# CLI Parameter: --effort

Override the reasoning effort level passed to the `claude` subprocess. `clr`
injects `--effort max` automatically on every invocation; this flag overrides
that default to any supported level.

- **Type:** [`EffortLevel`](../type/07_effort_level.md)
- **Default:** max (injected automatically; override with this flag or suppress entirely with `--no-effort-max`)
- **Command:** [`run`](../command/01_run.md)
- **Group:** [Claude-Native Flags](../param_group/01_claude_native_flags.md)
- **Validation:** requires a value; unknown level → error listing valid values (`low`, `medium`, `high`, `max`)
- **JSON Key:** `"effort"`

```sh
clr "Fix the bug"                  # sends: --effort max (default)
clr --effort medium "Fix the bug"  # sends: --effort medium
clr --effort high "Fix the bug"    # sends: --effort high
```

**Note:** `max` is the default because `clr` is designed for agentic automation
tasks where full reasoning capacity is the correct default. The claude binary's
own default (`medium`) is intentionally overridden here.

**Note:** To suppress the `--effort` flag entirely (pass no effort flag to claude),
use `--no-effort-max`.

**Known Limitation (BUG-434 — Claude Code v2.1.78+):** `"max"` is not accepted by the
`claude` binary in interactive mode. Bare `clr` (no message, interactive REPL) fails
immediately with "Effort level 'max' is not available in interactive mode." Workaround:
use `--no-effort-max` or `--effort high` until TSK-447 lands a mode-aware injection fix.
`--effort max` in print mode (with a message) continues to work correctly.

### Referenced Type

| Type | Kind | Fundamental | Key Constraint |
|------|------|-------------|----------------|
| [`EffortLevel`](../type/07_effort_level.md) | Semantic | enumeration | low/medium/high/max |

### Referenced Parameter Groups

| # | Group | Membership | Co-members |
|---|-------|------------|------------|
| 1 | [Claude-Native Flags](../param_group/01_claude_native_flags.md) | Full | `--print`, `--model`, `--verbose`, `--json-schema`, `--mcp-config` |

### Referenced Commands

| # | Command | Default | Notes |
|---|---------|---------|-------|
| 1 | [`run`](../command/01_run.md) | max | clr default; claude binary default is medium |
| 5 | [`ask`](../command/05_ask.md) | max | Pure alias — same default as `run` |
| 3 | [`isolated`](../command/03_isolated.md) | max | Injected automatically; override with this flag or suppress entirely with `--no-effort-max`; env: `CLR_EFFORT` |
| 11 | [`topic`](../command/11_topic.md) | max | Identical to `ask`; delegates to `run`'s handler |

### Referenced User Stories

| # | User Story | Persona |
|---|------------|---------|
| 15 | [015_ask_mode.md](../user_story/015_ask_mode.md) | Developer |
| 20 | [020_suppress_effort_max.md](../user_story/020_suppress_effort_max.md) | Developer |
