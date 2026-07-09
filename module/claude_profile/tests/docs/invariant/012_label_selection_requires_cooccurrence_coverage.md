# Test: Invariant 012 — Label-Selection Requires Co-Occurrence Coverage

Property assertion cases for `docs/invariant/012_label_selection_requires_cooccurrence_coverage.md`.
Verifies that `reason_label()`'s branch-priority selection is tested against co-occurring flag
combinations, not just each flag individually against type-default values.

### Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| IN-1 | `reason_label()` reports `"occupied elsewhere"` when `cached` and `is_occupied_elsewhere` co-occur | Invariant holds (normal) |
| IN-2 | Single-flag-isolation tests alone do not satisfy co-occurrence coverage for the four-branch flag set | Invariant holds (boundary) |

**Total:** 2 IN cases

---

### IN-1: `reason_label()` reports `"occupied elsewhere"` when `cached` and `is_occupied_elsewhere` co-occur

- **Given:** An `AccountQuota` value with `is_owned == true`, `cached == true` (a non-expired cached
  quota), and `is_occupied_elsewhere == true` simultaneously — the producible co-occurrence
  identified by BUG-333 as "the DEFAULT/near-universal outcome for any occupied-elsewhere account
  after its first fetch" (per G1b's routing through `approximate_quota()`)
- **When:** `reason_label(aq, now_secs)` (`src/usage/refresh.rs:32-51`) is called on this value,
  using the BUG-333-fixed branch order (`!is_owned` → `is_occupied_elsewhere` → `cached` → `else`)
- **Then:** The function returns the `"occupied elsewhere"` label — the higher-priority
  `is_occupied_elsewhere` condition is checked and reported before the `cached` condition, so the
  more informative label is not masked by the earlier-checked (pre-fix) branch; this matches
  `mre_bug333_occupied_elsewhere_not_masked_by_cached` (`tests/usage/refresh_tests_b.rs`)
- **Source:** [docs/invariant/012_label_selection_requires_cooccurrence_coverage.md](../../../docs/invariant/012_label_selection_requires_cooccurrence_coverage.md)

---

### IN-2: Single-flag-isolation tests alone do not satisfy co-occurrence coverage for the four-branch flag set

- **Given:** A test matrix for `reason_label()` containing only single-flag-isolation cases —
  `reason_label_cached_valid`/`reason_label_cached_expired` (both hold `is_occupied_elsewhere:
  false`) and a test exercising `is_occupied_elsewhere: true` alone (holding `cached: false`) — the
  exact pre-BUG-333 state of `tests/usage/refresh_tests_b.rs`
- **When:** This matrix is projected onto the two independent, producible-together flags `cached`
  and `is_occupied_elsewhere` and checked against the invariant's measurable threshold (zero rows
  where two co-occurring flags are both left at their mutually-exclusive default)
- **Then:** The matrix fails the threshold — every row leaves one of the two flags at its type
  default `false`, so the co-occurring case `cached: true ∧ is_occupied_elsewhere: true` is never
  constructed, even though naive branch-coverage tooling reports both branches as 100% "hit"; this
  is the formal violation the invariant targets, distinct from and undetected by ordinary branch
  coverage
- **Source:** [docs/invariant/012_label_selection_requires_cooccurrence_coverage.md](../../../docs/invariant/012_label_selection_requires_cooccurrence_coverage.md)
