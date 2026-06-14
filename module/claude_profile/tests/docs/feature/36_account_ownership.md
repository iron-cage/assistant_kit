# FT — Feature 036: Account Ownership

### Scope

- **Purpose**: Test cases for account ownership enforcement — owner auto-capture, `unclaim::1`, seven enforcement gates (G1–G7), backward compatibility, and `is_owned` JSON field.
- **Source**: `docs/feature/036_account_ownership.md`
- **Covers**: AC-01 through AC-14

### Test Cases

| FT | AC | Scenario | Source fn |
|----|----|----------|-----------|
| FT-01 | AC-01 | `.account.save` writes `current_identity()` as `owner` in `{name}.json`; re-save from different identity overwrites | `ft01_save_captures_owner` |
| FT-02 | AC-02 | `unclaim::1` writes `owner: ""` to `{name}.json`; all G1–G7 gates pass after | `ft02_unclaim_clears_owner` |
| FT-03 | AC-03 | No `owner::` CLI parameter exists — ownership only set by `.account.save` or cleared by `unclaim::1` | structural (no registration path) |
| FT-04 | AC-04 | Non-owned account: `fetch_quota_for_list` skips token read + HTTP; reads cache; `aq.is_owned = false` | `ft04_non_owned_uses_cache_not_http` |
| FT-05 | AC-05 | Non-owned account with cache: usage row renders with `~` prefix and age indicator; without cache: dashes | `ft05_non_owned_display_tilde_or_dashes` |
| FT-06 | AC-06 | `should_refresh()` returns `false` when `aq.is_owned == false` | `ft06_should_refresh_false_when_not_owned` |
| FT-07 | AC-07 | `apply_touch()` skips non-owned accounts; emits trace line with `"not owned"` when `trace::1` | `ft07_touch_skips_non_owned_with_trace` |
| FT-08 | AC-08 | `.account.use` with non-owned account exits 1 with ownership violation message | `ft08_use_exits_1_when_not_owned` |
| FT-09 | AC-09 | `.account.delete` with non-owned account exits 1 with ownership violation message | `ft09_delete_exits_1_when_not_owned` |
| FT-10 | AC-10 | `.account.relogin` with non-owned account exits 1 with ownership violation message | `ft10_relogin_exits_1_when_not_owned` |
| FT-11 | AC-11 | Account without `owner` field passes all G1–G7 gates; behavior identical to pre-feature | `ft11_no_owner_field_backward_compat` |
| FT-12 | AC-12 | `format::json` includes `"is_owned": true` or `"is_owned": false` per account | `ft12_json_output_includes_is_owned` |
| FT-13 | AC-13 | `dry::1` on `.account.use`/`.account.delete`/`.account.relogin` does NOT skip ownership check | `ft13_dry_run_does_not_skip_ownership` |
| FT-14 | AC-14 | `save()` called from `refresh_account_token()` / touch path preserves existing `owner` field unchanged | `ft14_background_save_preserves_owner` |

### Notes

- FT-01 and FT-02 are unit tests in `claude_profile_core/tests/account_test.rs` — test `save()` writes the `owner` field and `unclaim::1` writes `""`.
- FT-03 is structural: `src/lib.rs` registers `unclaim::` but no `owner::` parameter; verified by negative grep `assert!(!registered_params.contains("owner::"))`.
- FT-04 is a unit test in `src/usage/fetch.rs` — mock-free: verify no `read_token()` call path was exercised and cache JSON is the returned value.
- FT-05 is a render test in `src/usage/render_tests.rs` — uses `AccountQuota { is_owned: false, cached: true, ... }` and asserts `~` prefix; also tests `cached: false, is_owned: false` giving dashes.
- FT-06 is a unit test in `src/usage/refresh_predicate.rs` `#[cfg(test)]` module.
- FT-07 is a unit test in `src/usage/touch.rs` `#[cfg(test)]` module using `gag::BufferRedirect::stderr()` for trace capture.
- FT-08 through FT-10 are integration tests via `verb/test` — verify exit code 1 and message text.
- FT-11 is a unit test in `claude_profile_core/tests/account_test.rs` — `{name}.json` with no `owner` key reads as `is_owned = true`.
- FT-12 is a render test in `src/usage/render_tests.rs` — verifies `"is_owned": true`/`"is_owned": false` in JSON object.
- FT-13 exercises G5/G6/G7 with `dry::1` flag set — ownership guard runs first; exit 1 regardless.
- FT-14 is a unit test in `claude_profile_core/tests/account_test.rs` — save with `owner: None` on an account with `owner: "alice@host"` leaves `owner: "alice@host"` in `{name}.json`.

---

### FT-01: `.account.save` auto-captures `current_identity()` as `owner`

- **Given:** Account `alice` has no saved profile. `current_identity()` resolves to `"user@host1"`.
- **When:** `.account.save name::alice` is called (owner: Some("user@host1") passed to `save()`).
- **Then:** `alice.json` contains `"owner": "user@host1"`. A second save from `"user@host2"` overwrites `owner` to `"user@host2"`.
- **Exit:** Ok(())
- **Source fn:** `ft01_save_captures_owner`
- **Source:** [036_account_ownership.md AC-01](../../../docs/feature/036_account_ownership.md)

---

### FT-02: `unclaim::1` clears ownership; all gates pass

- **Given:** Account `alice` has `alice.json` with `"owner": "user@host1"`. `is_owned` is `false` on the current machine.
- **When:** `.account.save name::alice unclaim::1` is called (owner: Some("") passed to `save()`).
- **Then:** `alice.json` contains `"owner": ""`. `is_owned("", current_identity())` returns `true`. All other `alice.json` fields are preserved via read-merge.
- **Exit:** Ok(())
- **Source fn:** `ft02_unclaim_clears_owner`
- **Source:** [036_account_ownership.md AC-02](../../../docs/feature/036_account_ownership.md)

---

### FT-03: No `owner::` CLI parameter exists

- **Given:** Any environment.
- **When:** The registered parameter list for `.account.save` is inspected.
- **Then:** `"owner"` does not appear as a registered parameter key. `"unclaim"` does appear.
- **Exit:** structural assertion
- **Source fn:** structural (no registration path)
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
- **Then:** `is_owned()` returns `true`. G1–G7 all pass. Behavior is byte-identical to pre-feature operation.
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
- **Source:** [036_account_ownership.md AC-14](../../../docs/feature/036_account_ownership.md)
