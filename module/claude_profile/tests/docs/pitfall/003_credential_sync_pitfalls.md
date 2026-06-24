# Pitfall Tests: Credential Sync Pitfalls

Test cases verifying that each guard documented in `docs/pitfall/003_credential_sync_pitfalls.md`
is in place and prevents the described credential corruption failure mode.

**Source:** [docs/pitfall/003_credential_sync_pitfalls.md](../../../docs/pitfall/003_credential_sync_pitfalls.md)
**Case prefix:** `PP-` (Pitfall Protection)

### Pitfall Guard Index

| ID | Pitfall | Bug | Guard Verified By |
|----|---------|-----|-------------------|
| PP-1 | `expiresAt` not updated by subprocess | BUG-162 / BUG-170 | `test_jwt_exp_ms_mre_bug162`, `test_parse_u064_from_str_mre_bug170_extracts_expires_at` |
| PP-2 | Writing to live session file during batch refresh | BUG-221 | `test_apply_refresh_lifecycle_active_marker_unchanged` (FT-13+) |
| PP-3 | Snapshot/restore of `_active` marker creates races | BUG-208 / BUG-211 | `test_apply_refresh_mre_bug208_restore_trace_emitted` (FT-17) |
| PP-4 | Rotation touch leaves stale token in live session | BUG-310 | `cc01_rotate_offline_copies_store_to_live`, `cc02_rotate_touch_offline_syncs_live_after_touch` |
| PP-5 | Stale `is_active` guard in race-recovery (TOCTOU) | BUG-316 | `mre_bug316_stale_is_active_race_recovery_copies_wrong_account_creds` (FT-26) |

---

### PP-1: Pitfall 1 guard — post-refresh expiry derived from JWT, not from stale file field

- **Given:** An isolated subprocess successfully refreshes a credential, writing a new `accessToken`
  to the credential file. The file's `expiresAt` field still contains the old expired timestamp.
- **When:** `refresh_account_token` computes `expires_at_ms` after the subprocess returns.
- **Then:** `expires_at_ms` is derived from the JWT `exp` claim in the new `accessToken` — not from
  the `expiresAt` field in the file. For opaque `sk-ant-oat01-*` tokens where JWT decode returns
  `None`, `expiresAt` from the response JSON is used as the fallback.
- **Note:** BUG-162 / BUG-170 guard. Verified by `test_jwt_exp_ms_mre_bug162` and
  `test_parse_u064_from_str_mre_bug170_extracts_expires_at` in `account_refresh_test.rs`.

---

### PP-2: Pitfall 2 guard — batch refresh never writes the live session file

- **Given:** `apply_refresh` cycles through all saved accounts including the current account.
  `~/.claude/.credentials.json` is the live session file.
- **When:** `refresh_account_token` is called for any account in the batch.
- **Then:** `~/.claude/.credentials.json` is NOT written during the batch refresh cycle. Only
  `{store}/{name}.credentials.json` per-account files are updated via `account::save()` with the
  `creds` parameter; the live file is untouched.
- **Note:** BUG-221 guard. Verified by the `apply_refresh does not write live` assertion
  (FT-13+) in `account_refresh_test.rs`. See also FT-13 (no `switch_account` called).

---

### PP-3: Pitfall 3 guard — `_active` marker never written during batch credential operations

- **Given:** `apply_refresh` cycles through all accounts. Each account is processed in turn.
- **When:** The batch refresh loop runs.
- **Then:** The `_active_{host}_{user}` marker file is NOT written at any point during the cycle.
  `save(update_marker=false)` suppresses marker writes during per-account processing. The marker
  state before and after the cycle is byte-identical.
- **Note:** BUG-208 / BUG-211 guard. Verified by `test_apply_refresh_mre_bug208_restore_trace_emitted`
  (FT-17) in `account_refresh_test.rs`. The snapshot+restore pattern was removed in BUG-211.

---

### PP-4: Pitfall 4 guard — live session re-synced from store after rotation touch

- **Given:** `.usage rotate::1` runs the rotation sequence: `switch_account(winner)` copies
  stored credentials to the live session file, then `apply_touch(winner)` may call
  `refresh_account_token()`, which writes refreshed credentials to the store only.
- **When:** The rotation sequence completes.
- **Then:** `std::fs::copy(store_path → live_path)` is called after `apply_touch` — the live
  session file is overwritten with the (potentially refreshed) token from the store. No stale
  `token_A` remains in the live session after the rotation.
- **Note:** BUG-310 guard (Feature 038 AC-11). Verified by `cc01_rotate_offline_copies_store_to_live`
  and `cc02_rotate_touch_offline_syncs_live_after_touch` in `usage_test.rs`.

---

### PP-5: Pitfall 5 guard — active marker re-read after `run_isolated` window (BUG-316 MRE)

- **Given:** `refresh_token_with_live_path("A", store, ...)` is entered. The `_active_{host}_{user}`
  marker initially contains `"A"`. During the ~35-second `run_isolated` window, a concurrent
  `switch_account("B")` overwrites the marker with `"B"` and overwrites the live credentials file
  with B's JSON. `run_isolated` returns `credentials=None`.
- **When:** The race-recovery block (`credentials=None` arm) executes.
- **Then:** The active marker is re-read fresh inside the guard. The fresh read returns `"B"`, not
  `"A"`. `is_active_now == false`. Race recovery does NOT copy the live file to the store.
  `A.credentials.json` in the store is unchanged — B's credentials are NOT written into A's slot.
- **Note:** BUG-316 guard. Verified by `mre_bug316_stale_is_active_race_recovery_copies_wrong_account_creds`
  in `claude_profile_core/tests/account_refresh_test.rs`. See also FT-26 in
  `tests/docs/feature/17_token_refresh.md` for full behavioral description.
