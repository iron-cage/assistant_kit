# Parameter :: `bind`

Edge case tests for the `bind` parameter. Tests validate the
localhost-only default and network-accessible binding.

**Source:** [param/16_bind.md](../../../../docs/cli/param/16_bind.md)

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| EC-1 | Absent -> binds to 127.0.0.1 only | Default |
| EC-2 | `bind::0.0.0.0` -> network-accessible bind | Parsing |

## Test Coverage Summary

- Default: 1 test (EC-1)
- Parsing: 1 test (EC-2)

**Total:** 2 edge cases

## Test Cases

---

### EC-1: Absent -> binds to 127.0.0.1 only

- **Given:** clean environment
- **When:** `clj .serve`
- **Then:** exit 0 on shutdown; server is reachable only from localhost, per invariant INV-002
- **Exit:** 0
- **Source:** [param/16_bind.md](../../../../docs/cli/param/16_bind.md)

---

### EC-2: `bind::0.0.0.0` -> network-accessible bind

- **Given:** clean environment
- **When:** `clj .serve bind::0.0.0.0`
- **Then:** exit 0 on shutdown; server is reachable from other hosts on the network
- **Exit:** 0
- **Source:** [param/16_bind.md](../../../../docs/cli/param/16_bind.md)
