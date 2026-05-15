# Feature: Delete Account

### Scope

- **Purpose**: Remove a named account from the store; when the active account is deleted, the `_active` marker is also removed.
- **Responsibility**: Documents the `account::delete()` API and `.account.delete` CLI command (FR-10).
- **In Scope**: Credential file removal, snapshot cleanup (`.claude.json`, `.settings.json`), `_active` marker cleanup when deleting the active account, dry-run mode.
- **Out of Scope**: Switching accounts before deletion (caller responsibility).

### Design

`claude_profile` must remove `{credential_store}/{name}.credentials.json` from the account store.

**Active account deletion:** When `name` matches the `_active` marker, deletion proceeds normally. After removing the credential file, the `_active` marker is also removed (best-effort — no error if already absent). This leaves the system in a "no active account" state; the user must run `.account.use` or `.account.save` to restore an active account.

**Operation steps:**
1. Validate `name`.
2. Remove `{credential_store}/{name}.credentials.json` → `NotFound` if absent.
3. Best-effort: remove `{credential_store}/{name}.claude.json` if present (silently skip if absent).
4. Best-effort: remove `{credential_store}/{name}.settings.json` if present (silently skip if absent).
5. Best-effort: read `_active` marker; if it matches `name`, remove `{credential_store}/_active`.

**Dry-run mode** (`dry::1`): Print `[dry-run] would delete account '{name}'` without removing any files.

**Exit codes:**
- 0: success (including active account deletion)
- 1: invalid name (usage error)
- 2: account not found (runtime error)

### Acceptance Criteria

- **AC-01**: `clp .account.delete name::alice@oldco.com` exits 0 and removes `{credential_store}/alice@oldco.com.credentials.json`.
- **AC-02**: `clp .account.delete name::alice@acme.com` (active account) exits 0; removes the credential file and the `_active` marker, leaving no active account.
- **AC-03**: `clp .account.delete name::ghost@example.com` (non-existent) exits 2 with not-found error.
- **AC-04**: `clp .account.delete name::alice@oldco.com dry::1` exits 0 with `[dry-run]` prefix; no files removed.
- **AC-05**: After a successful delete, `{credential_store}/{name}.claude.json` and `{credential_store}/{name}.settings.json` are also removed if they existed; absent snapshot files cause no error.

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| source | `src/account.rs` | `delete()` — validate, remove file, clear `_active` if active |
| source | `src/commands.rs` | `account_delete_routine()` — CLI handler |
| test | `tests/account_tests.rs::delete_active_account_succeeds` | Verifies active account deletion clears `_active` |
| doc | [cli/commands.md](../cli/commands.md#command--6-accountdelete) | CLI command specification |
