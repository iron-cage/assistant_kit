# Type :: `Port`

Validation tests for the `Port` semantic type. Tests validate the
ephemeral-port shortcut, the unprivileged range, out-of-range rejection,
and bind-failure handling.

**Source:** [type/10_port.md](../../../../docs/cli/type/10_port.md)

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| TC-1 | `0` -> OS assigns an ephemeral port | Special Value |
| TC-2 | Value in 1024-65535 -> accepted, unprivileged | Parsing |
| TC-3 | Value > 65535 -> exit 1 | Error Handling |
| TC-4 | Port already in use -> exit 1, bind failure | Error Handling |

## Test Coverage Summary

- Special Value: 1 test (TC-1)
- Parsing: 1 test (TC-2)
- Error Handling: 2 tests (TC-3, TC-4)

**Total:** 4 test cases

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
- **Then:** exit 1; stderr indicates the port value is out of range
- **Exit:** 1
- **Source:** [type/10_port.md](../../../../docs/cli/type/10_port.md)

---

### TC-4: Port already in use -> exit 1, bind failure

- **Given:** another process is already bound to port 8411
- **When:** `clj .serve port::8411`
- **Then:** exit 1; stderr contains a bind failure message
- **Exit:** 1
- **Source:** [type/10_port.md](../../../../docs/cli/type/10_port.md), [command/05_serve.md](../../../../docs/cli/command/05_serve.md)
