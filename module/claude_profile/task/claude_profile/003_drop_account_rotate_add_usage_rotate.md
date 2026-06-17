# Drop `.account.rotate`; Add `rotate::1` to `.usage` with Strategy-Driven Account Rotation

## Execution State

- **Executor Type:** ai
- **Actor:** null
- **Claimed At:** null
- **Reopen Count:** 0
- **State:** ✅ (Completed)
- **Closes:** null
- **Blocked Reason:** null
- **Validated By:** null
- **Validation Date:** null

## Goal

**Why (null hypothesis answered):** Skipping this task leaves `.account.rotate` using
`auto_rotate()` → `max_by_key(expires_at_ms)` — it picks the account with the longest-lived
OAuth token, which has no relationship to quota availability. Meanwhile, `.usage` shows a `→`
winner selected by `find_next_for_strategy()` (quota-aware: renew/endurance/drain). A user who
sees the `→` marker in the quota table and then runs `clp .account.rotate` is switched to a
different account than the recommended one. This semantic mismatch is a UX defect with no
workaround short of manually running `.account.use`. Additionally, `.account.rotate` has no G5
ownership gate, no strategy selection, and must call `pre_switch_touch_ctx()` for a redundant
API fetch — all three problems disappear when rotation is moved into `.usage rotate::1`.
`.account.rotate` is deprecated (Feature 008); its removal is a committed, documented
requirement (Feature 038, param 059). Skipping this task preserves a defective command.

Implement `rotate::1` on `.usage`: after the quota table renders, switch to the `→` winner via
`find_next_for_strategy()` (reusing the already-fetched `AccountQuota`; no extra API call).
Enforce G5 ownership gate on rotation eligibility; bypass with `force::1`. Add `live::1` mutual
exclusion guard. Remove `auto_rotate()` from `claude_profile_core` and
`account_rotate_routine()` from `src/commands/account_ops.rs`. Register `.account.rotate` as an
exit-1 redirector pointing to `.usage rotate::1`.

Task is complete when ALL of the following hold:
1. `clp .usage rotate::1` — quota table rendered; switches to the `→` winner (default `next::renew`); stdout ends with `switched to '{name}'`; exit 0
2. `clp .usage rotate::1 dry::1` — quota table rendered; stdout ends with `[dry-run] would switch to '{name}'`; credential store unchanged; exit 0
3. `clp .usage rotate::1` — no eligible owned non-current non-active account exists; exit 1; stdout or stderr contains `"no eligible account to rotate to"`; table still rendered
4. `clp .usage rotate::1 live::1` — exits 1 before any fetch; stderr or stdout contains mutual exclusion message; table NOT rendered
5. `clp .usage rotate::1` — two accounts: `owned@acme.com` (owned) and `foreign@acme.com` (non-owned); non-owned account receives no `→` marker; `owned@acme.com` is switched to; exit 0
6. `clp .usage rotate::1 force::1` — non-owned account eligible; may be selected as winner; exit 0; no ownership-violation error
7. `clp .usage rotate::1 next::endurance` — switches to endurance strategy winner; stdout ends with `switched to '{name}'`; exit 0
8. `clp .usage rotate::1 next::drain` — switches to drain strategy winner; stdout ends with `switched to '{name}'`; exit 0
9. `clp .account.rotate` — exits 1; stdout contains the migration message `"use 'clp .usage rotate::1' instead"` (exact wording may differ but must point to `.usage rotate::1`)
10. `RUSTFLAGS="-D warnings" cargo nextest run --all-features` exits 0 with no test failures and no compilation warnings

## In Scope

**P-1 — UsageParams extension (`src/usage/types.rs`):**
- Add `rotate: bool` field to `UsageParams` struct

**P-2 — Parameter parsing (`src/usage/params.rs`):**
- Parse `rotate::` bool param (default `0`) in `parse_usage_params()`
- Add mutual exclusion guard: if `rotate && live` → return `Err` with `"rotate::1 and live::1 are mutually exclusive"` message before any fetch

**P-3 — Eligibility gate (`src/usage/sort_next.rs` and `src/usage/render.rs`):**
- Add `gate_ownership: bool` parameter to `find_next_for_strategy()` in `src/usage/sort_next.rs`
- When `gate_ownership: true`: extend the `extra` predicate in all three strategy arms (`Renew`, `Endurance`, `Drain`) to also require `aq.is_owned`; combined predicate: `prefer_weekly(aq, prefer) > 5.0 && aq.is_owned`
- When `gate_ownership: false`: behavior unchanged (predicate remains `prefer_weekly(aq, prefer) > 5.0`)
- Update `render_text()` in `src/usage/render.rs` line ~50: the call to `find_next_for_strategy()` that sets `best_idx` (which drives the `→` table-body marker) must now pass `gate_ownership = params.rotate && !params.force`; `render_text()` must accept `rotate: bool` and `force: bool` (or equivalent) to compute this
- Update the footer block in `render_text()` (line ~270) — footer calls `find_next_for_strategy` for all three strategies unconditionally; pass `gate_ownership: false` here (footer recommendations are always ungated)
- Update the api.rs rotation dispatch call to `find_next_for_strategy` to also pass `gate_ownership = params.rotate && !params.force`

**P-4 — Rotation dispatch (`src/usage/api.rs`):**
- Note: the `live::1` mutual exclusion guard (P-2) fires during parameter parsing — before any fetch or render — so by the time P-4's dispatch block is reached, `params.live` is guaranteed `false`. No `live` check needed in P-4.
- After the quota table is rendered in `usage_routine()`, add rotation dispatch block (only entered when `params.rotate == true`):
  1. Call `find_next_for_strategy(accounts, params.next, params.prefer, now_secs, params.rotate && !params.force)` → `winner`
  2. If `winner.is_none()` → emit `"no eligible account to rotate to"` → exit 1
  3. If `params.dry` → emit `"[dry-run] would switch to '{name}'"` → exit 0
  4. G5 gate enforced via `gate_ownership` in step 1; no additional ownership check needed here
  5. Call `switch_account(winner_name, credential_store, paths)` from `claude_profile_core::account`
  6. Call `apply_touch(&mut winner_aq, credential_store, paths, trace, imodel, effort)` from `src/usage/touch.rs` — uses the winner's `AccountQuota` already in memory; no additional quota API call
  7. Emit `"switched to '{name}'"` → exit 0

**P-5 — Post-switch touch reuse (`src/usage/touch.rs`):**
- No new function needed. The existing `apply_touch(aq: &mut AccountQuota, ...)` in `src/usage/touch.rs` takes a mutable reference to the winner's already-fetched `AccountQuota`. Call it directly after `switch_account()` in P-4 step 6. This reuses the in-memory quota data, eliminating the `pre_switch_touch_ctx()` API call that `.account.use` requires.

**P-6 — Remove `auto_rotate()` (`claude_profile_core/src/account.rs`):**
- Delete `auto_rotate()` function (lines ~480–493); it is exported as `pub fn` by its own visibility — no separate `pub use` re-export exists; deleting the function is sufficient
- Update the doc comment in `claude_profile_core/src/lib.rs` (line ~46) that references `account::auto_rotate()` — remove the example or replace with a `.usage rotate::1` reference

**P-7 — Remove `account_rotate_routine()` (`src/commands/account_ops.rs`):**
- Delete `account_rotate_routine()` function
- Remove its export from `src/commands/mod.rs`

**P-8 — Register `.account.rotate` redirector (`src/registry.rs`):**
- Replace `.account.rotate` command registration with an exit-1 redirector that emits a message pointing users to `clp .usage rotate::1`

**P-9 — Test updates (`tests/cli/`):**
- Write failing tests (red phase) for criteria 1–9 before implementing P-3/P-4/P-5
- Test file: `tests/cli/usage_rotate_test.rs` (new) or extend `tests/cli/usage_test.rs`
- Tests for live network cases (criteria 1, 5, 6, 7, 8) use the `lim_it` guard
- Update or delete any `.account.rotate` command test files in `tests/cli/`

## Out of Scope

- Changes to `find_next_for_strategy()` strategy algorithms (Feature 023 — stable)
- Changes to `sort_indices()` sort logic
- Ownership gate changes on any command other than the `.usage rotate::1` path
- Adding `switched_to` or any new field to `format::json` output
- Changes to `next::`, `prefer::`, `sort::`, or other existing `.usage` params
- Changes to how `.usage` handles `live::`, `dry::`, `force::` in non-rotate paths
- Documentation edits (completed in the doc update pass before this task)

## Work Procedure

1. **Red (all tests)** — create `tests/cli/usage_rotate_test.rs` (or extend `tests/cli/usage_test.rs`); write ALL failing test functions before any implementation; tests fail to compile (red) until step 2 adds `rotate: bool` to `UsageParams`:
   - (a) [criterion 4] `rotate::1 live::1` → exit 1; mutual exclusion error; no table
   - (b) [criterion 3] `rotate::1` no eligible candidate → exit 1; `"no eligible account to rotate to"`; table rendered
   - (c) [criterion 2] `rotate::1 dry::1` → preview message; no credential change; exit 0
   - (d) [criterion 9] `clp .account.rotate` → exit 1; migration message contains `.usage rotate::1`
   - (e) [criterion 1] `rotate::1` core switch — lim_it-gated; switches to `→` winner; stdout ends with `switched to 'name'`; exit 0
   - (f) [criterion 5] `rotate::1` G5 gate — lim_it-gated; non-owned account gets no `→`; owned account selected
   - (g) [criterion 6] `rotate::1 force::1` — lim_it-gated; non-owned account eligible; exit 0
   - (h) [criterion 7] `rotate::1 next::endurance` — lim_it-gated; endurance winner selected; exit 0
   - (i) [criterion 8] `rotate::1 next::drain` — lim_it-gated; drain winner selected; exit 0
2. **Params** — add `rotate: bool` to `UsageParams` in `src/usage/types.rs`; add `rotate::` parsing in `parse_usage_params()` in `src/usage/params.rs`; add mutual exclusion guard (fails with error before any fetch; criterion 4 turns green; tests from step 1 now compile and can run)
3. **Eligibility gate** — add `gate_ownership: bool` param to `find_next_for_strategy()` in `src/usage/sort_next.rs`; update `extra` predicate in all three arms; update `render_text()` in `src/usage/render.rs` to accept and pass `gate_ownership = params.rotate && !params.force` to the `best_idx` call (~line 50); pass `gate_ownership: false` to footer calls (~line 270); update api.rs dispatch call with same flag
4. **Dispatch** — add post-render rotation dispatch block in `usage_routine()` in `src/usage/api.rs` following the order in P-4; wire `apply_touch()` from `src/usage/touch.rs` with `&mut winner_aq` (criteria 3, 2 turn green; live tests 1, 5–8 become runnable)
5. **Cleanup** — delete `auto_rotate()` from `claude_profile_core/src/account.rs` (lines ~480–493); update the doc comment in `claude_profile_core/src/lib.rs` (~line 46) that references `auto_rotate()`; delete `account_rotate_routine()` from `src/commands/account_ops.rs`; remove its re-export from `src/commands/mod.rs`; register `.account.rotate` exit-1 redirector in `src/registry.rs` (criterion 9 turns green); update or delete any existing `.account.rotate` test files in `tests/cli/`
6. **Green** — run `./verb/test` (project container test runner); fix compilation errors and test failures until all pass; criterion 10 satisfied

## Test Matrix

| Input Scenario | Config Under Test | Expected Behavior | Criterion |
|----------------|-------------------|-------------------|-----------|
| `clp .usage rotate::1 live::1` — any environment; guard fires in params layer pre-fetch | mutual exclusion guard | Exit 1; error message before any fetch or table render | 4 |
| `clp .usage rotate::1` — all accounts are current/active/non-owned | no eligible candidate | Exit 1; `"no eligible account to rotate to"`; table rendered | 3 |
| `clp .usage rotate::1 dry::1` — one eligible owned non-current account | dry-run preview | Exit 0; `[dry-run] would switch to 'name'`; credential store unchanged | 2 |
| `clp .usage rotate::1` — two accounts: current + eligible owned non-current | core switch (live) | Exit 0; table shows `→` on winner; stdout ends with `switched to 'name'`; active marker updated | 1 |
| `clp .usage rotate::1` — eligible account is non-owned, no force:: | G5 gate active | Non-owned account gets no `→`; owned account selected; exit 0 | 5 |
| `clp .usage rotate::1 force::1` — only non-owned eligible account | G5 gate bypassed | Non-owned account eligible; switch executed; exit 0 | 6 |
| `clp .usage rotate::1 next::endurance` | endurance strategy | Switches to account with most 5h session quota; exit 0 | 7 |
| `clp .usage rotate::1 next::drain` | drain strategy | Switches to account with least non-zero weekly quota; exit 0 | 8 |
| `clp .account.rotate` | removed command redirector | Exit 1; stdout/stderr contains `"clp .usage rotate::1"` migration hint | 9 |
| `RUSTFLAGS="-D warnings" cargo nextest run --all-features` | full suite | Exit 0; no test failures; no compilation warnings | 10 |

## Affected Entities

- `src/usage/` — `types.rs`, `params.rs`, `sort_next.rs`, `api.rs`, `touch.rs` (core implementation)
- `src/commands/account_ops.rs` — removal of `account_rotate_routine()`
- `src/registry.rs` — `.account.rotate` redirector registration
- `claude_profile_core/src/account.rs` — removal of `auto_rotate()`
- `tests/cli/` — new rotate test cases; removal of `.account.rotate` test cases

## Related Documentation

- `docs/feature/038_usage_strategy_rotate.md` — 11 ACs defining complete behavior
- `docs/cli/param/059_rotate.md` — `rotate::` param specification
- `docs/feature/008_auto_rotate.md` — deprecated predecessor (removal documented here)
- `docs/cli/command/006_usage.md` — `.usage` command spec including `rotate::` param row and step 12
- `docs/cli/command/001_account.md` — `.account.rotate` section removed in doc pass
- `docs/feature/023_next_account_strategies.md` — `find_next_for_strategy()` algorithm (stable; reused)
- `docs/feature/036_account_ownership.md` — G5 ownership gate design
- `docs/feature/004_account_use.md` — `switch_account()` primitive
- `docs/feature/024_session_touch.md` — post-switch touch design
- `tests/docs/feature/38_usage_strategy_rotate.md` — 11 FT cases (FT-01 – FT-11)
- `tests/docs/cli/command/09_usage.md` — IT-75 – IT-80 rotate:: integration test group

## History

- **2026-06-17** `CREATED` — Drop `.account.rotate` and add `rotate::1` to `.usage` with strategy-driven account rotation, G5 ownership gate, dry-run preview, and touch reuse from in-memory quota.
- **2026-06-17** `COMPLETED` — All 6 plan phases executed. `./verb/test` exits 0; 4/4 jobs pass (Local nextest, Workspace nextest, Doc tests, Clippy). All 10 criteria satisfied.

## Verification Findings

**Round 2 (2026-06-17) — All 4 PASS → task verified.** Findings from Round 1 fixed (see below).

---

**Round 1 (2026-06-17) — FAIL on 2 of 4 dimensions:**

**Scope Coherence — FAIL:**
- P-5 said "implement or wire `apply_post_switch_touch_from_quota()`" — this function does not exist; unclear whether new or existing. Fixed: P-5 now says call the existing `apply_touch()` with `&mut winner_aq` directly.
- Criterion 4 ("exits 1 before any fetch; table NOT rendered") was ambiguous with P-4 (post-render dispatch) — created structural contradiction about which layer owns the early exit. Fixed: P-3/P-4 clarified that the `live::1` mutual exclusion guard lives entirely in P-2 (params layer); P-4's dispatch block is only reached when `params.live == false`.
- Criterion 5 asserted `→` marker behavior (non-owned accounts receive no `→`) but P-3 did not explicitly name `render_text()` as needing an update. Fixed: P-3 now explicitly states `render_text()` in `src/usage/render.rs` must pass `gate_ownership` to the `find_next_for_strategy()` call that sets `best_idx`.

**Implementation Readiness — FAIL:**
- Step 7 said "remove exports" without naming the file. Fixed: P-6 clarifies that `auto_rotate` is `pub fn` in `claude_profile_core/src/account.rs` — exported by function visibility, not a `pub use` re-export; deletion of the function is sufficient.
- Live tests (criteria 1, 5, 6, 7, 8) were scheduled for step 6 (after implementation), violating TDD red-before-green for those criteria. Fixed: step 1 now writes ALL tests including lim_it-gated ones; they fail to compile (red) before `rotate: bool` exists in `UsageParams`.
- Test Matrix row for criterion 4 used "empty store" as Config Under Test — irrelevant to the guard mechanism. Fixed: now says "any environment; guard fires in params layer pre-fetch".

## Verification Record

- **Date:** 2026-06-17
- **Validators:** 4 independent Agent subagents (adversarial mandate)
- **Dimensions checked:** Scope Coherence, MOST Goal Quality, Value/YAGNI, Implementation Readiness
- **Result:** Round 1 — FAIL (Scope Coherence, Implementation Readiness). Round 2 — All 4 PASS.
- **Notes:** Round 1 failures required 3 Scope Coherence fixes (P-5 "implement or wire" ambiguity resolved to existing `apply_touch()`; criterion 4/P-4 mutual exclusion layer ambiguity clarified; P-3 updated to name `render_text()` as a call site for `gate_ownership`), and 3 Implementation Readiness fixes (P-6 export removal file named explicitly; lim_it test writing moved to step 1 for TDD compliance; Test Matrix criterion 4 config under test clarified). Round 2 passed all 4 dimensions independently.
