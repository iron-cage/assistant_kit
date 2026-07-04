# Test: `.list`

### Scope

- **Purpose**: Verify `.list` filters, sorts, limits, and formats journal events correctly.
- **Responsibility**: Test case coverage for all 12 `.list` parameters and their combinations.
- **In Scope**: Filter params (since, until, type, command, exit, model, dir, creds), limit cap, sort/reverse, format selection.
- **Out of Scope**: Individual parameter edge cases (-> `../param/`), real-time following (-> `02_tail.md`).

Test case planning for [command/01_list.md](../../../../docs/cli/command/01_list.md).

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| IT-1 | No args -> last 50 events, table format | Default |
| IT-2 | `since::1h` -> only events from last hour | Time Filter |
| IT-3 | `type::execution command::ask` -> combined filter | Combined Filter |
| IT-4 | `sort::cost reverse::1 since::7d` -> most expensive first | Sort |
| IT-5 | `format::json limit::100` -> JSON output, capped at 100 | Format + Limit |
| IT-6 | `exit::2 model::opus` -> filter by exit code and model substring | Combined Filter |
| IT-7 | `format::badvalue` -> exit 1, error message | Error Handling |

## Test Coverage Summary

- Default: 1 test (IT-1)
- Time Filter: 1 test (IT-2)
- Combined Filter: 2 tests (IT-3, IT-6)
- Sort: 1 test (IT-4)
- Format + Limit: 1 test (IT-5)
- Error Handling: 1 test (IT-7)

**Total:** 7 tests

---

### IT-1: No args -> last 50 events, table format

- **Given:** journal with more than 50 events
- **When:** `clj .list`
- **Then:** exit 0; exactly 50 most recent events shown in table format
- **Exit:** 0
- **Source:** [command/01_list.md](../../../../docs/cli/command/01_list.md)

---

### IT-2: `since::1h` -> only events from last hour

- **Given:** journal with events spanning more than 1 hour
- **When:** `clj .list since::1h`
- **Then:** exit 0; only events within the last hour are shown
- **Exit:** 0
- **Source:** [command/01_list.md](../../../../docs/cli/command/01_list.md), [param/01_since.md](../../../../docs/cli/param/01_since.md)

---

### IT-3: `type::execution command::ask` -> combined filter

- **Given:** journal with mixed event types and commands
- **When:** `clj .list type::execution command::ask`
- **Then:** exit 0; only execution-type events with command `ask` are shown
- **Exit:** 0
- **Source:** [command/01_list.md](../../../../docs/cli/command/01_list.md), [param/03_type.md](../../../../docs/cli/param/03_type.md), [param/04_command.md](../../../../docs/cli/param/04_command.md)

---

### IT-4: `sort::cost reverse::1 since::7d` -> most expensive first

- **Given:** journal with events of varying cost over the last 7 days
- **When:** `clj .list since::7d sort::cost reverse::1`
- **Then:** exit 0; events ordered highest cost first
- **Exit:** 0
- **Source:** [command/01_list.md](../../../../docs/cli/command/01_list.md), [param/11_sort.md](../../../../docs/cli/param/11_sort.md), [param/12_reverse.md](../../../../docs/cli/param/12_reverse.md)

---

### IT-5: `format::json limit::100` -> JSON output, capped at 100

- **Given:** journal with more than 100 events
- **When:** `clj .list format::json limit::100`
- **Then:** exit 0; output is valid JSON; exactly 100 events returned
- **Exit:** 0
- **Source:** [command/01_list.md](../../../../docs/cli/command/01_list.md), [param/10_format.md](../../../../docs/cli/param/10_format.md), [param/09_limit.md](../../../../docs/cli/param/09_limit.md)

---

### IT-6: `exit::2 model::opus` -> filter by exit code and model substring

- **Given:** journal with events across multiple exit codes and models
- **When:** `clj .list exit::2 model::opus`
- **Then:** exit 0; only events with exit code 2 and model name containing `opus` are shown
- **Exit:** 0
- **Source:** [command/01_list.md](../../../../docs/cli/command/01_list.md)

---

### IT-7: `format::badvalue` -> exit 1, error message

- **Given:** clean environment
- **When:** `clj .list format::badvalue`
- **Then:** exit 1; stderr contains an error naming the invalid format value
- **Exit:** 1
- **Source:** [command/01_list.md](../../../../docs/cli/command/01_list.md), [type/06_output_format.md](../../../../docs/cli/type/06_output_format.md)
