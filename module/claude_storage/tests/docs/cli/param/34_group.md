# Parameter :: `group::`

Edge case tests for the `group::` parameter. Tests validate each of the four aggregation dimensions on `.rollup` — which sessions collapse into the same row — plus enum validation and composition with the other `.rollup` parameters.

**Source:** [param/34_group.md](../../../../docs/cli/param/34_group.md)

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| EC-1 | Default `group::session` → one row per session | Default |
| EC-2 | `group::project` sums sessions sharing a `cwd` | Happy Path |
| EC-3 | `group::model` separates rows by model name | Happy Path |
| EC-4 | `group::day` separates rows by calendar day | Happy Path |
| EC-5 | Invalid `group::` value rejected | Input Validation |
| EC-6 | `group::` composes with `sort::`/`order::`/`columns::`/`limit::` | Composition |

## Test Coverage Summary

- Default: 1 test (EC-1)
- Happy Path: 3 tests (EC-2, EC-3, EC-4)
- Input Validation: 1 test (EC-5)
- Composition: 1 test (EC-6)

**Total:** 6 edge cases

**Behavioral Divergence Pair:** EC-1 (`session`, no summing — one row per session) ↔ EC-2 (`project`, sessions sharing a `cwd` summed into one row)

## Test Cases

---

### EC-1: Default `group::session` → one row per session

- **Commands:** `.rollup`
- **Given:** several sessions in one project
- **When:** `clg .rollup` with no `group::`
- **Then:** the finest granularity applies — one row per session, no summing
- **Exit:** 0
- **Covered by:** `cli_cmd_rollup_test.rs` — `int_1_default_group_session_one_row_per_session`
- **Source:** [param/34_group.md](../../../../docs/cli/param/34_group.md)

---

### EC-2: `group::project` sums sessions sharing a `cwd`

- **Commands:** `.rollup`
- **Given:** multiple sessions recorded under the same project `cwd`
- **When:** `clg .rollup group::project`
- **Then:** they collapse into a single row whose token columns are the sum of the contributing sessions; `Sessions` reports how many contributed
- **Exit:** 0
- **Covered by:** `cli_cmd_rollup_test.rs` — `int_2_group_project_sums_sessions_into_one_row`
- **Source:** [param/34_group.md](../../../../docs/cli/param/34_group.md)

---

### EC-3: `group::model` separates rows by model name

- **Commands:** `.rollup`
- **Given:** sessions recorded against more than one model
- **When:** `clg .rollup group::model`
- **Then:** one row per distinct recorded model name
- **Exit:** 0
- **Covered by:** `cli_cmd_rollup_test.rs` — `int_3_group_model_separates_rows_by_model`
- **Source:** [param/34_group.md](../../../../docs/cli/param/34_group.md)

---

### EC-4: `group::day` separates rows by calendar day

- **Commands:** `.rollup`
- **Given:** sessions whose `first_timestamp` falls on different UTC calendar dates
- **When:** `clg .rollup group::day`
- **Then:** one row per distinct date, as recorded
- **Exit:** 0
- **Covered by:** `cli_cmd_rollup_test.rs` — `int_4_group_day_separates_rows_by_calendar_day`
- **Source:** [param/34_group.md](../../../../docs/cli/param/34_group.md)

---

### EC-5: Invalid `group::` value rejected

- **Commands:** `.rollup`
- **Given:** clean environment
- **When:** `clg .rollup group::user`
- **Then:** Exit 1; error names the four valid values and echoes the rejected one
- **Exit:** 1
- **Covered by:** `cli_cmd_rollup_test.rs` — `int_16_invalid_group_rejected`
- **Source:** [param/34_group.md](../../../../docs/cli/param/34_group.md)

---

### EC-6: `group::` composes with `sort::`/`order::`/`columns::`/`limit::`

- **Commands:** `.rollup`
- **Given:** three models with distinct session counts
- **When:** `clg .rollup group::model sort::sessions order::asc columns::group,sessions limit::1`
- **Then:** all five parameters apply together — grouping picks the model dimension, ascending sort by session count picks the smallest, `limit::1` keeps only it, and no unrequested column label appears
- **Exit:** 0
- **Covered by:** `cli_cmd_rollup_test.rs` — `int_22_multiple_parameters_compose_correctly_together`
- **Source:** [param/34_group.md](../../../../docs/cli/param/34_group.md)
