# Parameter :: `type`

Edge case tests for the `type` parameter. Tests validate absence
behavior (all types) and per-command default variance.

**Source:** [param/03_type.md](../../../../docs/cli/param/03_type.md)

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| EC-1 | Absent on `.list` -> all event types shown | Default |
| EC-2 | `type::retry` on `.tail` -> follows only retry events | Parsing |
| EC-3 | Absent on `.stats` -> defaults to execution events | Per-Command Default |

## Test Coverage Summary

- Default: 1 test (EC-1)
- Parsing: 1 test (EC-2)
- Per-Command Default: 1 test (EC-3)

**Total:** 3 edge cases

## Test Cases

---

### EC-1: Absent on `.list` -> all event types shown

- **Given:** journal with events of multiple types
- **When:** `clj .list`
- **Then:** exit 0; events of all 8 `EventType` variants are shown
- **Exit:** 0
- **Source:** [param/03_type.md](../../../../docs/cli/param/03_type.md)

---

### EC-2: `type::retry` on `.tail` -> follows only retry events

- **Given:** a running journal producing events of multiple types
- **When:** `clj .tail type::retry`
- **Then:** exit 0 on interrupt; only new `retry`-type events appear in the stream
- **Exit:** 0
- **Source:** [param/03_type.md](../../../../docs/cli/param/03_type.md)

---

### EC-3: Absent on `.stats` -> defaults to execution events

- **Given:** journal with events of multiple types including `execution`
- **When:** `clj .stats`
- **Then:** exit 0; aggregate statistics are computed over `execution`-type events only
- **Exit:** 0
- **Source:** [param/03_type.md](../../../../docs/cli/param/03_type.md)
