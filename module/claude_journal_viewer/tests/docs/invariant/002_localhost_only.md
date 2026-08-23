# Test: Invariant — Localhost Only

Test case planning for [invariant/002_localhost_only.md](../../../docs/invariant/002_localhost_only.md). Tests validate that `.serve` binds to `127.0.0.1` unconditionally, and that the `bind::` override — a Phase 2 deliverable — is not yet honored.

**Source:** [invariant/002_localhost_only.md](../../../docs/invariant/002_localhost_only.md)
**Related:** [feature/002_web_viewing.md](../../../docs/feature/002_web_viewing.md)

## Test Case Index

| ID | Test Name | Category | Status |
|----|-----------|----------|--------|
| IN-1 | `.serve` binds to `127.0.0.1` at an OS-assigned port | Default Bind | ✅ |
| IN-2 | `bind::0.0.0.0` does not widen the bind — still loopback-only | Unwired Override | ✅ |
| IN-3 | `bind::` becomes an honored override | Explicit Bind | ⏳ Phase 2 |

## Test Coverage Summary

- Default Bind: 1 test (IN-1)
- Unwired Override: 1 test (IN-2)
- Explicit Bind: 1 test (IN-3, deferred)

**Total:** 3 invariant test cases (2 executable, 1 blocked on Phase 2)

## Architectural Constraint

The invariant is `Status: Planned`, and the reason is that its own opt-out
mechanism does not exist yet. `cmd_serve()` (`src/cli_main.rs:168`) builds its
address as the literal `format!( "127.0.0.1:{port}" )` and never reads a `bind`
key, so loopback-only holds by construction rather than by default. That makes
IN-1 and IN-2 testable today and IN-3 untestable until `bind::` is wired.

There is also no fixed port to assert against: the port resolves `port::` →
`CLJ_PORT` → `0`, so an unpinned `.serve` gets an OS-assigned ephemeral port.
Every case below therefore pins the port explicitly, or recovers the actual one
from the `Listening on http://localhost:{port}` startup line, which is flushed
immediately for exactly this reason.

---

### IN-1: `.serve` binds to `127.0.0.1` at an OS-assigned port

- **Given:** temp journal dir; `.serve` started with no `port::` and no `CLJ_PORT`
- **When:** read the startup line from stdout, parse the port out of it, then `GET http://127.0.0.1:{port}/`
- **Then:** startup line matches `Listening on http://localhost:{port}` with a nonzero port; HTTP 200 from loopback
- **Source:** [invariant/002_localhost_only.md](../../../docs/invariant/002_localhost_only.md) Threshold: bind address is 127.0.0.1

---

### IN-2: `bind::0.0.0.0` does not widen the bind — still loopback-only

- **Given:** temp journal dir; `.serve bind::0.0.0.0 port::8412` started
- **When:** read startup stdout; `GET http://127.0.0.1:8412/`; then attempt a connection to the same port via a non-loopback local address
- **Then:** exit 0 on shutdown and HTTP 200 from loopback — the parameter is accepted-and-ignored, not rejected — while the non-loopback attempt is refused, proving `bind::` did not take effect. This test asserts present behavior; it is the one that must be inverted when IN-3 lands
- **Source:** [invariant/002_localhost_only.md](../../../docs/invariant/002_localhost_only.md) Sources: `bind::` is not yet wired into `cmd_serve()`

---

### IN-3: `bind::` becomes an honored override — ⏳ Phase 2

- **Given:** temp journal dir; `.serve bind::0.0.0.0 port::8413` started, after `bind::` is wired
- **When:** read startup stdout; connect from a non-loopback local address
- **Then:** the server accepts the connection, and the startup line reports the configured address rather than `localhost`
- **Blocked on:** `cmd_serve()` reading a `bind` key at all. Until then this case cannot fail for the right reason — it would pass vacuously against a hardcoded loopback bind only because loopback is a subset of `0.0.0.0`
- **Source:** [invariant/002_localhost_only.md](../../../docs/invariant/002_localhost_only.md) Description: explicit `bind::0.0.0.0` is the consent mechanism
