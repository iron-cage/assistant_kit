# Parameter :: `order::`

Edge case tests for the `order::` parameter. Tests validate that the sort direction flips whichever column [`sort::`](35_sort.md) named, independently of which key that is, plus enum validation.

**Source:** [param/36_order.md](../../../../docs/cli/param/36_order.md)

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| EC-1 | Default `order::desc` puts the largest value first | Default |
| EC-2 | `order::asc` reverses the same `sort::` result | Happy Path |
| EC-3 | Invalid `order::` value rejected | Input Validation |
| EC-4 | `order::asc` composes with the other `.rollup` parameters | Composition |

## Test Coverage Summary

- Default: 1 test (EC-1)
- Happy Path: 1 test (EC-2)
- Input Validation: 1 test (EC-3)
- Composition: 1 test (EC-4)

**Total:** 4 edge cases

**Behavioral Divergence Pair:** EC-1 (`desc`, largest first) ↔ EC-2 (`asc`, the identical row set in reverse) — deliberately run over the same `sort::calls` fixture so direction is the only variable

## Test Cases

---

### EC-1: Default `order::desc` puts the largest value first

- **Commands:** `.rollup`
- **Given:** rows with distinct call counts
- **When:** `clg .rollup sort::calls order::desc`
- **Then:** the highest call count leads — the natural "what cost the most" reading, and the same order a bare `.rollup` produces
- **Exit:** 0
- **Covered by:** `cli_cmd_rollup_test.rs` — `int_5_sort_calls_desc_orders_by_call_count`
- **Source:** [param/36_order.md](../../../../docs/cli/param/36_order.md)

---

### EC-2: `order::asc` reverses the same `sort::` result

- **Commands:** `.rollup`
- **Given:** the identical fixture EC-1 uses
- **When:** `clg .rollup sort::calls order::asc`
- **Then:** the row sequence is EC-1's reversed — direction is orthogonal to the choice of sort key
- **Exit:** 0
- **Covered by:** `cli_cmd_rollup_test.rs` — `int_6_order_asc_reverses_sort_calls_result`
- **Source:** [param/36_order.md](../../../../docs/cli/param/36_order.md)

---

### EC-3: Invalid `order::` value rejected

- **Commands:** `.rollup`
- **Given:** clean environment
- **When:** `clg .rollup order::descending`
- **Then:** Exit 1; error names `asc|desc` and echoes the rejected value — a prefix of a valid value is not accepted
- **Exit:** 1
- **Covered by:** `cli_cmd_rollup_test.rs` — `int_18_invalid_order_rejected`
- **Source:** [param/36_order.md](../../../../docs/cli/param/36_order.md)

---

### EC-4: `order::asc` composes with the other `.rollup` parameters

- **Commands:** `.rollup`
- **Given:** three models with distinct session counts
- **When:** `clg .rollup group::model sort::sessions order::asc columns::group,sessions limit::1`
- **Then:** the ascending direction — opposite the default every other case uses — selects the smallest row, which is then what `limit::1` keeps
- **Exit:** 0
- **Covered by:** `cli_cmd_rollup_test.rs` — `int_22_multiple_parameters_compose_correctly_together`
- **Source:** [param/36_order.md](../../../../docs/cli/param/36_order.md)
