# CLI Parameter: bind

HTTP server bind address. Defaults to `127.0.0.1` (localhost only,
per invariant INV-002). Set to `0.0.0.0` for network-accessible
binding — use with caution as journal data may contain sensitive content.

- **Type:** [`String`](../type/03_string.md)
- **Default:** 127.0.0.1
- **Required:** No

```bash
clj .serve                            # Bind to 127.0.0.1
clj .serve bind::0.0.0.0             # Network-accessible
clj .serve bind::192.168.1.5 port::9090  # Specific interface
```

### Referenced Type

| Type | Kind | Fundamental | Key Constraint |
|------|------|-------------|----------------|
| [`String`](../type/03_string.md) | Fundamental | String | Valid IPv4/IPv6 address |

### Referenced Parameter Groups

| # | Group | Membership |
|---|-------|------------|
| 5 | [Global](../param_group/05_global.md) | Partial (serve only) |

### Referenced Commands

| # | Command | Default | Notes |
|---|---------|---------|-------|
| 5 | [`.serve`](../command/05_serve.md) | 127.0.0.1 | Localhost by default |

### Referenced User Stories

| # | User Story | Persona |
|---|------------|---------|
| 1 | [Cost Tracking](../user_story/001_cost_tracking.md) | Developer |
