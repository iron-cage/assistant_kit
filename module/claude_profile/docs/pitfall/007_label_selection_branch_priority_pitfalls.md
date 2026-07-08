# Pitfall: Label Selection Branch-Priority Pitfalls

### Scope

- **Purpose**: Document the recurring failure mode where a branch-priority label-selection function silently masks one true condition behind another that is checked earlier, when both are true simultaneously.
- **Responsibility**: Covers the full recurrence history of this pattern against `reason_label()`'s trace-reason derivation in `apply_refresh()` — four defects across the same function, escalating from an inline three-branch expression to an extracted, directly-testable function that still carries the same class of gap.
- **In Scope**: `reason_label()` (`src/usage/refresh.rs:32-51`) and its inline predecessor; BUG-295, BUG-298, BUG-306, BUG-333.
- **Out of Scope**: Missing `is_occupied_elsewhere` *guards* in control-flow gates (G1b/G2/G4 — a different defect shape: an absent check, not a masked one; → pitfall/005, which covers BUG-302/303/305/306/320 from the gate-completeness angle); the general co-occurrence-testing requirement (→ invariant/012); the function's full branch table (→ algorithm/012).

### Pattern

Branch-priority label functions built as a chain of `if`/`else if` conditions silently drop whichever condition is checked *later* whenever it co-occurs with a condition checked *earlier* — the first match wins and every subsequent condition is never evaluated, regardless of its truth value. This has recurred four times against the exact same trace-reason derivation site in `apply_refresh()`: each fix added one new branch to resolve one specific masking case, raised single-flag branch coverage back to 100%, and left the *next* possible co-occurrence (between the newly-added branch and an existing one) completely untested — because no fix in the chain ever added a test constructing two non-default flags at once. `pitfall/005_ownership_gate_pitfalls.md` documents a structurally related but distinct family (BUG-302/303/305/306/320: control-flow gates that were missing an occupancy *check entirely*); this pitfall documents defects where the check exists and fires correctly, but an earlier-priority branch masks its result in the trace string.

### Pitfall 1 — Ownership masks result-derived reason (BUG-295)

The original inline trace-reason expression at `refresh.rs:55` (`aq.result.as_ref().err().map_or("ok", String::as_str)`) never checked `!aq.is_owned` at all. For a non-owned account whose `aq.result` was `Ok(cached_data)` (the G1 cache-read path), `.err()` returned `None` and the expression produced the literal string `"ok"` — even though the actual cause of `should_retry=false` was the ownership gate (`refresh_predicate.rs:60`), not a healthy result. `fetch`/`touch` trace lines for the same account correctly showed `"not owned"` in the same invocation, exposing the contradiction.

**Fix:** Added a `!aq.is_owned` branch, checked before the result-derived expression: `if !aq.is_owned { "not owned" } else { aq.result.as_ref().err().map_or("ok", String::as_str) }`.

**Rule:** When `should_retry`/`should_refresh` can return `false` via a gate unrelated to `aq.result` state, the trace reason must be derived from the gate that fired, not from `aq.result.err()` — `aq.result` and the skip reason are independent variables that can diverge.

### Pitfall 2 — Cache fallback masks the true trigger, "ok" for two different root causes (BUG-298)

BUG-295's fix added the `!aq.is_owned` branch but left the `else` arm as `aq.result.as_ref().err().map_or("ok", String::as_str)`. A separate feature (BUG-255's `cached && expired` guard) had already introduced a new `should_retry=true` trigger path that converts `Err → Ok` at the fetch layer (`fetch.rs:306-313`) and sets `aq.cached = true`. For an owned, cached, expired account, `aq.result` was `Ok(_)` post-fallback, so `.err()` returned `None` and the expression again produced the constant `"ok"` — this time for two accounts (`i5@wbox.pro`: underlying HTTP 429; `i13@wbox.pro`: underlying token-expired) whose actual triggers were completely different but both displayed identically as `"ok"`.

**Fix:** Added an `aq.cached` branch between `!aq.is_owned` and the result-derived `else`: `else if aq.cached { "cached-expired" }`.

**Rule:** Any trigger path that converts `Err → Ok` before the reason expression runs destroys the reason expression's only signal (`aq.result.err()`). Every such conversion site must have its own dedicated branch in the reason function, added at the time the conversion is introduced — not retrofitted after a bug report.

### Pitfall 3 — `is_occupied_elsewhere` masked behind `cached`, non-co-occurring case only (BUG-306)

By this point the expression had three branches: `!is_owned` → `"not owned"`, `cached` → `"cached-expired"`, `else` → result-derived. None checked `is_occupied_elsewhere`, even though the G2 predicate gate (`refresh_predicate.rs:60`, BUG-303 fix) already treated `is_occupied_elsewhere` as an independent skip condition. For an owned, **non-cached**, occupied-elsewhere account with an `Ok` result, execution fell through to the `else` arm and produced `"ok"` — while `touch`'s trace line for the same account correctly showed `"occupied elsewhere"`.

**Fix:** Extracted the inline expression into a standalone `reason_label(aq: &AccountQuota) -> &'static str` function (making it directly unit-testable) and added a fourth branch, `else if aq.is_occupied_elsewhere { "occupied elsewhere" }`, positioned *after* `cached` and *before* the final `else`. The fix's own MRE test (`mre_bug306_refresh_trace_reason_occupied_elsewhere`, `tests/usage/refresh_tests_b.rs:701-721`) hardcodes `cached: false` and asserts the label is `"occupied elsewhere"` — it does not exercise `cached: true` simultaneously. The test's own doc comment (lines 697-698) states: *"Branch order matters: `is_occupied_elsewhere` must come after `cached` because cached accounts have their own trace reason regardless of occupancy status"* — an assertion of intentional design that was never validated against the co-occurring case.

**Rule:** Extracting an inline expression into a named, directly-testable function does not by itself close a branch-priority masking gap — it only makes the *existing* branches individually testable. The extraction must be paired with an audit of every *pairwise* combination among the branches, not just each branch in isolation against its type-default siblings.

### Pitfall 4 — `cached` masks `is_occupied_elsewhere` when both are true (BUG-333, fourth recurrence)

Prior to this fix, `reason_label()` (post-BUG-306, `refresh.rs:32-51`) checked `!is_owned` → `cached` (Priority 2) → `is_occupied_elsewhere` (Priority 3) → `else`. Because `cached` was checked *before* `is_occupied_elsewhere`, any account satisfying both conditions returned `"cached"`/`"cached-expired"` and never reached the occupancy branch. Per feature/036 G1b, this was not a rare edge case: `fetch_quota_for_list()` unconditionally routes any owned, non-current, occupied-elsewhere account through `approximate_quota()`, which independently sets `cached` (from cache-history presence) and `is_occupied_elsewhere` (hardcoded `true` by the caller) — making the co-occurrence "the DEFAULT/near-universal outcome for any occupied-elsewhere account after its first fetch" (BUG-333 Impact). Confirmed reproducible for two independent accounts (`i13@wbox.pro`, `illia.tt@wbox.pro`) in the same captured transcript, where `fetch`/`touch` correctly showed `"occupied elsewhere"` while `refresh` showed `"cached"` for the identical accounts in the identical invocation.

BUG-333's own investigation explicitly tested and disproved the hypothesis that this was intentional design (H2): BUG-306's rationalizing comment ("branch order matters... cached accounts have their own trace reason regardless of occupancy status") was never backed by a test of the co-occurring case — it was an unvalidated assumption, not a validated design decision.

**Fix (BUG-333, fix applied 2026-07-08; state 🎯 Verified pending independent validation):** Of the bug's two candidate fixes — (a) reorder branches to check `is_occupied_elsewhere` before `cached`, or (b) compound the label (e.g., `"occupied elsewhere (cached)"`) — option (a) was applied: the two branches in `reason_label()` (`refresh.rs:32-51`) were reordered so `is_occupied_elsewhere` is now checked before `cached`, closing the masking gap with a same-function, zero-side-effect change. `mre_bug333_occupied_elsewhere_not_masked_by_cached` (`tests/usage/refresh_tests_b.rs`) constructs the co-occurrence directly and confirms `"occupied elsewhere"` is now returned.

**Rule:** A branch-priority label function's coverage must be measured in co-occurrence terms, not per-branch terms. Every prior fix in this chain (BUG-295, BUG-298, BUG-306) added exactly one new isolated test for its new branch, achieving 100% single-flag branch coverage each time, while the *pairwise* combination between the newest branch and every previously-existing branch was never constructed by any test. See `invariant/012_label_selection_requires_cooccurrence_coverage.md` for the formal invariant and enforcement procedure this recurrence motivated.

### Recurrence Summary

| Bug | Masking pair (earlier-checked → later-checked) | Branch added | Co-occurrence tested by the fix? |
|-----|--------------------------------------------------|---------------|-----------------------------------|
| BUG-295 | (no gate check at all) → `aq.result.err()` | `!is_owned` | N/A — first branch added |
| BUG-298 | `!is_owned` → `aq.result.err()` | `cached` (between them) | No — new branch tested alone (`cached: true`, `is_occupied_elsewhere` not yet a concept in this expression) |
| BUG-306 | `cached` → `aq.result.err()` | `is_occupied_elsewhere` (between `cached` and `else`) | No — new branch tested alone (`cached: false` hardcoded in the fix's own MRE test) |
| BUG-333 | `cached` → `is_occupied_elsewhere` | _(fix: reordered `is_occupied_elsewhere` before `cached` — no new branch)_ | Yes — `mre_bug333_occupied_elsewhere_not_masked_by_cached` constructs `cached: true` ∧ `is_occupied_elsewhere: true` together |

Each fix closed exactly the gap the filing bug reported and no more — the pattern of "add one branch, add one isolated test for that branch" repeated four times without ever adding a co-occurrence test between the new branch and prior branches, which is why the fourth branch (`is_occupied_elsewhere`, added by BUG-306) went on to have its own masking defect (BUG-333) discovered against the third branch (`cached`) added two bugs earlier.

### Algorithms

| File | Relationship |
|------|-------------|
| [algorithm/012_refresh_trace_reason_classification.md](../algorithm/012_refresh_trace_reason_classification.md) | `reason_label()` full branch order and entry point this pitfall's recurrence history applies to |

### Invariants

| File | Relationship |
|------|-------------|
| [invariant/012_label_selection_requires_cooccurrence_coverage.md](../invariant/012_label_selection_requires_cooccurrence_coverage.md) | Formal co-occurrence coverage requirement violated by all four bugs in this recurrence chain |

### Features

| File | Relationship |
|------|-------------|
| [feature/036_account_ownership.md](../feature/036_account_ownership.md) | AC-22 (BUG-295), AC-24 (BUG-306), AC-25 (BUG-333) define `reason_label()`'s branches and the co-occurrence coverage requirement; G1b defines the `approximate_quota()` call site producing the `cached ∧ is_occupied_elsewhere` co-occurrence |

### Related Pitfalls

| File | Relationship |
|------|-------------|
| [pitfall/005_ownership_gate_pitfalls.md](../pitfall/005_ownership_gate_pitfalls.md) | Sibling recurrence family on the same `is_owned`/`is_occupied_elsewhere` state — covers BUG-302/303/305/306/320, where the defect shape is a *missing* control-flow guard rather than a masked trace label; BUG-306 appears in both pitfall docs because its fix simultaneously closed a gate-completeness gap (this doc's Pitfall 3 predecessor context) and introduced the branch that BUG-333 later found maskable |
