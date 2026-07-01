# CLI Parameter: --verbose

Enable Claude Code verbose output. Passed through to the claude
subprocess.

- **Type:** bool (standalone flag)
- **Default:** false
- **Command:** [`run`](../command/01_run.md)
- **Group:** [Claude-Native Flags](../param_group/01_claude_native_flags.md)
- **JSON Key:** `"verbose"`

```sh
clr --verbose "Debug this"
```

### Referenced Type

| Type | Kind | Fundamental | Key Constraint |
|------|------|-------------|----------------|
| bool | Primitive | bool | present/absent |

### Referenced Parameter Groups

| # | Group | Membership | Co-members |
|---|-------|------------|------------|
| 1 | [Claude-Native Flags](../param_group/01_claude_native_flags.md) | Full | `--print`, `--model`, `--effort`, `--json-schema`, `--mcp-config` |

### Referenced Commands

| # | Command | Default | Notes |
|---|---------|---------|-------|
| 1 | [`run`](../command/01_run.md) | false | — |
| 5 | [`ask`](../command/05_ask.md) | false | — |

### Referenced User Stories

*None — no user story directly exercises `--verbose`.*
