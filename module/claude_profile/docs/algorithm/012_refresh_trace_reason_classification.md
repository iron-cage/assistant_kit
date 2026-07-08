# Algorithm: Refresh Trace Reason Classification

### Scope

- **Purpose**: Define how `apply_refresh()`'s trace output selects a single human-readable reason string from an account's ownership, cache, occupancy, and result state.
- **Responsibility**: Documents `reason_label()`'s entry point, its four-branch priority order, the data sources that populate each branch's condition, and the co-occurrence hazard between the `cached` and `is_occupied_elsewhere` branches that BUG-333 found.
- **In Scope**: `reason_label(aq: &AccountQuota, now_secs: u64) -> &str` branch order and priority; the four condition sources (`is_owned`, `cached` + `expires_at_ms`, `is_occupied_elsewhere`, `result`) and where each is set upstream; the `cached ∧ is_occupied_elsewhere` co-occurrence hazard.
- **Out of Scope**: `should_refresh()` control-flow predicate that determines whether a refresh is actually attempted (→ `refresh_predicate.rs`, feature/036 G2 gate — a separate function `reason_label()` does not influence); `apply_touch()`'s independent reason derivation (→ feature/024); the general co-occurrence-testing requirement this algorithm's hazard is an instance of (→ invariant/012).

### Abstract

`reason_label()` answers one question for `apply_refresh()`'s trace output: given an account's current ownership, cache, occupancy, and result state, which single string best explains why `should_retry` came out the way it did? Four conditions are checked in a fixed priority order, and the first one that matches wins — every condition after the first match is never evaluated, meaning its information is silently dropped whenever it also happens to be true. The function was extracted from an inline three-branch block (feature/036 AC-24, Fix BUG-306) specifically to make the predicate–reason 1:1 contract unit-testable; BUG-333 confirmed that the branch priority itself had introduced the co-occurrence hazard this doc formalizes, independent of whether the function is extracted or inline, and its fix reordered the two affected branches to close the gap (see § The Co-Occurrence Hazard below).

### Algorithm

#### Entry Point

`claude_profile/src/usage/refresh.rs:32` — `pub fn reason_label( aq : &AccountQuota, now_secs : u64 ) -> &str`

Called from `apply_refresh()` at `refresh.rs:112`, once per account per invocation, immediately after `should_refresh(aq, now_secs)` is evaluated and only when `trace` is enabled:

```rust
let should_retry = should_refresh( aq, now_secs );
if trace
{
  eprintln!( "{}refresh  {}  should_retry={} (reason: {})", trace_ts(), aq.name, should_retry, reason_label( aq, now_secs ) );
}
```

`reason_label()`'s return value feeds only this trace string. It has no feedback path into `should_retry` or any other control-flow decision — `should_refresh()` (`refresh_predicate.rs`) evaluates its own independent conditions and is called separately, one line above.

#### Branch Priority Table

| Priority | Branch (`refresh.rs` line) | Condition | Returns | Condition source (upstream) |
|----------|---------------------------|-----------|---------|------------------------------|
| 1 | 34 | `!aq.is_owned` | `"not owned"` | `is_owned(account)` predicate (feature/036) — ownership comparison against `current_identity()` |
| 2 | 38–41 | `aq.is_occupied_elsewhere` | `"occupied elsewhere"` | `other_machines_active()` marker-file detection, assigned at `fetch.rs:117,188,321` |
| 3 | 42–46 | `aq.cached` (nested: `(aq.expires_at_ms / 1000) <= now_secs`) | `"cached-expired"` (token also expired) or `"cached"` (token still valid) | `aq.cached` set by cache-fallback logic in `fetch.rs` (G1/G1b path or 429+cache fallback, `fetch.rs:306-313`) |
| 4 | 47–50 (`else`) | — (residual) | `aq.result.as_ref().err().map_or("ok", String::as_str)` | `aq.result` — live fetch/refresh outcome |

Branches are checked top-to-bottom; the first matching condition returns immediately. Priorities 2 and 3 are independently-set boolean flags (see column 5) — nothing in either flag's assignment logic prevents both from being `true` simultaneously for the same account. BUG-333's fix reordered these two branches from their pre-fix sequence (`cached` checked before `is_occupied_elsewhere`) to the current one (`is_occupied_elsewhere` checked before `cached`) — see § The Co-Occurrence Hazard below.

#### The Co-Occurrence Hazard (BUG-333)

Per `feature/036_account_ownership.md` G1b, `fetch_quota_for_list()` unconditionally routes any owned, non-current, occupied-elsewhere account through `approximate_quota()` (`fetch.rs:161-167`, `fetch.rs:415-448`) rather than a live HTTP fetch. `approximate_quota()` independently determines:

- `aq.cached` — from `read_cached_quota()` / cache-history presence (Feature 040 approximation availability)
- `aq.is_occupied_elsewhere` — hardcoded `true` by the caller, since G1b only fires when this is already known to be true

These are two unrelated data sources evaluated in the same call. Consequently, **any occupied-elsewhere account with a prior cache entry — the common case after its first fetch — has both `cached = true` and `is_occupied_elsewhere = true` when `reason_label()` runs.** Prior to BUG-333's fix, `cached` (then Priority 2) was checked before `is_occupied_elsewhere` (then Priority 3), so the function returned `"cached"` or `"cached-expired"` and the occupancy information was silently dropped — even though `fetch`/`touch` trace lines for the same account in the same invocation correctly reported `"occupied elsewhere"` (BUG-333 Symptom, verbatim transcript captured pre-fix):

```
2026-07-06 · 17:23:20 · fetch     i13@wbox.pro  skipped (reason: occupied elsewhere)
2026-07-06 · 17:23:24 · refresh   i13@wbox.pro  should_retry=false (reason: cached)
2026-07-06 · 17:23:26 · touch     i13@wbox.pro  skipped (reason: occupied elsewhere)
```

BUG-333's fix reordered the two branches so `is_occupied_elsewhere` (now Priority 2) is checked before `cached` (now Priority 3), and the function now correctly returns `"occupied elsewhere"` for this co-occurring case. This was purely a diagnostic/trace-string defect (BUG-333 Severity: Low) — `should_refresh()`'s own occupancy gate (feature/036 G2: `!is_owned || is_occupied_elsewhere`) already independently returned `false` for this account regardless of what `reason_label()` reported, so no quota value, credential mutation, or renewal date was ever affected. See `invariant/012_label_selection_requires_cooccurrence_coverage.md` for the formal invariant this hazard violates, and `pitfall/007_label_selection_branch_priority_pitfalls.md` for the four-bug recurrence history of this same function/seam.

### Features

| File | Relationship |
|------|--------------|
| [feature/036_account_ownership.md](../feature/036_account_ownership.md) | AC-22/AC-23/AC-24/AC-25 define `reason_label()`'s branches; G1b defines the `approximate_quota()` call site that produces the `cached ∧ is_occupied_elsewhere` co-occurrence |

### Sources

| File | Relationship |
|------|--------------|
| `src/usage/refresh.rs:32-51` | `reason_label()` — the four-branch function this doc describes |
| `src/usage/refresh.rs:76-201` | `apply_refresh()` — sole caller, line 112 |
| `src/usage/fetch.rs:161-167,415-448` | G1b gate and `approximate_quota()` — the production site where `cached` and `is_occupied_elsewhere` are independently set on the same account |
| `src/usage/refresh_predicate.rs` | `should_refresh()` — the separate, independently-evaluated control-flow predicate `reason_label()`'s output does not feed into |

### Tests

| File | Relationship |
|------|--------------|
| `tests/usage/refresh_tests_b.rs` | `reason_label_not_owned`, `reason_label_cached_expired`, `reason_label_cached_valid`, `reason_label_ok`, `reason_label_err`, `mre_bug298_apply_refresh_trace_reason_cached_expired`, `mre_bug_gap20_refresh_trace_reason_ok_owned_non_cached_ok`, `mre_bug306_refresh_trace_reason_occupied_elsewhere` — single-flag-isolation coverage of each branch, none construct the `cached ∧ is_occupied_elsewhere` co-occurrence; `mre_bug333_occupied_elsewhere_not_masked_by_cached` (added 2026-07-08) closes that gap — constructs `cached: true` ∧ `is_occupied_elsewhere: true` together and asserts `"occupied elsewhere"` (per invariant/012) |

### Invariants

| File | Relationship |
|------|-------------|
| [invariant/012_label_selection_requires_cooccurrence_coverage.md](../invariant/012_label_selection_requires_cooccurrence_coverage.md) | Formal co-occurrence coverage requirement this algorithm's hazard is a concrete instance of |

### Pitfalls

| File | Relationship |
|------|-------------|
| [pitfall/007_label_selection_branch_priority_pitfalls.md](../pitfall/007_label_selection_branch_priority_pitfalls.md) | BUG-295/298/306/333 — recurrence history of masking defects in this same function |
| [pitfall/005_ownership_gate_pitfalls.md](../pitfall/005_ownership_gate_pitfalls.md) | Pitfall 4 — prior BUG-306 fix to this same function, from the ownership-gate-family angle |
