# Algorithm 012: Refresh Trace Reason Classification

AC test cases for `docs/algorithm/012_refresh_trace_reason_classification.md`. Tests `reason_label(aq: &AccountQuota, now_secs: u64) -> &str` in `src/usage/refresh.rs:32-51`.

**Type note:** `reason_label()` selects one string from a fixed four-branch priority order (`!is_owned` → `is_occupied_elsewhere` → `cached` → residual `result`/`"ok"`), checked top-to-bottom; the first matching condition returns immediately, so any condition after the first match is never evaluated even if also true. Post-BUG-333 fix, `is_occupied_elsewhere` is Priority 2 (checked before `cached`, now Priority 3) — this is the current, correct order these cases verify.

### AC Case Index

| AC | Short Name | Category | Status |
|----|------------|----------|--------|
| AC-1 | Not-owned account → `"not owned"` regardless of other flags | Nominal | ✅ |
| AC-2 | Owned, cached, valid token, not occupied elsewhere → `"cached"` | Nominal | ✅ |
| AC-3 | Owned, cached, `expires_at_ms` boundary at `now_secs` → `"cached-expired"` (boundary inclusive) | Boundary | ✅ |
| AC-4 | Owned, not cached, not occupied elsewhere, `result = Err` → residual error string | Error | ✅ |
| AC-5 | Owned, cached AND occupied elsewhere simultaneously → `"occupied elsewhere"` wins, not masked by `cached` (BUG-333) | Regression (BUG-333) | ✅ |

---

### AC-1: Not-owned account returns `"not owned"` regardless of cache or occupancy state

- **Given:** `aq.is_owned = false`, with `aq.cached = true` and `aq.is_occupied_elsewhere = true` also set (deliberately, to prove Priority 1 short-circuits before either is evaluated)
- **When:** `reason_label(&aq, now_secs)` is called
- **Then:** Returns `"not owned"` — Priority 1 (`!aq.is_owned`) matches first and returns immediately; the `cached`/`is_occupied_elsewhere` values are never inspected

### AC-2: Owned, cached, valid token, not occupied elsewhere returns `"cached"`

- **Given:** `aq.is_owned = true`, `aq.is_occupied_elsewhere = false`, `aq.cached = true`, `aq.expires_at_ms` set to a value whose `/1000` is greater than `now_secs` (token not yet expired)
- **When:** `reason_label(&aq, now_secs)` is called
- **Then:** Returns `"cached"` — Priority 1 and 2 both fail (`is_owned = true`, `is_occupied_elsewhere = false`), Priority 3 matches (`aq.cached = true`), and the nested expiry check (`(aq.expires_at_ms / 1000) <= now_secs`) is `false`, selecting the non-expired variant

### AC-3: Owned, cached, token expiry exactly at `now_secs` returns `"cached-expired"` (boundary inclusive)

- **Given:** `aq.is_owned = true`, `aq.is_occupied_elsewhere = false`, `aq.cached = true`, `aq.expires_at_ms = now_secs * 1000` — `(aq.expires_at_ms / 1000) == now_secs` exactly
- **When:** `reason_label(&aq, now_secs)` is called
- **Then:** Returns `"cached-expired"` — the nested condition is `(aq.expires_at_ms / 1000) <= now_secs`; at exact equality this is `true`, so the boundary is inclusive on the expired side

### AC-4: Owned, not cached, not occupied elsewhere, with a fetch error returns the residual error string

- **Given:** `aq.is_owned = true`, `aq.is_occupied_elsewhere = false`, `aq.cached = false`, `aq.result = Err("some fetch error".to_string())`
- **When:** `reason_label(&aq, now_secs)` is called
- **Then:** Returns `"some fetch error"` — Priorities 1-3 all fail, falling through to Priority 4 (`aq.result.as_ref().err().map_or("ok", String::as_str)`), which extracts the error string; had `aq.result` been `Ok(_)` instead, this same residual branch would return `"ok"`

### AC-5: Owned account that is both cached and occupied elsewhere returns `"occupied elsewhere"`, not masked by the cached branch (BUG-333)

- **Given:** `aq.is_owned = true`, `aq.cached = true` (token valid, not expired), `aq.is_occupied_elsewhere = true` — the near-universal co-occurring case per `feature/036_account_ownership.md` G1b, where `fetch_quota_for_list()` unconditionally routes any owned, non-current, occupied-elsewhere account through `approximate_quota()`, which independently sets `cached` (from cache-history presence) and `is_occupied_elsewhere` (hardcoded `true` by the caller) as two unrelated data sources that can both be true on the same account
- **When:** `reason_label(&aq, now_secs)` is called
- **Then:** Returns `"occupied elsewhere"` — Priority 2 (`aq.is_occupied_elsewhere`) is checked before Priority 3 (`aq.cached`), so the occupancy branch matches first and the function returns before the cached branch is ever reached
- **Note:** BUG-333 regression — before the fix, `cached` was Priority 2 and `is_occupied_elsewhere` was Priority 3, so this exact co-occurrence returned `"cached"`/`"cached-expired"` and silently dropped the occupancy signal, even though the `fetch` and `touch` trace lines for the same account in the same invocation correctly reported `"occupied elsewhere"` (verbatim pre-fix transcript: `refresh   i13@wbox.pro  should_retry=false (reason: cached)` alongside `fetch     i13@wbox.pro  skipped (reason: occupied elsewhere)`). Purely a diagnostic/trace-string defect (BUG-333 Severity: Low) — `should_refresh()`'s own occupancy gate (feature/036 G2) already independently returned `false` for this account regardless of what `reason_label()` reported, so no quota value, credential mutation, or renewal date was ever affected. This is the fourth defect in the same masked-label-selection lineage as BUG-295/298/306 (see `docs/pitfall/007_label_selection_branch_priority_pitfalls.md`), each of which fixed a different single-flag branch of this same function — BUG-333 is the one that specifically targeted the `cached ∧ is_occupied_elsewhere` co-occurrence this AC verifies.
