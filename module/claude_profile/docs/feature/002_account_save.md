# Feature: Save Account

### Scope

- **Purpose**: Snapshot the current active credentials as a named account profile for later restoration.
- **Responsibility**: Documents the `account::save()` API and the `.account.save` CLI command (FR-7).
- **In Scope**: Credential copy, `oauthAccount` snapshot from `~/.claude.json`, org identity snapshot (`{name}.json` via endpoint 005), name validation, directory init, CLI dry-run behaviour, idempotent re-save as metadata refresh. Ownership-neutral: `account_save_routine()` passes `owner: None` to `save()` — the `owner` field in `{name}.json` is always preserved unchanged. Ownership assignment and release are dedicated operations on `.accounts` (see Feature 036, Feature 037).
- **Out of Scope**: Account switching (→ 004_account_use.md), store initialization (→ 001_account_store_init.md).

### Design

`claude_profile` must copy `~/.claude/.credentials.json` to `{credential_store}/{name}.credentials.json`, creating or overwriting the named entry. It
also extracts the `oauthAccount` subtree from `~/.claude.json` and writes it into `{name}.json` — containing `{"oauthAccount": {...}}` alongside other
per-account metadata — to preserve account identity for later enumeration by `.accounts`, except that a re-save can keep the slot's stored
`oauthAccount` instead (step 6a). Machine-global state (`commands.*`, `mcpServers`, `projects`) is never captured in the per-account snapshot. The
credential store path is resolved per FR-6 (see `001_account_store_init.md`).

**Name resolution** — when `name::` is omitted, the one saved slot already holding the live session names it: the `{name}.credentials.json` whose
`refreshToken` equals the live one, or whose `accessToken` does when the live file is a redirect key, which carries no refresh token
(`slots_holding_session()`, BUG-582). A running Claude process rewrites `~/.claude.json` from its own profile, so after a switch
`oauthAccount.emailAddress` can still name the previous account, while the slot match follows the credential itself; a live redirect key names the slot
holding it, so step 1a skips the save. When no single slot holds the session (a fresh login, a session refreshed outside clp since its last save, or two slots
holding it), `oauthAccount.emailAddress` from `~/.claude.json` is the next source (this field is updated by both clp account switches and external OAuth
login). If absent or empty, the per-machine active marker file (`_active_{hostname}_{user}` — see Feature 025) is the fallback. If no source provides a
name, exits 1 with a clear message directing the user to pass `name::` explicitly.

**One session, one slot** — a save that copies the live session never files it under a second slot. Before copying it (no `creds` given), `save()` asks
`slots_holding_session()` for every slot holding the live session: the same `claudeAiOauth.refreshToken`, or the same `accessToken` for a redirect key. When one of them
isn't `name`, it refuses with `live session is already saved as '{holder}' — refusing to also save it as '{name}'` before writing any account file (the
store directory and its lock file may already exist by then), and `.account.save` exits 2. Re-saving the holder's own slot, and saving a session no slot
holds yet, proceed as before. `dry::1` previews the resolved name only; the refusal comes from the real save.

**Known limitation** — refresh tokens are single-use, so a slot matches the live session only until that session is refreshed outside clp (e.g. by a
running Claude process). From then on its new token matches no slot: the guard above can't place it, and a bare save falls back to `~/.claude.json` and
the marker as before, so a stale `~/.claude.json` can still file it under the wrong name (BUG-582). Token refresh isn't guarded either:
`refresh_account_token()` copies a live session that differs from the stored one into the slot this machine's active marker names, and hands
`save()` those bytes as `creds`, which it doesn't check. `.accounts assignee::` or a login outside clp can leave that marker naming another account
than the live session, so a refresh can still put one session in two slots, or in only the wrong one, which then counts as its holder: a bare save names
that slot and keeps its stored identity (step 6a), and a save under the session's own name is refused. A redirect key re-saved with a new `api_key::`
while it's live leaves the old key live and held by no slot, so the same fallback applies to it.

**Name validation** — account names must be valid email addresses:
- Non-empty
- Must contain `@` with non-empty local part and domain

**Operation steps:**
1. Resolve name: use explicit `name::` if provided; otherwise the one saved slot holding the live session (its `refreshToken`, or a redirect key's
   `accessToken`); else read `oauthAccount.emailAddress` from `~/.claude.json`; if absent or empty, fall back to the per-machine active marker file
   (`{credential_store}/_active_{hostname}_{user}` via `active_marker_filename()` — see Feature 025); if no source provides a non-empty name, exit 1.
1a. Redirect skip gate: if `backend::` and `preset::` are both absent and the resolved name's stored `{name}.json` holds `backend: redirect`, exit 0 with a skip notice and touch no store file — a redirect account holds a static key with no live OAuth session to snapshot, and proceeding would re-backend it destructively (see [071](071_redirect_backend_accounts.md) AC-19, BUG-549).
2. Validate `name` against the rules above (exit 1 on violation).
3. Resolve credential store directory and ensure it exists (`create_dir_all` — see FR-6).
4. Read `~/.claude/.credentials.json`.
4a. Session guard (Anthropic backend, copying the live file): if a slot other than `name` holds the live session (the same `refreshToken`, or the same
    `accessToken` for a redirect key), refuse with `live session is already saved as '{holder}' — refusing to also save it as '{name}'`; no account file
    is written — not the credentials, `{name}.json` or the marker (BUG-582). A re-backend from `redirect` (071 AC-15) runs this check before it resets
    `{name}.json`.
5. Write contents to `{credential_store}/{name}.credentials.json` (creates or overwrites).
6. Extract `oauthAccount` subtree from `~/.claude.json`. If `{credential_store}/{name}.json` already exists, read its current content and merge: update the `oauthAccount` key while preserving all other existing keys (notably `_renewal_at` written by `.account.renewal`). Write the merged result to `{credential_store}/{name}.json` (best-effort: skip silently if source absent or `oauthAccount` key missing; merge failure silently falls back to overwrite).
6a. Stored identity on a re-save (Anthropic backend, copying the live file): when step 4a found the live session already in `name`'s own slot,
    `{name}.json` already records an `oauthAccount.emailAddress`, and `~/.claude.json` names neither that account nor `name` (or names none), step 6
    keeps the stored `oauthAccount`: a running Claude process can leave `~/.claude.json` naming the previous account (BUG-582). A session no slot holds
    still takes `~/.claude.json`'s.
7. Capture model preference (anthropic backend only): read `model` field from `~/.claude/settings.json`; if present, write `{"model": "<value>"}` into `{credential_store}/{name}.json` (merged with existing content); if `~/.claude/settings.json` is absent or lacks a `model` field, skip silently — save still succeeds. This enables `switch_account()` to restore the per-account model preference on next switch. For `backend: redirect` saves the live pin belongs to whichever account was previously active, so the snapshot is skipped and any stray `model` field already in `{name}.json` is removed (see [071](071_redirect_backend_accounts.md) AC-18).
8. Call `claude_quota::fetch_claude_cli_roles(&access_token)` (feature-gated, best-effort): on success, write response into `{credential_store}/{name}.json`; on any failure, skip silently — save still succeeds.
9. Write `{credential_store}/_active_{hostname}_{user}` = `{name}` via `active_marker_filename()` — mark the saved account as the current active account (per-machine marker, see feature 025) (when invoked from `.account.save` or `.account.relogin`; background refresh callers pass `update_marker=false` and do not write the marker).

**Idempotency as metadata refresh:** `save()` overwrites all snapshot data on every invocation, except an `oauthAccount` step 6a keeps. Re-running
`clp .account.save` for an existing account name re-fetches endpoint 005 and overwrites org identity fields in `{name}.json` alongside all other metadata.
This is the canonical mechanism for refreshing cached metadata when org membership or role changes — no separate command required.

**Model preference capture (BUG-222 fix):** `save()` reads the `model` field from `~/.claude/settings.json` and writes it into `{credential_store}/{name}.json`. This snapshot enables `switch_account()` to restore the per-account model preference on the next switch. If `~/.claude/settings.json` is absent or lacks a `model` field, no model field is written and save still succeeds. Anthropic backend only: `backend: redirect` saves never capture the live pin (it is another account's preference) and remove a stray pre-gate `model` field — see [071](071_redirect_backend_accounts.md) AC-18.

**Ownership-neutral save:** `account_save_routine()` passes `owner: None` to `save()`, which preserves any existing `owner` field in `{name}.json` via read-merge — identical to the behaviour of background refresh callers. `.account.save` never writes to the `owner` field. Ownership assignment (`write_owner(name, store, identity)`) and release (`write_owner(name, store, "")`) are separate explicit operations on `.accounts` (see Feature 036, Feature 037). `.account.assign` is also ownership-neutral — marker-only write.

**Dry-run mode** (`dry::1`): Print `[dry-run] would save current credentials as '{name}'` without modifying any files.

### Acceptance Criteria

- **AC-01**: `clp .account.save name::alice@acme.com` exits 0 and creates `{credential_store}/alice@acme.com.credentials.json`.
- **AC-02**: `clp .account.save name::` (empty) exits 1 with `account name must not be empty`.
- **AC-03**: `clp .account.save name::notanemail` exits 1 with `must be an email address`.
- **AC-04**: `clp .account.save name::alice@acme.com dry::1` prints `[dry-run] would save current credentials as 'alice@acme.com'` and creates no files.
- **AC-05**: When `~/.claude.json` contains an `oauthAccount` key, `{credential_store}/{name}.json` is created alongside the credential file, containing `{"oauthAccount": {...}}`.
- **AC-07**: When `~/.claude.json` is absent or lacks an `oauthAccount` key, no `oauthAccount` snapshot is written to `{name}.json` — save still succeeds.
- **AC-08**: `clp .account.save` (no `name::`) names the save after the one saved slot already holding the live session (matching `refreshToken`, or
  `accessToken` for a redirect key); when no single slot does, it reads `oauthAccount.emailAddress` from `~/.claude.json`; if absent, falls back to the
  per-machine active marker file; saves normally with output `saved current credentials as '{name}'`.
- **AC-09**: `clp .account.save` (no `name::`) when no single saved slot holds the live session and both `oauthAccount.emailAddress` in `~/.claude.json`
  and the per-machine active marker file are absent or empty exits 1 with `cannot infer account name: no active account set — pass name:: explicitly`.
- **AC-10**: After a successful `clp .account.save`, `{credential_store}/_active_{hostname}_{user}` (the per-machine active marker) contains the saved account name; `clp .credentials.status` shows `Account: {name}` immediately.
- **AC-11**: `clp .account.save name::a/b@c.com` exits 1 — path-unsafe characters (`/`, `\`, `*`) in the email local part are rejected by `validate_name()` before any filesystem operation.
- **AC-12**: When endpoint 005 responds successfully, org identity fields are written to `{credential_store}/{name}.json` alongside the credential file.
- **AC-13**: When endpoint 005 fails (network error, scope issue, or feature not enabled), no org identity fields are written to `{name}.json` and save still exits 0.
- **AC-14**: Re-running `clp .account.save` for an existing account name overwrites org identity fields in `{name}.json` with a fresh endpoint 005 response; this is the metadata refresh mechanism.
- **AC-15**: Background refresh calls to `save()` (via `refresh_account_token`) pass `update_marker=false`; the per-machine active marker is not written; any concurrent `.account.use` switch is not disturbed.
- **AC-16**: `clp .account.save` (no `name::`) when `~/.claude.json` contains `oauthAccount.emailAddress = "i5@example.com"`, the per-machine active marker
  contains `"i2@example.com"` (stale from a prior clp session) and no saved slot holds the live session saves credentials as `i5@example.com`;
  `i2@example.com.credentials.json` is NOT created or modified. (BUG-212 regression guard.)
- **AC-17**: When `{credential_store}/{name}.json` already exists and contains a `_renewal_at` key, re-running `clp .account.save` preserves the
  `_renewal_at` value in the updated snapshot (read-merge, not full overwrite); the `oauthAccount` key is updated unless step 6a keeps the stored one,
  and all other keys are retained.
- **AC-18**: `clp .account.save name::alice@acme.com` when `~/.claude/settings.json` contains `{"model": "sonnet"}` writes `{"model": "sonnet"}` into `{credential_store}/alice@acme.com.json`. When `~/.claude/settings.json` is absent or has no `model` field, no model field is written to `alice@acme.com.json` and exit code is still 0. (Anthropic backend only — redirect saves never capture `model`; see [071](071_redirect_backend_accounts.md) AC-18.)
- **AC-19**: `clp .account.save name::alice@acme.com` does NOT modify the `owner` field in `alice@acme.com.json` — `account_save_routine()` passes `owner: None` to `save()`, preserving any existing value via read-merge. Background refresh callers also pass `owner: None` (same behaviour). Ownership assignment and release are explicit operations on `.accounts` (see Feature 036, Feature 037).
- **AC-20**: After `clp .account.use name::bob@example.com`, with `~/.claude.json` rewritten to name `alice@example.com` (as a Claude process still running
  under alice does), `clp .account.save` (no `name::`) saves as `bob@example.com`, the slot already holding the live session. `alice@example.com.credentials.json`
  stays byte-identical, the marker names bob, and `bob@example.com.json` keeps bob's `oauthAccount` (step 6a). (BUG-582 regression guard.)
- **AC-21**: `clp .account.save name::alice@example.com` while `bob@example.com`'s slot already holds the live session exits 2 with
  `live session is already saved as 'bob@example.com' — refusing to also save it as 'alice@example.com'`; alice's slot and the marker are unchanged. When two
  slots already hold the live session, every save of it that reaches step 4a refuses this way, whatever name it resolves; one resolving to a stored redirect
  slot without `backend::` stops at step 1a's skip instead. Re-saving bob's own slot, and saving a fresh login no slot holds, still succeed.
  A `backend::anthropic` re-save of a redirect slot refused this way keeps that slot's `{name}.json` and key. (BUG-582 regression guard.)
- **AC-22**: After `clp .account.use name::kimi` (a redirect slot), with `~/.claude.json` rewritten to name `alice@example.com`, `clp .account.save`
  (no `name::`) names the save `kimi`, the slot holding the live key, and skips it (step 1a); `clp .account.save name::alice@example.com` exits 2 with
  `live session is already saved as 'kimi' — refusing to also save it as 'alice@example.com'`. Alice's slot stays byte-identical. (BUG-582 regression guard.)

### Bugs

| File | Relationship |
|------|--------------|
| BUG-222 | BUG-222 ✅ Fixed (TSK-234): `save()` now writes model to `{name}.json` (step 7); `switch_account()` restores model on switch |
| BUG-549 | BUG-549 ✅ Fixed: bare save (no `backend::`/`preset::`) on a stored-redirect target silently re-backended it to anthropic (absent `backend::` → type default) and destroyed the redirect record via 071 AC-15's delete-and-rewrite; step 1a's redirect skip gate (071 AC-19) now exits 0 without touching the store |
| BUG-582 | BUG-582 ✅ Fixed: a bare save filed the live session under whichever account `~/.claude.json` named; the holding slot now names it (step 1), and step 4a refuses one another slot holds (AC-20 to AC-22) |

### Related Commands

| File | Relationship |
|------|--------------|
| [036_account_ownership.md](036_account_ownership.md) | ownership model — `owner::0` clears ownership (Feature 064; formerly `.account.unclaim`); G1–G8 enforcement gates |

### Commands

| File | Relationship |
|------|--------------|
| [command/001_account.md](../cli/command/001_account.md#command-4-accountsave) | CLI command specification |

### Features

| File | Relationship |
|------|--------------|
| [001_account_store_init.md](001_account_store_init.md) | Directory initialization triggered by save |
| [014_rich_account_metadata.md](014_rich_account_metadata.md) | Metadata fields snapshotted by `save()` |
| [022_org_identity_snapshot.md](022_org_identity_snapshot.md) | Org identity metadata lifecycle and org fields in `{name}.json` |
| [025_per_machine_active_marker.md](025_per_machine_active_marker.md) | Per-machine marker naming convention used in step 8 |
| [030_account_renewal_override.md](030_account_renewal_override.md) | `_renewal_at` field written by `.account.renewal`; preserved by `save()` read-merge (AC-17) |
| [036_account_ownership.md](036_account_ownership.md) | Ownership model — `.account.save` is ownership-neutral (owner: None); `owner::0` clears ownership (Feature 064; formerly `.account.unclaim`); `assignee::USER@MACHINE` is marker-only (Feature 065; formerly `.account.assign`) |
| [032_account_assign.md](032_account_assign.md) | `.account.assign` — marker-only write; does NOT call `write_owner()`; does NOT modify `owner` field in `{name}.json` |

### Referenced Commands

| # | Command | Role |
|---|---------|------|
| 1 | [`.account.save`](../cli/command/001_account.md#command-4-accountsave) | CLI surface for this feature |

### Sources

| File | Relationship |
|------|--------------|
| `claude_profile_core/src/account/store.rs` | `save()` implementation — validate, init dir, copy credentials |
| `src/commands/account_ops.rs` | `account_save_routine()` — CLI handler |

### Tests

| File | Relationship |
|------|--------------|
| `tests/cli/accounts_list_test.rs` | Verifies credential file and metadata snapshots created with correct content |
| `claude_profile_core/tests/account_test.rs` | `as_save_writes_active_marker` — unit test: active marker written after `save()` |
| `tests/cli/credentials_test.rs` | `cred14` — CLI: `.credentials.status` shows `Account: {name}` after `.account.save` |
| `tests/cli/account_mutations_test.rs` | `as15` — name inferred from active marker (AC-08) |
| `tests/cli/account_mutations_test_b.rs` | `as16` — active marker written after save (AC-10); `as17`/`as18` — path-unsafe chars in local part exit 1 (AC-11) |
| `tests/cli/account_relogin_test_b.rs` | `mre_bug_209_account_save_uses_active_marker_not_stale_email` — BUG-209 regression: stale `emailAddress` ignored (AC-08) |
| `tests/cli/account_relogin_test_b.rs` | `mre_bug_212_account_save_stale_marker_uses_oauth_email` — BUG-212 regression: stale `_active` overridden by `oauthAccount.emailAddress` (AC-16) |
| `tests/cli/account_relogin_test_b.rs` | `mre_bug_582_bare_save_files_live_session_under_its_own_slot` — BUG-582 regression: the holding slot names a bare save (AC-20) |
| `tests/cli/account_relogin_test_b.rs` | `bug_582_explicit_save_refuses_session_another_slot_holds`, `bug_582_bare_save_refuses_when_two_slots_hold_live_session`, `bug_582_refused_rebackend_save_keeps_redirect_record` — refusals (AC-21) |
| `claude_profile_core/tests/account_test.rs` | `mre_bug_582_save_refuses_session_another_slot_holds`, `bug_582_guard_allows_own_slot_and_unheld_session` — the store guard (AC-21) |
| `claude_profile_core/tests/account_test.rs` | `slots_holding_session_names_every_holder_in_name_order` — the slot lookup behind step 1 and step 4a |
| `tests/cli/account_relogin_test_b.rs` | `bug_582_live_redirect_key_never_filed_under_another_slot` — a live redirect key names its own slot, never alice's (AC-22) |
| `claude_profile_core/tests/account_test.rs` | `bug_582_live_redirect_key_matched_by_access_token` — lookup and guard match a redirect key by `accessToken` (AC-22) |
| `claude_profile_core/tests/account_test.rs` | `bug_582_resave_keeps_slot_identity_when_claude_json_names_another` — a re-save of the holding slot keeps its `oauthAccount` (step 6a, AC-20) |
| [tests/docs/feature/002_account_save.md](../../tests/docs/feature/002_account_save.md) | FT-01...FT-19 test case planning for Feature 002 |

### Schema

| File | Relationship |
|------|-------------|
| [schema/001_credentials_json.md](../schema/001_credentials_json.md) | `{name}.credentials.json` written at step 5 — credential file schema |
| [schema/002_account_json.md](../schema/002_account_json.md) | `{name}.json` written at steps 6-8 — metadata snapshot field table |
