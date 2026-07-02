# Subprocess: Credential Write-Back Protocol

### Scope

- **Purpose**: Define how refreshed OAuth credentials flow from an isolated subprocess back to the credential store and live session.
- **Responsibility**: Authoritative protocol for credential write-back in `refresh_account_token()` — step sequence, safety rules, expiry derivation, and RT rotation mechanism.
- **In Scope**: `refresh_account_token()` protocol steps; live session write prohibition (BUG-221 fix); post-refresh expiry derivation; RT rotation via `expiresAt=1`; post-rotation live sync (BUG-310 fix).
- **Out of Scope**: `run_isolated()` API contract (→ `subprocess/001`); per-invocation trigger predicates (→ `subprocess/003–004`); credential file format (→ `schema/001`).

### Purpose

Define how refreshed OAuth credentials flow from an isolated subprocess back to the credential store and (when applicable) the live session, without disrupting the user's active Claude session.

### Protocol Steps (via `refresh_account_token()`)

```
1. read credentials   — load {name}.credentials.json from credential store
2. manipulate         — set expiresAt: "1" to force Claude CLI to treat token as expired (AC-32)
3. run_isolated       — spawn Claude with manipulated creds; collect IsolatedRunResult
4. write credentials  — if credentials=Some(new_json): account::save() writes new_json
                         to {name}.credentials.json only (never to ~/.claude/.credentials.json)
5. save metadata      — save() updates {name}.json (oauthAccount snapshot, org identity)
                         with update_marker=false — _active marker never written during cycling
```

### Key Safety Rule: Never Write to Live Session File

`~/.claude/.credentials.json` is NEVER written during batch refresh or touch. Writing to the live file was BUG-221 (fixed TSK-230). The `save()` path that receives `creds: Some(new_json)` writes directly to `{name}.credentials.json` only.

**Exception:** When `paths` is `Some` AND the account being refreshed is the current account, `refresh_account_token()` optionally syncs live→store as a consistency check (AC-33) — but the direction is always live→store, never store→live (except BUG-310 fix: after `apply_touch` the store is copied to live for the rotated account).

### Expiry Derivation (Post-Refresh)

After `credentials = Some(new_json)`, derive `expires_at_ms` via two-step fallback:

```
1. jwt_exp_ms(new_json.accessToken)    — preferred for JWT-format tokens (decode "exp" claim)
2. parse_u64_field(new_json, "expiresAt")  — fallback for opaque sk-ant-oat01-* tokens
3. unchanged                           — last-resort if both strategies fail
```

`expiresAt` in the credentials file is NOT updated by the subprocess (BUG-162) — the subprocess only writes `accessToken` and `refreshToken`. Post-refresh expiry MUST be derived from the new token content.

### RT Rotation Prevention

`refresh_account_token()` sets `expiresAt: "1"` in the in-memory credential copy before passing to `run_isolated`. Without this, Claude CLI would use the valid `accessToken` as-is (no refresh needed), returning `credentials = None` — the `refreshToken` would age silently until server-side expiry. Setting `expiresAt: "1"` forces the CLI to treat the AT as expired, perform a full RT→AT+RT exchange, and rotate the RT on every refresh_account_token call.

### Post-Rotation Live Sync (BUG-310 Fix)

After `.usage rotate::1` calls `switch_account(winner)` then `apply_touch(winner)`:
- `switch_account(winner)` copies stored creds → live
- `apply_touch(winner)` may refresh the token, writing ONLY to store
- BUG-310: live session retains stale `token_A` while store has `token_B`
- Fix (AC-11 Feature 038): after `apply_touch`, `std::fs::copy(store → live)` re-syncs for the current-account case

### Features

| File | Relationship |
|------|-------------|
| [feature/017_token_refresh.md](../feature/017_token_refresh.md) | AC-20/AC-25/AC-29/AC-32/AC-33 (write-back, expiry, live safety) |

### Schema

| File | Relationship |
|------|-------------|
| [schema/001](../schema/001_credentials_json.md) | `{name}.credentials.json` format |

### Subprocess

| File | Relationship |
|------|-------------|
| [subprocess/001](001_run_isolated_contract.md) | `run_isolated()` API and `IsolatedRunResult` |
| [subprocess/003](003_token_refresh_invocation.md) | Token refresh invocation |
| [subprocess/004](004_session_touch_invocation.md) | Session touch invocation |

### Invariants

| File | Relationship |
|------|-------------|
| [invariant/008](../invariant/008_single_token_refresh_entry.md) | Single-entry-point invariant |
