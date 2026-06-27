# CLI Parameter: port

HTTP server port for the embedded web viewer. The server
listens on this port after successful bind. Exit 1 on bind failure.

- **Type:** [`Port`](../type/10_port.md)
- **Default:** 8411
- **Required:** No

```bash
clj .serve                            # Default port 8411
clj .serve port::9090                 # Custom port
clj .serve port::0                    # OS-assigned port
```

### Referenced Type

| Type | Kind | Fundamental | Key Constraint |
|------|------|-------------|----------------|
| [`Port`](../type/10_port.md) | Semantic | Integer | 0-65535; 0 = OS-assigned |

### Referenced Parameter Groups

| # | Group | Membership |
|---|-------|------------|
| 5 | [Global](../param_group/05_global.md) | Partial (serve only) |

### Referenced Commands

| # | Command | Default | Notes |
|---|---------|---------|-------|
| 5 | [`.serve`](../command/05_serve.md) | 8411 | Default viewer port |

### Referenced User Stories

| # | User Story | Persona |
|---|------------|---------|
| 1 | [Cost Tracking](../user_story/01_cost_tracking.md) | Developer |
| 3 | [Automation Audit](../user_story/03_automation_audit.md) | Developer |
