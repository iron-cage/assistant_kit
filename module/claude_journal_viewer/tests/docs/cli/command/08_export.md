# Test: `.export`

### Scope

- **Purpose**: Verify `.export` writes filtered events in the selected format to file or stdout, with no limit cap.
- **Responsibility**: Test case coverage for all 6 `.export` parameters.
- **In Scope**: Format selection, time/type/command filters, file vs stdout output, uncapped result count.
- **Out of Scope**: Interactive listing with limit cap (-> `01_list.md`), aggregate export (-> `03_stats.md`).

Test case planning for [command/08_export.md](../../../../docs/cli/command/08_export.md).

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| IT-1 | `format::csv since::7d output::PATH` -> CSV written to file | File Output |
| IT-2 | `format::json since::30d` -> JSON written to stdout | Stdout Output |
| IT-3 | `format::jsonl type::execution` -> raw JSONL filtered by type | Format Selection |
| IT-4 | `format::table since::1d command::ask` -> table format, combined filter | Combined Filter |
| IT-5 | No `limit` cap applied even with many matching events | Uncapped Export |
| IT-6 | `format::badvalue` -> exit 1, error message | Error Handling |

## Test Coverage Summary

- File Output: 1 test (IT-1)
- Stdout Output: 1 test (IT-2)
- Format Selection: 1 test (IT-3)
- Combined Filter: 1 test (IT-4)
- Uncapped Export: 1 test (IT-5)
- Error Handling: 1 test (IT-6)

**Total:** 6 tests

---

### IT-1: `format::csv since::7d output::PATH` -> CSV written to file

- **Given:** journal with events in the last 7 days; writable output path
- **When:** `clj .export format::csv since::7d output::/tmp/week.csv`
- **Then:** exit 0; `/tmp/week.csv` contains valid CSV rows for the last 7 days' events; nothing printed to stdout
- **Exit:** 0
- **Source:** [command/08_export.md](../../../../docs/cli/command/08_export.md), [param/23_output.md](../../../../docs/cli/param/23_output.md)

---

### IT-2: `format::json since::30d` -> JSON written to stdout

- **Given:** journal with events in the last 30 days; no `output` given
- **When:** `clj .export format::json since::30d`
- **Then:** exit 0; stdout contains valid JSON for the last 30 days' events
- **Exit:** 0
- **Source:** [command/08_export.md](../../../../docs/cli/command/08_export.md)

---

### IT-3: `format::jsonl type::execution` -> raw JSONL filtered by type

- **Given:** journal with mixed event types
- **When:** `clj .export format::jsonl type::execution`
- **Then:** exit 0; stdout contains one JSON object per line, execution-type events only
- **Exit:** 0
- **Source:** [command/08_export.md](../../../../docs/cli/command/08_export.md), [param/03_type.md](../../../../docs/cli/param/03_type.md)

---

### IT-4: `format::table since::1d command::ask` -> table format, combined filter

- **Given:** journal with `ask` and non-`ask` events in the last day
- **When:** `clj .export format::table since::1d command::ask`
- **Then:** exit 0; stdout renders a table containing only `ask` command events from the last day
- **Exit:** 0
- **Source:** [command/08_export.md](../../../../docs/cli/command/08_export.md), [param/04_command.md](../../../../docs/cli/param/04_command.md)

---

### IT-5: No `limit` cap applied even with many matching events

- **Given:** journal with more than 50 matching events (the `.list` default cap)
- **When:** `clj .export format::jsonl`
- **Then:** exit 0; all matching events are exported, not capped at 50
- **Exit:** 0
- **Source:** [command/08_export.md](../../../../docs/cli/command/08_export.md)

---

### IT-6: `format::badvalue` -> exit 1, error message

- **Given:** clean environment
- **When:** `clj .export format::badvalue`
- **Then:** exit 1; stderr contains an error naming the invalid format value
- **Exit:** 1
- **Source:** [command/08_export.md](../../../../docs/cli/command/08_export.md), [type/06_output_format.md](../../../../docs/cli/type/06_output_format.md)
