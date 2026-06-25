# FT — Feature 036: Account Ownership

### Scope

- **Purpose**: Test cases for account ownership enforcement — ownership-neutral `.account.save` and `.accounts assignee::USER@MACHINE name::X`, `.accounts owner::0 name::X` with G8 gate (Feature 064/065; formerly `assign::1`/`unclaim::1`), nine enforcement gates (G1, G1b, G2–G8), backward compatibility, and `is_owned` JSON field.
- **Source**: `docs/feature/036_account_ownership.md`
- **Covers**: AC-01 through AC-24

### Test Cases

| FT | AC | Scenario | Source fn |
|----|----|----------|-----------|
| FT-01 | AC-01 | `.account.save` does NOT modify `owner` — `account_save_routine()` passes `owner: None`; existing value preserved | ✅ `ft01_save_does_not_stamp_owner` |
| FT-02 | AC-02 | `.accounts owner::0 name::alice` exits 0; writes `owner: ""`; `write_owner()` called directly — no credential re-save (Feature 064; formerly `unclaim::1`) | ✅ `ft02_unclaim_clears_owner` |
| FT-03 | AC-03 | `owner::` NOT on `.account.save` (exits 1); `.account.unclaim` and `.account.assign` fully deregistered (generic "unknown command"); `.accounts unclaim::1` and `.accounts assign::1` exit 1 with REMOVED_TOGGLE migration message (Feature 064) | ✅ `ft03_unclaim_param_placement` |
| FT-04 | AC-04 | Non-owned account: `fetch_quota_for_list` skips token read + HTTP; reads cache; `aq.is_owned = false` | ✅ `ft04_non_owned_uses_cache_not_http` |
| FT-05 | AC-05 | Non-owned account with cache: usage row renders with `~` prefix and age indicator; without cache: dashes | ✅ `ft05_non_owned_display_tilde_or_dashes` |
| FT-06 | AC-06 | `should_refresh()` returns `false` when `aq.is_owned == false` OR `aq.is_occupied_elsewhere == true`; gate condition: `!is_owned \|\| is_occupied_elsewhere` | ✅ `ft06_should_refresh_false_when_not_owned` |
| FT-07 | AC-07 | `apply_touch()` skips accounts where `!is_owned \|\| is_occupied_elsewhere`; trace emits `"not owned"` or `"occupied elsewhere"` matching the fired gate | ✅ `ft07_touch_skips_non_owned_with_trace` |
| FT-08 | AC-08 | `.account.use` with non-owned account exits 1 with ownership violation message | ✅ `ft08_use_exits_1_when_not_owned` |
| FT-09 | AC-09 | `.account.delete` with non-owned account exits 1 with ownership violation message | ✅ `ft09_delete_exits_1_when_not_owned` |
| FT-10 | AC-10 | `.account.relogin` with non-owned account exits 1 with ownership violation message | ✅ `ft10_relogin_exits_1_when_not_owned` |
| FT-11 | AC-11 | Account without `owner` field passes all G1–G8 gates; behavior identical to pre-feature | ✅ `ft11_no_owner_field_backward_compat` |
| FT-12 | AC-12 | `format::json` includes `"is_owned": true` or `"is_owned": false` per account | ✅ `ft12_json_output_includes_is_owned` |
| FT-13 | AC-13 | `dry::1` on `.account.use`/`.account.delete`/`.account.relogin` does NOT skip ownership check | ✅ `ft13_dry_run_does_not_skip_ownership` |
| FT-14 | AC-14 | Background `save()` callers pass `owner: None` — existing `owner` preserved; `accounts_routine()` assign path does NOT call `write_owner()` | ✅ `ft14_background_save_preserves_owner` |
| FT-15 | AC-15 | `.account.save owner::0` exits 1 — `owner::` not registered on `.account.save`; `.accounts unclaim::1` exits 1 with REMOVED_TOGGLE migration message; `.accounts assign::1` exits 1 with REMOVED_TOGGLE migration message (Feature 064) | ✅ `ft15_unclaim_not_on_save_or_assign` |
| FT-16 | AC-16 | `.accounts owner::0 name::X` with account owned by different identity exits 1 with ownership violation; already-unowned account exits 0 (Feature 064; formerly `unclaim::1`) | ✅ `ft16_unclaim_g8_gate` |
| FT-17 | AC-17 | `.accounts owner::0 name::alice dry::1` prints `[dry-run]` line; `alice.json` unchanged (Feature 064; formerly `unclaim::1 dry::1`) | ✅ `ft17_unclaim_dry_run` |
| FT-18 | AC-18 | `.account.use name::X force::1` when X owned by different identity bypasses G5; exits 0; `switch_account()` called | ✅ `ft18_use_force_bypasses_g5` |
| FT-19 | AC-19 | `.account.delete name::X force::1` when X owned by different identity bypasses G6; exits 0; files deleted | ✅ `ft19_delete_force_bypasses_g6` |
| FT-20 | AC-20 | `.account.relogin name::X force::1` when X owned by different identity bypasses G7; exits 0; 6-step relogin proceeds | ✅ `ft20_relogin_force_bypasses_g7` |
| FT-21 | AC-21 | `force::1 dry::1` on G5/G6/G7 commands bypasses ownership gate but previews without writing; exits 0; `[dry-run]` printed | ✅ `ft21_force_dry_bypasses_gate_previews` |
| FT-22 | AC-22 | `apply_refresh()` emits `[trace] refresh  <name>  should_retry=false (reason: not owned)` when `trace::1` and `aq.is_owned == false` — reason is `"not owned"`, not `"ok"` (BUG-295) | ✅ `mre_bug295_apply_refresh_trace_reason_not_owned` |
| FT-23 | AC-04 | G1 non-owned path applies polynomial approximation when history available (BUG-304) | ✅ `ft23_g1_non_owned_applies_approximation` |
| FT-24 | AC-23 | Owned + occupied-elsewhere + non-current account: `fetch_quota_for_list` skips token read + HTTP; calls `approximate_quota()`; emits `[trace] fetch  <name>  skipped (reason: occupied elsewhere)` when `trace::1` — Fix(BUG-305) | ✅ `mre_bug305_fetch_skips_occupied_elsewhere_with_trace` |
| FT-25 | AC-24 | `reason_label(aq, now_secs)` returns `"occupied elsewhere"` for owned + non-cached + occupied-elsewhere + Ok-result account — Fix(BUG-306); `apply_refresh()` trace emits correct reason | ✅ `mre_bug306_refresh_trace_reason_occupied_elsewhere` |

### Notes

- FT-01 is an integration test — calls `clp .account.save name::alice` and asserts existing `owner` field is UNCHANGED (`account_save_routine()` passes `owner: None`; ownership-neutral).
- FT-02 is an integration test — calls `clp .accounts owner::0 name::alice` and asserts exit 0, `owner: ""` written, and credential file NOT re-saved (`alice.credentials.json` mtime unchanged). The `accounts_routine()` owner::0 path calls `write_owner()` directly. (Feature 064; formerly `unclaim::1`.)
- FT-03 is structural with three cases: (a) `.account.save` rejects `owner::0` (exits 1 — not registered); (b) `.account.unclaim` and `.account.assign` produce generic "unknown command" error (fully deregistered); (c) `.accounts unclaim::1` and `.accounts assign::1` exit 1 with REMOVED_TOGGLE migration messages (Feature 064).
- FT-04 is a unit test in `src/usage/fetch.rs` — mock-free: verify no `read_token()` call path was exercised and cache JSON is the returned value.
- FT-05 is a render test in `src/usage/render_tests.rs` — uses `AccountQuota { is_owned: false, cached: true, ... }` and asserts `~` prefix; also tests `cached: false, is_owned: false` giving dashes.
- FT-06 is a unit test in `src/usage/refresh_predicate.rs` `#[cfg(test)]` module.
- FT-07 is a unit test in `src/usage/touch_tests.rs` using `gag::BufferRedirect::stderr()` for trace capture.
- FT-08 through FT-10 are integration tests via `verb/test` — verify exit code 1 and message text.
- FT-11 is a unit test in `claude_profile_core/tests/account_test.rs` — `{name}.json` with no `owner` key reads as `is_owned = true`.
- FT-12 is a render test in `src/usage/render_tests.rs` — verifies `"is_owned": true`/`"is_owned": false` in JSON object.
- FT-13 exercises G5/G6/G7 with `dry::1` flag set — ownership guard runs first; exit 1 regardless.
- FT-14 is a unit test in `claude_profile_core/tests/account_test.rs` — background `save()` with `owner: None` (e.g. `refresh_account_token`) on an account with `owner: "alice@host"` leaves `owner: "alice@host"` in `{name}.json`. All `save()` callers pass `owner: None` (preserves existing owner) — both background refresh and interactive `account_save_routine()` are ownership-neutral. The `accounts_routine()` assign path does NOT call `write_owner()`.
- FT-18 through FT-20 are integration tests via `./verb/test` — verify exit 0 and that the expected mutation (switch/delete/relogin) proceeds despite non-owned account.
- FT-21 is an integration test via `./verb/test` — three sub-cases (use, delete, relogin), each verifying: exit 0, `[dry-run]` line printed, no files modified. The G8 case (force+dry on `owner::0`) is deferred to Feature 037 tests (`37_accounts_usage_param_unification.md`).
- FT-18–21 require `force::` (`058`) to be registered on `.account.use`, `.account.delete`, `.account.relogin` — Task 002 prerequisite.
- FT-22 is a unit test in `src/usage/refresh_tests.rs` — uses `gag::BufferRedirect::stderr()` for trace capture. Reproduces BUG-295: verifies `apply_refresh()` emits `reason: not owned` (not `reason: ok`) when `aq.is_owned == false`.
- FT-24 is a unit test in `src/usage/fetch.rs` or `tests/cli/usage_test.rs` — creates a temp credential store with an `alice.json` owned by current identity and a `_active_{remote_host}_{remote_user}` marker file naming `alice`. Verifies no HTTP call fires and trace line emitted. Reproduces BUG-305.
- FT-25 is a unit test in `src/usage/refresh_tests.rs` — directly calls `reason_label(&aq, 0)` with `is_owned=true, cached=false, is_occupied_elsewhere=true, result=Ok(...)`. Verifies return value is `"occupied elsewhere"`. No `apply_refresh()` call needed — the extracted function is directly testable. Reproduces BUG-306.

---

### FT-01: `.account.save` does NOT modify `owner` field — ownership-neutral

- **Given:** Account `alice` exists in credential store. `alice.json` contains `"owner": "testuser@testmachine"`. `current_identity()` resolves to `"testuser@testmachine"`.
- **When:** `clp .account.save name::alice` — `account_save_routine()` passes `owner: None` to `save()`.
- **Then:** `alice.json` still contains `"owner": "testuser@testmachine"` — unchanged. Credentials re-saved. The `owner` field is preserved via read-merge; `account_save_routine()` does NOT call `write_owner()` or pass `Some(...)` for owner.
- **Exit:** 0
- **Source fn:** `ft01_save_does_not_stamp_owner`
- **Source:** [036_account_ownership.md AC-01](../../../docs/feature/036_account_ownership.md)

---

### FT-02: `.accounts owner::0 name::X` exits 0; writes `owner: ""`; no credential re-save (Feature 064)

- **Given:** Account `alice` exists in credential store. `alice.json` has `"owner": "testuser@testmachine"`. `current_identity()` resolves to `"testuser@testmachine"`. Record mtime of `alice.credentials.json`.
- **When:** `clp .accounts owner::0 name::alice` — `accounts_routine()` owner::0 path calls `write_owner("alice", store, "")` directly. (Formerly `unclaim::1 name::alice` — Feature 064.)
- **Then:** Exits 0. `alice.json` contains `"owner": ""`. All G1–G8 enforcement gates disabled for `alice`. mtime of `alice.credentials.json` is unchanged (no credential re-save).
- **Exit:** 0
- **Source fn:** `ft02_unclaim_clears_owner`
- **Source:** [036_account_ownership.md AC-02](../../../docs/feature/036_account_ownership.md)

---

### FT-03: `owner::` NOT on `.account.save`; `.account.unclaim`/`.account.assign` deregistered; REMOVED_TOGGLE stubs on `.accounts` (Feature 064)

- **Given:** Any environment.
- **When (case A):** `clp .account.save name::alice owner::0` is executed.
- **Then (case A):** Exits 1 — `owner::` not registered on `.account.save`. `alice.json` unchanged.
- **When (case B):** `clp .account.unclaim name::alice` is executed; separately `clp .account.assign name::alice` is executed.
- **Then (case B):** Both exit 1 with generic "unknown command" error — fully deregistered (not redirect stubs; same error as any unrecognized command).
- **When (case C):** `clp .accounts unclaim::1 name::alice` is executed; separately `clp .accounts assign::1 name::alice` is executed.
- **Then (case C):** Both exit 1 with REMOVED_TOGGLE migration messages: `unclaim::1` → "REMOVED — use `owner::0 name::X`"; `assign::1` → "REMOVED — use `assignee::USER@MACHINE name::X`". (Feature 064.)
- **Exit:** 1 (all cases)
- **Source fn:** `ft03_unclaim_param_placement`
- **Source:** [036_account_ownership.md AC-03](../../../docs/feature/036_account_ownership.md)

---

### FT-04: Non-owned account bypasses token read and HTTP call; uses cache

- **Given:** Account `alice` has `alice.json` with `"owner": "other@remote"` (not current identity). Cache contains valid quota data.
- **When:** `fetch_quota_for_list()` is called for `alice`.
- **Then:** `read_token()` is NOT called; `fetch_oauth_usage()` is NOT called; returned `AccountQuota` has `cached: true`, `is_owned: false`, and quota values from the cache.
- **Exit:** Ok(cached_data) with aq.is_owned=false
- **Source fn:** `ft04_non_owned_uses_cache_not_http`
- **Source:** [036_account_ownership.md AC-04](../../../docs/feature/036_account_ownership.md)

---

### FT-05: Non-owned display — `~` prefix when cache present; dashes when absent

- **Given (case A):** `AccountQuota { is_owned: false, cached: true, cache_age_secs: 600, ... }` with quota data.
- **When (case A):** Usage row rendered as text.
- **Then (case A):** Rendered line contains `~` prefix on the utilization value and age indicator (e.g., `10m`).
- **Given (case B):** `AccountQuota { is_owned: false, cached: false }` with no quota data.
- **When (case B):** Usage row rendered as text.
- **Then (case B):** All quota columns show `—`.
- **Exit:** rendered string assertions
- **Source fn:** `ft05_non_owned_display_tilde_or_dashes`
- **Source:** [036_account_ownership.md AC-05](../../../docs/feature/036_account_ownership.md)

---

### FT-06: `should_refresh()` returns `false` when `is_owned == false` OR `is_occupied_elsewhere == true`

- **Given (case A):** `AccountQuota { is_owned: false, is_occupied_elsewhere: false, ... }` — non-owned account.
- **Given (case B):** `AccountQuota { is_owned: true, is_occupied_elsewhere: true, ... }` — owned but occupied by another machine.
- **When:** `should_refresh(&aq)` is called for each case.
- **Then:** Returns `false` in both cases. Gate condition: `!is_owned || is_occupied_elsewhere`. No refresh is initiated.
- **Exit:** false (both cases)
- **Source fn:** `ft06_should_refresh_false_when_not_owned`
- **Source:** [036_account_ownership.md AC-06](../../../docs/feature/036_account_ownership.md)

---

### FT-07: `apply_touch()` skips non-owned or occupied-elsewhere accounts; emits matching trace when `trace::1`

- **Given (case A):** Account `alice` with `aq.is_owned = false, aq.is_occupied_elsewhere = false`. `trace::1` enabled.
- **When (case A):** `apply_touch()` processes the account list containing `alice`.
- **Then (case A):** No subprocess spawned. Stderr contains `[trace] touch  alice  skipped (reason: not owned)`.
- **Given (case B):** Account `alice` with `aq.is_owned = true, aq.is_occupied_elsewhere = true`. `trace::1` enabled.
- **When (case B):** `apply_touch()` processes `alice`.
- **Then (case B):** No subprocess spawned. Stderr contains `[trace] touch  alice  skipped (reason: occupied elsewhere)`.
- **Exit:** Ok(()) with no subprocess; matching trace line emitted per case
- **Source fn:** `ft07_touch_skips_non_owned_with_trace`
- **Note:** Gate condition: `!aq.is_owned || aq.is_occupied_elsewhere`. The trace reason mirrors the specific gate that fired.
- **Source:** [036_account_ownership.md AC-07](../../../docs/feature/036_account_ownership.md)

---

### FT-08: `.account.use` exits 1 when account not owned

- **Given:** Account `alice` has `alice.json` with `"owner": "other@remote"`. Current identity ≠ `"other@remote"`.
- **When:** `clp .account.use name::alice` is executed.
- **Then:** Exits 1. Stderr contains `"ownership violation: this account is owned by other@remote"`. `switch_account()` is NOT called.
- **Exit:** 1
- **Source fn:** `ft08_use_exits_1_when_not_owned`
- **Source:** [036_account_ownership.md AC-08](../../../docs/feature/036_account_ownership.md)

---

### FT-09: `.account.delete` exits 1 when account not owned

- **Given:** Account `alice` has `alice.json` with `"owner": "other@remote"`. Current identity ≠ `"other@remote"`.
- **When:** `clp .account.delete name::alice` is executed.
- **Then:** Exits 1. Stderr contains `"ownership violation: this account is owned by other@remote"`. No files are deleted.
- **Exit:** 1
- **Source fn:** `ft09_delete_exits_1_when_not_owned`
- **Source:** [036_account_ownership.md AC-09](../../../docs/feature/036_account_ownership.md)

---

### FT-10: `.account.relogin` exits 1 when account not owned

- **Given:** Account `alice` has `alice.json` with `"owner": "other@remote"`. Current identity ≠ `"other@remote"`.
- **When:** `clp .account.relogin name::alice` is executed.
- **Then:** Exits 1. Stderr contains `"ownership violation: this account is owned by other@remote"`. The 6-step relogin procedure is NOT started.
- **Exit:** 1
- **Source fn:** `ft10_relogin_exits_1_when_not_owned`
- **Source:** [036_account_ownership.md AC-10](../../../docs/feature/036_account_ownership.md)

---

### FT-11: No `owner` field in `{name}.json` — backward compatible; all gates pass

- **Given:** Account `alice` has `alice.json` with no `owner` key (legacy profile).
- **When:** `is_owned()` is evaluated for `alice` during any gate check.
- **Then:** `is_owned()` returns `true`. G1–G8 all pass. Behavior is byte-identical to pre-feature operation.
- **Exit:** true; full pass-through
- **Source fn:** `ft11_no_owner_field_backward_compat`
- **Source:** [036_account_ownership.md AC-11](../../../docs/feature/036_account_ownership.md)

---

### FT-12: `format::json` includes `"is_owned"` field per account object

- **Given:** Mixed account list: `alice` owned by current identity, `bob` owned by a different identity.
- **When:** `clp .usage format::json` is executed.
- **Then:** JSON output includes `"is_owned": true` for `alice` and `"is_owned": false` for `bob` in their respective account objects.
- **Exit:** json with is_owned per account
- **Source fn:** `ft12_json_output_includes_is_owned`
- **Source:** [036_account_ownership.md AC-12](../../../docs/feature/036_account_ownership.md)

---

### FT-13: `dry::1` does NOT skip G5/G6/G7 ownership check

- **Given:** Account `alice` not owned by current identity.
- **When:** `clp .account.use name::alice dry::1` (same for delete and relogin).
- **Then:** Exits 1 with ownership violation message. The dry-run acknowledgment is NOT printed. Ownership check runs BEFORE dry-run logic.
- **Exit:** 1 with ownership message
- **Source fn:** `ft13_dry_run_does_not_skip_ownership`
- **Source:** [036_account_ownership.md AC-13](../../../docs/feature/036_account_ownership.md)

---

### FT-14: Background `save()` callers preserve existing `owner` field

- **Given:** Account `alice` has `alice.json` with `"owner": "alice@host1"`. `refresh_account_token()` is called for `alice`.
- **When:** `refresh_account_token()` internally calls `save()` with `owner: None`.
- **Then:** `alice.json` retains `"owner": "alice@host1"` unchanged after the save. No other `alice.json` fields are affected.
- **Exit:** Ok(()); owner field preserved
- **Source fn:** `ft14_background_save_preserves_owner`
- **Note:** All `save()` callers pass `owner: None` (preserves existing owner) — both background refresh and interactive `account_save_routine()` are ownership-neutral. The `accounts_routine()` assign path does NOT call `write_owner()`. See Feature 002 FT-09 for the `update_marker` side.
- **Source:** [036_account_ownership.md AC-14](../../../docs/feature/036_account_ownership.md)

---

### FT-15: `owner::` NOT on `.account.save`; `.accounts unclaim::1`/`assign::1` exit 1 with REMOVED_TOGGLE (Feature 064)

- **Given:** Account `alice` exists in credential store.
- **When (case A):** `clp .account.save name::alice owner::0` is executed.
- **Then (case A):** Exits 1 — `owner::` not registered on `.account.save`. `alice.json` unchanged. No file written.
- **When (case B):** `clp .accounts unclaim::1 name::alice` is executed.
- **Then (case B):** Exits 1 — REMOVED_TOGGLE migration message: "REMOVED — use `owner::0 name::X`". No file written. (Feature 064.)
- **When (case C):** `clp .accounts assign::1 name::alice` is executed.
- **Then (case C):** Exits 1 — REMOVED_TOGGLE migration message: "REMOVED — use `assignee::USER@MACHINE name::X`". No file written. (Feature 065: `active::` itself is now also REMOVED_TOGGLE; use `assignee::`.)
- **Exit:** 1 (all cases)
- **Source fn:** `ft15_unclaim_not_on_save_or_assign` (in `tests/cli/account_mutations_test.rs`)
- **Source:** [036_account_ownership.md AC-15](../../../docs/feature/036_account_ownership.md)

---

### FT-16: G8 gate — `.accounts owner::0 name::X` exits 1 when caller is not the owner; exits 0 when unowned (Feature 064)

- **Given (case A):** Account `alice` has `alice.json` with `"owner": "other@remote"`. Current identity ≠ `"other@remote"`.
- **When (case A):** `clp .accounts owner::0 name::alice` is executed. (Formerly `unclaim::1 name::alice` — Feature 064.)
- **Then (case A):** Exits 1. Stderr contains `"ownership violation: this account is owned by other@remote"`. `alice.json` unchanged. Gate evaluates BEFORE any write.
- **Given (case B):** Account `alice` has `alice.json` with `"owner": ""` (already unowned). Current identity = `"testuser@testmachine"`.
- **When (case B):** `clp .accounts owner::0 name::alice` is executed.
- **Then (case B):** Exits 0. Gate passes (unowned). `write_owner()` writes `""` again — idempotent. `alice.json` still has `"owner": ""`.
- **Exit:** 1 (case A), 0 (case B)
- **Source fn:** `ft16_unclaim_g8_gate`
- **Source:** [036_account_ownership.md AC-16](../../../docs/feature/036_account_ownership.md)

---

### FT-17: `.accounts owner::0 name::alice dry::1` prints dry-run line; no file written (Feature 064)

- **Given:** Account `alice` has `alice.json` with `"owner": "testuser@testmachine"`. Current identity = `"testuser@testmachine"` (G8 gate passes).
- **When:** `clp .accounts owner::0 name::alice dry::1` is executed. (Formerly `unclaim::1 name::alice dry::1` — Feature 064.)
- **Then:** Exits 0. stdout contains `[dry-run] would clear owner of alice`. `alice.json` still contains `"owner": "testuser@testmachine"` — unchanged. `write_owner()` is NOT called.
- **Exit:** 0
- **Source fn:** `ft17_unclaim_dry_run`
- **Source:** [036_account_ownership.md AC-17](../../../docs/feature/036_account_ownership.md)

---

### FT-18: `.account.use force::1` bypasses G5 when account is non-owned

- **Given:** Account `alice` has `alice.json` with `"owner": "other@remote"`. Current identity ≠ `"other@remote"`.
- **When:** `clp .account.use name::alice force::1` is executed.
- **Then:** Exits 0. G5 ownership gate is bypassed — no exit 1 with ownership violation. `switch_account()` is called; `~/.claude/.credentials.json` is updated to `alice`'s credentials. Active marker updated.
- **Exit:** 0
- **Source fn:** `ft18_use_force_bypasses_g5`
- **Source:** [036_account_ownership.md AC-18](../../../docs/feature/036_account_ownership.md)

---

### FT-19: `.account.delete force::1` bypasses G6 when account is non-owned

- **Given:** Account `alice` has `alice.json` with `"owner": "other@remote"`. Current identity ≠ `"other@remote"`.
- **When:** `clp .account.delete name::alice force::1` is executed.
- **Then:** Exits 0. G6 ownership gate is bypassed — no exit 1 with ownership violation. Deletion proceeds: `alice.credentials.json` and `alice.json` are removed from the credential store.
- **Exit:** 0
- **Source fn:** `ft19_delete_force_bypasses_g6`
- **Source:** [036_account_ownership.md AC-19](../../../docs/feature/036_account_ownership.md)

---

### FT-20: `.account.relogin force::1` bypasses G7 when account is non-owned

- **Given:** Account `alice` has `alice.json` with `"owner": "other@remote"`. Current identity ≠ `"other@remote"`.
- **When:** `clp .account.relogin name::alice force::1` is executed.
- **Then:** Exits 0. G7 ownership gate is bypassed — no exit 1 with ownership violation. The 6-step relogin procedure is initiated (switch, spawn claude binary, detect credential change, save, restore session).
- **Exit:** 0
- **Source fn:** `ft20_relogin_force_bypasses_g7`
- **Source:** [036_account_ownership.md AC-20](../../../docs/feature/036_account_ownership.md)

---

### FT-21: `force::1 dry::1` bypasses ownership gate but previews without writing

- **Given:** Account `alice` has `alice.json` with `"owner": "other@remote"`. Current identity ≠ `"other@remote"`.
- **When (case A):** `clp .account.use name::alice force::1 dry::1` is executed.
- **Then (case A):** Exits 0. Ownership gate bypassed (no exit 1). stdout contains `[dry-run]` preview line. `~/.claude/.credentials.json` is NOT modified. Active marker NOT updated.
- **When (case B):** `clp .account.delete name::alice force::1 dry::1` is executed.
- **Then (case B):** Exits 0. Ownership gate bypassed. stdout contains `[dry-run]` preview line. No files deleted.
- **When (case C):** `clp .account.relogin name::alice force::1 dry::1` is executed.
- **Then (case C):** Exits 0. Ownership gate bypassed. stdout contains `[dry-run]` preview line. No relogin procedure initiated.
- **Exit:** 0 (all cases)
- **Source fn:** `ft21_force_dry_bypasses_gate_previews`
- **Note:** G8 (unclaim + force + dry) is tested in `37_accounts_usage_param_unification.md` FT-18 (AC-18 there maps to Feature 037 AC-18). Force always runs before dry — gate bypassed, write suppressed.
- **Source:** [036_account_ownership.md AC-21](../../../docs/feature/036_account_ownership.md)

---

### FT-22: `apply_refresh()` emits `reason: not owned` when `aq.is_owned == false` (BUG-295)

- **Given:** `AccountQuota` with `is_owned: false` and `result: Ok(cached_data)` (non-owned cache path, as set by G1 in `fetch.rs`). Env var `TRACE=1` (or `trace::1`) active.
- **When:** `apply_refresh()` is called with this `aq`.
- **Then:** stderr contains `[trace] refresh  <name>  should_retry=false (reason: not owned)`. The reason string is `"not owned"` — derived from the ownership gate check (`!aq.is_owned`), NOT from `aq.result.err()`.
- **Exit:** reason = `"not owned"` (not `"ok"`)
- **Source fn:** `mre_bug295_apply_refresh_trace_reason_not_owned`
- **Note:** Reproduces BUG-295. Before fix: `aq.result = Ok(cached_data)` for non-owned accounts causes `.err()` to return `None`, yielding `reason: ok`. After fix: ownership gate checked first; emits `"not owned"` before consulting `aq.result`. Consistent with AC-07 / FT-07 (`apply_touch` trace pattern).
- **Source:** [036_account_ownership.md AC-22](../../../docs/feature/036_account_ownership.md)

---

### FT-23: G1 non-owned path applies polynomial approximation when history is available (BUG-304)

- **Given:** Account `alice` has `alice.json` with `"owner": "other@remote"`. Quota cache present with `five_hour.utilization = 40.0`. History contains 3 entries within the current 5h window with utilization values 10.0, 25.0, 40.0 at timestamps `t0`, `t1`, `t2` (monotonically increasing toward `now_secs`). Current identity ≠ `"other@remote"` (G1 fires).
- **When:** `fetch_quota_for_list()` processes `alice` (G1 non-owned path executes via `read_cached_quota()`).
- **Then:** `AccountQuota.result` is `Ok(data)` where `data.five_hour.utilization` ≠ `40.0` — the quadratic LS polynomial approximation produces a value greater than the raw cached 40.0 (trend is increasing). `cached: true`. `is_owned: false`. The approximated utilization is closer to the current position on the polynomial curve than the stale raw value.
- **Exit:** Ok(()); `five_hour.utilization` ≠ raw `40.0`
- **Source fn:** `ft23_g1_non_owned_applies_approximation`
- **Note:** Reproduces BUG-304 integration path. Before fix: G1 path called `read_quota_cache()` directly and reconstructed `OauthUsageData` with raw stale values — no approximation applied. After fix: G1 calls `read_cached_quota()` which applies Feature 040 polynomial approximation when `history.len() >= 2`. This test verifies the G1 approximation correction that was previously missing (the G1 violation site). Feature 040 FT-04 covers the approximation algorithm itself; this case covers G1 as a new consumer of that algorithm.
- **Source:** [036_account_ownership.md AC-04](../../../docs/feature/036_account_ownership.md)

---

### FT-24: Owned + occupied-elsewhere + non-current account skips HTTP; calls `approximate_quota()`; emits occupied-elsewhere trace (BUG-305)

- **Given:** Account `alice` has `alice.json` with `"owner": "testuser@testmachine"` (current identity = `"testuser@testmachine"`, so `is_owned = true`). A `_active_{remote_host}_{remote_user}` marker file exists naming `alice` (so `is_occupied_elsewhere = true`). `alice` is NOT the live session on this machine (`is_current = false`). Quota cache present with valid data. `trace::1` enabled.
- **When:** `fetch_quota_for_list()` processes `alice` (G1 passes — owned; G1b fires — occupied elsewhere and not current).
- **Then:** No `read_token()` call for credential reading; no HTTP GET to `fetch_oauth_usage`. `approximate_quota()` is called and returns `AccountQuota` with `cached: true` and quota values from cache/approximation. Stderr contains `[trace] fetch  alice  skipped (reason: occupied elsewhere)`. `is_owned: true`, `is_occupied_elsewhere: true`.
- **Exit:** Ok(cached/approximated data); trace line emitted; no HTTP call
- **Source fn:** `mre_bug305_fetch_skips_occupied_elsewhere_with_trace`
- **Note:** Reproduces BUG-305. Before fix: `occupied_elsewhere` computed at line 74 but only stamped on result at line 337 — no gate fires; full HTTP pipeline executes. After fix: G1b gate fires for `is_owned && is_occupied_elsewhere && !is_current`; `approximate_quota()` returns cached/approximated data without HTTP. Sister test to BUG-302 (touch) and BUG-303 (refresh) MRE tests.
- **Source:** [036_account_ownership.md AC-23](../../../docs/feature/036_account_ownership.md)

---

### FT-25: `reason_label(aq, now_secs)` returns `"occupied elsewhere"` for owned + non-cached + occupied-elsewhere account (BUG-306)

- **Given:** `AccountQuota { is_owned: true, cached: false, is_occupied_elsewhere: true, result: Ok(Default::default()) }` — owned account, no cache hit, active on another machine, fetch succeeded.
- **When:** `reason_label(&aq, 0)` is called (extracted function in `refresh.rs`).
- **Then:** Returns `"occupied elsewhere"`. The function branches: `!is_owned` → false; `cached` → false; `is_occupied_elsewhere` → true → `"occupied elsewhere"`. The cached sub-branch (expired vs valid) and the `else` branch (`aq.result.err().map_or("ok", ...)`) are NOT reached.
- **Exit:** `"occupied elsewhere"` (not `"ok"`)
- **Source fn:** `mre_bug306_refresh_trace_reason_occupied_elsewhere`
- **Note:** Reproduces BUG-306. Before fix: no `is_occupied_elsewhere` branch in the inline reason block — else fires → `"ok"`. After fix: `reason_label(aq, now_secs)` is a dedicated function with the `is_occupied_elsewhere` branch inserted after `cached`. The function is directly testable without running the full `apply_refresh()` pipeline. Third instance of predicate-gate vs reason-branch omission (after BUG-295, BUG-298). The extracted function enforces the predicate–reason 1:1 contract: each branch in `reason_label()` must mirror a gate in `should_refresh()`.
- **Source:** [036_account_ownership.md AC-24](../../../docs/feature/036_account_ownership.md)
