# Schema 001: Credentials JSON — `{name}.credentials.json`

SC test cases for `docs/schema/001_credentials_json.md`. Verifies the on-disk format
contract of the per-account OAuth credential snapshot: required fields, encoding format,
write callers, read callers, and isolation from the live session credential file.

**Source:** [docs/schema/001_credentials_json.md](../../../../docs/schema/001_credentials_json.md)

### SC Case Index

| ID | Short Name | Category | Status |
|----|------------|----------|--------|
| SC-1 | Save copies all 3 credential fields to named file | Field Presence | ✅ |
| SC-2 | accessToken is NOT updated by run_isolated (BUG-162) | Field Semantics | ✅ |
| SC-3 | Write caller is save() only — never live credentials file | Write Isolation | ✅ |
| SC-4 | Credential store created automatically when missing | Write Semantics | ✅ |
| SC-5 | Absent credential file is graceful (returns None/empty, no panic) | Error Path | ✅ |

---

### SC-1: Save copies all 3 credential fields to named file

- **Given:** `~/.claude/.credentials.json` contains `accessToken`, `refreshToken`, and `expiresAt` fields with valid values
- **When:** `.account.save` is invoked with a valid account name
- **Then:** `{name}.credentials.json` in the credential store contains all 3 fields with values matching the source
- **Source fn:** `save_copies_credentials_to_named_file` (account_tests.rs)
- **Source:** [docs/schema/001_credentials_json.md §Fields](../../../../docs/schema/001_credentials_json.md)

---

### SC-2: `expiresAt` is NOT updated by `run_isolated` token refresh (BUG-162)

- **Given:** An account whose `{name}.credentials.json` has `expiresAt = T0` (original issue time)
- **When:** A token refresh via `run_isolated` succeeds and writes a new `accessToken`
- **Then:** `expiresAt` in `{name}.credentials.json` remains `T0` — `run_isolated` does not update this field; callers must use the JWT `exp` claim for expiry instead
- **Note:** BUG-162 root cause — `run_isolated` subprocess refresh never updates `expiresAt` (the OAuth server controls this value at token issuance; the subprocess writes only `accessToken` and `refreshToken` during rotation)
- **Source fn:** `sc2_001_expires_at_stays_t0_manipulate_expires_at_in_memory_only` (account_tests.rs)
- **Source:** [docs/schema/001_credentials_json.md §Write Callers](../../../../docs/schema/001_credentials_json.md)

---

### SC-3: Write caller is `account::save()` only — never writes to live credentials file

- **Given:** An active session where `~/.claude/.credentials.json` exists with specific token values
- **When:** `.account.save` or any token refresh path runs
- **Then:** `~/.claude/.credentials.json` is NOT modified — all writes go to `{name}.credentials.json` in the credential store only (BUG-221 fix)
- **Source fn:** `reach_bulk_touch_does_not_write_live_credentials` (usage/touch_tests_b.rs)
- **Source:** [docs/schema/001_credentials_json.md §Write Callers](../../../../docs/schema/001_credentials_json.md)

---

### SC-4: Credential store directory created automatically when missing

- **Given:** No credential store directory exists at `{root}/.persistent/claude/credential/`
- **When:** `.account.save` is invoked
- **Then:** The credential store directory is created before writing `{name}.credentials.json` — no error occurs due to missing directory
- **Source fn:** `save_creates_credential_store_when_missing` (account_tests.rs)
- **Source:** [docs/schema/001_credentials_json.md §Write Callers](../../../../docs/schema/001_credentials_json.md)

---

### SC-5: Absent credential file is graceful

- **Given:** No `{name}.credentials.json` exists for the account
- **When:** Any read caller (`.accounts`, `.usage`, token refresh) attempts to read the file
- **Then:** The read returns a graceful empty/None result — no panic, no unhandled error
- **Source fn:** `e05_credential_store_absent_list_empty` (cli/cross_cutting_test.rs)
- **Source:** [docs/schema/001_credentials_json.md §Read Callers](../../../../docs/schema/001_credentials_json.md)
