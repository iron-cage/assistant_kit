# CLI Parameter: port

HTTP server port for the embedded web viewer. The server
listens on this port after successful bind. Exit 1 on bind failure.

Resolution order is `port::` → `CLJ_PORT` → `0`. There is no fixed
default port: absent both, the OS assigns an ephemeral one, and the
actual port is reported on the startup line
(`Listening on http://localhost:{port}`) — read it there rather than
assuming a well-known number.

A value that is not a port exits 1 with
`Error: invalid integer '<input>' for parameter 'port'`, before anything
binds. That applies to whichever source supplied it: `CLJ_PORT` resolves
into the value of `port::`, so a bad one is rejected identically rather
than falling through to an OS-assigned port.

Only an *absent* `CLJ_PORT` reaches the `0` default. `CLJ_PORT=` — set but
empty — is a value, and an empty string is not a port, so it exits 1 like
any other bad one. Use `unset CLJ_PORT` to return to the default:

```bash
CLJ_PORT=nope clj .serve; echo "exit=$?"    # exit=1
CLJ_PORT= clj .serve;     echo "exit=$?"    # exit=1 — empty is a value, not an absence
unset CLJ_PORT; clj .serve                  # OS-assigned; read the startup line
```

- **Type:** [`Port`](../type/10_port.md)
- **Default:** `0` (OS-assigned)
- **Required:** No

```bash
clj .serve                            # OS-assigned port; read it from the startup line
clj .serve port::9090                 # Custom port
clj .serve port::0                    # OS-assigned port (explicit)
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
| 5 | [`.serve`](../command/05_serve.md) | `0` | OS-assigned unless `port::`/`CLJ_PORT` is set |

### Referenced User Stories

| # | User Story | Persona |
|---|------------|---------|
| 1 | [Cost Tracking](../user_story/001_cost_tracking.md) | Developer |
| 3 | [Automation Audit](../user_story/003_automation_audit.md) | Developer |
