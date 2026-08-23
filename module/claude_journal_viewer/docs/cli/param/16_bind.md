# CLI Parameter: bind

HTTP server bind address. Intended to default to `127.0.0.1` (localhost
only, per invariant INV-002), with `0.0.0.0` opting into network-accessible
binding — journal data may contain sensitive content, so exposure must be a
conscious choice.

**Status: not implemented (Phase 2 deliverable).** `cmd_serve()`
(`src/cli_main.rs:168`) hardcodes `format!( "127.0.0.1:{port}" )` and never
reads a `bind` key, so `bind::` is currently accepted-and-ignored rather than
honored: `.serve` is loopback-only regardless of what is passed. INV-002
(`docs/invariant/002_localhost_only.md`) is `Status: Planned` for exactly this
reason. Treat every example below as the intended Phase 2 contract, not
present behavior.

- **Type:** [`String`](../type/03_string.md)
- **Default:** 127.0.0.1 (hardcoded, not yet configurable)
- **Required:** No

```bash
clj .serve                            # Bind to 127.0.0.1 — the only behavior available today
clj .serve bind::0.0.0.0             # Phase 2: network-accessible (currently ignored)
clj .serve bind::192.168.1.5 port::9090  # Phase 2: specific interface (currently ignored)
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
