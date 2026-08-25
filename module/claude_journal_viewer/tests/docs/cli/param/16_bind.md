# Parameter :: `bind`

Edge case tests for the `bind` parameter. `bind::` is read by `cmd_serve()`
and passed through to `tiny_http::Server::http()`, so the cases below cover
the default, an honored override, and the invalid-value path.

**Source:** [param/16_bind.md](../../../../docs/cli/param/16_bind.md)

## Test Case Index

| ID | Test Name | Category | Status | Implemented as |
|----|-----------|----------|--------|----------------|
| EC-1 | Absent -> binds to 127.0.0.1, no exposure warning | Default | ✅ | IN-1 |
| EC-2 | `bind::0.0.0.0` -> honored; startup line and stderr warning report it | Consent Signal | ✅ | IN-2 |
| EC-3 | `bind::127.0.0.2` -> selects that interface; 127.0.0.1 refused | Interface Select | ✅ | IN-3 |
| EC-4 | Unbindable address -> exit 1 at startup, not a parse error | Invalid Value | ✅ | `it4_busy_pinned_port_exits_1` (shared failure path) |

## Test Coverage Summary

- Default: 1 test (EC-1)
- Consent Signal: 1 test (EC-2)
- Interface Select: 1 test (EC-3)
- Invalid Value: 1 test (EC-4)

**Total:** 4 edge cases (all executable)

## Test Cases

---

### EC-1: Absent -> binds to 127.0.0.1

- **Given:** clean environment
- **When:** `clj .serve port::0`
- **Then:** the startup line reports `localhost`, `GET http://127.0.0.1:{port}/` returns 200, and stderr carries no exposure warning — per invariant INV-002
- **Implemented as:** IN-1 (`ft1_in1_serve_starts_on_loopback_and_prints_url`)
- **Source:** [param/16_bind.md](../../../../docs/cli/param/16_bind.md)

---

### EC-2: `bind::0.0.0.0` -> honored, and the widening is surfaced

- **Given:** clean environment
- **When:** `clj .serve bind::0.0.0.0 port::19412`
- **Then:** the startup line is exactly `Listening on http://0.0.0.0:19412` and stderr contains `reachable beyond this machine`. Loopback still answers, because `0.0.0.0` includes it — which is precisely why reachability is not the assertion here
- **Implemented as:** IN-2 (`in2_non_loopback_bind_is_honored_and_warned`)
- **Source:** [param/16_bind.md](../../../../docs/cli/param/16_bind.md)

---

### EC-3: `bind::127.0.0.2` -> selects that interface

- **Given:** clean environment
- **When:** `clj .serve bind::127.0.0.2 port::19413`
- **Then:** `127.0.0.2:19413` answers 200 while `127.0.0.1:19413` is refused. This is the case that can actually fail against an ignored `bind::`, since a hardcoded `127.0.0.1` bind would invert both halves
- **Implemented as:** IN-3 (`in3_bind_selects_the_interface`)
- **Source:** [param/16_bind.md](../../../../docs/cli/param/16_bind.md)

---

### EC-4: Unbindable address -> exit 1 at startup

- **Given:** an address/port combination the OS will refuse (an already-held port, or an address not assigned to any local interface)
- **When:** `clj .serve bind::<addr> port::<port>`
- **Then:** exit 1 with `Error: could not start server on {addr}: {e}` on stderr. The value is not validated at parse time — it is handed to `tiny_http::Server::http()` and the OS decides, so a malformed address surfaces as a bind failure rather than a parameter rejection
- **Exit:** 1
- **Implemented as:** `it4_busy_pinned_port_exits_1` exercises this exact failure path via a held port; the address half shares one code path with it, so a separate case would assert the same branch twice
- **Source:** [param/16_bind.md](../../../../docs/cli/param/16_bind.md)
