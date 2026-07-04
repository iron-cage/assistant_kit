# Parameter :: `include_stdout`

Edge case tests for the `include_stdout` parameter. Tests validate
the default (message-only search) and the extended search scope.

**Source:** [param/28_include_stdout.md](../../../../docs/cli/param/28_include_stdout.md)

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| EC-1 | Absent -> search only matches message field | Default |
| EC-2 | `include_stdout::1` -> search also matches stdout/stderr fields | Parsing |

## Test Coverage Summary

- Default: 1 test (EC-1)
- Parsing: 1 test (EC-2)

**Total:** 2 edge cases

## Test Cases

---

### EC-1: Absent -> search only matches message field

- **Given:** journal with an event whose `stdout` field contains "Fix bug" but whose `message` field does not
- **When:** `clj .search pattern::"Fix bug"`
- **Then:** exit 0; the event is not matched, since only the `message` field is searched
- **Exit:** 0
- **Source:** [param/28_include_stdout.md](../../../../docs/cli/param/28_include_stdout.md)

---

### EC-2: `include_stdout::1` -> search also matches stdout/stderr fields

- **Given:** journal with an event whose `stdout` field contains "Fix bug" but whose `message` field does not
- **When:** `clj .search pattern::"Fix bug" include_stdout::1`
- **Then:** exit 0; the event is matched, since `stdout` and `stderr` fields are also searched
- **Exit:** 0
- **Source:** [param/28_include_stdout.md](../../../../docs/cli/param/28_include_stdout.md)
