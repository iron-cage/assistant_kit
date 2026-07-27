# Parameter Group :: Tool Listing

Test case spec for [07_tool_listing.md](../../../../docs/cli/param_group/07_tool_listing.md).

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| G7-CC1 | All 5 params consumed by `dispatch_tools()` — none affect subprocess execution | Consumption Pattern |
| G7-CC2 | `--name`, `--category`, `--columns` accepted by `clr tools` without error | Acceptance |
| G7-CC3 | `clr tools --name cron --category Scheduling` → both filters apply with AND logic | Interaction |
| G7-CC4 | None of `--name`, `--category`, `--value` appear in `clr run --help` output | Exclusivity |
| G7-CC5 | `clr tools --value name --inspect` → exit 1, mutually exclusive | Interaction |
| G7-CC6 | `clr tools --columns name --value category` → `--columns` ignored, bare value printed | Interaction |

## Test Coverage Summary

- Consumption Pattern: 1 test (G7-CC1)
- Acceptance: 1 test (G7-CC2)
- Interaction: 3 tests (G7-CC3, G7-CC5, G7-CC6)
- Exclusivity: 1 test (G7-CC4)

**Total:** 6 tests

---

### G7-CC1: Params consumed by `dispatch_tools()` only

- **Setup:** none (static `TOOLS` array, no external process or filesystem state)
- **Command:** `clr tools --name Bash --columns name,category`
- **Expected behavior:** Exit 0; params control table output without affecting any subprocess; no subprocess is spawned
- **Exit:** 0
- **Source:** [07_tool_listing.md](../../../../docs/cli/param_group/07_tool_listing.md)
- **Note:** `tools` is a static, read-only listing command — it never spawns a `claude` subprocess

---

### G7-CC2: Core params accepted without error

- **Command:** `clr tools --name Bash --category Shell --columns name,category`
- **Expected behavior:** Exit 0; no error on stderr about unknown flags
- **Exit:** 0
- **Source:** [07_tool_listing.md](../../../../docs/cli/param_group/07_tool_listing.md)
- **Note:** `--value` and `--inspect` are tested separately in their own EC files and in G7-CC5/G7-CC6; this test validates the core 3 filter/projection params are accepted together

---

### G7-CC3: `--name` and `--category` combine with AND logic

- **Command:** `clr tools --name cron --category Scheduling`
- **Expected behavior:** Exit 0; stdout contains "CronCreate", "CronDelete", "CronList"; stdout does NOT contain "RemoteTrigger" or "ScheduleWakeup" (same category, but name does not match "cron")
- **Exit:** 0
- **Source:** [07_tool_listing.md](../../../../docs/cli/param_group/07_tool_listing.md)

---

### G7-CC4: Tools-exclusive params not in `clr run --help`

- **Command:** `clr run --help` (or `clr --help`)
- **Expected behavior:** Exit 0; stdout does NOT contain `--name`, `--category`, or `--value` as tools-scoped flags
- **Exit:** 0
- **Source:** [07_tool_listing.md](../../../../docs/cli/param_group/07_tool_listing.md)
- **Note:** Verifies semantic coherence — `--name`/`--category`/`--value` are exclusive to `clr tools`. `--columns`/`--inspect` are shared with the Session Listing group and their exclusivity from `run` is already covered by G5-CC4; not duplicated here.

---

### G7-CC5: `--value` and `--inspect` are mutually exclusive

- **Command:** `clr tools --value name --inspect`
- **Expected behavior:** Exit 1; stderr states the two flags cannot be combined
- **Exit:** 1
- **Source:** [07_tool_listing.md](../../../../docs/cli/param_group/07_tool_listing.md)

---

### G7-CC6: `--columns` ignored when `--value` active

- **Command:** `clr tools --columns name --value category`
- **Expected behavior:** Exit 0; output is bare `category` values only (one per line) — not a table restricted to the `name` column
- **Exit:** 0
- **Source:** [07_tool_listing.md](../../../../docs/cli/param_group/07_tool_listing.md)
- **Note:** mirrors G5-CC7's "--inspect ignores --columns" pattern, applied to `--value` mode specifically; the `--inspect`+`--columns` interaction is separately covered by IT-20 in `tests/docs/cli/command/08_tools.md`
