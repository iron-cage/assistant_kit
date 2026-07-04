# Test: Feature 029 — Account Host and Role Metadata

### Scope

- **Purpose**: Test cases for account host/role profile metadata capture, display, and isolation from credentials.
- **Source**: `docs/feature/029_account_host_metadata.md`
- **Covers**: AC-01 through AC-10

Feature behavioral requirement test cases for `docs/feature/029_account_host_metadata.md`. Each FT case maps to one acceptance criterion.

### AC Coverage Index

| FT | Criterion | AC | Notes |
|----|-----------|-----|-------|
| FT-01 | `host::` and `role::` params write `{name}.json` | AC-01 | Integration |
| FT-02 | Omitting `host::` auto-captures `$USER@<hostname>` via fallback chain | AC-02 | Integration |
| FT-03 | Missing `$USER`: save succeeds with `@<hostname>` host | AC-03 | Edge case |
| FT-04 | Re-save with new `host::` overwrites `{name}.json` | AC-04 | Integration |
| FT-05 | `cols::+host` shows Host column from profile | AC-05 | Integration |
| FT-06 | `cols::+role` shows Role column from profile | AC-06 | Integration |
| FT-07 | `cols::+host get::host` extracts host as bare value | AC-07 | Extraction |
| FT-08 | `clp .accounts host::1 role::1` shows Host/Role fields | AC-08 | Integration |
| FT-09 | Absent `{name}.json` — no command exits non-zero | AC-09 | Resilience |
| FT-10 | Re-save with `host::` updates host without affecting credentials | AC-10 | Isolation |

### Test Case Index

| ID | Test Name | AC | Category |
|----|-----------|-----|----------|
| FT-01 | host:: and role:: write `{name}.json` | AC-01 | Profile Write |
| FT-02 | Auto-capture $USER@<hostname> via resolve_hostname() when host:: omitted | AC-02 | Auto-Capture |
| FT-03 | Missing $USER: save stores @<hostname>, succeeds | AC-03 | Resilience |
| FT-04 | Re-save overwrites `{name}.json` | AC-04 | Idempotency |
| FT-05 | cols::+host shows Host column | AC-05 | Display |
| FT-06 | cols::+role shows Role column | AC-06 | Display |
| FT-07 | get::host extracts host as bare string | AC-07 | Extraction |
| FT-08 | .accounts host::1 role::1 shows fields | AC-08 | .accounts |
| FT-09 | No `{name}.json` — no non-zero exits | AC-09 | Resilience |
| FT-10 | Re-save host:: does not affect credentials file | AC-10 | Isolation |

**Total:** 10 FT cases

---

### FT-01: `host::` and `role::` params write `{name}.json`

- **Given:** No pre-existing account for `test@example.com`.
- **When:** `clp .account.save name::test@example.com host::mybox role::work`
- **Then:** Exits 0. `{credential_store}/test@example.com.json` contains `{"host": "mybox", "role": "work"}`.
- **Exit:** 0
- **Source fn:** `as_save_writes_profile_json` (in `tests/cli/account_mutations_test.rs`)
- **Source:** [feature/029_account_host_metadata.md AC-01](../../../docs/feature/029_account_host_metadata.md)

---

### FT-02: Auto-capture `$USER@<hostname>` via fallback chain when `host::` omitted

- **Given:** `$USER=alice` set in environment. Machine hostname resolves to `workstation` via `resolve_hostname()` (`$HOSTNAME` env → `/etc/hostname` → `"local"`).
- **When:** `clp .account.save name::test@example.com` (no `host::` param)
- **Then:** Exits 0. `{credential_store}/test@example.com.json` contains `"host": "alice@workstation"`.
- **Exit:** 0
- **Source fn:** `as24_host_auto_capture_user_hostname` (in `tests/cli/account_mutations_test.rs`)
- **Source:** [feature/029_account_host_metadata.md AC-02](../../../docs/feature/029_account_host_metadata.md)

---

### FT-03: Missing `$USER` — save succeeds with `@<hostname>` host

- **Given:** `$USER` unset. Hostname resolves via `resolve_hostname()` fallback chain (always produces a non-empty value: env → file → `"local"`).
- **When:** `clp .account.save name::test@example.com` (no `host::` param)
- **Then:** Exits 0. `{credential_store}/test@example.com.json` contains `"host": "@<hostname>"` (user portion empty, hostname always resolved). No error.
- **Exit:** 0
- **Source fn:** `as28_host_missing_user_stores_at_resolved_hostname` (in `tests/cli/account_mutations_test.rs`)
- **Source:** [feature/029_account_host_metadata.md AC-03](../../../docs/feature/029_account_host_metadata.md)

---

### FT-04: Re-save overwrites `{name}.json`

- **Given:** Account `test@example.com` already saved with `host::oldbox role::personal`.
- **When:** `clp .account.save name::test@example.com host::newbox role::dev`
- **Then:** Exits 0. `{credential_store}/test@example.com.json` now contains `{"host": "newbox", "role": "dev"}`.
- **Exit:** 0
- **Source fn:** `as26_host_resave_overwrites` (in `tests/cli/account_mutations_test.rs`)
- **Source:** [feature/029_account_host_metadata.md AC-04](../../../docs/feature/029_account_host_metadata.md)

---

### FT-05: `cols::+host` shows Host column from profile

- **Given:** Account `test@example.com` saved with `host::mybox`.
- **When:** `clp .usage cols::+host`
- **Then:** Exits 0. Table header contains "Host". The row for `test@example.com` shows "mybox" in that column.
- **Exit:** 0
- **Live:** yes
- **Source fn:** `it202_cols_host_shows_host_column` (in `tests/cli/usage_test.rs`)
- **Source:** [feature/029_account_host_metadata.md AC-05](../../../docs/feature/029_account_host_metadata.md)

---

### FT-06: `cols::+role` shows Role column from profile

- **Given:** Account `test@example.com` saved with `role::work`.
- **When:** `clp .usage cols::+role`
- **Then:** Exits 0. Table header contains "Role". The row for `test@example.com` shows "work" in that column.
- **Exit:** 0
- **Live:** yes
- **Source fn:** `it203_cols_role_shows_role_column` (in `tests/cli/usage_test.rs`)
- **Source:** [feature/029_account_host_metadata.md AC-06](../../../docs/feature/029_account_host_metadata.md)

---

### FT-07: `cols::+host get::host` extracts host as bare string

- **Given:** Account `test@example.com` saved with `host::mybox`. Only account in store.
- **When:** `clp .usage cols::+host get::host`
- **Then:** Exits 0. Stdout is exactly `mybox` (bare string, no headers or footer).
- **Exit:** 0
- **Live:** yes
- **Source fn:** `it220_ft029_07_get_host_extracts_bare` (in `tests/cli/usage_test.rs`)
- **Source:** [feature/029_account_host_metadata.md AC-07](../../../docs/feature/029_account_host_metadata.md)

---

### FT-08: `.accounts host::1 role::1` shows Host and Role fields

- **Given:** Account `test@example.com` saved with `host::mybox role::work`.
- **When:** `clp .accounts host::1 role::1`
- **Then:** Exits 0. Output contains `Host: mybox` and `Role: work` for the account.
- **Exit:** 0
- **Source fn:** `acc49_accounts_host_role_shows_profile_metadata` (in `tests/cli/accounts_test.rs`)
- **Source:** [feature/029_account_host_metadata.md AC-08](../../../docs/feature/029_account_host_metadata.md)

---

### FT-09: Absent `{name}.json` — no command exits non-zero

- **Given:** Account `test@example.com` saved WITHOUT `host::` — `{name}.json` absent (or omit entirely).
- **When-A:** `clp .usage cols::+host`
- **When-B:** `clp .accounts host::1`
- **Then-A:** Exits 0. Host column present; `test@example.com` row shows empty cell.
- **Then-B:** Exits 0. Host field absent from output (not an error).
- **Exit:** 0 both
- **Source fn:** `it221_ft029_09_usage_no_profile_shows_empty_host` (When-A, in `tests/cli/usage_test.rs`); `acc50_accounts_host_no_profile_json_exits_0` (When-B, in `tests/cli/accounts_test.rs`)
- **Source:** [feature/029_account_host_metadata.md AC-09](../../../docs/feature/029_account_host_metadata.md)

---

### FT-10: Re-save with `host::` does not affect credentials file

- **Given:** Account `test@example.com` saved. Record SHA-256 of `test@example.com.credentials.json`.
- **When:** `clp .account.save name::test@example.com host::newbox`
- **Then:** Exits 0. `test@example.com.credentials.json` SHA-256 is unchanged. `{name}.json` updated.
- **Exit:** 0
- **Source fn:** `as29_resave_credentials_unchanged` (in `tests/cli/account_mutations_test.rs`)
- **Source:** [feature/029_account_host_metadata.md AC-10](../../../docs/feature/029_account_host_metadata.md)
