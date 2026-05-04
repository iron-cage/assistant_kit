# Feature: Switch Account

### Scope

- **Purpose**: Atomically rotate the active credential set to a named account without credential corruption risk.
- **Responsibility**: Documents the `account::switch_account()` API and `.account.switch` CLI command (FR-9).
- **In Scope**: Atomic write-then-rename, `_active` marker update, not-found guard, dry-run.
- **Out of Scope**: Selecting which account to switch to (→ 008_auto_rotate.md), process termination (caller responsibility).

### Design

`claude_profile` must switch the active account by:

1. Read `{credential_store}/{name}.credentials.json` → fail with `NotFound` if absent.
2. Write contents to a temp file adjacent to `~/.claude/.credentials.json`.
3. Rename temp file to `~/.claude/.credentials.json` — atomic on same filesystem (POSIX rename semantics).
4. Write account name to `{credential_store}/_active`.

**Atomicity guarantee:** The rename in step 3 ensures that a crash between steps 2 and 4 leaves either the old credentials or the new ones in place — never a partially-written file. Step 4 (_active marker) is a best-effort metadata update; a crash after step 3 leaves the credentials correct but the marker stale.

**Dry-run mode** (`dry::1`): Print `[dry-run] would switch to '{name}'` without modifying any files.

**Exit codes:**
- 0: success
- 1: invalid name characters (usage error)
- 2: account not found (runtime error)

### Acceptance Criteria

- **AC-01**: `clp .account.switch name::alice@home.com` exits 0, `~/.claude/.credentials.json` contains alice@home.com's credentials, `_active` contains `alice@home.com`.
- **AC-02**: `clp .account.switch name::ghost@example.com` (no such account) exits 2 with actionable error.
- **AC-03**: Concurrent crash during rename leaves credentials in valid state (never partial write).
- **AC-04**: `clp .account.switch name::alice@home.com dry::1` exits 0 with `[dry-run]` prefix; no files changed.

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| source | `src/account.rs` | `switch_account()` — read, temp write, atomic rename, _active update |
| source | `src/commands.rs` | `account_switch_routine()` — CLI handler |
| test | `tests/account_tests.rs::switch_account_overwrites_credentials_file` | Verifies atomic overwrite + _active update |
| doc | [invariant/005_atomic_switching.md](../invariant/005_atomic_switching.md) | Atomicity invariant for this feature |
| doc | [cli/commands.md](../cli/commands.md#command--6-accountswitch) | CLI command specification |
