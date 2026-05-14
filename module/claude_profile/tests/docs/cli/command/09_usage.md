# Test: `.usage`

Integration test planning for the `.usage` command. See [commands.md](../../../../docs/cli/commands.md#command--9-usage) for specification.

### Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| IT-1 | Default invocation shows quota table with new column headers | Basic Invocation |
| IT-2 | Active account has `✓` in flag column; inactive accounts do not | Active Marker |
| IT-3 | Account with missing accessToken shows `—` columns and error reason | Error Inline |
| IT-4 | `format::json` produces valid JSON array with `expires_in_secs` and `_left_pct` fields | Output Format |
| IT-5 | Empty credential store exits 0 with `(no accounts configured)` | Edge Case |
| IT-6 | Credential store unreadable exits 2 | Error Handling |
| IT-7 | HOME unset exits 2 | Error Handling |
| IT-8 | Multiple accounts displayed in alphabetical order | Ordering |
| IT-9 | Account with missing token file shows `—` with error reason | Error Inline |
| IT-10 | Account with expired token shows `EXPIRED` in Expires column | Expires Column |
| IT-11 | Best non-active account is marked with `→` in flag column | Recommendation |
| IT-12 | Footer line shows valid count and recommended next account | Footer |

### Test Coverage Summary

- Basic Invocation: 1 test
- Active Marker: 1 test
- Error Inline: 2 tests
- Output Format: 1 test
- Edge Case: 1 test
- Error Handling: 2 tests
- Ordering: 1 test
- Expires Column: 1 test
- Recommendation: 1 test
- Footer: 1 test

**Total:** 12 integration tests

---

### IT-1: Default invocation shows quota table with new column headers

- **Given:** At least one saved account with a valid token exists in the credential store.
- **When:** `clp .usage`
- **Then:** Stdout contains a table with "Quota" heading and rows showing columns: "Expires", "5h Left", "5h Reset", "7d Left", "7d Reset". Exit 0.
- **Exit:** 0
- **Source:** [commands.md — .usage](../../../../docs/cli/commands.md#command--9-usage)

---

### IT-2: Active account has `✓` in flag column; inactive accounts do not

- **Given:** At least two saved accounts, one marked active via `_active`.
- **When:** `clp .usage`
- **Then:** A line in stdout contains both `✓` and the active account name; no line contains both `✓` and any inactive account name. Exit 0.
- **Exit:** 0
- **Source:** [commands.md — .usage](../../../../docs/cli/commands.md#command--9-usage)

---

### IT-3: Account with missing accessToken shows `—` columns and error reason

- **Given:** One account whose credential file has no `accessToken` field (but has a future `expiresAt`).
- **When:** `clp .usage`
- **Then:** That account's row shows `—` for 5h Left and 7d Left; Status column shows an inline error reason. Expires column shows "in" (not "EXPIRED") because token has a future expiry. Exit 0.
- **Exit:** 0
- **Source:** [commands.md — .usage](../../../../docs/cli/commands.md#command--9-usage)

---

### IT-4: `format::json` produces valid JSON array with `expires_in_secs` and `_left_pct` fields

- **Given:** At least one saved account with a valid token.
- **When:** `clp .usage format::json`
- **Then:** Valid JSON array on stdout. Each element has `account` (string), `active` (boolean), and `expires_in_secs` (number). Successful elements have `session_5h_left_pct` and `weekly_7d_left_pct` (not `session_5h_pct` or `weekly_7d_pct`). Exit 0.
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

---

### IT-10: Account with expired token shows `EXPIRED` in Expires column

- **Given:** One saved account whose `expiresAt` in the credential file is a past timestamp (e.g., `PAST_MS`).
- **When:** `clp .usage`
- **Then:** That account's row shows `EXPIRED` in the Expires column. The quota columns show `—`. Exit 0.
- **Exit:** 0
- **Source:** [commands.md — .usage](../../../../docs/cli/commands.md#command--9-usage)

---

### IT-11: Best non-active account is marked with `→` in flag column

- **Given:** Two accounts — one active with quota data, one non-active with valid token and quota data showing lower session usage than the active account.
- **When:** `clp .usage`
- **Then:** A line in stdout contains both `→` and the non-active account name. No line contains both `→` and the active account name. Exit 0.
- **Exit:** 0
- **Live:** yes (requires real tokens for both accounts to return quota data)
- **Source:** [commands.md — .usage](../../../../docs/cli/commands.md#command--9-usage)

---

### IT-12: Footer line shows valid count and recommended next account

- **Given:** At least two accounts with valid tokens that return quota data.
- **When:** `clp .usage`
- **Then:** Stdout contains a footer line matching "Valid: N / M" and "Next:" with the recommended account name. Exit 0.
- **Exit:** 0
- **Live:** yes (requires ≥2 accounts with live quota headers)
- **Source:** [commands.md — .usage](../../../../docs/cli/commands.md#command--9-usage)
