# FT — Feature 036: Account Ownership

### Scope

- **Purpose**: Test cases for account ownership enforcement — owner stamp via `.account.save`, `.account.unclaim` command with G8 gate, eight enforcement gates (G1–G8), backward compatibility, and `is_owned` JSON field.
- **Source**: `docs/feature/036_account_ownership.md`
- **Covers**: AC-01 through AC-17

### Test Cases

| FT | AC | Scenario | Source fn |
|----|----|----------|-----------|
| FT-01 | AC-01 | `.account.save` stamps `current_identity()` as `owner` in `{name}.json` | `ft01_save_stamps_owner` |
| FT-02 | AC-02 | `.account.unclaim name::alice` exits 0; writes `owner: ""`; `write_owner()` called directly — no credential re-save | `ft02_unclaim_clears_owner` |
| FT-03 | AC-03 | No `owner::` CLI param; `unclaim::` NOT on `.account.save`; `.account.unclaim` lists `name::`, `dry::`, `trace::`; `.account.assign` does NOT list `unclaim` | `ft03_unclaim_param_placement` |
| FT-04 | AC-04 | Non-owned account: `fetch_quota_for_list` skips token read + HTTP; reads cache; `aq.is_owned = false` | `ft04_non_owned_uses_cache_not_http` |
| FT-05 | AC-05 | Non-owned account with cache: usage row renders with `~` prefix and age indicator; without cache: dashes | `ft05_non_owned_display_tilde_or_dashes` |
| FT-06 | AC-06 | `should_refresh()` returns `false` when `aq.is_owned == false` | `ft06_should_refresh_false_when_not_owned` |
| FT-07 | AC-07 | `apply_touch()` skips non-owned accounts; emits trace line with `"not owned"` when `trace::1` | `ft07_touch_skips_non_owned_with_trace` |
| FT-08 | AC-08 | `.account.use` with non-owned account exits 1 with ownership violation message | `ft08_use_exits_1_when_not_owned` |
| FT-09 | AC-09 | `.account.delete` with non-owned account exits 1 with ownership violation message | `ft09_delete_exits_1_when_not_owned` |
| FT-10 | AC-10 | `.account.relogin` with non-owned account exits 1 with ownership violation message | `ft10_relogin_exits_1_when_not_owned` |
| FT-11 | AC-11 | Account without `owner` field passes all G1–G8 gates; behavior identical to pre-feature | `ft11_no_owner_field_backward_compat` |
| FT-12 | AC-12 | `format::json` includes `"is_owned": true` or `"is_owned": false` per account | `ft12_json_output_includes_is_owned` |
| FT-13 | AC-13 | `dry::1` on `.account.use`/`.account.delete`/`.account.relogin` does NOT skip ownership check | `ft13_dry_run_does_not_skip_ownership` |
| FT-14 | AC-14 | Background `save()` callers pass `owner: None` — existing `owner` preserved; `account_assign_routine()` does NOT call `write_owner()` | `ft14_background_save_preserves_owner` |
| FT-15 | AC-15 | `.account.save unclaim::1` exits 1 — `unclaim::` removed from `.account.save`; `.account.assign unclaim::1` exits 1 — unknown parameter | `ft15_unclaim_not_on_save_or_assign` |
| FT-16 | AC-16 | `.account.unclaim` with account owned by different identity exits 1 with ownership violation; already-unowned account exits 0 | `ft16_unclaim_g8_gate` |
| FT-17 | AC-17 | `.account.unclaim name::alice dry::1` prints `[dry-run]` line; `alice.json` unchanged | `ft17_unclaim_dry_run` |

### Notes

- FT-01 is an integration test — calls `clp .account.save name::alice` and asserts `owner` is stamped as `current_identity()` (`account_save_routine()` passes `Some(&owner_val)` to `save()`).
- FT-02 is an integration test — calls `clp .account.unclaim name::alice` and asserts exit 0, `owner: ""` written, and credential file NOT re-saved (`alice.credentials.json` mtime unchanged).
- FT-03 is structural with three cases: (a) `.account.save` help does NOT list `unclaim`; (b) `.account.unclaim` help DOES list `name::`, `dry::`, `trace::`; (c) `.account.assign` help does NOT list `unclaim`.
- FT-04 is a unit test in `src/usage/fetch.rs` — mock-free: verify no `read_token()` call path was exercised and cache JSON is the returned value.
- FT-05 is a render test in `src/usage/render_tests.rs` — uses `AccountQuota { is_owned: false, cached: true, ... }` and asserts `~` prefix; also tests `cached: false, is_owned: false` giving dashes.
- FT-06 is a unit test in `src/usage/refresh_predicate.rs` `#[cfg(test)]` module.
- FT-07 is a unit test in `src/usage/touch_tests.rs` using `gag::BufferRedirect::stderr()` for trace capture.
- FT-08 through FT-10 are integration tests via `verb/test` — verify exit code 1 and message text.
- FT-11 is a unit test in `claude_profile_core/tests/account_test.rs` — `{name}.json` with no `owner` key reads as `is_owned = true`.
- FT-12 is a render test in `src/usage/render_tests.rs` — verifies `"is_owned": true`/`"is_owned": false` in JSON object.
- FT-13 exercises G5/G6/G7 with `dry::1` flag set — ownership guard runs first; exit 1 regardless.
- FT-14 is a unit test in `claude_profile_core/tests/account_test.rs` — background `save()` with `owner: None` (e.g. `refresh_account_token`) on an account with `owner: "alice@host"` leaves `owner: "alice@host"` in `{name}.json`. Background callers pass `owner: None` (preserves existing); interactive `account_save_routine()` passes `Some(&owner_val)` (stamps owner). `account_assign_routine()` does NOT call `write_owner()`.

---

### FT-01: `.account.save` stamps `current_identity()` as `owner`

- **Given:** Account `alice` exists in credential store. `current_identity()` resolves to `"testuser@testmachine"`.
- **When:** `clp .account.save name::alice` — `account_save_routine()` passes `Some(&owner_val)` to `save()` where `owner_val = current_identity()`.
- **Then:** `alice.json` contains `"owner": "testuser@testmachine"`. Credentials re-saved. Owner field is always written on interactive save.
- **Exit:** 0
- **Source fn:** `ft01_save_stamps_owner`
- **Source:** [036_account_ownership.md AC-01](../../../docs/feature/036_account_ownership.md)

---

### FT-02: `.account.unclaim` exits 0; writes `owner: ""`; no credential re-save

- **Given:** Account `alice` exists in credential store. `alice.json` has `"owner": "testuser@testmachine"`. `current_identity()` resolves to `"testuser@testmachine"`. Record mtime of `alice.credentials.json`.
- **When:** `clp .account.unclaim name::alice` — `account_unclaim_routine()` calls `write_owner("alice", store, "")` directly.
- **Then:** Exits 0. `alice.json` contains `"owner": ""`. All G1–G8 enforcement gates disabled for `alice`. mtime of `alice.credentials.json` is unchanged (no credential re-save).
- **Exit:** 0
- **Source fn:** `ft02_unclaim_clears_owner`
- **Source:** [036_account_ownership.md AC-02](../../../docs/feature/036_account_ownership.md)

---

### FT-03: `unclaim::` NOT on `.account.save`; `.account.unclaim` has dedicated params; `.account.assign` does NOT list `unclaim`

- **Given:** Any environment.
- **When (case A):** `.account.save.help` output inspected.
- **Then (case A):** `.account.save` help does NOT list `unclaim`; `owner::` does NOT appear on any command.
- **When (case B):** `.account.unclaim.help` output inspected (or command `--help` equivalent).
- **Then (case B):** `.account.unclaim` lists `name::`, `dry::`, `trace::`; no other parameters.
- **When (case C):** `.account.assign.help` output inspected for `unclaim`.
- **Then (case C):** `.account.assign` help does NOT list `unclaim`.
- **Exit:** structural assertion (cases A + B + C)
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

### FT-06: `should_refresh()` returns `false` when `is_owned == false`

- **Given:** `AccountQuota { is_owned: false, ... }` — regardless of token state or cache age.
- **When:** `should_refresh(&aq)` is called.
- **Then:** Returns `false`. No refresh is initiated.
- **Exit:** false
- **Source fn:** `ft06_should_refresh_false_when_not_owned`
- **Source:** [036_account_ownership.md AC-06](../../../docs/feature/036_account_ownership.md)

---

### FT-07: `apply_touch()` skips non-owned account; emits trace when `trace::1`

- **Given:** Account `alice` with `aq.is_owned = false`. `trace::1` enabled.
- **When:** `apply_touch()` processes the account list containing `alice`.
- **Then:** No subprocess is spawned for `alice`. Stderr contains `[trace] touch  alice  skipped (reason: not owned)`.
- **Exit:** Ok(()) with no subprocess; trace line emitted
- **Source fn:** `ft07_touch_skips_non_owned_with_trace`
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
- **Note:** Background `save()` callers pass `owner: None` (preserves existing owner); interactive `account_save_routine()` passes `Some(&owner_val)`. `account_assign_routine()` does NOT call `write_owner()`. See Feature 002 FT-09 for the `update_marker` side.
- **Source:** [036_account_ownership.md AC-14](../../../docs/feature/036_account_ownership.md)

---

### FT-15: `.account.save unclaim::1` exits 1; `.account.assign unclaim::1` exits 1 — `unclaim::` removed from both

- **Given:** Account `alice` exists in credential store.
- **When (case A):** `clp .account.save name::alice unclaim::1` is executed.
- **Then (case A):** Exits 1 — `unclaim::` is not registered on `.account.save`. `alice.json` unchanged. No file written.
- **When (case B):** `clp .account.assign name::alice unclaim::1` is executed.
- **Then (case B):** Exits 1 — `unclaim::` is not registered on `.account.assign`. `alice.json` unchanged. Marker NOT written.
- **Exit:** 1 (both cases)
- **Source fn:** `ft15_unclaim_not_on_save_or_assign` (cases in `tests/cli/account_mutations_test.rs` and `tests/cli/account_assign_test.rs`)
- **Source:** [036_account_ownership.md AC-15](../../../docs/feature/036_account_ownership.md)

---

### FT-16: G8 gate — `.account.unclaim` exits 1 when caller is not the owner; exits 0 when unowned

- **Given (case A):** Account `alice` has `alice.json` with `"owner": "other@remote"`. Current identity ≠ `"other@remote"`.
- **When (case A):** `clp .account.unclaim name::alice` is executed.
- **Then (case A):** Exits 1. Stderr contains `"ownership violation: this account is owned by other@remote"`. `alice.json` unchanged. Gate evaluates BEFORE any write.
- **Given (case B):** Account `alice` has `alice.json` with `"owner": ""` (already unowned). Current identity = `"testuser@testmachine"`.
- **When (case B):** `clp .account.unclaim name::alice` is executed.
- **Then (case B):** Exits 0. Gate passes (unowned). `write_owner()` writes `""` again — idempotent. `alice.json` still has `"owner": ""`.
- **Exit:** 1 (case A), 0 (case B)
- **Source fn:** `ft16_unclaim_g8_gate`
- **Source:** [036_account_ownership.md AC-16](../../../docs/feature/036_account_ownership.md)

---

### FT-17: `.account.unclaim dry::1` prints dry-run line; no file written

- **Given:** Account `alice` has `alice.json` with `"owner": "testuser@testmachine"`. Current identity = `"testuser@testmachine"` (G8 gate passes).
- **When:** `clp .account.unclaim name::alice dry::1` is executed.
- **Then:** Exits 0. stdout contains `[dry-run] would unclaim alice`. `alice.json` still contains `"owner": "testuser@testmachine"` — unchanged. `write_owner()` is NOT called.
- **Exit:** 0
- **Source fn:** `ft17_unclaim_dry_run`
- **Source:** [036_account_ownership.md AC-17](../../../docs/feature/036_account_ownership.md)
