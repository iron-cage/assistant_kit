# Parameter :: `--new-session`

Edge case tests for the new session flag. Tests validate continuation suppression and help documentation.

**Source:** [params.md](../../../../docs/cli/params.md#parameter--7---new-session)

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| EC-1 | Default → `-c` (continuation) in assembled command | Default |
| EC-2 | `--new-session` → no `-c` in assembled command | Suppression |
| EC-3 | `--new-session` without message → accepted, no error | Edge Case |
| EC-4 | `--new-session` + message → both handled correctly | Interaction |
| EC-5 | `--help` lists `--new-session` | Documentation |
| EC-6 | `--new-session` + `--session-dir` → both accepted, no conflict | Interaction |

## Test Coverage Summary

- Default: 1 test (EC-1)
- Suppression: 1 test (EC-2)
- Edge Case: 1 test (EC-3)
- Interaction: 2 tests (EC-4, EC-6)
- Documentation: 1 test (EC-5)

**Total:** 6 edge cases

**Behavioral Divergence Pair:** EC-1 (valid/expected path) ↔ EC-2 (invalid/rejected path)

## Test Cases
---

### EC-1: Default → `-c` in assembled command:

- **Given:** clean environment
- **When:** `clr --dry-run "Fix bug"`
- **Then:** Assembled command contains `-c` (continuation flag)
- **Exit:** 0
- **Source:** [params.md](../../../../docs/cli/params.md#parameter--7---new-session)
---

### EC-2: `--new-session` suppresses `-c`:

- **Given:** clean environment
- **When:** `clr --dry-run --new-session "Fix bug"`
- **Then:** Assembled command does NOT contain `-c`
- **Exit:** 0
- **Source:** [params.md](../../../../docs/cli/params.md#parameter--7---new-session)
---

### EC-3: `--new-session` without message → no error:

- **Given:** clean environment
- **When:** `clr --dry-run --new-session`
- **Then:** Exit 0; assembled command has no `-c`; no rejection
- **Exit:** 0
- **Source:** [params.md](../../../../docs/cli/params.md#parameter--7---new-session)
---

### EC-4: `--new-session` + message → both handled:

- **Given:** clean environment
- **When:** `clr --dry-run --new-session "Fix bug"`
- **Then:** Message present in assembled command; `-c` absent
- **Exit:** 0
- **Source:** [params.md](../../../../docs/cli/params.md#parameter--7---new-session)
---

### EC-5: `--help` lists `--new-session`:

- **Given:** clean environment
- **When:** `clr --help`
- **Then:** Stdout contains `--new-session`
- **Exit:** 0
- **Source:** [commands.md](../../../../docs/cli/commands.md#command--2-help)
---

### EC-6: `--new-session` + `--session-dir` → no conflict:

- **Given:** clean environment
- **When:** `clr --dry-run --new-session --session-dir /tmp/sessions "Fix bug"`
- **Then:** Both flags accepted; assembled command contains `--session-dir` and no `-c`
- **Exit:** 0
- **Source:** [params.md](../../../../docs/cli/params.md#parameter--7---new-session)
