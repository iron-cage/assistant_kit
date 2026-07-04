# Test: `.stats`

### Scope

- **Purpose**: Verify `.stats` aggregates journal events correctly across all grouping dimensions.
- **Responsibility**: Test case coverage for all 6 `.stats` parameters and default aggregation behavior.
- **In Scope**: Default time window/type, `by` grouping dimensions (model, day, error, command, hour), verbosity, journal_dir override.
- **Out of Scope**: Raw event listing (-> `01_list.md`), export of aggregates (-> `08_export.md`).

Test case planning for [command/03_stats.md](../../../../docs/cli/command/03_stats.md).

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| IT-1 | No args -> daily stats for last 7 days | Default |
| IT-2 | `by::model since::30d` -> per-model cost/token breakdown | Grouping |
| IT-3 | `by::error since::7d` -> error class distribution | Grouping |
| IT-4 | `by::command` -> run vs ask breakdown | Grouping |
| IT-5 | `by::hour since::1d` -> hourly activity breakdown | Grouping |
| IT-6 | `by::badvalue` -> exit 1, error message | Error Handling |

## Test Coverage Summary

- Default: 1 test (IT-1)
- Grouping: 4 tests (IT-2, IT-3, IT-4, IT-5)
- Error Handling: 1 test (IT-6)

**Total:** 6 tests

---

### IT-1: No args -> daily stats for last 7 days

- **Given:** journal with execution events spanning more than 7 days
- **When:** `clj .stats`
- **Then:** exit 0; output shows one row per day for the last 7 days, execution events only, with a totals row and success rate
- **Exit:** 0
- **Source:** [command/03_stats.md](../../../../docs/cli/command/03_stats.md)

---

### IT-2: `by::model since::30d` -> per-model cost/token breakdown

- **Given:** journal with events across multiple models within the last 30 days
- **When:** `clj .stats by::model since::30d`
- **Then:** exit 0; output contains one row per model with count, cost, and token aggregates
- **Exit:** 0
- **Source:** [command/03_stats.md](../../../../docs/cli/command/03_stats.md), [param/06_model.md](../../../../docs/cli/param/06_model.md)

---

### IT-3: `by::error since::7d` -> error class distribution

- **Given:** journal with failed events of varying error classes in the last 7 days
- **When:** `clj .stats by::error since::7d`
- **Then:** exit 0; output contains one row per error class with a count
- **Exit:** 0
- **Source:** [command/03_stats.md](../../../../docs/cli/command/03_stats.md)

---

### IT-4: `by::command` -> run vs ask breakdown

- **Given:** journal with both `run` and `ask` command events
- **When:** `clj .stats by::command`
- **Then:** exit 0; output contains separate aggregate rows for `run` and `ask`
- **Exit:** 0
- **Source:** [command/03_stats.md](../../../../docs/cli/command/03_stats.md)

---

### IT-5: `by::hour since::1d` -> hourly activity breakdown

- **Given:** journal with events spread across today
- **When:** `clj .stats by::hour since::1d`
- **Then:** exit 0; output contains one row per hour with activity for the current day
- **Exit:** 0
- **Source:** [command/03_stats.md](../../../../docs/cli/command/03_stats.md)

---

### IT-6: `by::badvalue` -> exit 1, error message

- **Given:** clean environment
- **When:** `clj .stats by::badvalue`
- **Then:** exit 1; stderr contains an error naming the invalid grouping value
- **Exit:** 1
- **Source:** [command/03_stats.md](../../../../docs/cli/command/03_stats.md), [type/09_group_by.md](../../../../docs/cli/type/09_group_by.md)
