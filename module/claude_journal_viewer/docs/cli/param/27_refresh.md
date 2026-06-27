# CLI Parameter: refresh

Auto-refresh interval in seconds for the web viewer. The dashboard
page polls for new data at this interval. Set to `0` to disable
auto-refresh (manual reload only).

- **Type:** [`Integer`](../type/04_integer.md)
- **Default:** 10
- **Required:** No

```bash
clj .serve                            # 10-second auto-refresh
clj .serve refresh::30               # 30-second auto-refresh
clj .serve refresh::0                # Manual refresh only
clj .serve refresh::5 open::1        # Fast refresh, auto-open
```

### Referenced Type

| Type | Kind | Fundamental | Key Constraint |
|------|------|-------------|----------------|
| [`Integer`](../type/04_integer.md) | Fundamental | Integer | Non-negative; 0 = disabled |

### Referenced Parameter Groups

| # | Group | Membership |
|---|-------|------------|
| 5 | [Global](../param_group/05_global.md) | Partial (serve only) |

### Referenced Commands

| # | Command | Default | Notes |
|---|---------|---------|-------|
| 5 | [`.serve`](../command/05_serve.md) | 10 | 10-second refresh |

### Referenced User Stories

| # | User Story | Persona |
|---|------------|---------|
| 1 | [Cost Tracking](../user_story/01_cost_tracking.md) | Developer |
