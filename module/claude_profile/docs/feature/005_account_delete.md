# Feature: Delete Account

### Scope

- **Purpose**: Remove a named account from the store; when the active account is deleted, the per-machine active marker is also removed.
- **Responsibility**: Documents the `account::delete()` API and `.account.delete` CLI command (FR-10).
- **In Scope**: Credential file removal, snapshot cleanup (`{name}.json`), per-machine active marker (`_active_{hostname}_{user}`) cleanup when deleting the active account, dry-run mode, ownership guard (exit 1 when account is owned by a different identity — G6 gate from [036_account_ownership.md](036_account_ownership.md)).
- **Out of Scope**: Switching accounts before deletion (caller responsibility).

### Design

`claude_profile` must remove `{credential_store}/{name}.credentials.json` from the account store.

**Active account deletion:** When `name` matches the per-machine active marker (`_active_{hostname}_{user}` via `active_marker_filename()`), deletion proceeds normally. After removing the credential file, the per-machine active marker is also removed (best-effort — no error if already absent). This leaves the system in a "no active account" state; the user must run `.account.use` or `.account.save` to restore an active account.

**Operation steps:**
1. Validate `name`.
2. Remove `{credential_store}/{name}.credentials.json` → `NotFound` if absent.
3. Best-effort: remove `{credential_store}/{name}.json` if present (silently skip if absent).
4. Best-effort: read per-machine active marker (`active_marker_filename()`); if it matches `name`, remove `{credential_store}/_active_{hostname}_{user}`.

**Ownership guard (G6):** Before executing step 1, `account_delete_routine()` reads the `owner` field from `{name}.json`. If `owner` is non-empty and does not match `current_identity()`, the command exits 1 with `"ownership violation: this account is owned by {owner}"`. This check runs before `dry::1` output — a dry-run on a non-owned account still exits 1. See [036_account_ownership.md](036_account_ownership.md).

**Dry-run mode** (`dry::1`): Print `[dry-run] would delete account '{name}'` without removing any files.

**Exit codes:**
- 0: success (including active account deletion)
- 1: invalid name (usage error); or ownership violation (G6 gate)
- 2: account not found (runtime error)

### Acceptance Criteria

- **AC-01**: `clp .account.delete name::alice@oldco.com` exits 0 and removes `{credential_store}/alice@oldco.com.credentials.json`.
- **AC-02**: `clp .account.delete name::alice@acme.com` (active account) exits 0; removes the credential file and the per-machine active marker (`_active_{hostname}_{user}`), leaving no active account.
- **AC-03**: `clp .account.delete name::ghost@example.com` (non-existent) exits 2 with not-found error.
- **AC-04**: `clp .account.delete name::alice@oldco.com dry::1` exits 0 with `[dry-run]` prefix; no files removed.
- **AC-05**: After a successful delete, `{credential_store}/{name}.json` is also removed if it existed; absent snapshot file causes no error.
- **AC-06**: `clp .account.delete name::alice@other.com` when `alice@other.com`'s owner ≠ `current_identity()` exits 1 with `"ownership violation: this account is owned by {owner}"`. No files are modified. (G6 ownership gate — [036_account_ownership.md](036_account_ownership.md) AC-09.)
- **AC-07**: Ownership check runs before `dry::1` output — `clp .account.delete name::alice@other.com dry::1` with ownership violation exits 1 without printing the dry-run message.

### Commands

| File | Relationship |
|------|--------------|
| [command/001_account.md](../cli/command/001_account.md#command-6-accountdelete) | CLI command specification |

### Features

| File | Relationship |
|------|--------------|
| [015_name_shortcut_syntax.md](015_name_shortcut_syntax.md) | Positional and prefix shortcut for `name::` on this command |
| [022_org_identity_snapshot.md](022_org_identity_snapshot.md) | Org identity metadata lifecycle — delete removes it best-effort |
| [025_per_machine_active_marker.md](025_per_machine_active_marker.md) | Per-machine active marker naming convention used in deletion step |
| [036_account_ownership.md](036_account_ownership.md) | G6: ownership guard — exit 1 before any deletion when account is owned by different identity |

### Referenced Commands

| # | Command | Role |
|---|---------|------|
| 1 | [`.account.delete`](../cli/command/001_account.md#command-6-accountdelete) | CLI surface for this feature |

### Sources

| File | Relationship |
|------|--------------|
| `src/account.rs` | `delete()` — validate, remove file, clear per-machine active marker if active |
| `src/commands/account_ops.rs` | `account_delete_routine()` — CLI handler |

### Tests

| File | Relationship |
|------|--------------|
| `tests/account_tests.rs::delete_active_account_succeeds` | Verifies active account deletion clears per-machine active marker |
