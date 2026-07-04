# Parameter :: `pattern`

Edge case tests for the `pattern` parameter. Tests validate the
required-parameter constraint and regex matching against the message field.

**Source:** [param/14_pattern.md](../../../../docs/cli/param/14_pattern.md)

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| EC-1 | Absent on `.search` -> error, required parameter missing | Required |
| EC-2 | `pattern::"rate limit"` -> matches message field | Parsing |
| EC-3 | `pattern::"(?i)panic"` -> case-insensitive via regex flag | Parsing |

## Test Coverage Summary

- Required: 1 test (EC-1)
- Parsing: 2 tests (EC-2, EC-3)

**Total:** 3 edge cases

## Test Cases

---

### EC-1: Absent on `.search` -> error, required parameter missing

- **Given:** clean environment
- **When:** `clj .search`
- **Then:** exit 1; stderr indicates `pattern` is a required parameter
- **Exit:** 1
- **Source:** [param/14_pattern.md](../../../../docs/cli/param/14_pattern.md)

---

### EC-2: `pattern::"rate limit"` -> matches message field

- **Given:** journal with an event whose message field contains "rate limit"
- **When:** `clj .search pattern::"rate limit"`
- **Then:** exit 0; the matching event is shown
- **Exit:** 0
- **Source:** [param/14_pattern.md](../../../../docs/cli/param/14_pattern.md)

---

### EC-3: `pattern::"(?i)panic"` -> case-insensitive via regex flag

- **Given:** journal with events containing "Panic" and "panic" in their message field
- **When:** `clj .search pattern::"(?i)panic"`
- **Then:** exit 0; both events are matched regardless of case
- **Exit:** 0
- **Source:** [param/14_pattern.md](../../../../docs/cli/param/14_pattern.md)
