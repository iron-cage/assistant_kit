# Test: Invariant 011 — Shared Predicate Consistency

Property assertion cases for `docs/invariant/011_shared_predicate_consistency.md`. Verifies that
the `billing_type=="none"` + `result` conjunction (`AccountQuota::is_no_subscription()`) is
evaluated identically at every render-layer call site, distinguishing genuinely cancelled accounts
(population 1) from live non-stripe accounts (population 2).

### Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| IN-1 | `is_no_subscription()` returns `false` for a `billing_type=="none"` account with `result=Ok` (population 2) | Invariant holds (normal) |
| IN-2 | `is_no_subscription()` returns `true` only when both conjuncts hold — `billing_type=="none"` alone is insufficient | Invariant holds (boundary) |

**Total:** 2 IN cases

---

### IN-1: `is_no_subscription()` returns `false` for a `billing_type=="none"` account with `result=Ok` (population 2)

- **Given:** An `AccountQuota` value where `account.billing_type == "none"` (a non-stripe
  team/enterprise account) and `result` is `Ok(...)` with valid quota data — i.e., population 2 as
  defined in the invariant's concrete grounding, distinct from a genuinely cancelled subscription
- **When:** `AccountQuota::is_no_subscription()` (`src/usage/types.rs:226-229`) is called on this
  value, and the same value is independently rendered through `render.rs:108`, `render.rs:374`
  (`GetField::Renews` arm), and `render_tsv.rs:72`
- **Then:** `is_no_subscription()` returns `false` at all three call sites — the conjunction
  `billing_type=="none" AND result.is_err()` is not satisfied because `result.is_err()` is false;
  the renderer shows the real renewal date instead of `"—"`, confirming population 2 is not
  misclassified as having no active subscription
- **Source:** [docs/invariant/011_shared_predicate_consistency.md](../../../docs/invariant/011_shared_predicate_consistency.md)

---

### IN-2: `is_no_subscription()` returns `true` only when both conjuncts hold — `billing_type=="none"` alone is insufficient

- **Given:** Two `AccountQuota` values at the boundary of the predicate's two conjuncts:
  (a) `billing_type == "none"` AND `result.is_err()` (population 1, genuinely cancelled), and
  (b) `billing_type == "none"` AND `result` is `Ok(...)` (population 2, the degenerate case where
  only the first conjunct holds)
- **When:** `AccountQuota::is_no_subscription()` is called on both values
- **Then:** Case (a) returns `true` (both conjuncts satisfied — no active subscription) while case
  (b) returns `false` (only the `billing_type` conjunct holds, `result.is_err()` does not) — the
  predicate never degenerates to evaluating `billing_type == "none"` in isolation, which is exactly
  the drift BUG-332 documents at the three fixed call sites
- **Source:** [docs/invariant/011_shared_predicate_consistency.md](../../../docs/invariant/011_shared_predicate_consistency.md)
