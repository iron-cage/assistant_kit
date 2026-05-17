# Parameter :: `--no-persist`

Edge case coverage for the `--no-persist` parameter. See [22_no_persist.md](../../../../docs/cli/param/22_no_persist.md) for specification.

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| EC-1 | `--no-persist` → `--no-session-persistence` in assembled command | Behavioral Divergence |
| EC-2 | Default (no `--no-persist`) → no `--no-session-persistence` in assembled command | Behavioral Divergence |
| EC-3 | `--no-persist` without message → accepted, no error | Edge Case |
| EC-4 | `--help` output contains `--no-persist` | Documentation |
| EC-5 | `--no-persist` + `--new-session` → both accepted, coexist | Interaction |
| EC-6 | `--no-persist` + `--dry-run` → preview shows `--no-session-persistence` | Interaction |

## Test Coverage Summary

- Behavioral Divergence: 2 tests (EC-1, EC-2)
- Edge Case: 1 test
- Interaction: 2 tests
- Documentation: 1 test

**Total:** 6 edge cases

---

### EC-1: `--no-persist` forwards `--no-session-persistence`

- **Given:** clean environment
- **When:** `clr --dry-run --no-persist "Fix the bug"`
- **Then:** Assembled command contains `--no-session-persistence`
- **Exit:** 0
- **Source:** [--no-persist](../../../../docs/cli/param/22_no_persist.md)

---

### EC-2: Default → no `--no-session-persistence`

- **Given:** clean environment
- **When:** `clr --dry-run "Fix the bug"`
- **Then:** Assembled command does NOT contain `--no-session-persistence`
- **Exit:** 0
- **Source:** [--no-persist](../../../../docs/cli/param/22_no_persist.md)

---

### EC-3: `--no-persist` without message → accepted, no error

- **Given:** clean environment
- **When:** `clr --dry-run --no-persist`
- **Then:** Exit 0; assembled command contains `--no-session-persistence`
- **Exit:** 0
- **Source:** [--no-persist](../../../../docs/cli/param/22_no_persist.md)

---

### EC-4: `--help` lists `--no-persist`

- **Given:** clean environment
- **When:** `clr --help`
- **Then:** Stdout contains `--no-persist`
- **Exit:** 0
- **Source:** [command.md — help](../../../../docs/cli/command.md#command--2-help)

---

### EC-5: `--no-persist` + `--new-session` → both accepted

- **Given:** clean environment
- **When:** `clr --dry-run --no-persist --new-session "Fix the bug"`
- **Then:** Assembled command contains `--no-session-persistence` and does NOT contain `-c`
- **Exit:** 0
- **Source:** [--no-persist](../../../../docs/cli/param/22_no_persist.md)

---

### EC-6: `--no-persist` + `--dry-run` → preview shows flag

- **Given:** clean environment
- **When:** `clr --dry-run --no-persist "Fix the bug"`
- **Then:** Stdout contains `--no-session-persistence`; stderr empty
- **Exit:** 0
- **Source:** [--no-persist](../../../../docs/cli/param/22_no_persist.md)
