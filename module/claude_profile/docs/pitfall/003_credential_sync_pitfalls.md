# Pitfall: Credential Sync Pitfalls

### Pattern

Managing multiple credential copies (live `~/.claude/.credentials.json`, per-account `{name}.credentials.json`, and the in-memory subprocess view) requires careful sequencing. Several bugs stem from the wrong copy being written, read, or discarded.

### Pitfall 1 — `expiresAt` is NOT updated by `run_isolated` (BUG-162)

The isolated subprocess updates `accessToken` and `refreshToken` but does NOT write a new `expiresAt`. Reading `expiresAt` from the file after refresh yields the original expired timestamp — causing `EXPIRED` display even after successful refresh.

**Fix:** Derive post-refresh expiry from the JWT `exp` claim embedded in the new `accessToken` (BUG-162 fix). Fall back to `expiresAt` field from response JSON for opaque `sk-ant-oat01-*` tokens (BUG-170 fix).

**Rule:** Never read `expiresAt` from the credential file after a refresh. Derive it from the new token content.

### Pitfall 2 — Writing to live session file during batch refresh corrupts the user's session (BUG-221)

Early implementations called `switch_account()` or wrote directly to `~/.claude/.credentials.json` during per-account batch refresh. This overwrote whatever account the user currently had active, silently disrupting their Claude session.

**Fix:** `account::save()` gained `creds: Option<&[u8]>` — when provided, writes directly to `{name}.credentials.json` only. The live session file is never touched during batch refresh.

**Rule:** Batch operations (refresh, touch) MUST NOT write to `~/.claude/.credentials.json`. Only `.account.use` and relogin write to the live file.

### Pitfall 3 — Snapshot/restore of `_active` marker creates race conditions (BUG-208, BUG-211)

Earlier `apply_refresh`/`apply_touch` loops snapshotted the current `_active` marker and restored it after per-account processing. This caused races: if two machines simultaneously ran `.usage`, each restore clobbered the other's active account.

**Fix (BUG-211):** Snapshot+restore removed entirely. `save(update_marker=false)` suppresses `_active` writes during per-account cycling. The marker is only written by `.account.use`.

**Rule:** The `_active_{host}_{user}` marker is only written by switch operations. Never write it during batch credential operations.

### Pitfall 4 — Rotation touch leaves live session with stale token (BUG-310)

`.usage rotate::1` sequence: `switch_account(winner)` copies stored creds→live, then `apply_touch(winner)` may call `refresh_account_token()`, which writes new credentials ONLY to the store. Live session retains `token_A` while store has `token_B`. Next `.usage` sees the live session as out-of-date.

**Fix (AC-11 Feature 038):** After `apply_touch`, `std::fs::copy(store → live)` re-syncs the current account's credential file.

**Rule:** After any touch/refresh operation on the account that's being rotated to, re-sync the live credential file from the store.

### Pitfall 5 — Stale `is_active` guard in race-recovery corrupts credential store slot (BUG-316)

`refresh_token_with_live_path` computes `is_active` once by reading `_active_{host}_{user}` at function entry, then reuses that boolean in the race-recovery block ~35 seconds later (after `run_isolated` completes). If a concurrent `switch_account("B")` runs during the subprocess window, the active marker changes to "B" — but `is_active` still holds `true`. Race recovery then reads the live file (now containing B's credentials) and writes B's credentials into A's credential store slot.

**Fix (BUG-316):** Re-read the active marker independently inside the `credentials=None` branch, immediately before the race-recovery guard. Only proceed with race recovery if the freshly-read marker still points to `name`.

**Rule:** Never cache a filesystem-derived boolean across a blocking call (subprocess, network I/O) in a multi-process environment. Re-read the active marker at each use site in `refresh_token_with_live_path` — independently before the pre-sync block and independently before the race-recovery block.

### Cross-References

| File | Relationship |
|------|-------------|
| [subprocess/002](../subprocess/002_credential_writeback.md) | Credential write-back protocol |
| [schema/001](../schema/001_credentials_json.md) | `{name}.credentials.json` fields |
| [state_machine/002](../state_machine/002_oauth_token_lifecycle.md) | Token validity states |
