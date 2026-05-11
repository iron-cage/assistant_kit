# Test: `.usage`

Integration test planning for the `.usage` command. See [commands.md](../../../../docs/cli/commands.md#command--9-usage) for specification.

> **Implementation state:** IT-1 through IT-9 below describe the target test coverage for the live-API implementation (per `docs/feature/009_token_usage.md`). The current interim implementation reads `stats-cache.json` and its tests are in `tests/cli/usage_test.rs` (u01–u24). IT-1..IT-9 will be implemented when `data_fmt` is added to the workspace.

### Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| IT-1 | Default invocation shows quota table for all accounts | Basic Invocation |
| IT-2 | Active account marked with `(✓)` in Account column | Active Marker |
| IT-3 | Expired token account shows `—` columns and error reason | Error Inline |
| IT-4 | `format::json` produces valid JSON array | Output Format |
| IT-5 | Empty credential store exits 0 with `(no accounts configured)` | Edge Case |
| IT-6 | Credential store unreadable exits 2 | Error Handling |
| IT-7 | HOME unset exits 2 | Error Handling |
| IT-8 | Multiple accounts displayed in alphabetical order | Ordering |
| IT-9 | Account with missing token file shows `—` with error reason | Error Inline |

### Test Coverage Summary

- Basic Invocation: 1 test
- Active Marker: 1 test
- Error Inline: 2 tests
- Output Format: 1 test
- Edge Case: 1 test
- Error Handling: 2 tests
- Ordering: 1 test

**Total:** 9 integration tests

---

### IT-1: Default invocation shows quota table for all accounts

- **Given:** At least one saved account with a valid token exists in the credential store.
- **When:** `clp .usage`
- **Then:** Stdout contains a table with "Quota" heading and rows for each account showing Session (5h) and Weekly (7d) columns. Exit 0.
- **Exit:** 0
- **Source:** [commands.md — .usage](../../../../docs/cli/commands.md#command--9-usage)

---

### IT-2: Active account marked with `(✓)` in Account column

- **Given:** At least two saved accounts, one marked active via `_active`.
- **When:** `clp .usage`
- **Then:** Active account's row shows `(✓)` inline in the Account column; other rows do not. Exit 0.
- **Exit:** 0
- **Source:** [commands.md — .usage](../../../../docs/cli/commands.md#command--9-usage)

---

### IT-3: Expired token account shows `—` columns and error reason

- **Given:** One valid account and one account with an expired token.
- **When:** `clp .usage`
- **Then:** Expired account row shows `—` for Session and Weekly columns; Status column shows an inline error reason (e.g., `(expired token)`). Valid account row shows quota data. Exit 0.
- **Exit:** 0
- **Source:** [commands.md — .usage](../../../../docs/cli/commands.md#command--9-usage)

---

### IT-4: `format::json` produces valid JSON array

- **Given:** At least one saved account with a valid token.
- **When:** `clp .usage format::json`
- **Then:** Valid JSON array on stdout. Each element has `account`, `active`, and either quota fields or `error` field. Exit 0.
- **Exit:** 0
- **Source:** [commands.md — .usage](../../../../docs/cli/commands.md#command--9-usage)

---

### IT-5: Empty credential store shows empty table

- **Given:** Credential store exists but contains no `*.credentials.json` files.
- **When:** `clp .usage`
- **Then:** Stdout contains `(no accounts configured)`. Exit 0.
- **Exit:** 0
- **Source:** [commands.md — .usage](../../../../docs/cli/commands.md#command--9-usage)

---

### IT-6: Credential store unreadable exits 2

- **Given:** `HOME` is set but credential store directory cannot be read (permissions error).
- **When:** `clp .usage`
- **Then:** Error on stderr. Exit 2.
- **Exit:** 2
- **Source:** [commands.md — .usage](../../../../docs/cli/commands.md#command--9-usage)

---

### IT-7: HOME unset exits 2

- **Given:** `HOME` environment variable is unset.
- **When:** `env -u HOME clp .usage`
- **Then:** Error on stderr. Exit 2.
- **Exit:** 2
- **Source:** [commands.md — .usage](../../../../docs/cli/commands.md#command--9-usage)

---

### IT-8: Multiple accounts displayed in alphabetical order

- **Given:** Three saved accounts: `c@x.com`, `a@x.com`, `b@x.com`.
- **When:** `clp .usage`
- **Then:** Rows appear in order `a@x.com`, `b@x.com`, `c@x.com`. Exit 0.
- **Exit:** 0
- **Source:** [commands.md — .usage](../../../../docs/cli/commands.md#command--9-usage)

---

### IT-9: Account with missing token file shows `—` with error reason

- **Given:** Credential store entry exists but the `.credentials.json` file for that account is missing.
- **When:** `clp .usage`
- **Then:** That account's row shows `—` for quota columns and a missing-token error reason in Status. Exit 0.
- **Exit:** 0
- **Source:** [commands.md — .usage](../../../../docs/cli/commands.md#command--9-usage)
