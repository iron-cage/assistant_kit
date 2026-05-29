# Feature: Switch Account

### Scope

- **Purpose**: Atomically rotate the active credential set to a named account without credential corruption risk.
- **Responsibility**: Documents the `account::switch_account()` API and `.account.use` CLI command (FR-9).
- **In Scope**: Atomic write-then-rename, active marker (`_active_{hostname}_{user}`) update, best-effort `oauthAccount` patch in `~/.claude.json`, not-found guard, dry-run.
- **Out of Scope**: Selecting which account to switch to (→ 008_auto_rotate.md), process termination (caller responsibility), post-switch subprocess activation (→ 027_account_use_post_switch_touch.md).

### Design

`claude_profile` must switch the active account by:

1. Read `{credential_store}/{name}.credentials.json` → fail with `NotFound` if absent.
2. Write contents to a temp file adjacent to `~/.claude/.credentials.json`.
3. Rename temp file to `~/.claude/.credentials.json` — atomic on same filesystem (POSIX rename semantics).
4. Write account name to `{credential_store}/_active_{hostname}_{user}` (per-machine marker via `active_marker_filename()`).
5. Best-effort patch `~/.claude.json["oauthAccount"]` from `{credential_store}/{name}.claude.json`, enforcing `oauthAccount.emailAddress = name` after extraction regardless of what the snapshot contains; all other keys in `~/.claude.json` (machine-global state: `commands.*`, `mcpServers`, `projects`) are preserved untouched. Missing snapshot is silently skipped.

**BUG-219 (open):** `organizationName`, `displayName`, and `organizationUuid` inside `oauthAccount` are copied verbatim from the snapshot and NOT enforced to reflect the switched-to account. When a snapshot was captured while a different account's session was active, these org-identity fields carry stale cross-account data, causing Claude Code's native `/usage` display to show the wrong organization name. See bug/219 for fix location and prevention.

**Atomicity guarantee:** The rename in step 3 ensures that a crash between steps 2 and 4 leaves either the old credentials or the new ones in place — never a partially-written file. Step 4 (active marker) is a best-effort metadata update; step 5 is a best-effort `oauthAccount` patch. A crash after step 3 always leaves the credentials correct; the marker and companion files may be stale but are not load-bearing for authentication.

**Dry-run mode** (`dry::1`): Print `[dry-run] would switch to '{name}'` without modifying any files.

**Exit codes:**
- 0: success
- 1: invalid name characters (usage error)
- 2: account not found (runtime error)
- 3: account credentials expired — see [027_account_use_post_switch_touch.md AC-17](027_account_use_post_switch_touch.md) for the expired-token guard added to the post-switch touch path

### Acceptance Criteria

- **AC-01**: `clp .account.use name::alice@home.com` exits 0, `~/.claude/.credentials.json` contains alice@home.com's credentials, `_active_{hostname}_{user}` contains `alice@home.com`.
- **AC-02**: `clp .account.use name::ghost@example.com` (no such account) exits 2 with actionable error.
- **AC-03**: Concurrent crash during rename leaves credentials in valid state (never partial write).
- **AC-04**: `clp .account.use name::alice@home.com dry::1` exits 0 with `[dry-run]` prefix; no files changed.
- **AC-05**: `clp .credentials.status` after `.account.use name::alice@home.com` shows `Email: alice@home.com` (not the previously active account's email).
- **AC-06**: `clp .account.use name::a/b@c.com` exits 1 — path-unsafe characters (`/`, `\`, `*`) in the email local part are rejected by `validate_name()` before any filesystem operation.
- **AC-07**: `.account.use name::alice@acme.com` patches `~/.claude.json oauthAccount.emailAddress` to `'alice@acme.com'` regardless of what the `alice@acme.com.claude.json` snapshot contains — the account name always wins over stored snapshot data. This prevents stale snapshot state from propagating to the shared identity file.

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| source | `src/account.rs` | `switch_account()` — read, temp write, atomic rename, active marker update, best-effort `oauthAccount` patch in `~/.claude.json` with `emailAddress` enforced to equal `name` |
| source | `src/commands.rs` | `account_use_routine()` — CLI handler |
| test | `tests/cli/account_mutations_test.rs` (aw01–aw11) | Verifies atomic overwrite, active marker update, dry-run, path-unsafe char rejection, edge cases |
| test | `tests/cli/account_mutations_test.rs::switch_restores_claude_json` | Verifies `~/.claude.json` restored after switch (issue-122) |
| doc | [invariant/005_atomic_switching.md](../invariant/005_atomic_switching.md) | Atomicity invariant for this feature |
| doc | [command/001_account.md](../cli/command/001_account.md#command--5-accountuse) | CLI command specification |
| doc | [027_account_use_post_switch_touch.md](027_account_use_post_switch_touch.md) | Post-switch subprocess activation of idle 5h session window; AC-17 adds expiry guard before switch |
| bug | `task/claude_profile/bug/213_account_use_switches_to_expired_token_silently.md` | BUG-213 ✅ Fixed by TSK-216: expiry guard inserted in `account_use_routine()` before `switch_account()`; exits 3 when `now_ms > expiresAt` on the fetch-failed path (→ Feature 027 AC-17) |
| bug | `task/claude_profile/bug/217_switch_account_corrupts_claude_json_with_stale_snapshot_emailaddress.md` | BUG-217 🟢 Fixed: `switch_account()` now enforces `emailAddress == name` before inserting `oauthAccount`; `oauth["emailAddress"] = name` assignment added at `account.rs:335` |
| bug | `task/claude_profile/bug/219_switch_account_stale_oauthaccount_org_fields.md` | BUG-219 🔴 Open: BUG-217 partial fix — `organizationName`, `displayName`, `organizationUuid` in `oauthAccount` not enforced; stale org-identity installed in `~/.claude.json` after switch; Claude Code `/usage` shows wrong org |
