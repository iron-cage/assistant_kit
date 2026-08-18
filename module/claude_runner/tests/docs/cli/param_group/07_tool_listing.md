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

## Implementation Map

All contract cases live in `tests/tools_command_test.rs`, whose own numbering is `IT-N`
(from `command/08_tools.md`). `G7-CCN` identifiers are cross-references, written with the
`G7-` prefix at the call site so they never collide with another group's `CCN`.

| CC | Test function |
|----|---------------|
| G7-CC1 | `it32_tools_spawns_no_subprocess` |
| G7-CC2 | `it30_tools_core_params_accepted_together` |
| G7-CC3 | `it12_tools_name_and_category_and_logic` |
| G7-CC4 | `it31_tools_exclusive_flags_absent_from_run_help` |
| G7-CC5 | `it19_tools_value_inspect_mutually_exclusive` |
| G7-CC6 | `it27_tools_columns_ignored_when_value_active` |

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
