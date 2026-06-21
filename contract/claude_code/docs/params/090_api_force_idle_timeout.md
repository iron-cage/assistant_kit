# api_force_idle_timeout

Forces an idle timeout for API connections.

### Forms

| | Value |
|-|-------|
| CLI Flag | — |
| Env Var | `API_FORCE_IDLE_TIMEOUT` |
| Config Key | — |

### Type

integer (milliseconds)

### Default

— (none; uses default connection behavior)

### Since

v2.1.169

### Description

Forces an idle timeout on the HTTP connection to the Anthropic API. When set,
connections that have been idle for longer than this duration are closed and
re-established on the next request. Useful for environments with aggressive
connection-level firewalls or NAT timeouts.

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| doc | [readme.md](readme.md) | Master parameter table |
