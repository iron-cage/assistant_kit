# Parameter :: `--name`

Edge case coverage for the `--name` parameter. See [078_name.md](../../../../docs/cli/param/078_name.md) for specification.

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| EC-1 | `clr tools --name task` shows only name-matching tools (substring) | Behavioral |
| EC-2 | `clr tools --name TASK` (uppercase) matches same tools as lowercase | Behavioral |
| EC-3 | `clr tools --name doesnotexist` — zero matches, exit 0, no rows | Boundary |
| EC-4 | `clr tools --name cron --category Scheduling` — combines with `--category` (AND) | Interaction |
| EC-5 | `clr tools` with no `--name` flag shows all tools (default = no filter) | Default |
| EC-6 | `clr tools --help` output contains `--name` | Documentation |

## Test Coverage Summary

- Behavioral: 2 tests (EC-1, EC-2)
- Boundary: 1 test (EC-3)
- Interaction: 1 test (EC-4)
- Default: 1 test (EC-5)
- Documentation: 1 test (EC-6)

**Total:** 6 edge cases

---

### EC-1: Substring match on tool name

- **Command:** `clr tools --name task`
- **Expected behavior:** Exit 0; stdout contains `TaskCreate`, `TaskGet`, `TaskList`, `TaskOutput`, `TaskStop`, `TaskUpdate`; stdout does NOT contain `Bash` or `Read`
- **Exit:** 0
- **Source:** [078_name.md](../../../../docs/cli/param/078_name.md)

---

### EC-2: Case-insensitive match

- **Command:** `clr tools --name TASK`
- **Expected behavior:** Exit 0; stdout is identical to `clr tools --name task` — the same 6 `Task*` tools are shown
- **Exit:** 0
- **Source:** [078_name.md](../../../../docs/cli/param/078_name.md)

---

### EC-3: Zero matches is not an error

- **Command:** `clr tools --name doesnotexist`
- **Expected behavior:** Exit 0; stdout contains the table heading/caption but no tool rows
- **Exit:** 0
- **Source:** [078_name.md](../../../../docs/cli/param/078_name.md)

---

### EC-4: Combines with `--category` using AND logic

- **Command:** `clr tools --name cron --category Scheduling`
- **Expected behavior:** Exit 0; stdout contains `CronCreate`, `CronDelete`, `CronList`; stdout does NOT contain `RemoteTrigger` or `ScheduleWakeup` (same category, name does not match "cron")
- **Exit:** 0
- **Source:** [078_name.md](../../../../docs/cli/param/078_name.md)

---

### EC-5: Default shows all tools

- **Command:** `clr tools` (no `--name` flag)
- **Expected behavior:** Exit 0; stdout contains tools with widely differing names (e.g. `Bash` and `WebFetch`) — no filtering applied
- **Exit:** 0
- **Source:** [078_name.md](../../../../docs/cli/param/078_name.md)

---

### EC-6: Help output contains `--name`

- **Command:** `clr tools --help`
- **Expected behavior:** Exit 0; stdout contains `--name`
- **Exit:** 0
- **Source:** [078_name.md](../../../../docs/cli/param/078_name.md)
