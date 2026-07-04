# Parameter :: `command`

Edge case tests for the `command` parameter. Tests validate absence
behavior (all commands) and the exact-match constraint.

**Source:** [param/04_command.md](../../../../docs/cli/param/04_command.md)

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| EC-1 | Absent -> all commands shown | Default |
| EC-2 | `command::ask` -> only ask invocations | Parsing |
| EC-3 | `command::as` (partial) -> no match, exact match required | Exact Match |

## Test Coverage Summary

- Default: 1 test (EC-1)
- Parsing: 1 test (EC-2)
- Exact Match: 1 test (EC-3)

**Total:** 3 edge cases

## Test Cases

---

### EC-1: Absent -> all commands shown

- **Given:** journal with events from `run`, `ask`, and `isolated` commands
- **When:** `clj .list`
- **Then:** exit 0; events from all CLR commands are shown
- **Exit:** 0
- **Source:** [param/04_command.md](../../../../docs/cli/param/04_command.md)

---

### EC-2: `command::ask` -> only ask invocations

- **Given:** journal with events from multiple commands including `ask`
- **When:** `clj .list command::ask`
- **Then:** exit 0; only events with `command` field exactly equal to `ask` are shown
- **Exit:** 0
- **Source:** [param/04_command.md](../../../../docs/cli/param/04_command.md)

---

### EC-3: `command::as` (partial) -> no match, exact match required

- **Given:** journal with events from the `ask` command
- **When:** `clj .list command::as`
- **Then:** exit 0; no events are shown, since `as` does not exactly match `ask` (substring match is not performed)
- **Exit:** 0
- **Source:** [param/04_command.md](../../../../docs/cli/param/04_command.md)
