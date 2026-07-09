# Pitfall Tests: Label Selection Branch-Priority Pitfalls

Test cases verifying that each guard documented in
`docs/pitfall/007_label_selection_branch_priority_pitfalls.md` is in place and prevents the
described branch-priority masking failure mode in `reason_label()`.

**Source:** [docs/pitfall/007_label_selection_branch_priority_pitfalls.md](../../../docs/pitfall/007_label_selection_branch_priority_pitfalls.md)
**Case prefix:** `PP-` (Pitfall Protection)

### Pitfall Guard Index

| ID | Pitfall | Bug | Guard Verified By |
|----|---------|-----|-------------------|
| PP-1 | Ownership masks result-derived reason | BUG-295 | `reason_label()` `!is_owned` branch checked before result-derived `else` |
| PP-2 | Cache fallback masks the true trigger, `"ok"` for two different root causes | BUG-298 | `reason_label()` `cached` branch checked before result-derived `else` |
| PP-3 | `is_occupied_elsewhere` masked behind `cached`, non-co-occurring case only | BUG-306 | `mre_bug306_refresh_trace_reason_occupied_elsewhere` in `tests/usage/refresh_tests_b.rs` |
| PP-4 | `cached` masks `is_occupied_elsewhere` when both are true (fourth recurrence) | BUG-333 | `mre_bug333_occupied_elsewhere_not_masked_by_cached` in `tests/usage/refresh_tests_b.rs` |

---

### PP-1: Ownership gate is checked before the result-derived reason expression

- **Given:** A non-owned account (`aq.is_owned = false`) whose `aq.result` is `Ok(cached_data)`
  (the G1 cache-read path) — the result-derived expression alone would see `.err() = None`.
- **When:** `reason_label(aq)` derives the trace reason for the same account that skipped
  refresh via the ownership gate (`refresh_predicate.rs:60`).
- **Then:** Returns `"not owned"` — not the literal `"ok"` that a result-only expression would
  produce despite the actual skip cause being the ownership gate, not a healthy result. Fix
  BUG-295.
- **Rule:** When `should_retry`/`should_refresh` can return `false` via a gate unrelated to
  `aq.result` state, the trace reason must be derived from the gate that fired, not from
  `aq.result.err()` — `aq.result` and the skip reason are independent variables that can
  diverge.
- **Source fn:** `reason_label()` in `src/usage/refresh.rs:32-51` (`!aq.is_owned` branch,
  checked first)
- **Source:** [pitfall/007_label_selection_branch_priority_pitfalls.md §Pitfall 1](../../../docs/pitfall/007_label_selection_branch_priority_pitfalls.md)

---

### PP-2: Cache-fallback conversions get their own dedicated branch, not the result-derived `else`

- **Given:** Two owned, cached, expired accounts whose `aq.result` is `Ok(_)` post-fallback
  (BUG-255's `cached && expired` guard converts `Err → Ok` at the fetch layer) but whose
  underlying triggers differ — one HTTP 429, one token-expired.
- **When:** `reason_label(aq)` derives the trace reason for both accounts.
- **Then:** Both return `"cached-expired"` (a dedicated `aq.cached` branch, not the
  result-derived `else`) — not the ambiguous constant `"ok"` that would make two accounts with
  completely different root causes display identically. Fix BUG-298.
- **Rule:** Any trigger path that converts `Err → Ok` before the reason expression runs
  destroys the reason expression's only signal (`aq.result.err()`). Every such conversion site
  must have its own dedicated branch in the reason function, added at the time the conversion
  is introduced — not retrofitted after a bug report.
- **Source fn:** `reason_label()` in `src/usage/refresh.rs:32-51` (`aq.cached` branch,
  positioned between `!is_owned` and the result-derived `else`)
- **Source:** [pitfall/007_label_selection_branch_priority_pitfalls.md §Pitfall 2](../../../docs/pitfall/007_label_selection_branch_priority_pitfalls.md)

---

### PP-3: `is_occupied_elsewhere` has its own branch, distinct from the result-derived `else`

- **Given:** An owned, non-cached, occupied-elsewhere account with an `Ok` result — before
  this fix, none of the three existing branches (`!is_owned`, `cached`, result-derived `else`)
  checked `is_occupied_elsewhere`, so execution fell through to `else`.
- **When:** `mre_bug306_refresh_trace_reason_occupied_elsewhere` calls the extracted
  `reason_label(aq: &AccountQuota) -> &'static str` function with `cached: false` and
  `is_occupied_elsewhere: true`.
- **Then:** Returns `"occupied elsewhere"` — not the literal `"ok"` the result-derived `else`
  would otherwise produce, and matching `touch`'s trace line for the same account. Fix
  BUG-306.
- **Rule:** Extracting an inline expression into a named, directly-testable function does not
  by itself close a branch-priority masking gap — it only makes the *existing* branches
  individually testable. The extraction must be paired with an audit of every *pairwise*
  combination among the branches, not just each branch in isolation against its type-default
  siblings.
- **Source fn:** `mre_bug306_refresh_trace_reason_occupied_elsewhere` in
  `tests/usage/refresh_tests_b.rs:701-721`
- **Source:** [pitfall/007_label_selection_branch_priority_pitfalls.md §Pitfall 3](../../../docs/pitfall/007_label_selection_branch_priority_pitfalls.md)

---

### PP-4: `is_occupied_elsewhere` is checked before `cached` — co-occurrence no longer masked

- **Given:** An account satisfying both `aq.cached = true` and `aq.is_occupied_elsewhere =
  true` simultaneously — per feature/036 G1b, `fetch_quota_for_list()` unconditionally routes
  any owned, non-current, occupied-elsewhere account through `approximate_quota()`, which
  independently sets both flags, making this co-occurrence the near-universal outcome for any
  occupied-elsewhere account after its first fetch.
- **When:** `mre_bug333_occupied_elsewhere_not_masked_by_cached` constructs the co-occurrence
  directly and calls `reason_label(aq)`.
- **Then:** Returns `"occupied elsewhere"` — not `"cached"`/`"cached-expired"`, which is what
  the pre-fix branch order (`cached` checked before `is_occupied_elsewhere`) produced for every
  account satisfying both conditions, contradicting `fetch`/`touch`'s correct
  `"occupied elsewhere"` trace for the identical accounts in the identical invocation. Fix
  BUG-333.
- **Rule:** A branch-priority label function's coverage must be measured in co-occurrence
  terms, not per-branch terms. Every prior fix in this chain (BUG-295, BUG-298, BUG-306) added
  exactly one new isolated test for its new branch, achieving 100% single-flag branch coverage
  each time, while the *pairwise* combination between the newest branch and every
  previously-existing branch was never constructed by any test.
- **Source fn:** `mre_bug333_occupied_elsewhere_not_masked_by_cached` in
  `tests/usage/refresh_tests_b.rs`
- **Source:** [pitfall/007_label_selection_branch_priority_pitfalls.md §Pitfall 4](../../../docs/pitfall/007_label_selection_branch_priority_pitfalls.md)
