# CLI Parameter: bind

HTTP server bind address. Defaults to `127.0.0.1` (localhost only, per
invariant INV-002), with `0.0.0.0` opting into network-accessible binding —
journal data may contain sensitive content, so exposure must be a conscious
choice.

The value is passed through to `tiny_http::Server::http()` unvalidated, so an
unresolvable or already-bound address fails at startup with
`Error: could not start server on {addr}: {e}` and exit 1 rather than being
rejected at parse time. IPv6 addresses take the bracketed form
(`bind::[::1]`), matching the socket-address syntax the server itself parses.

Binding anywhere other than loopback prints a one-line warning to stderr
(`Warning: bound to {addr} — journal data is reachable beyond this machine`)
and makes the startup line report the real address instead of `localhost`, so
a widened bind can never be mistaken for the default.

- **Type:** [`String`](../type/03_string.md)
- **Default:** 127.0.0.1
- **Required:** No

```bash
clj .serve                               # Bind to 127.0.0.1 (default)
clj .serve bind::0.0.0.0                 # Network-accessible — warns on stderr
clj .serve bind::192.168.1.5 port::9090  # Specific interface
```

### Referenced Type

| Type | Kind | Fundamental | Key Constraint |
|------|------|-------------|----------------|
| [`String`](../type/03_string.md) | Fundamental | String | Valid IPv4/IPv6 address (IPv6 bracketed); validated at bind time, not parse time |

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
