# Feature: Delete Account

### Scope

- **Purpose**: Remove a named account from the store with a safety guard that prevents deleting the currently active account.
- **Responsibility**: Documents the `account::delete()` API and `.account.delete` CLI command (FR-10).
- **In Scope**: Credential file removal, active-account guard, dry-run mode.
- **Out of Scope**: Switching accounts before deletion (caller responsibility).

### Design

`claude_profile` must remove `~/.claude/accounts/{name}.credentials.json` from the account store.

**Active-account guard:** Before deletion, check if `name` matches the `_active` marker. If so, return `PermissionDenied` with an actionable message instructing the user to switch accounts first.

**Operation steps:**
1. Validate `name`.
2. Read `_active` marker; if `name` matches → error: cannot delete active account.
3. Remove `~/.claude/accounts/{name}.credentials.json` → `NotFound` if absent.

**Dry-run mode** (`dry::1`): Print `[dry-run] would delete account '{name}'` without removing any files.

**Exit codes:**
- 0: success
- 1: invalid name (usage error)
- 2: account not found, or account is active (runtime error)

### Acceptance Criteria

- **AC-01**: `clp .account.delete name::old` exits 0 and removes `~/.claude/accounts/old.credentials.json`.
- **AC-02**: `clp .account.delete name::work` (active account) exits 2 with message directing user to switch first.
- **AC-03**: `clp .account.delete name::ghost` (non-existent) exits 2 with not-found error.
- **AC-04**: `clp .account.delete name::old dry::1` exits 0 with `[dry-run]` prefix; no files removed.

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| source | `src/account.rs` | `delete()` — validate, guard, remove file |
| source | `src/commands.rs` | `account_delete_routine()` — CLI handler |
| test | `tests/account_tests.rs::delete_returns_error_if_account_is_active` | Verifies active-account guard |
| doc | [cli/commands.md](../cli/commands.md#command--7-accountdelete) | CLI command specification |
