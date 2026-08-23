# Parameter :: `bind`

Edge case tests for the `bind` parameter. `bind::` is documented but not
wired into `cmd_serve()`, so the executable cases below assert that it is
accepted-and-ignored; the honored-override case is deferred to Phase 2.

**Source:** [param/16_bind.md](../../../../docs/cli/param/16_bind.md)

## Test Case Index

| ID | Test Name | Category | Status |
|----|-----------|----------|--------|
| EC-1 | Absent -> binds to 127.0.0.1 | Default | ✅ |
| EC-2 | `bind::0.0.0.0` -> accepted, ignored, still loopback-only | Unwired Override | ✅ |
| EC-3 | `bind::0.0.0.0` -> network-accessible bind | Parsing | ⏳ Phase 2 |

## Test Coverage Summary

- Default: 1 test (EC-1)
- Unwired Override: 1 test (EC-2)
- Parsing: 1 test (EC-3, deferred)

**Total:** 3 edge cases (2 executable, 1 blocked on Phase 2)

## Test Cases

---

### EC-1: Absent -> binds to 127.0.0.1

- **Given:** clean environment
- **When:** `clj .serve port::8414`
- **Then:** exit 0 on shutdown; `GET http://127.0.0.1:8414/` returns 200, per invariant INV-002
- **Exit:** 0
- **Source:** [param/16_bind.md](../../../../docs/cli/param/16_bind.md)

---

### EC-2: `bind::0.0.0.0` -> accepted, ignored, still loopback-only

- **Given:** clean environment
- **When:** `clj .serve bind::0.0.0.0 port::8415`
- **Then:** exit 0 on shutdown — the unknown key is neither rejected nor honored. The server is still reachable on `127.0.0.1:8415` and still refuses a connection to that port via a non-loopback local address, because `cmd_serve()` (`src/cli_main.rs:168`) hardcodes `format!( "127.0.0.1:{port}" )`
- **Exit:** 0
- **Source:** [param/16_bind.md](../../../../docs/cli/param/16_bind.md) — Status: not implemented

---

### EC-3: `bind::0.0.0.0` -> network-accessible bind — ⏳ Phase 2

- **Given:** clean environment, after `bind::` is wired
- **When:** `clj .serve bind::0.0.0.0 port::8416`
- **Then:** exit 0 on shutdown; the server is reachable from other hosts on the network
- **Blocked on:** `cmd_serve()` reading a `bind` key. Written today this case would pass vacuously — loopback is a subset of `0.0.0.0`, so a naive "can I reach it on 127.0.0.1" assertion succeeds against the hardcoded bind and proves nothing. EC-2 is the inverse assertion that must be flipped when this lands
- **Source:** [param/16_bind.md](../../../../docs/cli/param/16_bind.md)
