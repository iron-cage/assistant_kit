# Parameter Group :: Display

Interaction tests for the Display group: `limit`, `format`, `sort`, `reverse`,
`verbosity`, `output`, `wide`, `columns`. Tests validate co-dependency,
mutual exclusivity, and precedence rules between display parameters.

**Source:** [param_group/02_display.md](../../../../docs/cli/param_group/02_display.md)

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| CC-1 | `sort::cost reverse::1` -> reverse affects only the sort field | Co-Dependency |
| CC-2 | `wide::1 columns::"time,cost"` -> `columns` takes precedence over `wide` | Mutual Exclusivity |
| CC-3 | `format::json wide::1` -> `wide` has no effect on non-table format | Format Scoping |
| CC-4 | `sort::cost limit::5` -> limit applied after sort | Ordering |
| CC-5 | `output` only takes effect on `.export`, ignored elsewhere | Command Scoping |
| CC-6 | `verbosity::5` -> clamped to 2 | Boundary |

## Test Coverage Summary

- Co-Dependency: 1 test (CC-1)
- Mutual Exclusivity: 1 test (CC-2)
- Format Scoping: 1 test (CC-3)
- Ordering: 1 test (CC-4)
- Command Scoping: 1 test (CC-5)
- Boundary: 1 test (CC-6)

**Total:** 6 corner cases

## Test Cases
---

### CC-1: `sort::cost reverse::1` -> reverse affects only the sort field

- **Given:** journal with events of varying cost and varying timestamps
- **When:** `clj .list sort::cost reverse::1`
- **Then:** events are ordered by cost descending; time ordering is not separately reversed
- **Exit:** 0
- **Source:** [param_group/02_display.md](../../../../docs/cli/param_group/02_display.md)
---

### CC-2: `wide::1 columns::"time,cost"` -> `columns` takes precedence over `wide`

- **Given:** journal with events containing many fields
- **When:** `clj .list wide::1 columns::"time,cost"`
- **Then:** only the `time` and `cost` columns are shown; `wide` is overridden
- **Exit:** 0
- **Source:** [param_group/02_display.md](../../../../docs/cli/param_group/02_display.md)
---

### CC-3: `format::json wide::1` -> `wide` has no effect on non-table format

- **Given:** journal with events containing many fields
- **When:** `clj .list format::json wide::1`
- **Then:** output is standard JSON, unaffected by `wide` (which only applies to `format::table`)
- **Exit:** 0
- **Source:** [param_group/02_display.md](../../../../docs/cli/param_group/02_display.md)
---

### CC-4: `sort::cost limit::5` -> limit applied after sort

- **Given:** journal with more than 5 events of varying cost
- **When:** `clj .list sort::cost reverse::1 limit::5`
- **Then:** exactly the 5 highest-cost events are shown, not an arbitrary 5 events subsequently sorted
- **Exit:** 0
- **Source:** [param_group/02_display.md](../../../../docs/cli/param_group/02_display.md)
---

### CC-5: `output` only takes effect on `.export`, ignored elsewhere

- **Given:** clean environment
- **When:** `clj .list output::/tmp/should_not_be_used.txt`
- **Then:** `.list` writes to stdout as normal; `/tmp/should_not_be_used.txt` is not created (`output` is not a recognized `.list` member)
- **Exit:** 0
- **Source:** [param_group/02_display.md](../../../../docs/cli/param_group/02_display.md)
---

### CC-6: `verbosity::5` -> clamped to 2

- **Given:** journal directory with multiple files
- **When:** `clj .status verbosity::5`
- **Then:** output matches the per-file breakdown produced at `verbosity::2`, not an error
- **Exit:** 0
- **Source:** [param_group/02_display.md](../../../../docs/cli/param_group/02_display.md)
