# Test: `tools`

Integration test planning for the `tools` command. See [command/08_tools.md](../../../../docs/cli/command/08_tools.md) for specification.

`tools` lists all Claude Code built-in tools (40 as of the contract doc's current tool count —
see [invariant/015](../../../docs/invariant/015_tools_array_doc_sync.md) for the sync guard that
keeps this accurate) with name, category, and description in a plain-style table, with optional
`--name`/`--category` filtering, `--columns`/`--value` projection, and `--inspect` record format.
Tests verify that the table is printed with correct tool names, categories, and a caption; that
filters combine with AND logic; that projection/format modes behave correctly and interact
correctly with each other; that help flags work; and that `tools` is listed in main help output.

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| IT-1 | `clr tools` exits 0 | Happy path |
| IT-2 | Stdout lists core tool names | Content |
| IT-3 | Stdout lists tool categories | Content |
| IT-4 | Stdout contains table caption with correct count | Content |
| IT-5 | `clr tools --help` exits 0 | Help flag |
| IT-6 | Stdout lists scheduling and mode tools | Content |
| IT-7 | `clr --help` mentions `tools` command | Help listing |
| IT-8 | `clr tools -h` exits 0 | Help flag |
| IT-9 | `clr tools <unexpected-arg>` exits 1 | Error rejection |
| IT-10 | `clr tools --name task` filters to name-matching tools only | Filter |
| IT-11 | `clr tools --category Web` filters to category-matching tools only | Filter |
| IT-12 | `clr tools --name cron --category Scheduling` combines filters with AND | Filter |
| IT-13 | `clr tools --name doesnotexist` — zero matches exits 0 with empty table | Filter Boundary |
| IT-14 | `clr tools --columns name,category` shows only the 2 selected columns | Projection |
| IT-15 | `clr tools --columns badkey` exits 1 with valid-keys error | Projection Error |
| IT-16 | `clr tools --value name` prints bare names, one per line, no table decoration | Value Mode |
| IT-17 | `clr tools --name Bash --value category` prints exactly one bare value | Value Mode |
| IT-18 | `clr tools --inspect` prints key:value record blocks for all tools | Inspect Mode |
| IT-19 | `clr tools --value name --inspect` exits 1 (mutually exclusive) | Error Rejection |
| IT-20 | `clr tools --columns name --inspect` ignores `--columns`, shows full record | Interaction |
| IT-21 | `clr tools` stdout tool count matches `contract/claude_code/docs/tool/readme.md` row count | Sync Guard |

## Test Coverage Summary

- Happy path: 1 test (IT-1)
- Content: 4 tests (IT-2, IT-3, IT-4, IT-6)
- Help flag: 2 tests (IT-5, IT-8)
- Help listing: 1 test (IT-7)
- Error rejection: 2 tests (IT-9, IT-19)
- Filter: 3 tests (IT-10, IT-11, IT-12)
- Filter Boundary: 1 test (IT-13)
- Projection: 1 test (IT-14)
- Projection Error: 1 test (IT-15)
- Value Mode: 2 tests (IT-16, IT-17)
- Inspect Mode: 1 test (IT-18)
- Interaction: 1 test (IT-20)
- Sync Guard: 1 test (IT-21)

**Total:** 21 tests

---

### IT-1: `clr tools` exits 0

- **Command:** `clr tools`
- **Expected behavior:** exit 0; stdout is non-empty (table is printed)
- **Exit:** 0
- **Source:** [command/08_tools.md](../../../../docs/cli/command/08_tools.md)

---

### IT-2: Stdout lists core tool names

- **Command:** `clr tools`
- **Expected behavior:** exit 0; stdout contains "Read", "Write", "Edit", "Bash", and "Agent"
- **Exit:** 0
- **Source:** [command/08_tools.md](../../../../docs/cli/command/08_tools.md)

---

### IT-3: Stdout lists tool categories

- **Command:** `clr tools`
- **Expected behavior:** exit 0; stdout contains "File Operations", "Shell", "Search", and "Web"
- **Exit:** 0
- **Source:** [command/08_tools.md](../../../../docs/cli/command/08_tools.md)

---

### IT-4: Stdout contains table caption with correct count

- **Command:** `clr tools`
- **Expected behavior:** exit 0; stdout contains "Claude Code Tools" and the current `TOOLS.len()` value (read dynamically from the built binary — do not hardcode a literal count in the test assertion, since this value must track the contract doc per invariant/015, not stay fixed)
- **Exit:** 0
- **Source:** [command/08_tools.md](../../../../docs/cli/command/08_tools.md)

---

### IT-5: `--help` flag

- **Command:** `clr tools --help`
- **Expected behavior:** exit 0; stdout contains "tools"
- **Exit:** 0
- **Source:** [command/08_tools.md](../../../../docs/cli/command/08_tools.md)

---

### IT-6: Stdout lists scheduling and mode tools

- **Command:** `clr tools`
- **Expected behavior:** exit 0; stdout contains "CronCreate", "CronDelete", "CronList", "EnterPlanMode", and "ExitPlanMode"
- **Exit:** 0
- **Source:** [command/08_tools.md](../../../../docs/cli/command/08_tools.md)

---

### IT-7: `clr --help` mentions `tools` command

- **Command:** `clr --help`
- **Expected behavior:** exit 0; stdout contains "tools"
- **Exit:** 0
- **Source:** [command/08_tools.md](../../../../docs/cli/command/08_tools.md)

---

### IT-8: `-h` short flag

- **Command:** `clr tools -h`
- **Expected behavior:** exit 0; stdout contains "tools"
- **Exit:** 0
- **Source:** [command/08_tools.md](../../../../docs/cli/command/08_tools.md)

---

### IT-9: Unexpected argument rejected

- **Command:** `clr tools some-unknown-arg`
- **Expected behavior:** exit 1; stderr contains "does not accept arguments"
- **Exit:** 1
- **Source:** [command/08_tools.md](../../../../docs/cli/command/08_tools.md)

---

### IT-10: `--name` filters to name-matching tools only

- **Command:** `clr tools --name task`
- **Expected behavior:** exit 0; stdout contains "TaskCreate", "TaskGet", "TaskList"; stdout does NOT contain "Bash" or "Read"
- **Exit:** 0
- **Source:** [param/078_name.md](../../../../docs/cli/param/078_name.md)

---

### IT-11: `--category` filters to category-matching tools only

- **Command:** `clr tools --category Web`
- **Expected behavior:** exit 0; stdout contains "WebFetch" and "WebSearch"; stdout does NOT contain "Bash"
- **Exit:** 0
- **Source:** [param/079_category.md](../../../../docs/cli/param/079_category.md)

---

### IT-12: `--name` and `--category` combine with AND logic

- **Command:** `clr tools --name cron --category Scheduling`
- **Expected behavior:** exit 0; stdout contains "CronCreate", "CronDelete", "CronList"; stdout does NOT contain "RemoteTrigger" or "ScheduleWakeup" (same category, name does not match "cron")
- **Exit:** 0
- **Source:** [param_group/07_tool_listing.md](../../../../docs/cli/param_group/07_tool_listing.md)

---

### IT-13: Zero matches after filtering is not an error

- **Command:** `clr tools --name doesnotexist`
- **Expected behavior:** exit 0; stdout contains the table heading/caption but no tool rows
- **Exit:** 0
- **Source:** [param/078_name.md](../../../../docs/cli/param/078_name.md)

---

### IT-14: `--columns` narrows displayed columns

- **Command:** `clr tools --columns name,category`
- **Expected behavior:** exit 0; stdout header row contains "Tool" and "Category" but not "Description"
- **Exit:** 0
- **Source:** [param/059_columns.md](../../../../docs/cli/param/059_columns.md)

---

### IT-15: Unknown `--columns` key rejected

- **Command:** `clr tools --columns badkey`
- **Expected behavior:** exit 1; stderr lists valid keys (`idx`, `name`, `category`, `desc`)
- **Exit:** 1
- **Source:** [param/059_columns.md](../../../../docs/cli/param/059_columns.md)

---

### IT-16: `--value` prints bare column values

- **Command:** `clr tools --value name`
- **Expected behavior:** exit 0; stdout contains "Bash" on its own line with no table header row, no "#"/"Category"/"Description" column headers, and no heading/caption line
- **Exit:** 0
- **Source:** [param/080_value.md](../../../../docs/cli/param/080_value.md)

---

### IT-17: `--value` combined with a single-row filter prints exactly one value

- **Command:** `clr tools --name Bash --value category`
- **Expected behavior:** exit 0; stdout is exactly `Shell` (plus trailing newline) — a single bare cell
- **Exit:** 0
- **Source:** [param/080_value.md](../../../../docs/cli/param/080_value.md)

---

### IT-18: `--inspect` prints key:value record blocks

- **Command:** `clr tools --inspect`
- **Expected behavior:** exit 0; stdout contains "name:", "category:", "desc:" key labels; stdout does NOT contain a table header row
- **Exit:** 0
- **Source:** [param/069_inspect.md](../../../../docs/cli/param/069_inspect.md)

---

### IT-19: `--value` and `--inspect` are mutually exclusive

- **Command:** `clr tools --value name --inspect`
- **Expected behavior:** exit 1; stderr states the two flags cannot be combined
- **Exit:** 1
- **Source:** [param_group/07_tool_listing.md](../../../../docs/cli/param_group/07_tool_listing.md)

---

### IT-20: `--columns` ignored when `--inspect` active

- **Command:** `clr tools --columns name --inspect`
- **Expected behavior:** exit 0; output is inspect-format record blocks showing all 4 attributes, not a table restricted to the `name` column
- **Exit:** 0
- **Source:** [param/059_columns.md](../../../../docs/cli/param/059_columns.md)

---

### IT-21: `TOOLS` array count matches contract doc (sync guard)

- **Command:** `clr tools` (or direct unit-level comparison in the sync-guard test itself)
- **Expected behavior:** the number of tool rows printed equals the row count of `contract/claude_code/docs/tool/readme.md`'s Tool Table at test time — this is the integration-level manifestation of the [invariant/015](../../../docs/invariant/015_tools_array_doc_sync.md) sync guard; the guard's own primary test lives alongside the crate's unit tests, this case just pins the user-visible symptom
- **Exit:** 0
- **Source:** [invariant/015_tools_array_doc_sync.md](../../../docs/invariant/015_tools_array_doc_sync.md)
