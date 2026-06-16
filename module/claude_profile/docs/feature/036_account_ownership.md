# Feature: Account Ownership

### Scope

- **Purpose**: Allow a saved account to declare which host/user identity owns it, so that non-owner machines can safely read cached quota but never touch, refresh, switch to, delete, or re-authenticate the account's credentials.
- **Responsibility**: Documents the `owner` field in `{name}.json`, identity resolution via `current_identity()`, ownership stamp at `.account.save` time via `account_save_routine()` always passing `Some(&current_identity())` to `save()`, `.account.unclaim` command (clears ownership by calling `write_owner(name, store, "")` directly), eight enforcement gates (G1–G8), and the `is_owned` flag propagated to `AccountQuota` and `format::json` output.
- **In Scope**: `owner` field in `{name}.json`; `current_identity()` resolution (`$USER@<hostname>` via `resolve_hostname()`); ownership stamp at `.account.save` time via `account_save_routine()` passing `Some(&current_identity())` to `save()`; `.account.unclaim` command — calls `write_owner(name, store, "")` directly to clear ownership without touching credentials; background refresh callers pass `owner: None` to `save()` (preserves existing); `.account.assign` is ownership-neutral — marker-only write, does not call `write_owner()`; `is_owned(account)` predicate; G1 quota-fetch gate (bypass token read + HTTP for non-owned, use cache directly); G2 `should_refresh()` early-false for non-owned; G3 `apply_refresh` loop skip; G4 `apply_touch` skip; G5 `.account.use` ownership guard; G6 `.account.delete` ownership guard; G7 `.account.relogin` ownership guard; G8 `.account.unclaim` ownership guard; `AccountQuota.is_owned` flag; `is_owned` field in `format::json` output.
- **Out of Scope**: Credential file access control (enforcement is logical, not filesystem-level); shared-store cross-machine sync mechanism (out of scope for this project); `host::` display label (remains user-settable; see [029_account_host_metadata.md](029_account_host_metadata.md)). ~~Owner display column~~ — moved to Feature 037: Owner column now visible by default on both `.accounts` and `.usage`.

> **CLI surface migration (Feature 037):** `.account.unclaim` is being absorbed into `.accounts` as `unclaim::1` parameter. The standalone command will be removed. All acceptance criteria and G8 enforcement below remain valid — they apply via `clp .accounts unclaim::1 name::X` instead of `clp .account.unclaim name::X`. Batch unclaim (no `name::`, applies to filtered set) is a new capability. See [037_accounts_usage_param_unification.md](037_accounts_usage_param_unification.md).

### Design

**Ownership model:** Each saved account optionally carries an `owner` field in `{name}.json`. An empty or absent `owner` means no enforcement — the account behaves exactly as before this feature (backward compatible). A non-empty `owner` identifies the single `USER@MACHINE` identity that may operate on the account's credentials. All other identities may only read the quota cache.

**Identity resolution:** `current_identity() = "$USER@<hostname>"` where hostname is resolved via `resolve_hostname()` — the same fallback chain as `active_marker_filename()` in Feature 025: `$HOSTNAME` env → `/etc/hostname` file → `"local"`. The components are NOT sanitized (unlike `active_marker_filename()`): `$USER` and hostname are used as-is. This makes `current_identity()` round-trip with the auto-captured `owner` value without requiring sanitization at comparison time.

**Ownership predicate:**
```
is_owned(account) = account.owner.is_empty() || account.owner == current_identity()
```
An account is "owned by this machine" when: (a) owner is empty or absent (no enforcement — all identities pass), or (b) the stored owner string exactly matches `current_identity()`. Any other value means "owned by someone else" — enforcement gates apply.

**Owner capture at save time:** When `.account.save` executes, `account_save_routine()` always sets `owner_val = current_identity()` and passes `Some(&owner_val)` to `save()`, which writes it to the `owner` field in `{name}.json`. Ownership is stamped on every interactive save. To release ownership, use `clp .account.unclaim name::EMAIL` — a dedicated command that calls `write_owner(name, store, "")` directly without touching credentials or the active marker. **`.account.assign` is ownership-neutral — it writes only the marker file and does NOT call `write_owner()` or modify the `owner` field in any way.**

**`save()` owner handling:** `save()` accepts an `owner: Option<&str>` parameter. When `Some(s)`, the value is written to `{name}.json`. When `None`, the existing `owner` field in `{name}.json` is preserved — read-merge semantics identical to `host` and `role`. `account_save_routine()` always passes `Some(&current_identity())` (stamps ownership on every interactive save). Background refresh callers pass `owner: None` (preserves existing owner). To clear ownership, use `clp .account.unclaim name::EMAIL` — calls `write_owner(name, store, "")` directly without touching credentials.

**Enforcement gates (G1–G8):**

| Gate | Location | Action when `!is_owned` | `force::1` bypass |
|------|----------|------------------------|-------------------|
| G1 | `fetch_quota_for_list()` in `fetch.rs` | Skip `read_token()` and HTTP fetch; read quota from cache (`read_quota_cache()`) directly; set `aq.is_owned = false` | No bypass — read-side gate; `force::` not accepted by fetch path |
| G2 | `should_refresh()` in `refresh_predicate.rs` | Return `false` early — no refresh attempt | No bypass — read-side gate |
| G3 | `apply_refresh()` loop in `refresh.rs` | Skip account — `should_refresh()` already returns `false` via G2 | No bypass — read-side gate |
| G4 | `apply_touch()` in `touch.rs` | Skip account — emit trace `"skipped (reason: not owned)"` when `trace::1` | No bypass — read-side gate |
| G5 | `account_use_routine()` in `account_ops.rs` | Exit 1 with `"ownership violation: this account is owned by {owner}"` | Yes — `force::1` skips gate; proceeds to `switch_account()` |
| G6 | `account_delete_routine()` in `account_ops.rs` | Exit 1 with `"ownership violation: this account is owned by {owner}"` | Yes — `force::1` skips gate; proceeds to deletion |
| G7 | `account_relogin_routine()` in `account_relogin.rs` | Exit 1 with `"ownership violation: this account is owned by {owner}"` | Yes — `force::1` skips gate; proceeds to 6-step relogin |
| G8 | `accounts_routine()` (`unclaim::1` path) in `commands/accounts.rs` (Feature 037) | Exit 1 with `"ownership violation: this account is owned by {owner}"` | Yes — `force::1` skips gate; proceeds to `write_owner(name, store, "")` |

**G1 detail (cache-as-primary for non-owned accounts):** When `is_owned = false`, `fetch_quota_for_list()` skips `read_token()` (avoids touching the credential file) and skips the HTTP call to `fetch_oauth_usage`. Instead, it calls `read_quota_cache(credential_store, name)` and returns the cached values if present — the same path as Feature 033 cache-fallback, but triggered by ownership rather than API failure. The row is rendered with `~` prefix and age indicator identical to the cache-fallback path. If no cache exists, the row shows `—` for all quota columns. `aq.is_owned = false` is set in all cases. `aq.cached = true` is set when cache data is used.

**`AccountQuota.is_owned` field:** New `bool` field on `AccountQuota`. Set to `true` when `is_owned(account)` at fetch time; `false` otherwise. Propagated to `format::json` output as `"is_owned": bool`. Not used for display column — ownership is operational, not display.

**Backward compatibility:** Any `{name}.json` without an `owner` field (or with `owner: ""`) behaves identically to the pre-feature behavior. No migration needed. All existing accounts are effectively unowned after the feature ships.

**No `host::` collision:** The `host::` parameter (Feature 029) is a user-customizable display label (e.g., `"workstation"`, `"laptop"`). It is independent of `owner`. The `owner` field is set by `.account.save` (via `account_save_routine()` passing `Some(&owner_val)`) and cleared via `.account.unclaim` (calls `write_owner(name, store, "")`); it is never user-specified as a direct value via CLI parameter. Both fields coexist in `{name}.json` without ambiguity.

**`force::` bypass:** All commands enforcing G5–G8 accept a `force::1` parameter that bypasses the ownership check. When `force::1` is present, the gate is skipped regardless of whether `current_identity() == owner`. `force::1` does not affect G1–G4 (read-side gates — touch and refresh are intentionally suppressed for non-owned accounts even with force). `force::1` always runs BEFORE `dry::1` evaluation — when both are set, the ownership check is bypassed but the mutation is still previewed without file writes.

**Dry-run interaction:** G5, G6, G7, G8 check ownership BEFORE evaluating `dry::1` — ownership violation exits 1 even in dry-run mode (unless `force::1` is also set). This prevents information leakage (a dry-run would still reveal that a switch is possible, which is incorrect if the caller isn't the owner).

**Trace interaction:** G4 emits a `[trace] touch  <name>  skipped (reason: not owned)` line when `trace::1` — identical format to other touch skip traces. G1 emits `[trace] fetch  <name>  skipped (reason: not owned)` when `trace::1`.

### Acceptance Criteria

- **AC-01**: `clp .account.save name::X` stamps `current_identity()` as `owner` in `{name}.json` via `account_save_routine()` passing `Some(&owner_val)` to `save()`. `clp .account.assign name::X` does NOT modify `owner` (marker-only — no `write_owner()` call). All other `{name}.json` fields are preserved via read-merge.
- **AC-02**: `clp .account.unclaim name::X` exits 0 and writes `owner: ""` to `{name}.json` via `write_owner(name, store, "")`. Credentials are NOT touched (no `save()` call, no credential file read or write). After unclaim, all G1–G8 enforcement gates pass (account behaves as unowned — no enforcement).
- **AC-03**: There is no `owner::` CLI parameter — ownership is never user-specified as a direct string value. `unclaim::` is not registered on `.account.save` or `.account.assign` (exits 1 on unknown parameter). Ownership release is via the dedicated `.account.unclaim` command. `.account.assign` is marker-only — does NOT call `write_owner()`, does NOT touch the `owner` field in `{name}.json`.
- **AC-04**: For an account where `is_owned = false`, `fetch_quota_for_list()` does NOT read the credential file and does NOT call `fetch_oauth_usage`. It calls `read_quota_cache()` and returns cached quota (with `cached = true`) when available, or dashes when no cache exists. `aq.is_owned = false` is always set.
- **AC-05**: Non-owned accounts in `.usage` output show quota values with `~` prefix and `(Xm ago)` age indicator when cache is present — identical to Feature 033 cache-fallback display. When no cache exists, columns show `—`.
- **AC-06**: `should_refresh()` returns `false` when `aq.is_owned == false`. No refresh subprocess is spawned for non-owned accounts regardless of token state.
- **AC-07**: `apply_touch()` skips accounts where `aq.is_owned == false`. When `trace::1`, emits `[trace] touch  <name>  skipped (reason: not owned)` for each skipped account.
- **AC-08**: `clp .account.use name::alice@other.com` when `alice@other.com`'s owner ≠ `current_identity()` exits 1 with message `"ownership violation: this account is owned by {owner}"`. Ownership check runs before `switch_account()`.
- **AC-09**: `clp .account.delete name::alice@other.com` when `alice@other.com`'s owner ≠ `current_identity()` exits 1 with message `"ownership violation: this account is owned by {owner}"`. Ownership check runs before any deletion.
- **AC-10**: `clp .account.relogin name::alice@other.com` when `alice@other.com`'s owner ≠ `current_identity()` exits 1 with message `"ownership violation: this account is owned by {owner}"`. Ownership check runs before step 1 of the 6-step relogin procedure.
- **AC-11**: Accounts without `owner` field in `{name}.json`, or with `owner: ""`, pass all G1–G8 gates — behavior is byte-identical to pre-feature operation. No regression.
- **AC-12**: `format::json` output includes `"is_owned": true` or `"is_owned": false` per account object. Value matches the `is_owned(account)` predicate at fetch time.
- **AC-13**: `dry::1` on `.account.use`, `.account.delete`, `.account.relogin`, or `.account.unclaim` does NOT skip the ownership check — G5/G6/G7/G8 exit 1 before printing the dry-run message when not owned.
- **AC-14**: `account_save_routine()` always passes `Some(&current_identity())` to `save()` (stamps ownership on every interactive save). `account_unclaim_routine()` calls `write_owner(name, store, "")` directly to clear ownership. Background refresh callers pass `owner: None` (preserves existing owner via read-merge). `account_assign_routine()` does NOT call `write_owner()` and does NOT modify the `owner` field.
- **AC-15**: `clp .account.save name::X` stamps `current_identity()` as `owner` in `{name}.json`. `clp .account.unclaim name::X` writes `owner: ""` to `{name}.json` via `write_owner()` — credentials NOT touched. Both `.account.save` and `.account.assign` reject `unclaim::1` (exits 1 on unknown parameter). `clp .account.assign name::X` writes only the per-machine marker file `_active_{machine}_{user}` — the `owner` field in `{name}.json` is untouched.
- **AC-16**: `clp .account.unclaim name::X` evaluates G8 ownership gate: `read_owner()` → `is_owned()`. If non-owner → exit 1 with `"ownership violation: this account is owned by {owner}"`. Gate runs BEFORE `dry::1` check. If account is unowned (`owner == ""`), gate passes — `write_owner()` writes `""` again (idempotent no-op).
- **AC-17**: `clp .account.unclaim name::X dry::1` prints `[dry-run] would unclaim X` and exits 0. No files modified. G8 gate still runs before dry-run check — non-owner gets exit 1 even in dry-run mode.
- **AC-18**: `clp .account.use name::X force::1` when `X` is owned by a different identity bypasses G5 — proceeds to `switch_account()` and exits 0. `force::1` is registered on `.account.use` as a `bool` param defaulting to `0`.
- **AC-19**: `clp .account.delete name::X force::1` when `X` is owned by a different identity bypasses G6 — proceeds with deletion and exits 0. `force::1` is registered on `.account.delete` as a `bool` param defaulting to `0`.
- **AC-20**: `clp .account.relogin name::X force::1` when `X` is owned by a different identity bypasses G7 — proceeds with the 6-step relogin procedure and exits 0. `force::1` is registered on `.account.relogin` as a `bool` param defaulting to `0`.
- **AC-21**: `force::1` with `dry::1` on any G5–G8 command bypasses the ownership gate (no exit 1) but still previews without writing — `[dry-run]` message is printed and exits 0. Ownership check is bypassed; write suppression is not.

### Bugs

| File | Relationship |
|------|--------------|
| *(none filed yet)* | — |

### Features

| File | Relationship |
|------|--------------|
| [002_account_save.md](002_account_save.md) | `.account.save` — stamps `owner` via `account_save_routine()` passing `Some(&current_identity())` |
| [032_account_assign.md](032_account_assign.md) | `.account.assign` — marker-only write; does NOT call `write_owner()`; does NOT accept `unclaim::` |
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
| [cli/param/058_force.md](../cli/param/058_force.md) | `force::` — bypass G5–G8 ownership enforcement on mutation commands |

### Commands

| File | Relationship |
|------|--------------|
| [cli/command/001_account.md](../cli/command/001_account.md) | `.account.save` — Command 4; `.account.unclaim` — Command 17; `.account.use` (G5 + force::); `.account.delete` (G6 + force::); `.account.relogin` (G7 + force::) |

### Sources

| File | Relationship |
|------|--------------|
| `src/usage/fetch.rs` | G1: `fetch_quota_for_list()` — ownership check; skip token read + HTTP; cache-as-primary path; `aq.is_owned` assignment |
| `src/usage/refresh_predicate.rs` | G2: `should_refresh()` — early `false` when `!aq.is_owned` |
| `src/usage/refresh.rs` | G3: `apply_refresh()` loop — non-owned skip (via G2 predicate) |
| `src/usage/touch.rs` | G4: `apply_touch()` — non-owned skip with trace |
| `src/commands/account_ops.rs` | G5/G6: `account_use_routine()` / `account_delete_routine()` — ownership guard before mutation; G8: `account_unclaim_routine()` — ownership guard before unclaim; `account_save_routine()` — stamps ownership via `Some(&current_identity())` |
| `src/commands/account_assign.rs` | `account_assign_routine()` — marker-only write; does NOT call `write_owner()`; does NOT modify `owner` field |
| `claude_profile_core/src/account.rs` | `save()` with `owner: Option<&str>` — writes when `Some`, preserves when `None`; `current_identity()`; `read_owner()`; `is_owned()`; `write_owner()` — used by `account_unclaim_routine()` for direct owner writes |
| `src/commands/account_relogin.rs` | G7: `account_relogin_routine()` — ownership guard before 6-step procedure |
| `src/usage/types.rs` | `AccountQuota.is_owned: bool` field |
