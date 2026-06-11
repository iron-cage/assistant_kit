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
5. Patch `~/.claude.json["oauthAccount"]`: unconditionally set `oauthAccount.emailAddress = name` — this field must always reflect the switched-to account regardless of metadata file state (BUG-254 fix). When `{credential_store}/{name}.json` exists and contains a parseable `oauthAccount` snapshot, restore the full block into `~/.claude.json` with `emailAddress` enforced to `name` and `organizationName`/`organizationUuid` overridden from org identity metadata if present and non-empty (BUG-219 fix). When `{name}.json` is absent or unparseable, only the `emailAddress` field is patched — all other `oauthAccount` fields retain their previous values. All other keys in `~/.claude.json` (machine-global state: `commands.*`, `mcpServers`, `projects`) are preserved untouched.
6. Best-effort restore model preference: read `model` from `{credential_store}/{name}.json`; if present, set `model` in `~/.claude/settings.json` to that value (creating or patching the file); if `{name}.json` lacks a `model` field, delete the `model` key from `~/.claude/settings.json` if it exists (clearing stale model from prior account). All other keys in `~/.claude/settings.json` are preserved untouched. Missing or malformed files at either location are silently skipped.

**BUG-219 ✅ Fixed (TSK-221):** `organizationName` and `organizationUuid` inside `oauthAccount` are now overridden from org identity metadata in `{name}.json` after the BUG-217 `emailAddress` insert. Best-effort: silently skipped if absent or malformed. `displayName` and `accountUuid` are left as-is (low display impact).

**BUG-254 fix:** `emailAddress` patch lifted out of the metadata-file-conditional block — now fires unconditionally. When `{name}.json` is absent or unparseable, `switch_account` still patches `~/.claude.json oauthAccount.emailAddress = name`. This prevents stale emailAddress from propagating to subsequent `save()` name inference (BUG-212 downstream dependency).

**Atomicity guarantee:** The rename in step 3 ensures that a crash between steps 2 and 4 leaves either the old credentials or the new ones in place — never a partially-written file. Step 4 (active marker) is a best-effort metadata update; step 5 is a best-effort `oauthAccount` patch. A crash after step 3 always leaves the credentials correct; the marker and companion files may be stale but are not load-bearing for authentication.

**Model preference restore (BUG-222 fix):** `switch_account()` reads `model` from `{credential_store}/{name}.json` and updates `~/.claude/settings.json` accordingly. When the snapshot contains a `model` field, that value is written into `~/.claude/settings.json`, restoring the per-account preference. When the snapshot lacks `model`, the `model` key is removed from `~/.claude/settings.json` to prevent the prior account's model from persisting. All other keys in `~/.claude/settings.json` are preserved.

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
- **AC-07**: `.account.use name::alice@acme.com` patches `~/.claude.json oauthAccount.emailAddress` to `'alice@acme.com'` regardless of what the `alice@acme.com.json` snapshot contains — the account name always wins over stored snapshot data. Additionally, `oauthAccount.organizationName` and `oauthAccount.organizationUuid` are overridden from org identity metadata in `alice@acme.com.json` when present and non-empty (BUG-219 fix — prevents stale cross-account org identity from propagating to `~/.claude.json`).
- **AC-08**: `.account.use name::alice@acme.com` when `{credential_store}/alice@acme.com.json` contains `{"model": "sonnet"}` writes `"model": "sonnet"` into `~/.claude/settings.json` while preserving all other keys; when `alice@acme.com.json` lacks `model`, removes the `model` key from `~/.claude/settings.json` (clearing stale model preference from prior account).
- **AC-09**: `.account.use name::bob@acme.com` when `{credential_store}/bob@acme.com.json` does not exist still patches `~/.claude.json oauthAccount.emailAddress` to `"bob@acme.com"`. All other `oauthAccount` fields and machine-global keys in `~/.claude.json` are preserved. (BUG-254 regression guard.)

### Bugs

| File | Relationship |
|------|--------------|
| `task/claude_profile/bug/213_account_use_switches_to_expired_token_silently.md` | BUG-213 ✅ Fixed by TSK-216: expiry guard inserted in `account_use_routine()` before `switch_account()`; exits 3 when `now_ms > expiresAt` on the fetch-failed path (→ Feature 027 AC-17) |
| `task/claude_profile/bug/217_switch_account_corrupts_claude_json_with_stale_snapshot_emailaddress.md` | BUG-217 🟢 Fixed: `switch_account()` now enforces `emailAddress == name` before inserting `oauthAccount`; `oauth["emailAddress"] = name` assignment added at `account.rs:335` |
| `task/claude_profile/bug/219_switch_account_stale_oauthaccount_org_fields.md` | BUG-219 ✅ Fixed by TSK-221: `switch_account()` now reads org identity from `{name}.json` and overrides `organizationName` + `organizationUuid` after the BUG-217 `emailAddress` insert |
| `task/claude_profile/bug/222_switch_account_model_preference_not_restored.md` | BUG-222 ✅ Fixed (TSK-234): `switch_account()` now reads `{name}.json` and restores/clears `model` in `~/.claude/settings.json` (step 6) |
| `task/claude_profile/bug/254_switch_account_skips_email_patch_when_metadata_file_absent.md` | BUG-254 ✅ Fixed (TSK-255): `switch_account()` emailAddress patch lifted out of metadata-file-conditional block — now fires unconditionally; AC-09 regression guard added |

### Commands

| File | Relationship |
|------|--------------|
| [command/001_account.md](../cli/command/001_account.md#command--5-accountuse) | CLI command specification |

### Features

| File | Relationship |
|------|--------------|
| [008_auto_rotate.md](008_auto_rotate.md) | Auto rotation primitive that uses `switch_account()` for the actual switch |
| [015_name_shortcut_syntax.md](015_name_shortcut_syntax.md) | Positional and prefix shortcut for `name::` on this command |
| [025_per_machine_active_marker.md](025_per_machine_active_marker.md) | Per-machine active marker written in step 4 of `switch_account()` |
| [027_account_use_post_switch_touch.md](027_account_use_post_switch_touch.md) | Post-switch subprocess activation of idle 5h session window; AC-17 adds expiry guard before switch |
| [032_account_assign.md](032_account_assign.md) | Marker-only write for any `USER@MACHINE` pair — contrast with full credential rotation here |
| [034_explicit_session_model_override.md](034_explicit_session_model_override.md) | Explicit session model override — `set_model::` runs after switch completes |

### Invariants

| File | Relationship |
|------|--------------|
| [invariant/005_atomic_switching.md](../invariant/005_atomic_switching.md) | Atomicity invariant for this feature |

### Referenced Commands

| # | Command | Role |
|---|---------|------|
| 1 | [`.account.use`](../cli/command/001_account.md#command--5-accountuse) | CLI surface for this feature |

### Sources

| File | Relationship |
|------|--------------|
| `src/account.rs` | `switch_account()` — read, temp write, atomic rename, active marker update, best-effort `oauthAccount` patch in `~/.claude.json` with `emailAddress` enforced to equal `name` |
| `src/commands/account_ops.rs` | `account_use_routine()` — CLI handler |

### Tests

| File | Relationship |
|------|--------------|
| `tests/cli/account_mutations_test.rs` (aw01–aw11) | Verifies atomic overwrite, active marker update, dry-run, path-unsafe char rejection, edge cases |
| `tests/cli/account_mutations_test.rs::switch_restores_claude_json` | Verifies `~/.claude.json` restored after switch (issue-122) |
