# Parameter :: `port`

Edge case tests for the `port` parameter. Tests validate the three-level
resolution order (`port::` -> `CLJ_PORT` -> `0`), a custom port, and the
OS-assigned ephemeral default.

**Source:** [param/15_port.md](../../../../docs/cli/param/15_port.md)

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| EC-1 | Absent, no `CLJ_PORT` -> OS-assigned ephemeral port | Default |
| EC-2 | `port::9090` -> binds to custom port | Parsing |
| EC-3 | `CLJ_PORT=9091`, no `port::` -> binds to the env port | Env Fallback |
| EC-4 | `port::9092` with `CLJ_PORT=9091` -> CLI wins | Precedence |

## Test Coverage Summary

- Default: 1 test (EC-1)
- Parsing: 1 test (EC-2)
- Env Fallback: 1 test (EC-3)
- Precedence: 1 test (EC-4)

**Total:** 4 edge cases

## Test Cases

---

### EC-1: Absent, no `CLJ_PORT` -> OS-assigned ephemeral port

- **Given:** `CLJ_PORT` unset in the child environment
- **When:** `clj .serve`
- **Then:** exit 0 on shutdown; the startup line `Listening on http://localhost:{port}` reports a nonzero port the OS assigned. There is no fixed default port to assert against — the port must be parsed out of that line
- **Exit:** 0
- **Source:** [param/15_port.md](../../../../docs/cli/param/15_port.md)

---

### EC-2: `port::9090` -> binds to custom port

- **Given:** port 9090 is free
- **When:** `clj .serve port::9090`
- **Then:** exit 0 on shutdown; startup line reports port 9090; `GET http://127.0.0.1:9090/` returns 200
- **Exit:** 0
- **Source:** [param/15_port.md](../../../../docs/cli/param/15_port.md)

---

### EC-3: `CLJ_PORT=9091`, no `port::` -> binds to the env port

- **Given:** `CLJ_PORT=9091` in the child environment; port 9091 free
- **When:** `clj .serve`
- **Then:** exit 0 on shutdown; startup line reports port 9091 — the env var is consulted only when `port::` is absent
- **Exit:** 0
- **Source:** [param/15_port.md](../../../../docs/cli/param/15_port.md), [002_env_param.md](../../../../docs/cli/002_env_param.md)

---

### EC-4: `port::9092` with `CLJ_PORT=9091` -> CLI wins

- **Given:** `CLJ_PORT=9091` in the child environment; ports 9091 and 9092 free
- **When:** `clj .serve port::9092`
- **Then:** exit 0 on shutdown; startup line reports 9092, not 9091 — CLI param always overrides the env var
- **Exit:** 0
- **Source:** [002_env_param.md](../../../../docs/cli/002_env_param.md) — Precedence

---

## Note on unparseable input

`port_str.parse().unwrap_or( 0 )` swallows any parse failure, so
`port::notanumber` silently falls back to an OS-assigned port rather than
exiting 1. That is present behavior, not a documented contract — no case above
asserts it, deliberately, so that tightening it later is not a test break.
