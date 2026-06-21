# Feature: Account Ownership

### Scope

- **Purpose**: Allow a saved account to declare which host/user identity owns it, so that non-owner machines can safely read cached quota but never touch, refresh, switch to, delete, or re-authenticate the account's credentials.
- **Responsibility**: Documents the `owner` field in `{name}.json`, identity resolution via `current_identity()`, the `write_owner()` API for explicit ownership assignment and release, nine enforcement gates (G1, G1b, G2–G8), and the `is_owned` flag propagated to `AccountQuota` and `format::json` output.
- **In Scope**: `owner` field in `{name}.json`; `current_identity()` resolution (`$USER@<hostname>` via `resolve_hostname()`); `write_owner(name, store, identity)` — explicit ownership assignment; `write_owner(name, store, "")` — ownership release (`.accounts unclaim::1` path); `.account.save` is ownership-neutral — `account_save_routine()` passes `owner: None` to `save()`, does not call `write_owner()`; `.accounts assign::1` is also ownership-neutral — does not call `write_owner()`; background refresh callers also pass `owner: None` to `save()` (all callers preserve existing); `is_owned(account)` predicate; G1 quota-fetch gate (bypass token read + HTTP for non-owned, use cache directly via `read_cached_quota()`); G1b quota-fetch gate (bypass token read + HTTP for owned accounts active on another machine — `is_occupied_elsewhere && !is_current` — use `approximate_quota()` instead of live fetch; emit `[trace] fetch  <name>  skipped (reason: occupied elsewhere)` when `trace::1`); G2 `should_refresh()` early-false for non-owned or occupied-elsewhere (`!is_owned || is_occupied_elsewhere`); G3 `apply_refresh` loop skip with matching trace reason; G4 `apply_touch` skip for non-owned or occupied-elsewhere (`!is_owned || is_occupied_elsewhere`) with matching trace reason; G5 `.account.use` ownership guard; G6 `.account.delete` ownership guard; G7 `.account.relogin` ownership guard; G8 `.accounts unclaim::1` ownership guard; `AccountQuota.is_owned` flag; `is_owned` field in `format::json` output.
- **Out of Scope**: Credential file access control (enforcement is logical, not filesystem-level); shared-store cross-machine sync mechanism (out of scope for this project); `host::` display label (remains user-settable; see [029_account_host_metadata.md](029_account_host_metadata.md)). ~~Owner display column~~ — moved to Feature 037: Owner column now visible by default on both `.accounts` and `.usage`.

> **CLI surface migration (Feature 037 — shipped):** `.account.unclaim` was absorbed into `.accounts` as `unclaim::1` parameter; the standalone command is **fully removed** (deregistered from the command registry — produces generic 'unknown command' error). All acceptance criteria and G8 enforcement below apply via `clp .accounts unclaim::1 name::X`. Batch unclaim (no `name::`, applies to filtered set) is available. See [037_accounts_usage_param_unification.md](037_accounts_usage_param_unification.md).

### Design

**Ownership model:** Each saved account optionally carries an `owner` field in `{name}.json`. An empty or absent `owner` means no enforcement — the account behaves exactly as before this feature (backward compatible). A non-empty `owner` identifies the single `USER@MACHINE` identity that may operate on the account's credentials. All other identities may only read the quota cache.

**Identity resolution:** `current_identity() = "$USER@<hostname>"` where hostname is resolved via `resolve_hostname()` — the same fallback chain as `active_marker_filename()` in Feature 025: `$HOSTNAME` env → `/etc/hostname` file → `"local"`. The components are NOT sanitized (unlike `active_marker_filename()`): `$USER` and hostname are used as-is. This makes `current_identity()` round-trip with the auto-captured `owner` value without requiring sanitization at comparison time.

**Ownership predicate:**
```
is_owned(account) = account.owner.is_empty() || account.owner == current_identity()
```
An account is "owned by this machine" when: (a) owner is empty or absent (no enforcement — all identities pass), or (b) the stored owner string exactly matches `current_identity()`. Any other value means "owned by someone else" — enforcement gates apply.

**Owner capture — explicit only:** Ownership is never stamped implicitly. `account_save_routine()` passes `owner: None` to `save()` — `.account.save` does not write to the `owner` field. All write paths to the `owner` field are explicit: `write_owner(name, store, identity)` for assignment, `write_owner(name, store, "")` for release. The only CLI-exposed release path is `.accounts unclaim::1` (calls `write_owner(name, store, "")` directly). **Both `.account.save` and `.accounts assign::1` are ownership-neutral — neither calls `write_owner()` or modifies the `owner` field.**

**`save()` owner handling:** `save()` accepts an `owner: Option<&str>` parameter. When `Some(s)`, the value is written to `{name}.json`. When `None`, the existing `owner` field in `{name}.json` is preserved — read-merge semantics identical to `host` and `role`. All callers — `account_save_routine()`, background refresh via `refresh_account_token()` — pass `owner: None`, preserving any existing owner field. Ownership can only be set by calling `write_owner()` directly (no CLI-exposed set path; release via `.accounts unclaim::1`).

**Enforcement gates (G1, G1b, G2–G8):**

| Gate | Location | Condition | Action | `force::1` bypass |
|------|----------|-----------|--------|-------------------|
| G1 | `fetch_quota_for_list()` in `fetch.rs` | `!is_owned` | Skip `read_token()` and HTTP fetch; call `read_cached_quota()` — reads cache and applies Feature 040 polynomial approximation when history is available; set `aq.is_owned = false` | No bypass — read-side gate |
| G1b | `fetch_quota_for_list()` in `fetch.rs` | `is_owned && is_occupied_elsewhere && !is_current` (Fix BUG-305) | Skip `read_token()` and HTTP fetch; call `approximate_quota()` — returns cached or polynomial-approximated data; emit `[trace] fetch  <name>  skipped (reason: occupied elsewhere)` when `trace::1` | No bypass — read-side gate |
| G2 | `should_refresh()` in `refresh_predicate.rs` | `!is_owned \|\| is_occupied_elsewhere` (Fix BUG-303) | Return `false` early — no refresh attempt | No bypass — read-side gate |
| G3 | `apply_refresh()` loop in `refresh.rs` | via G2 (Fix BUG-295, BUG-298, BUG-306) | Skip account; emit `[trace] refresh  <name>  should_retry=false (reason: <reason>)` where reason mirrors the gate that fired (`"not owned"` / `"cached-expired"` / `"occupied elsewhere"` / result-derived) | No bypass — read-side gate |
| G4 | `apply_touch()` in `touch.rs` | `!is_owned \|\| is_occupied_elsewhere` (Fix BUG-302) | Skip account; emit `[trace] touch  <name>  skipped (reason: not owned)` or `skipped (reason: occupied elsewhere)` when `trace::1` | No bypass — read-side gate |
| G5 | `account_use_routine()` in `account_ops.rs` | `!is_owned` | Exit 1 with `"ownership violation: this account is owned by {owner}"` | Yes — `force::1` skips gate; proceeds to `switch_account()` |
| G6 | `account_delete_routine()` in `account_ops.rs` | `!is_owned` | Exit 1 with `"ownership violation: this account is owned by {owner}"` | Yes — `force::1` skips gate; proceeds to deletion |
| G7 | `account_relogin_routine()` in `account_relogin.rs` | `!is_owned` | Exit 1 with `"ownership violation: this account is owned by {owner}"` | Yes — `force::1` skips gate; proceeds to 6-step relogin |
| G8 | `accounts_routine()` (`unclaim::1` path) in `commands/accounts.rs` (Feature 037) | `!is_owned` | Exit 1 with `"ownership violation: this account is owned by {owner}"` | Yes — `force::1` skips gate; proceeds to `write_owner(name, store, "")` |

**G1 detail (cache-as-primary for non-owned accounts):** When `is_owned = false`, `fetch_quota_for_list()` skips `read_token()` (avoids touching the credential file) and skips the HTTP call to `fetch_oauth_usage`. Instead, it calls `read_cached_quota(credential_store, name, now_secs)` — a centralized function that reads the `{name}.json` cache via `read_quota_cache()`, then reads `cache.history[]` and applies Feature 040 polynomial approximation independently for each period when 2+ history entries are available. When history is sufficient, the returned utilization values are approximated (more accurate than raw stale cache); when history is absent, raw cached values are returned. The row is rendered with `~` prefix and age indicator identical to the cache-fallback path. If no cache exists, the row shows `—` for all quota columns. `aq.is_owned = false` is set in all cases. `aq.cached = true` is set when cache data is used.

**G1b detail (cache-as-primary for occupied-elsewhere owned accounts — Fix BUG-305):** When `is_owned = true` and `is_occupied_elsewhere = true` and `!is_current`, `fetch_quota_for_list()` skips `read_token()` and the HTTP call and instead calls `approximate_quota(acct, credential_store, is_current, is_occupied_elsewhere, now_secs)` — the same helper used by the solo gate (Feature 061). `approximate_quota()` reads the `{name}.json` cache and applies Feature 040 polynomial approximation when `cache.history[]` has 2+ entries; falls back to raw cached values when history is absent; returns an `AccountQuota` with `cached=true` and `cache_age_secs` set when cache exists. When `trace::1`, emits `[trace] fetch  <name>  skipped (reason: occupied elsewhere)` before calling `approximate_quota()`. G1b fires AFTER G1 (owned accounts only) and AFTER `is_current` is resolved. The `is_current` account is never skipped by G1b (it is the live session on this machine and must have accurate data). This gate mirrors the occupancy guards in G2 (refresh predicate) and G4 (touch), completing the per-phase occupancy coverage for all three pipeline phases.

**`AccountQuota.is_owned` field:** New `bool` field on `AccountQuota`. Set to `true` when `is_owned(account)` at fetch time; `false` otherwise. Propagated to `format::json` output as `"is_owned": bool`. Not used for display column — ownership is operational, not display.

**Backward compatibility:** Any `{name}.json` without an `owner` field (or with `owner: ""`) behaves identically to the pre-feature behavior. No migration needed. All existing accounts are effectively unowned after the feature ships.

**No `host::` collision:** The `host::` parameter (Feature 029) is a user-customizable display label (e.g., `"workstation"`, `"laptop"`). It is independent of `owner`. The `owner` field is managed only via `write_owner()` (no direct CLI param); the only exposed path is release via `.accounts unclaim::1`. Both fields coexist in `{name}.json` without ambiguity.

**`force::` bypass:** All commands enforcing G5–G8 accept a `force::1` parameter that bypasses the ownership check. When `force::1` is present, the gate is skipped regardless of whether `current_identity() == owner`. `force::1` does not affect G1–G4 (read-side gates — touch and refresh are intentionally suppressed for non-owned accounts even with force). `force::1` always runs BEFORE `dry::1` evaluation — when both are set, the ownership check is bypassed but the mutation is still previewed without file writes.

**Dry-run interaction:** G5, G6, G7, G8 check ownership BEFORE evaluating `dry::1` — ownership violation exits 1 even in dry-run mode (unless `force::1` is also set). This prevents information leakage (a dry-run would still reveal that a switch is possible, which is incorrect if the caller isn't the owner).

**Trace interaction:** Trace messages mirror the gate that fired — reason string is never derived from `aq.result` when a non-result gate caused the skip.

- **G1 (not owned):** `[trace] fetch  <name>  skipped (reason: not owned)` — Fix(BUG-295): reason from ownership gate, not `aq.result`.
- **G1b (occupied elsewhere):** `[trace] fetch  <name>  skipped (reason: occupied elsewhere)` — Fix(BUG-305): occupancy gate, not `aq.result`.
- **G3 (via G2):** `[trace] refresh  <name>  should_retry=false (reason: <reason>)` where reason is derived in a dedicated `reason_label(aq)` function that mirrors all G2 gate branches — `"not owned"` (Fix BUG-295), `"cached-expired"` (Fix BUG-298), `"occupied elsewhere"` (Fix BUG-306), or `aq.result.err().map_or("ok", ...)` when no gate fired.
- **G4 (not owned or occupied):** `[trace] touch  <name>  skipped (reason: not owned)` when `!is_owned`; `[trace] touch  <name>  skipped (reason: occupied elsewhere)` when `is_occupied_elsewhere`.

**Predicate–reason contract:** Every early-return gate in `should_refresh()` (`refresh_predicate.rs`) must have a corresponding branch in `reason_label(aq)` (`refresh.rs`). The function is extracted specifically to enforce this contract and make the 1:1 mapping testable directly. When a new predicate gate is added, the matching reason branch is a mandatory co-change.

### Acceptance Criteria

- **AC-01**: `clp .account.save name::X` does NOT modify the `owner` field in `{name}.json` — `account_save_routine()` passes `owner: None` to `save()`, preserving any existing value via read-merge. `clp .accounts assign::1 name::X` also does NOT modify `owner` (marker-only — no `write_owner()` call). Both paths are ownership-neutral.
- **AC-02**: `clp .accounts unclaim::1 name::X` exits 0 and writes `owner: ""` to `{name}.json` via `write_owner(name, store, "")`. Credentials are NOT touched (no `save()` call, no credential file read or write). After unclaim, all G1–G8 enforcement gates pass (account behaves as unowned — no enforcement).
- **AC-03**: There is no `owner::` CLI parameter — ownership is never user-specified as a direct string value. `unclaim::` is not registered on `.account.save` (exits 1 on unknown parameter). Ownership release is via `.accounts unclaim::1`. `.accounts assign::1` is marker-only — does NOT call `write_owner()`, does NOT touch the `owner` field in `{name}.json`.
- **AC-04**: For an account where `is_owned = false`, `fetch_quota_for_list()` does NOT read the credential file and does NOT call `fetch_oauth_usage`. It calls `read_cached_quota(credential_store, name, now_secs)` which reads the `{name}.json` cache and applies Feature 040 polynomial approximation when `cache.history[]` has 2+ entries. Returns approximated quota when history is sufficient; returns raw cached values when history is absent; returns dashes when no cache exists. `aq.is_owned = false` is always set.
- **AC-05**: Non-owned accounts in `.usage` output show quota values with `~` prefix and `(Xm ago)` age indicator when cache is present — identical to Feature 033 cache-fallback display. When no cache exists, columns show `—`.
- **AC-06**: `should_refresh()` returns `false` when `aq.is_owned == false` OR when `aq.is_occupied_elsewhere == true`. No refresh subprocess is spawned for non-owned or occupied-elsewhere accounts regardless of token state. Gate condition: `!aq.is_owned || aq.is_occupied_elsewhere`.
- **AC-07**: `apply_touch()` skips accounts where `aq.is_owned == false` OR where `aq.is_occupied_elsewhere == true`. When `trace::1`, emits `[trace] touch  <name>  skipped (reason: not owned)` when `!is_owned`; emits `[trace] touch  <name>  skipped (reason: occupied elsewhere)` when `is_occupied_elsewhere`. Gate condition: `!aq.is_owned || aq.is_occupied_elsewhere`.
- **AC-08**: `clp .account.use name::alice@other.com` when `alice@other.com`'s owner ≠ `current_identity()` exits 1 with message `"ownership violation: this account is owned by {owner}"`. Ownership check runs before `switch_account()`.
- **AC-09**: `clp .account.delete name::alice@other.com` when `alice@other.com`'s owner ≠ `current_identity()` exits 1 with message `"ownership violation: this account is owned by {owner}"`. Ownership check runs before any deletion.
- **AC-10**: `clp .account.relogin name::alice@other.com` when `alice@other.com`'s owner ≠ `current_identity()` exits 1 with message `"ownership violation: this account is owned by {owner}"`. Ownership check runs before step 1 of the 6-step relogin procedure.
- **AC-11**: Accounts without `owner` field in `{name}.json`, or with `owner: ""`, pass all G1–G8 gates — behavior is byte-identical to pre-feature operation. No regression.
- **AC-12**: `format::json` output includes `"is_owned": true` or `"is_owned": false` per account object. Value matches the `is_owned(account)` predicate at fetch time.
- **AC-13**: `dry::1` on `.account.use`, `.account.delete`, `.account.relogin`, or `.account.unclaim` does NOT skip the ownership check — G5/G6/G7/G8 exit 1 before printing the dry-run message when not owned.
- **AC-14**: `account_save_routine()` passes `owner: None` to `save()` — ownership-neutral, preserves existing `owner` field. Background refresh callers also pass `owner: None`. The `.accounts unclaim::1` path in `accounts_routine()` calls `write_owner(name, store, "")` directly to clear ownership. The `.accounts assign::1` path in `accounts_routine()` does NOT call `write_owner()` and does NOT modify the `owner` field. No CLI-exposed path sets the `owner` field to a non-empty value; the field may only be cleared via `unclaim::1`.
- **AC-15**: `clp .account.save name::X` does NOT modify `owner` in `{name}.json` — passes `owner: None`. `clp .accounts unclaim::1 name::X` writes `owner: ""` to `{name}.json` via `write_owner()` — credentials NOT touched. `.account.save` rejects `unclaim::1` (exits 1 on unknown parameter). `clp .accounts assign::1 name::X` writes only the per-machine marker file `_active_{machine}_{user}` — the `owner` field in `{name}.json` is untouched.
- **AC-16**: `clp .accounts unclaim::1 name::X` evaluates G8 ownership gate: `read_owner()` → `is_owned()`. If non-owner → exit 1 with `"ownership violation: this account is owned by {owner}"`. Gate runs BEFORE `dry::1` check. If account is unowned (`owner == ""`), gate passes — `write_owner()` writes `""` again (idempotent no-op).
- **AC-17**: `clp .accounts unclaim::1 name::X dry::1` prints `[dry-run] would unclaim X` and exits 0. No files modified. G8 gate still runs before dry-run check — non-owner gets exit 1 even in dry-run mode.
- **AC-18**: `clp .account.use name::X force::1` when `X` is owned by a different identity bypasses G5 — proceeds to `switch_account()` and exits 0. `force::1` is registered on `.account.use` as a `bool` param defaulting to `0`.
- **AC-19**: `clp .account.delete name::X force::1` when `X` is owned by a different identity bypasses G6 — proceeds with deletion and exits 0. `force::1` is registered on `.account.delete` as a `bool` param defaulting to `0`.
- **AC-20**: `clp .account.relogin name::X force::1` when `X` is owned by a different identity bypasses G7 — proceeds with the 6-step relogin procedure and exits 0. `force::1` is registered on `.account.relogin` as a `bool` param defaulting to `0`.
- **AC-21**: `force::1` with `dry::1` on any G5–G8 command bypasses the ownership gate (no exit 1) but still previews without writing — `[dry-run]` message is printed and exits 0. Ownership check is bypassed; write suppression is not.
- **AC-22**: `apply_refresh()` emits `[trace] refresh  <name>  should_retry=false (reason: not owned)` when `trace::1` is set and `aq.is_owned == false`. The reason string is `"not owned"` — derived from the ownership gate decision, not from `aq.result`. Consistent with AC-07 (`apply_touch` trace pattern).
- **AC-23**: For an owned account where `is_occupied_elsewhere == true` and `is_current == false`, `fetch_quota_for_list()` does NOT read the credential file and does NOT call `fetch_oauth_usage`. It calls `approximate_quota()` to return cached or polynomial-approximated data (same path as solo gate, Feature 061). When `trace::1`, emits `[trace] fetch  <name>  skipped (reason: occupied elsewhere)` before calling `approximate_quota()`. The `is_current` account is never skipped by G1b. Gate condition: `is_owned && is_occupied_elsewhere && !is_current` (i.e., after G1 passes, check occupancy before token read). Fix(BUG-305).
- **AC-24**: `apply_refresh()` emits `[trace] refresh  <name>  should_retry=false (reason: occupied elsewhere)` when `trace::1` is set, `aq.is_owned == true`, `aq.cached == false`, and `aq.is_occupied_elsewhere == true`. The `reason_label(aq)` function is extracted from `apply_refresh()` and returns `"occupied elsewhere"` for this combination — the fourth branch after `"not owned"`, `"cached-expired"`, before the `else` result-derived branch. The function is directly testable. Fix(BUG-306).

### Bugs

| File | Relationship |
|------|--------------|
| [BUG-295 🟢 Fixed](../../../../task/claude_profile/bug/295_refresh_trace_misleads_reason_ok_for_not_owned.md) | `apply_refresh` emitted `reason: ok` instead of `reason: not owned` for non-owned accounts. Fixed: `!aq.is_owned` guard added to reason derivation at `refresh.rs` |
| [BUG-304 🟢 Fixed (TSK-316)](../../../../../task/claude_profile/bug/304_cache_read_bypasses_approximation.md) | G1 non-owned cache read path bypassed Feature 040 polynomial approximation — stale data in multi-machine setups. Fixed: centralized `read_cached_quota()` function replaces all 3 inline cache-read paths |
| [BUG-305 🟢 Fixed (TSK-317)](../../../../../task/claude_profile/bug/305_fetch_fires_for_occupied_elsewhere_accounts.md) | `fetch_quota_for_list` performed full HTTP fetch for owned+occupied-elsewhere accounts. Fixed: G1b gate added after solo gate — `!is_current && occupied_elsewhere.contains(&acct.name)` → `approximate_quota()` |
| [BUG-306 🟢 Fixed (TSK-317)](../../../../../task/claude_profile/bug/306_refresh_trace_reason_ok_for_occupied_elsewhere.md) | `apply_refresh` trace emitted `reason: ok` for owned+non-cached+occupied-elsewhere accounts. Fixed: `reason_label()` extracted with `is_occupied_elsewhere` branch |

### Features

| File | Relationship |
|------|--------------|
| [002_account_save.md](002_account_save.md) | `.account.save` — ownership-neutral; `account_save_routine()` passes `owner: None`; does NOT modify `owner` field |
| [032_account_assign.md](032_account_assign.md) | `.accounts assign::1` — marker-only write; does NOT call `write_owner()`; does NOT accept `unclaim::` |
| [004_account_use.md](004_account_use.md) | G5: `.account.use` ownership guard |
| [005_account_delete.md](005_account_delete.md) | G6: `.account.delete` ownership guard |
| [009_token_usage.md](009_token_usage.md) | `.usage` — non-owned accounts use G1 cache path; `is_owned` JSON field |
| [017_token_refresh.md](017_token_refresh.md) | G2/G3: non-owned accounts skip `should_refresh()` and `apply_refresh()` |
| [019_account_relogin.md](019_account_relogin.md) | G7: `.account.relogin` ownership guard |
| [024_session_touch.md](024_session_touch.md) | G4: `apply_touch()` skips non-owned accounts |
| [025_per_machine_active_marker.md](025_per_machine_active_marker.md) | `resolve_hostname()` — shared fallback chain for `current_identity()` |
| [029_account_host_metadata.md](029_account_host_metadata.md) | `{name}.json` structure — `owner` field extends the same file; `host::` is display label, not ownership |
| [033_quota_cache.md](033_quota_cache.md) | G1 non-owned path uses quota cache as primary source; same display as cache-fallback |
| [040_quota_measurement_history.md](040_quota_measurement_history.md) | Non-owned accounts skip history append (G1 gate) |
| [061_solo_token_conservation.md](061_solo_token_conservation.md) | Solo gate extends G1/G2/G4 with `is_current` check; non-current+owned accounts use `approximate_quota()` instead of live fetch |

### Parameters

| File | Relationship |
|------|--------------|
| [cli/param/058_force.md](../cli/param/058_force.md) | `force::` — bypass G5–G8 ownership enforcement on mutation commands |
| [cli/param/060_solo.md](../cli/param/060_solo.md) | `solo::` — extends G1/G2/G4 with `is_current` check; non-current+owned accounts use `approximate_quota()` instead of live fetch |

### Commands

| File | Relationship |
|------|--------------|
| [cli/command/001_account.md](../cli/command/001_account.md) | `.account.save` — Command 4; `.account.use` (G5 + force::); `.account.delete` (G6 + force::); `.account.relogin` (G7 + force::) |

### Sources

| File | Relationship |
|------|--------------|
| `src/usage/fetch.rs` | G1: `fetch_quota_for_list()` — ownership check; skip token read + HTTP; cache-as-primary path; `aq.is_owned` assignment |
| `src/usage/refresh_predicate.rs` | G2: `should_refresh()` — early `false` when `!aq.is_owned` |
| `src/usage/refresh.rs` | G3: `apply_refresh()` loop — non-owned skip (via G2 predicate) |
| `src/usage/touch.rs` | G4: `apply_touch()` — non-owned skip with trace |
| `src/commands/account_ops.rs` | G5/G6: `account_use_routine()` / `account_delete_routine()` — ownership guard before mutation; `account_save_routine()` — ownership-neutral (passes `owner: None`) |
| `src/commands/accounts.rs` | G8: `accounts_routine()` unclaim path — ownership guard before `write_owner(name, store, "")`; assign path — marker-only write; does NOT call `write_owner()` |
| `claude_profile_core/src/account.rs` | `save()` with `owner: Option<&str>` — writes when `Some`, preserves when `None`; `current_identity()`; `read_owner()`; `is_owned()`; `write_owner()` — used by `account_unclaim_routine()` for direct owner writes |
| `src/commands/account_relogin.rs` | G7: `account_relogin_routine()` — ownership guard before 6-step procedure |
| `src/usage/types.rs` | `AccountQuota.is_owned: bool` field |
