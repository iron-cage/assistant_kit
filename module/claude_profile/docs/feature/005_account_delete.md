# Feature: Delete Account

### Scope

- **Purpose**: Remove a named account from the store; when the deleted account is active on any machine, every `_active_*` marker naming it across all machines is also removed (Fix BUG-347).
- **Responsibility**: Documents the `account::delete()` API and `.account.delete` CLI command (FR-10).
- **In Scope**: Credential file removal, snapshot cleanup (`{name}.json`), cross-machine active marker cleanup (every `_active_*` file naming the deleted account across all machines, not only the calling machine's own) when deleting an active account, dry-run mode, ownership guard (exit 1 when account is owned by a different identity — G6 gate from [036_account_ownership.md](036_account_ownership.md)).
- **Out of Scope**: Switching accounts before deletion (caller responsibility).

### Design

`claude_profile` must remove `{credential_store}/{name}.credentials.json` from the account store.

**Active account deletion:** Deletion always proceeds normally regardless of active state — there is no refusal guard. After removing the credential file, every `_active_*` marker file in the credential store whose content matches `name` is also removed (best-effort, via `all_marker_files()` — no error if none match), not only the calling machine's own marker (Fix BUG-347: the pre-fix implementation resolved only the calling machine's own marker path via `active_marker_filename()`, leaving foreign-machine markers naming the same account orphaned). This can leave one or more machines in a "no active account" state; the user must run `.account.use` or `.account.save` on each affected machine to restore an active account.

**Operation steps:**
1. Validate `name`.
2. Remove `{credential_store}/{name}.credentials.json` → `NotFound` if absent.
3. Best-effort: remove `{credential_store}/{name}.json` if present (silently skip if absent).
4. Best-effort: scan every `_active_*` marker file in the credential store (`all_marker_files()`); remove any whose content matches `name` — not only the calling machine's own marker (Fix BUG-347).

**Ownership guard (G6):** Before executing step 1, `account_delete_routine()` reads the `owner` field from `{name}.json`. If `owner` is non-empty and does not match `current_identity()`, the command exits 1 with `"ownership violation: this account is owned by {owner}"`. This check runs before `dry::1` output — a dry-run on a non-owned account still exits 1. See [036_account_ownership.md](036_account_ownership.md).

**Dry-run mode** (`dry::1`): Print `[dry-run] would delete account '{name}'` without removing any files.

**Exit codes:**
- 0: success (including active account deletion)
- 1: invalid name (usage error); or ownership violation (G6 gate)
- 2: account not found (runtime error)

### Acceptance Criteria

- **AC-01**: `clp .account.delete name::alice@oldco.com` exits 0 and removes `{credential_store}/alice@oldco.com.credentials.json`.
- **AC-02**: `clp .account.delete name::alice@acme.com` (active account) exits 0; removes the credential file and every `_active_*` marker across all machines naming the account, leaving no active account on any of them.
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
| `src/account/store.rs` | `delete()` — validate, remove file, clear every `_active_*` marker naming this account across all machines (Fix BUG-347) |
| `src/commands/account_ops.rs` | `account_delete_routine()` — CLI handler |

### Tests

| File | Relationship |
|------|--------------|
| `tests/account_tests.rs::delete_active_account_succeeds` | Verifies active account deletion clears the calling machine's own marker |
| `claude_profile_core/tests/account_test.rs::test_ft14_025_delete_clears_foreign_machine_marker` | `bug_reproducer(BUG-347)` — verifies deletion clears a *foreign* machine's marker naming the same account |
