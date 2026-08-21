# Parameter :: `sort::`

Edge case tests for the `sort::` parameter. Tests validate that `.rollup`'s rows are ranked by the named computed column — always after [`group::`](34_group.md) aggregation, never before — plus enum validation and composition.

**Source:** [param/35_sort.md](../../../../docs/cli/param/35_sort.md)

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| EC-1 | Default `sort::total` ranks by combined tokens | Default |
| EC-2 | `sort::calls` ranks by call count, not by total | Happy Path |
| EC-3 | Invalid `sort::` value rejected | Input Validation |
| EC-4 | `sort::sessions` composes with the other `.rollup` parameters | Composition |

## Test Coverage Summary

- Default: 1 test (EC-1)
- Happy Path: 1 test (EC-2)
- Input Validation: 1 test (EC-3)
- Composition: 1 test (EC-4)

**Total:** 4 edge cases

**Behavioral Divergence Pair:** EC-1 (default `total`) ↔ EC-2 (`calls`) — the same rows, ranked into a different order by a different column

## Test Cases

---

### EC-1: Default `sort::total` ranks by combined tokens

- **Commands:** `.rollup`
- **Given:** the worked-example fixture from `docs/cli/command/14_rollup.md`
- **When:** `clg .rollup` with no `sort::`
- **Then:** the rendered table matches the published worked example byte-for-byte — which pins the default `total` ranking along with the rest of the default projection
- **Exit:** 0
- **Covered by:** `cli_cmd_rollup_test.rs` — `int_13_worked_example_byte_exact`
- **Source:** [param/35_sort.md](../../../../docs/cli/param/35_sort.md)

---

### EC-2: `sort::calls` ranks by call count, not by total

- **Commands:** `.rollup`
- **Given:** rows whose call-count ordering differs from their total-token ordering — so the two keys are distinguishable
- **When:** `clg .rollup sort::calls order::desc`
- **Then:** rows are ordered by deduplicated assistant-turn count, not by combined tokens
- **Exit:** 0
- **Covered by:** `cli_cmd_rollup_test.rs` — `int_5_sort_calls_desc_orders_by_call_count`
- **Source:** [param/35_sort.md](../../../../docs/cli/param/35_sort.md)

---

### EC-3: Invalid `sort::` value rejected

- **Commands:** `.rollup`
- **Given:** clean environment
- **When:** `clg .rollup sort::tokens`
- **Then:** Exit 1; error names all eight valid keys and echoes the rejected one
- **Exit:** 1
- **Covered by:** `cli_cmd_rollup_test.rs` — `int_17_invalid_sort_rejected`
- **Source:** [param/35_sort.md](../../../../docs/cli/param/35_sort.md)

---

### EC-4: `sort::sessions` composes with the other `.rollup` parameters

- **Commands:** `.rollup`
- **Given:** three models with distinct session counts
- **When:** `clg .rollup group::model sort::sessions order::asc columns::group,sessions limit::1`
- **Then:** `sessions` — the key that is only meaningful when `group::` is not `session` — ranks the aggregated rows, and the ascending winner survives `limit::1`
- **Exit:** 0
- **Covered by:** `cli_cmd_rollup_test.rs` — `int_22_multiple_parameters_compose_correctly_together`
- **Source:** [param/35_sort.md](../../../../docs/cli/param/35_sort.md)
