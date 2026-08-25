# Test: Invariant — Localhost Only

Test case planning for [invariant/002_localhost_only.md](../../../docs/invariant/002_localhost_only.md). Tests validate that `.serve` binds to `127.0.0.1` when `bind::` is absent, that the `bind::` override genuinely selects the interface, and that widening it is surfaced to the user rather than happening silently.

**Source:** [invariant/002_localhost_only.md](../../../docs/invariant/002_localhost_only.md)
**Related:** [feature/002_web_viewing.md](../../../docs/feature/002_web_viewing.md)

## Test Case Index

| ID | Test Name | Category | Status |
|----|-----------|----------|--------|
| IN-1 | `.serve` binds to `127.0.0.1` at an OS-assigned port, with no exposure warning | Default Bind | ✅ |
| IN-2 | `bind::0.0.0.0` is honored, reports its real address, and warns | Consent Signal | ✅ |
| IN-3 | `bind::127.0.0.2` selects that interface — `127.0.0.1` is refused | Explicit Bind | ✅ |

## Test Coverage Summary

- Default Bind: 1 test (IN-1, shared with FT-1)
- Consent Signal: 1 test (IN-2)
- Explicit Bind: 1 test (IN-3)

**Total:** 3 invariant test cases (all executable)

## Architectural Constraint

`0.0.0.0` is a poor discriminator for "did `bind::` take effect", because
loopback is a subset of it: a server that ignored the parameter and bound
`127.0.0.1` would still answer a loopback request on that port. IN-2 therefore
asserts on the two *observable consent signals* — the startup line reporting
the real address instead of `localhost`, and the stderr warning — rather than
on reachability.

IN-3 gets the decisive reachability assertion instead, using `127.0.0.2`. That
address is inside loopback (`127.0.0.0/8`), so nothing leaves the machine and
the test needs no network interface enumeration, yet it is a *different*
address from the old hardcoded `127.0.0.1` — so the case fails against an
unwired `bind::` rather than passing vacuously. The refusal check on
`127.0.0.1` runs only after a successful request to `127.0.0.2`, so the server
is provably listening by then and a refusal cannot be a startup race.

There is no fixed port to assert against by default: the port resolves `port::`
→ `CLJ_PORT` → `0`, so an unpinned `.serve` gets an OS-assigned ephemeral port.
IN-2 and IN-3 pin distinct high ports (19412, 19413) so they cannot collide
with each other or with FT-5 under nextest's parallel execution; IN-1 recovers
the actual port from the `Listening on http://{host}:{port}` startup line,
which is flushed immediately for exactly this reason.

---

### IN-1: `.serve` binds to `127.0.0.1` at an OS-assigned port

- **Given:** temp journal dir; `.serve` started with `port::0` and no `CLJ_PORT`
- **When:** read the startup line from stdout, parse the port out of it, then `GET http://127.0.0.1:{port}/`; afterwards read the captured stderr
- **Then:** startup line matches `Listening on http://localhost:{port}` with a nonzero port; HTTP 200 from loopback; stderr carries **no** exposure warning — the default must never look like an opt-in
- **Source:** [invariant/002_localhost_only.md](../../../docs/invariant/002_localhost_only.md) Threshold: bind address is 127.0.0.1

---

### IN-2: `bind::0.0.0.0` is honored, reports its real address, and warns

- **Given:** temp journal dir; `.serve bind::0.0.0.0 port::19412` started
- **When:** read startup stdout and the captured stderr; `GET http://127.0.0.1:19412/`
- **Then:** the startup line is exactly `Listening on http://0.0.0.0:19412` — not `localhost` — and stderr contains `reachable beyond this machine`; the loopback request still returns 200, since `0.0.0.0` includes loopback
- **Note:** the 200 is context, not the assertion. The startup line and the warning are what distinguish an honored `bind::` from an ignored one here
- **Source:** [invariant/002_localhost_only.md](../../../docs/invariant/002_localhost_only.md) Description: explicit `bind::0.0.0.0` is the consent mechanism

---

### IN-3: `bind::127.0.0.2` selects that interface

- **Given:** temp journal dir; `.serve bind::127.0.0.2 port::19413` started
- **When:** startup line read; `GET http://127.0.0.2:19413/`; then a bare TCP connect to `127.0.0.1:19413`
- **Then:** the startup line is `Listening on http://127.0.0.2:19413`; the request to `127.0.0.2` returns 200; the connect to `127.0.0.1` is **refused** — proving the parameter selected the interface rather than being ignored
- **Source:** [invariant/002_localhost_only.md](../../../docs/invariant/002_localhost_only.md) Method: `bind::` genuinely selects the interface
