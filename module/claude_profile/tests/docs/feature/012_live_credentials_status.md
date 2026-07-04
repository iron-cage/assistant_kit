# Test: Feature 012 — Live Credentials Status

### Scope

- **Purpose**: Test cases for `.credentials.status` — store-independent live credential inspection and field toggles.
- **Source**: `docs/feature/012_live_credentials_status.md`
- **Covers**: AC-01 through AC-07

Feature behavioral requirement test cases for `docs/feature/012_live_credentials_status.md` (FR-17). Each FT case maps to one or more acceptance criteria.

### AC Coverage Index

| FT | Criterion | AC |
|----|-----------|-----|
| FT-01 | Succeeds on machine with only `~/.claude/.credentials.json` (no store) | AC-01 |
| FT-02 | Default output shows 6 default-on fields | AC-02 |
| FT-03 | `format::json` returns all 12 fields | AC-03 |
| FT-04 | Absent credentials file exits 2 with actionable error | AC-04 |
| FT-05 | Missing email and absent active marker → both show `N/A` | AC-05 |
| FT-06 | Suppressing all default-on fields → only suppressed output | AC-06 |
| FT-07 | `file::1 saved::1` appends File and Saved lines | AC-07 |

### Test Case Index

| ID | Test Name | AC | Category |
|----|-----------|-----|----------|
| FT-01 | No credential store directory needed — only credentials file | AC-01 | Store Independence |
| FT-02 | Default output: account, sub, tier, token, expires, email lines | AC-02 | Default Output |
| FT-03 | `format::json` includes all 12 fields | AC-03 | JSON Format |
| FT-04 | Missing credentials file → exit 2 with path in error | AC-04 | Error Handling |
| FT-05 | No `~/.claude.json` → email and N/A fields show `N/A` | AC-05 | N/A Handling |
| FT-06 | `sub::0 tier::0 expires::0 email::0 account::0` → only Token | AC-06 | Field Toggles |
| FT-07 | `file::1 saved::1` → File and Saved lines shown | AC-07 | Opt-In Fields |

**Total:** 7 FT cases

---

### FT-01: No credential store directory needed — only credentials file

- **Given:** `~/.claude/.credentials.json` exists but no credential store directory is present.
- **When:** `clp .credentials.status`
- **Then:** Exits 0. Output shows status fields without requiring any saved account store.
- **Exit:** 0
- **Source fn:** `cred01_no_credential_store_succeeds`
- **Source:** [012_live_credentials_status.md AC-01](../../../docs/feature/012_live_credentials_status.md)

---

### FT-02: Default output: account, sub, tier, token, expires, email lines

- **Given:** `~/.claude/.credentials.json` exists with valid credentials. `~/.claude.json` exists with `oauthAccount.emailAddress`.
- **When:** `clp .credentials.status` (no extra params)
- **Then:** Output contains lines for `Account:`, `Sub:`, `Tier:`, `Token:`, `Expires:`, `Email:`. No `File:`, `Saved:`, `Display:`, `Role:`, `Billing:`, `Model:` lines by default.
- **Exit:** 0
- **Source fn:** `cred02_default_with_claude_json`
- **Source:** [012_live_credentials_status.md AC-02](../../../docs/feature/012_live_credentials_status.md)

---

### FT-03: `format::json` returns all 12 fields

- **Given:** Valid credentials and `~/.claude.json` present.
- **When:** `clp .credentials.status format::json`
- **Then:** Output is valid JSON with all 12 keys: `subscription`, `tier`, `token`, `expires_in_secs`, `email`, `account`, `file`, `saved`, `display_name`, `role`, `billing`, `model`.
- **Exit:** 0
- **Source fn:** `cred03_format_json`
- **Source:** [012_live_credentials_status.md AC-03](../../../docs/feature/012_live_credentials_status.md)

---

### FT-04: Missing credentials file → exit 2 with path in error

- **Given:** `~/.claude/.credentials.json` does not exist.
- **When:** `clp .credentials.status`
- **Then:** Exits 2. Error message names the full expected path to `.credentials.json`.
- **Exit:** 2
- **Source fn:** `cred04_missing_credentials_file_exits_nonzero`
- **Source:** [012_live_credentials_status.md AC-04](../../../docs/feature/012_live_credentials_status.md)

---

### FT-05: No `~/.claude.json` → email and N/A fields show `N/A`

- **Given:** `~/.claude/.credentials.json` exists. `~/.claude.json` is absent. No per-machine active marker.
- **When:** `clp .credentials.status`
- **Then:** `Email:` shows `N/A`. `Account:` shows `N/A`. No error.
- **Exit:** 0
- **Source fn:** `cred05_no_claude_json_shows_na`
- **Source:** [012_live_credentials_status.md AC-05](../../../docs/feature/012_live_credentials_status.md)

---

### FT-06: Suppressing all default-on fields → only Token remains

- **Given:** Valid credentials present.
- **When:** `clp .credentials.status sub::0 tier::0 expires::0 email::0 account::0`
- **Then:** Only the `Token:` line appears in output. All suppressed fields are absent.
- **Exit:** 0
- **Source fn:** `cred06_suppress_all_default_on`
- **Source:** [012_live_credentials_status.md AC-06](../../../docs/feature/012_live_credentials_status.md)

---

### FT-07: `file::1 saved::1` → File and Saved lines shown

- **Given:** Valid credentials present. Credential store contains 2 saved accounts.
- **When:** `clp .credentials.status file::1 saved::1`
- **Then:** Output appends a `File:` line (showing the path to `.credentials.json`) and a `Saved:` line (e.g., `2 account(s)`) after the default-on fields.
- **Exit:** 0
- **Source fn:** `cred07_opt_in_file_and_saved`
- **Source:** [012_live_credentials_status.md AC-07](../../../docs/feature/012_live_credentials_status.md)
