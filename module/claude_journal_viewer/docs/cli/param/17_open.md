# CLI Parameter: open

Auto-open the default browser when the web viewer starts.
Uses `xdg-open` on Linux and `open` on macOS. No-op on
platforms without a desktop environment.

- **Type:** [`Boolean`](../type/08_boolean.md)
- **Default:** 0
- **Required:** No

```bash
clj .serve open::1                    # Start and open browser
clj .serve open::1 port::9090        # Custom port, auto-open
```

### Referenced Type

| Type | Kind | Fundamental | Key Constraint |
|------|------|-------------|----------------|
| [`Boolean`](../type/08_boolean.md) | Fundamental | Integer | 0 or 1 |

### Referenced Parameter Groups

| # | Group | Membership |
|---|-------|------------|
| 5 | [Global](../param_group/05_global.md) | Partial (serve only) |

### Referenced Commands

| # | Command | Default | Notes |
|---|---------|---------|-------|
| 5 | [`.serve`](../command/05_serve.md) | 0 | No auto-open |

### Referenced User Stories

| # | User Story | Persona |
|---|------------|---------|
| 1 | [Cost Tracking](../user_story/001_cost_tracking.md) | Developer |
