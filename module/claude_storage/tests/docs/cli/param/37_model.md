# Parameter :: `model::`

Edge case tests for the `model::` parameter. Tests validate that the substring filter drops non-matching sessions *before* [`group::`](34_group.md) aggregation — so the `Pct` denominator shrinks with the filtered set rather than the filter merely hiding rows — and that a filter matching nothing is not an error.

**Source:** [param/37_model.md](../../../../docs/cli/param/37_model.md)

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| EC-1 | `model::` filters before grouping; `Pct` recomputes against the filtered total | Happy Path |
| EC-2 | `model::` matching zero sessions → exit 0, header only | Boundary Values |

## Test Coverage Summary

- Happy Path: 1 test (EC-1)
- Boundary Values: 1 test (EC-2)

**Total:** 2 edge cases

**Behavioral Divergence Pair:** EC-1 (filter matches — surviving rows' percentages rise) ↔ EC-2 (filter matches nothing — header-only table, still exit 0, not an error)

## Test Cases

---

### EC-1: `model::` filters before grouping; `Pct` recomputes against the filtered total

- **Commands:** `.rollup`
- **Given:** sessions across several models, including a heavy one the filter will exclude
- **When:** `clg .rollup model::<substring>`
- **Then:** non-matching sessions are dropped entirely, including from the `percent` denominator — the surviving rows' percentages are computed against the filtered total, not the unfiltered one
- **Exit:** 0
- **Covered by:** `cli_cmd_rollup_test.rs` — `int_9_model_filter_recomputes_percent_against_filtered_total`
- **Source:** [param/37_model.md](../../../../docs/cli/param/37_model.md)

---

### EC-2: `model::` matching zero sessions → exit 0, header only

- **Commands:** `.rollup`
- **Given:** a populated storage and a model substring matching none of its sessions
- **When:** `clg .rollup model::nonexistent-model-xyz`
- **Then:** the table prints its header with zero data rows; an empty result is a valid answer, distinct from the argument errors that `group::`/`sort::`/`order::`/`columns::` raise for unknown values
- **Exit:** 0
- **Covered by:** `cli_cmd_rollup_test.rs` — `int_23_model_filter_matching_zero_sessions_exits_0_header_only`
- **Source:** [param/37_model.md](../../../../docs/cli/param/37_model.md)
