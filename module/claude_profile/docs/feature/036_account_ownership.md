# Feature: Account Ownership

### Scope

- **Purpose**: Allow a saved account to declare which host/user identity owns it, so that non-owner machines can safely read cached quota but never touch, refresh, switch to, delete, or re-authenticate the account's credentials.
- **Responsibility**: Documents the `owner` field in `{name}.json`, identity resolution via `current_identity()`, the `unclaim::1` parameter on `.account.save`, seven enforcement gates (G1–G7), and the `is_owned` flag propagated to `AccountQuota` and `format::json` output.
- **In Scope**: `owner` field in `{name}.json`; `current_identity()` resolution (`$USER@<hostname>` via `resolve_hostname()`); auto-capture at `.account.save` time; `unclaim::1` param to clear ownership; `is_owned(account)` predicate; G1 quota-fetch gate (bypass token read + HTTP for non-owned, use cache directly); G2 `should_refresh()` early-false for non-owned; G3 `apply_refresh` loop skip; G4 `apply_touch` skip; G5 `.account.use` ownership guard; G6 `.account.delete` ownership guard; G7 `.account.relogin` ownership guard; `AccountQuota.is_owned` flag; `save()` `owner: Option<&str>` param with read-merge preservation; `is_owned` field in `format::json` output.
- **Out of Scope**: Credential file access control (enforcement is logical, not filesystem-level); shared-store cross-machine sync mechanism (out of scope for this project); owner display column in `.usage` or `.accounts` (not added — ownership is operational metadata, not display info); `host::` display label (remains user-settable; see [029_account_host_metadata.md](029_account_host_metadata.md)).

### Design

**Ownership model:** Each saved account optionally carries an `owner` field in `{name}.json`. An empty or absent `owner` means no enforcement — the account behaves exactly as before this feature (backward compatible). A non-empty `owner` identifies the single `USER@MACHINE` identity that may operate on the account's credentials. All other identities may only read the quota cache.

**Identity resolution:** `current_identity() = "$USER@<hostname>"` where hostname is resolved via `resolve_hostname()` — the same fallback chain as `active_marker_filename()` in Feature 025: `$HOSTNAME` env → `/etc/hostname` file → `"local"`. The components are NOT sanitized (unlike `active_marker_filename()`): `$USER` and hostname are used as-is. This makes `current_identity()` round-trip with the auto-captured `owner` value without requiring sanitization at comparison time.

**Ownership predicate:**
```
is_owned(account) = account.owner.is_empty() || account.owner == current_identity()
```
An account is "owned by this machine" when: (a) owner is empty or absent (no enforcement — all identities pass), or (b) the stored owner string exactly matches `current_identity()`. Any other value means "owned by someone else" — enforcement gates apply.

**Owner capture at save time:** When `.account.save` executes, `account_save_routine()` passes `owner: Some(current_identity())` into `save()`. This records which identity performed the save. If the account already has an owner from a previous save, the new save overwrites it with the current machine's identity — re-saving is an ownership transfer.

**`unclaim::1` parameter:** When passed to `.account.save`, `account_save_routine()` passes `owner: Some("")` into `save()`, writing an empty string to the `owner` field in `{name}.json`. An empty owner disables all enforcement, returning the account to shared/unowned mode. Other fields are preserved via read-merge.

**`save()` owner handling:** `save()` gains an `owner: Option<&str>` parameter. When `Some(s)`, the value is written to `{name}.json` (empty string clears ownership, non-empty string sets the owner). When `None`, the existing `owner` field in `{name}.json` is preserved — read-merge semantics identical to `host` and `role`. All callers that are NOT the CLI save routine (e.g., `refresh_account_token()`, the touch path via `refresh_account_token()`) pass `owner: None` — ownership is never modified by background credential operations.

**Enforcement gates (G1–G7):**

| Gate | Location | Action when `!is_owned` |
|------|----------|------------------------|
| G1 | `fetch_quota_for_list()` in `fetch.rs` | Skip `read_token()` and HTTP fetch; read quota from cache (`read_quota_cache()`) directly; set `aq.is_owned = false` |
| G2 | `should_refresh()` in `refresh_predicate.rs` | Return `false` early — no refresh attempt |
| G3 | `apply_refresh()` loop in `refresh.rs` | Skip account — `should_refresh()` already returns `false` via G2 |
| G4 | `apply_touch()` in `touch.rs` | Skip account — emit trace `"skipped (reason: not owned)"` when `trace::1` |
| G5 | `account_use_routine()` in `account_ops.rs` | Exit 1 with `"ownership violation: this account is owned by {owner}"` |
| G6 | `account_delete_routine()` in `account_ops.rs` | Exit 1 with `"ownership violation: this account is owned by {owner}"` |
| G7 | `account_relogin_routine()` in `account_relogin.rs` | Exit 1 with `"ownership violation: this account is owned by {owner}"` |

**G1 detail (cache-as-primary for non-owned accounts):** When `is_owned = false`, `fetch_quota_for_list()` skips `read_token()` (avoids touching the credential file) and skips the HTTP call to `fetch_oauth_usage`. Instead, it calls `read_quota_cache(credential_store, name)` and returns the cached values if present — the same path as Feature 033 cache-fallback, but triggered by ownership rather than API failure. The row is rendered with `~` prefix and age indicator identical to the cache-fallback path. If no cache exists, the row shows `—` for all quota columns. `aq.is_owned = false` is set in all cases. `aq.cached = true` is set when cache data is used.

**`AccountQuota.is_owned` field:** New `bool` field on `AccountQuota`. Set to `true` when `is_owned(account)` at fetch time; `false` otherwise. Propagated to `format::json` output as `"is_owned": bool`. Not used for display column — ownership is operational, not display.

**Backward compatibility:** Any `{name}.json` without an `owner` field (or with `owner: ""`) behaves identically to the pre-feature behavior. No migration needed. All existing accounts are effectively unowned after the feature ships.

**No `host::` collision:** The `host::` parameter (Feature 029) is a user-customizable display label (e.g., `"workstation"`, `"laptop"`). It is independent of `owner`. The `owner` field is always auto-captured and never user-specified via CLI parameter. Both fields coexist in `{name}.json` without ambiguity.

**Dry-run interaction:** G5, G6, G7 check ownership BEFORE evaluating `dry::1` — ownership violation exits 1 even in dry-run mode. This prevents information leakage (a dry-run would still reveal that a switch is possible, which is incorrect if the caller isn't the owner).

**Trace interaction:** G4 emits a `[trace] touch  <name>  skipped (reason: not owned)` line when `trace::1` — identical format to other touch skip traces. G1 emits `[trace] fetch  <name>  skipped (reason: not owned)` when `trace::1`.

### Acceptance Criteria

- **AC-01**: `clp .account.save` auto-captures `current_identity()` as the `owner` field in `{name}.json`. Two successive saves from different identities overwrite the owner — the last saver owns the account.
- **AC-02**: `clp .account.save unclaim::1` writes `owner: ""` to `{name}.json`. After unclaim, all G1–G7 gates pass (account behaves as unowned — no enforcement).
- **AC-03**: There is no `owner::` CLI parameter — ownership is never user-specified. The only way to set owner is to run `.account.save` (sets to `current_identity()`); the only way to clear it is `unclaim::1`.
- **AC-04**: For an account where `is_owned = false`, `fetch_quota_for_list()` does NOT read the credential file and does NOT call `fetch_oauth_usage`. It calls `read_quota_cache()` and returns cached quota (with `cached = true`) when available, or dashes when no cache exists. `aq.is_owned = false` is always set.
- **AC-05**: Non-owned accounts in `.usage` output show quota values with `~` prefix and `(Xm ago)` age indicator when cache is present — identical to Feature 033 cache-fallback display. When no cache exists, columns show `—`.
- **AC-06**: `should_refresh()` returns `false` when `aq.is_owned == false`. No refresh subprocess is spawned for non-owned accounts regardless of token state.
- **AC-07**: `apply_touch()` skips accounts where `aq.is_owned == false`. When `trace::1`, emits `[trace] touch  <name>  skipped (reason: not owned)` for each skipped account.
- **AC-08**: `clp .account.use name::alice@other.com` when `alice@other.com`'s owner ≠ `current_identity()` exits 1 with message `"ownership violation: this account is owned by {owner}"`. Ownership check runs before `switch_account()`.
- **AC-09**: `clp .account.delete name::alice@other.com` when `alice@other.com`'s owner ≠ `current_identity()` exits 1 with message `"ownership violation: this account is owned by {owner}"`. Ownership check runs before any deletion.
- **AC-10**: `clp .account.relogin name::alice@other.com` when `alice@other.com`'s owner ≠ `current_identity()` exits 1 with message `"ownership violation: this account is owned by {owner}"`. Ownership check runs before step 1 of the 6-step relogin procedure.
- **AC-11**: Accounts without `owner` field in `{name}.json`, or with `owner: ""`, pass all G1–G7 gates — behavior is byte-identical to pre-feature operation. No regression.
- **AC-12**: `format::json` output includes `"is_owned": true` or `"is_owned": false` per account object. Value matches the `is_owned(account)` predicate at fetch time.
- **AC-13**: `dry::1` on `.account.use`, `.account.delete`, or `.account.relogin` does NOT skip the ownership check — G5/G6/G7 exit 1 before printing the dry-run message when not owned.
- **AC-14**: `save()` called from `refresh_account_token()`, touch subprocess path, or any caller other than the CLI save routine passes `owner: None` — the `owner` field in `{name}.json` is preserved unchanged via read-merge.

### Bugs

| File | Relationship |
|------|--------------|
| *(none filed yet)* | — |

### Features

| File | Relationship |
|------|--------------|
| [002_account_save.md](002_account_save.md) | `.account.save` — ownership auto-capture and `unclaim::1` entry point |
| [004_account_use.md](004_account_use.md) | G5: `.account.use` ownership guard |
| [005_account_delete.md](005_account_delete.md) | G6: `.account.delete` ownership guard |
| [009_token_usage.md](009_token_usage.md) | `.usage` — non-owned accounts use G1 cache path; `is_owned` JSON field |
| [017_token_refresh.md](017_token_refresh.md) | G2/G3: non-owned accounts skip `should_refresh()` and `apply_refresh()` |
| [019_account_relogin.md](019_account_relogin.md) | G7: `.account.relogin` ownership guard |
| [024_session_touch.md](024_session_touch.md) | G4: `apply_touch()` skips non-owned accounts |
| [025_per_machine_active_marker.md](025_per_machine_active_marker.md) | `resolve_hostname()` — shared fallback chain for `current_identity()` |
| [029_account_host_metadata.md](029_account_host_metadata.md) | `{name}.json` structure — `owner` field extends the same file; `host::` is display label, not ownership |
| [033_quota_cache.md](033_quota_cache.md) | G1 non-owned path uses quota cache as primary source; same display as cache-fallback |

### Parameters

| File | Relationship |
|------|--------------|
| [cli/param/056_unclaim.md](../cli/param/056_unclaim.md) | `unclaim::` — clear ownership on `.account.save` |

### Sources

| File | Relationship |
|------|--------------|
| `src/usage/fetch.rs` | G1: `fetch_quota_for_list()` — ownership check; skip token read + HTTP; cache-as-primary path; `aq.is_owned` assignment |
| `src/usage/refresh_predicate.rs` | G2: `should_refresh()` — early `false` when `!aq.is_owned` |
| `src/usage/refresh.rs` | G3: `apply_refresh()` loop — non-owned skip (via G2 predicate) |
| `src/usage/touch.rs` | G4: `apply_touch()` — non-owned skip with trace |
| `src/commands/account_ops.rs` | G5/G6: `account_use_routine()` / `account_delete_routine()` — ownership guard before mutation |
| `src/commands/account_relogin.rs` | G7: `account_relogin_routine()` — ownership guard before 6-step procedure |
| `claude_profile_core/src/account.rs` | `save()` `owner: Option<&str>` param; `current_identity()`; `read_owner()` helper; `is_owned()` predicate |
| `src/usage/types.rs` | `AccountQuota.is_owned: bool` field |
