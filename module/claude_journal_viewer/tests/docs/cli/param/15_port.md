# Parameter :: `port`

Edge case tests for the `port` parameter. Tests validate the default
port, a custom port, and the OS-assigned ephemeral shortcut.

**Source:** [param/15_port.md](../../../../docs/cli/param/15_port.md)

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| EC-1 | Absent -> binds to 8411 | Default |
| EC-2 | `port::9090` -> binds to custom port | Parsing |
| EC-3 | `port::0` -> OS-assigned ephemeral port | Special Value |

## Test Coverage Summary

- Default: 1 test (EC-1)
- Parsing: 1 test (EC-2)
- Special Value: 1 test (EC-3)

**Total:** 3 edge cases

## Test Cases

---

### EC-1: Absent -> binds to 8411

- **Given:** port 8411 is free
- **When:** `clj .serve`
- **Then:** exit 0 on shutdown; server binds to port 8411
- **Exit:** 0
- **Source:** [param/15_port.md](../../../../docs/cli/param/15_port.md)

---

### EC-2: `port::9090` -> binds to custom port

- **Given:** port 9090 is free
- **When:** `clj .serve port::9090`
- **Then:** exit 0 on shutdown; server binds to port 9090
- **Exit:** 0
- **Source:** [param/15_port.md](../../../../docs/cli/param/15_port.md)

---

### EC-3: `port::0` -> OS-assigned ephemeral port

- **Given:** clean environment
- **When:** `clj .serve port::0`
- **Then:** exit 0 on shutdown; the OS assigns an ephemeral port, printed to stdout
- **Exit:** 0
- **Source:** [param/15_port.md](../../../../docs/cli/param/15_port.md)
