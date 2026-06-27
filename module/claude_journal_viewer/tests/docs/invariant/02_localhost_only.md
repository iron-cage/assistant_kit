# Test: Invariant — Localhost Only

Test case planning for [invariant/002_localhost_only.md](../../../docs/invariant/002_localhost_only.md). Tests validate that `.serve` binds to `127.0.0.1` by default and that explicit `bind::0.0.0.0` overrides the default.

**Source:** [invariant/002_localhost_only.md](../../../docs/invariant/002_localhost_only.md)
**Related:** [feature/002_web_viewing.md](../../../docs/feature/002_web_viewing.md)

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| IN-1 | Default `.serve` binds to `127.0.0.1:8411`; connection from non-loopback refused | Default Bind |
| IN-2 | `bind::0.0.0.0` overrides default — server accepts the configured address | Explicit Bind |

## Test Coverage Summary

- Default Bind: 1 test (IN-1)
- Explicit Bind: 1 test (IN-2)

**Total:** 2 invariant test cases

## Architectural Constraint

IN-1 verifies the default bind address by inspecting the startup URL printed to stdout (must contain `127.0.0.1`) and by successfully connecting to `127.0.0.1:8411` while confirming the port is NOT listening on all interfaces without explicit configuration.

IN-2 starts the server with `bind::0.0.0.0 port::8412` (alternate port to avoid collision) and confirms the server prints `0.0.0.0:8412` or similar.

---

### IN-1: Default bind is `127.0.0.1:8411`

- **Given:** temp journal dir; `.serve` started with no `bind::` param
- **When:** read startup stdout; attempt `GET http://127.0.0.1:8411/`
- **Then:** startup line contains `127.0.0.1`; HTTP 200 from `127.0.0.1:8411`; no binding on `0.0.0.0` (verified by unit test asserting default constant equals `"127.0.0.1"`)
- **Source:** [invariant/002_localhost_only.md](../../../docs/invariant/002_localhost_only.md) Threshold: default bind address is 127.0.0.1

---

### IN-2: `bind::0.0.0.0` overrides default

- **Given:** temp journal dir; `.serve bind::0.0.0.0 port::8412` started
- **When:** read startup stdout; attempt `GET http://127.0.0.1:8412/`
- **Then:** startup line contains `0.0.0.0:8412` or `http://0.0.0.0:8412`; HTTP 200 from `127.0.0.1:8412` (loopback is a subset of 0.0.0.0)
- **Source:** [invariant/002_localhost_only.md](../../../docs/invariant/002_localhost_only.md) Measurement: `bind::` param overrides default
