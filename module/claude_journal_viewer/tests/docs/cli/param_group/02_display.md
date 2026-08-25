# Parameter Group :: Display

Interaction tests for the Display group: `limit`, `format`, `sort`, `reverse`,
`verbosity`, `output`. Tests validate co-dependency, command scoping, ordering,
and boundary handling between display parameters.

**Source:** [param_group/02_display.md](../../../../docs/cli/param_group/02_display.md)

## Test Case Index

| ID | Test Name | Category | Status | Implemented as |
|----|-----------|----------|--------|----------------|
| CC-1 | `sort::cost reverse::1` -> reverse affects only the sort field | Co-Dependency | ✅ | `viewer_integration_test.rs::ec30_sort_orders_by_every_documented_field` |
| CC-4 | `sort::cost limit::5` -> limit applied after sort | Ordering | ✅ | `viewer_integration_test.rs::ec33_limit_applies_after_sort_and_zero_means_unlimited` |
| CC-5 | `output` belongs to `.export`; `.list output::` exits 1 | Command Scoping | ✅ | `viewer_integration_test.rs::ec28_unknown_param_exits_1` |
| CC-6 | `verbosity::9` -> clamped to 2 | Boundary | ✅ | `viewer_integration_test.rs::ec35_status_verbosity_levels_and_clamping` |

## Test Coverage Summary

- Co-Dependency: 1 test (CC-1)
- Ordering: 1 test (CC-4)
- Command Scoping: 1 test (CC-5)
- Boundary: 1 test (CC-6)

**Total:** 4 corner cases (4 executable)

IDs keep their historical numbers rather than being renumbered. CC-2 and CC-3
covered `wide`/`columns` precedence and format scoping; both parameters were
retracted rather than built, so the interactions they tested no longer exist —
see [param_group/02_display.md](../../../../docs/cli/param_group/02_display.md).

## Test Cases
---

### CC-1: `sort::cost reverse::1` -> reverse affects only the sort field

- **Given:** journal with events of varying cost and varying timestamps
- **When:** `clj .list sort::cost reverse::1`
- **Then:** events are ordered by cost descending; time ordering is not separately reversed
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

### CC-5: `output` belongs to `.export`; `.list output::` exits 1

- **Given:** clean environment
- **When:** `clj .list output::/tmp/should_not_be_used.txt`
- **Then:** exit 1 naming `output` as unrecognized for `.list`; the file is not created
- **Exit:** 1
- **Source:** [param_group/02_display.md](../../../../docs/cli/param_group/02_display.md), [param_group/05_global.md](../../../../docs/cli/param_group/05_global.md)

Silent acceptance would be the worse failure here: a caller who passed `output::`
would see exit 0 and reasonably conclude the file was written, while `.list`
printed to stdout and created nothing.
---

### CC-6: `verbosity::9` -> clamped to 2

- **Given:** journal directory with multiple files
- **When:** `clj .status verbosity::9`
- **Then:** output matches the per-file breakdown produced at `verbosity::2`, not an error
- **Exit:** 0
- **Source:** [param_group/02_display.md](../../../../docs/cli/param_group/02_display.md), [param/22_verbosity.md](../../../../docs/cli/param/22_verbosity.md)
