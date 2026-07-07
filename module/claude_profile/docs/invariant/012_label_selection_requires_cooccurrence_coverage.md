# Invariant: Label-Selection Requires Co-Occurrence Coverage

### Scope

- **Purpose**: Guarantee that any function selecting a single label from multiple independent boolean conditions is tested against every combination of conditions that can co-occur in production, not just each condition in isolation.
- **Responsibility**: Documents the formal co-occurrence coverage invariant for branch-priority label-selection functions, its detection procedure, and the enforcement gap that let the same violation recur four times in `reason_label()`.
- **In Scope**: Any function of the shape "branch over N independent boolean flags, return the first matching label" — specifically `reason_label(aq, now_secs)` in `src/usage/refresh.rs:25-44`; the general test-matrix requirement (`2^n` combinations) for such functions.
- **Out of Scope**: Control-flow predicates that gate behavior (`should_refresh()` in `refresh_predicate.rs` — governed by the G1–G8 gate contract in feature/036, not this invariant); functions with mutually-exclusive-by-construction flags (no co-occurrence risk); trace-string formatting mechanics unrelated to branch selection (→ algorithm/012).

### Invariant Statement

For any label-selection function `L` that branches over `N ≥ 2` independent boolean conditions `{f1, f2, …, fn}` — where "independent" means no two conditions are mutually exclusive by construction — and returns the first label whose condition matches, a test suite MUST contain at least one test case exercising every combination of flags that can actually co-occur during production execution of the calling code path, not merely each flag individually with all others held at a default value.

This is the formal statement BUG-333 codified as its own "Generalized Version" (`task/claude_profile/bug/333_reason_label_cached_masks_occupied_elsewhere.md`):

**Broken assumption:** "the first-matching branch in a priority-ordered label function is always the most informative one to report" — false whenever two conditions can co-occur and the later-checked one carries information the earlier-checked one does not.

**Detection invariant (BUG-333, verbatim anchor):**

```
∀ label function L branching over boolean flags {f1, f2, ..., fn}:
  a test must exist for every combination of flags that can co-occur in production,
  not just each flag individually with all others held at their "default" value
```

**Measurable threshold:** For a label-selection function reachable with `N` independent boolean flags that can co-occur, the test suite's flag-assignment matrix, projected onto those `N` flags, must have zero rows where two co-occurring flags are both left at their mutually-exclusive default (i.e., the matrix must include at least one row per producible flag pair/triple/…, not just per single flag). A test suite satisfying every single-flag case while leaving every multi-flag co-occurrence untested is a **violation**, even though every individual branch shows as "covered" by naive line/branch coverage tooling.

### Why Branch/Line Coverage Tooling Does Not Detect This

Standard coverage tools report a branch as "hit" the first time execution passes through it, regardless of which other flags were true or false at that moment. `reason_label()`'s `cached` branch (line 31 in `src/usage/refresh.rs`) shows 100% branch coverage from `reason_label_cached_valid` and `reason_label_cached_expired` alone (`tests/usage/refresh_tests_b.rs:741,760`) — both tests hold `is_occupied_elsewhere: false`. The `is_occupied_elsewhere` branch (line 36) shows 100% coverage from `mre_bug306_refresh_trace_reason_occupied_elsewhere` (`tests/usage/refresh_tests_b.rs:701`) alone — that test holds `cached: false`. Both branches report "covered." The co-occurring case (`cached: true` AND `is_occupied_elsewhere: true`) — the actual defect BUG-333 found — is never constructed by any test, and no coverage percentage reflects that gap. Co-occurrence coverage is a distinct measurement dimension from branch coverage; a function can be 100% branch-covered and 0% co-occurrence-covered simultaneously.

### Enforcement Procedure

1. **Identify candidate functions**: any function whose body is a chain of `if`/`else if` branches (or equivalent `match`) over `≥ 2` boolean-typed struct fields or parameters, where the fields are set by independent, unrelated code paths (not derived from each other).
2. **Enumerate the flag set**: list every boolean condition the function branches on, in the order checked.
3. **Determine producible combinations**: trace each flag back to its assignment site(s) in the calling code path; a combination is "producible" if there exists at least one reachable call site where both flags are simultaneously true (or false, per the combination under test) — not merely conceivable in the abstract type system.
4. **Cross-reference the test matrix**: for each producible combination, confirm at least one test constructs that exact combination and asserts the returned label. A test suite where every flag has individual-isolation coverage but zero rows have ≥2 co-occurring non-default flags fails this check.
5. **Flag gaps as invariant violations**: any producible combination absent from the test matrix is a violation of this invariant, filed as a bug per the project's Bug-Fixing Workflow, independent of whether the missing combination has actually manifested as an observed defect yet.

### Application to `reason_label()`

`reason_label(aq, now_secs)` (`src/usage/refresh.rs:25-44`) branches over four conditions in priority order: `!aq.is_owned` → `aq.cached` (nested: token-expired sub-branch) → `aq.is_occupied_elsewhere` → `else` (result-derived). Per `feature/036_account_ownership.md` G1b, `aq.cached` and `aq.is_occupied_elsewhere` are set by two unrelated data sources (`read_cached_quota()`/cache-fallback logic, and `other_machines_active()` marker-file detection respectively) and are producible simultaneously — G1b's own design routes owned+occupied-elsewhere accounts through `approximate_quota()`, which independently determines both flags. This is not a hypothetical combination; per BUG-333's Impact section, it is "the DEFAULT/near-universal outcome for any occupied-elsewhere account after its first fetch."

Prior to BUG-333, the test suite (`tests/usage/refresh_tests_b.rs`) held one of these two flags at its type-default `false` in every single test — `reason_label_cached_valid`/`reason_label_cached_expired` fix `is_occupied_elsewhere: false`; `mre_bug306_refresh_trace_reason_occupied_elsewhere` fixes `cached: false`. Zero tests constructed `cached: true` with `is_occupied_elsewhere: true` together. This is the exact violation this invariant targets: full single-flag coverage, zero co-occurrence coverage, on a function where the co-occurrence is the common case in production.

### Violation Consequences

- A branch-priority label function silently returns a less-informative label whenever a higher-priority (earlier-checked) condition co-occurs with a lower-priority (later-checked) one that the reader needed to see — no crash, no test failure, no functional regression; the defect is invisible to every mechanical check except a co-occurrence-aware test.
- The gap compounds across maintenance cycles: each time a new condition is added to the function (a new predicate gate, per feature/036's "Predicate–reason contract"), the natural instinct is to add one new isolated test for the new branch — which raises branch coverage to 100% again without ever testing the new condition against the conditions already covered. This is precisely how the same defect pattern recurred across six bug filings against the same function/seam (BUG-295, BUG-298, BUG-303, BUG-305, BUG-306, BUG-333 — see `pitfall/007_label_selection_branch_priority_pitfalls.md` for the full per-bug history).
- A prior fix's doc comment asserting a branch order as intentional (e.g., `refresh_tests_b.rs:696-698`'s "Branch order matters: `is_occupied_elsewhere` must come after `cached` because cached accounts have their own trace reason regardless of occupancy status") can read as settled precedent to future maintainers, discouraging further co-occurrence testing — this rationalization was BUG-333's own H2 hypothesis, disproved: an assumption asserted in a comment is not evidence the co-occurring case was deliberately evaluated.

### Sources

| File | Relationship |
|------|-------------|
| `src/usage/refresh.rs:25-44` | `reason_label()` — the four-branch label-selection function this invariant currently governs |
| `src/usage/fetch.rs:161-167` (G1b) | Unconditionally routes owned+non-current+occupied-elsewhere accounts through `approximate_quota()`, which independently sets `cached` and `is_occupied_elsewhere` — the producing site that makes the co-occurrence the default case |

### Tests

| File | Relationship |
|------|--------------|
| `tests/usage/refresh_tests_b.rs` | `reason_label_cached_valid`, `reason_label_cached_expired`, `mre_bug306_refresh_trace_reason_occupied_elsewhere` — pre-BUG-333 single-flag-isolation tests that satisfied branch coverage while leaving the `cached ∧ is_occupied_elsewhere` co-occurrence untested; any BUG-333 fix/regression test added to close this gap belongs in this file |

### Algorithms

| File | Relationship |
|------|-------------|
| [algorithm/012_refresh_trace_reason_classification.md](../algorithm/012_refresh_trace_reason_classification.md) | Documents `reason_label()`'s full branch order and the co-occurrence hazard this invariant formalizes |

### Pitfalls

| File | Relationship |
|------|-------------|
| [pitfall/007_label_selection_branch_priority_pitfalls.md](../pitfall/007_label_selection_branch_priority_pitfalls.md) | Full four-bug recurrence history (BUG-295/298/306/333) of this invariant being violated against the same function |
