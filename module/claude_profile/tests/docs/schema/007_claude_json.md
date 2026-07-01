# Schema 007: Claude State — `~/.claude.json`

SC test cases for `docs/schema/007_claude_json.md`. Verifies the read-only contract:
`clp` never writes to this file, absent fields and file produce graceful N/A output,
and the `oauthAccount.emailAddress` field is used as the default account name.

**Source:** [docs/schema/007_claude_json.md](../../../../docs/schema/007_claude_json.md)

### SC Case Index

| ID | Short Name | Category | Status |
|----|------------|----------|--------|
| SC-1 | Absent `~/.claude.json` — save succeeds, all metadata fields show N/A | Error Path | ✅ |
| SC-2 | `emailAddress` used as default account name when `name::` omitted | Field Semantics | ✅ |
| SC-3 | `clp` never writes to `~/.claude.json` (read-only contract) | Write Isolation | ✅ |
| SC-4 | Absent `oauthAccount` subfields show N/A without error | Graceful Missing | ✅ |

---

### SC-1: Absent `~/.claude.json` — save succeeds and all metadata shows N/A

- **Given:** `~/.claude.json` does not exist on disk
- **When:** `.account.save` is invoked
- **Then:** The save completes successfully; `displayName`, `organizationRole`, `billingType` fields in `{name}.json` are either absent or N/A — no error is raised for a missing source file
- **Source fn:** `acc27_save_succeeds_without_claude_json` (cli/accounts_list_test_b.rs)
- **Source:** [docs/schema/007_claude_json.md §Graceful Missing-Field Handling](../../../../docs/schema/007_claude_json.md)

---

### SC-2: `oauthAccount.emailAddress` used as default account name when `name::` omitted

- **Given:** `~/.claude.json` contains `oauthAccount.emailAddress = "alice@example.com"` and `.account.save` is invoked without `name::` parameter
- **When:** `account_save_routine()` resolves the account name
- **Then:** The account is saved as `alice@example.com` — the email address from `~/.claude.json` is used as the default name
- **Source fn:** `it025_synthetic_row_uses_claude_json_email` (cli/usage_live_test.rs; verifies email from claude.json is used in live context)
- **Source:** [docs/schema/007_claude_json.md §Fields Read by clp](../../../../docs/schema/007_claude_json.md)

---

### SC-3: `clp` never writes to `~/.claude.json` (read-only contract)

- **Given:** `~/.claude.json` exists with specific field values
- **When:** Any `clp` operation runs (`.account.save`, `.account.use`, `.usage`, `.model`, etc.)
- **Then:** `~/.claude.json` is NOT modified — file mtime and content unchanged after `clp` invocation
- **Source fn:** `reach_bulk_touch_does_not_write_live_credentials` (usage/touch_tests_b.rs; verifies live credential file untouched by touch operations)
- **Source:** [docs/schema/007_claude_json.md §Read-Only Contract](../../../../docs/schema/007_claude_json.md)

---

### SC-4: Absent `oauthAccount` subfields show N/A without error

- **Given:** `~/.claude.json` exists but specific subfields under `oauthAccount` are missing (`displayName` absent, `organizationRole` absent, etc.)
- **When:** `.account.save` or `.credentials.status` reads `~/.claude.json`
- **Then:** Missing subfields produce N/A in the corresponding output — no error, no panic for partial `oauthAccount` objects
- **Source fn:** `acc27_save_succeeds_without_claude_json` (cli/accounts_list_test_b.rs)
- **Source:** [docs/schema/007_claude_json.md §Graceful Missing-Field Handling](../../../../docs/schema/007_claude_json.md)
