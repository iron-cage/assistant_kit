# Type :: `Port`

Validation tests for the `Port` semantic type. Tests validate the
ephemeral-port shortcut, the unprivileged range, out-of-range rejection,
and bind-failure handling.

**Source:** [type/10_port.md](../../../../docs/cli/type/10_port.md)

## Test Case Index

| ID | Test Name | Category | Status | Implemented as |
|----|-----------|----------|--------|----------------|
| TC-1 | `0` -> OS assigns an ephemeral port | Special Value | ✅ | `ft1_in1_serve_starts_on_loopback_and_prints_url` |
| TC-2 | Value in 1024-65535 -> accepted, unprivileged | Parsing | ✅ | `ft5_port_override_binds_requested_port` |
| TC-3 | Value > 65535 -> exit 1 | Error Handling | ✅ | `ft14_serve_validates_port_and_open_before_binding` |
| TC-4 | Port already in use -> exit 1, bind failure | Error Handling | ✅ | `it4_busy_pinned_port_exits_1` |

## Test Coverage Summary

- Special Value: 1 test (TC-1)
- Parsing: 1 test (TC-2)
- Error Handling: 2 tests (TC-3, TC-4)

**Total:** 4 test cases

TC-3 and TC-4 are both "exit 1" and are deliberately checked by different
assertions. A bind failure and a rejected value produce different messages,
and until FT-14 landed only TC-4's was reachable: `port::` resolved through
`.unwrap_or( 0 )`, so an out-of-range value never failed at all — it bound an
OS-assigned port and reported success.

## Test Cases

---

### TC-1: `0` -> OS assigns an ephemeral port

- **Given:** clean environment
- **When:** `clj .serve port::0`
- **Then:** exit 0 on shutdown; server starts on an OS-assigned ephemeral port, printed to stdout
- **Exit:** 0
- **Source:** [type/10_port.md](../../../../docs/cli/type/10_port.md)

---

### TC-2: Value in 1024-65535 -> accepted, unprivileged

- **Given:** port 9090 is free
- **When:** `clj .serve port::9090`
- **Then:** exit 0 on shutdown; server binds to port 9090 without requiring elevated privileges
- **Exit:** 0
- **Source:** [type/10_port.md](../../../../docs/cli/type/10_port.md), [param/15_port.md](../../../../docs/cli/param/15_port.md)

---

### TC-3: Value > 65535 -> exit 1

- **Given:** clean environment
- **When:** `clj .serve port::70000`
- **Then:** exit 1 before anything binds; stderr contains `invalid integer '70000' for parameter 'port'`. `CLJ_PORT` resolves into the same value and fails identically
- **Exit:** 1
- **Source:** [type/10_port.md](../../../../docs/cli/type/10_port.md)

---

### TC-4: Port already in use -> exit 1, bind failure

- **Given:** another process is already bound to port 8411
- **When:** `clj .serve port::8411`
- **Then:** exit 1; stderr contains a bind failure message
- **Exit:** 1
- **Source:** [type/10_port.md](../../../../docs/cli/type/10_port.md), [command/05_serve.md](../../../../docs/cli/command/05_serve.md)
