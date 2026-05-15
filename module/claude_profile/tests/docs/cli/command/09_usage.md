# Test: `.usage`

Integration test planning for the `.usage` command. See [commands.md](../../../../docs/cli/commands.md#command--9-usage) for specification.

### Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| IT-1 | Default invocation shows quota table with new column headers | Basic Invocation |
| IT-2 | Current account (live-token match) has `✓` in flag column; others do not | Current Marker |
| IT-3 | Account with missing accessToken shows `—` columns and error reason | Error Inline |
| IT-4 | `format::json` produces valid JSON array with `expires_in_secs` and `_left_pct` fields | Output Format |
| IT-5 | Empty credential store exits 0 with `(no accounts configured)` | Edge Case |
| IT-6 | Credential store unreadable exits 2 | Error Handling |
| IT-7 | HOME unset exits 2 | Error Handling |
| IT-8 | Multiple accounts displayed in alphabetical order | Ordering |
| IT-9 | Account with missing token file shows `—` with error reason | Error Inline |
| IT-10 | Account with expired token shows `EXPIRED` in Expires column | Expires Column |
| IT-11 | Best non-current account is marked with `→` in flag column | Recommendation |
| IT-12 | Footer line shows valid count and recommended next account | Footer |
| IT-13 | `*` marks `_active` account when it differs from the current account | Active Divergence |
| IT-14 | When credentials file unreadable: no `✓`; `*` still marks `_active` account | Active Divergence |
| IT-15 | When current = active, only `✓` appears; no `*` on any row | Active Divergence |
| IT-16 | JSON output uses `is_current` (not `active`) and includes `is_active` per object | JSON Schema |

### Test Coverage Summary

- Basic Invocation: 1 test (IT-1)
- Current Marker: 1 test (IT-2)
- Error Inline: 2 tests (IT-3, IT-9)
- Output Format: 1 test (IT-4)
- Edge Case: 1 test (IT-5)
- Error Handling: 2 tests (IT-6, IT-7)
- Ordering: 1 test (IT-8)
- Expires Column: 1 test (IT-10)
- Recommendation: 1 test (IT-11)
- Footer: 1 test (IT-12)
- Active Divergence: 3 tests (IT-13, IT-14, IT-15)
- JSON Schema: 1 test (IT-16)

**Total:** 16 integration tests

---

### IT-1: Default invocation shows quota table with new column headers

- **Given:** At least one saved account with a valid token exists in the credential store.
- **When:** `clp .usage`
- **Then:** Stdout contains a table with "Quota" heading and rows showing columns: "Expires", "5h Left", "5h Reset", "7d Left", "7d Reset". Exit 0.
- **Exit:** 0
- **Source:** [commands.md — .usage](../../../../docs/cli/commands.md#command--9-usage)

---

### IT-2: Current account (live-token match) has `✓` in flag column

- **Given:** Two saved accounts; live `~/.claude/.credentials.json` has an `accessToken` matching `work@acme.com`'s stored token. `_active` also points to `work@acme.com` (current = active, normal case).
- **When:** `clp .usage`
- **Then:** A line in stdout contains both `✓` and `work@acme.com`; no line contains `✓` and any other account name; no `*` appears (current = active). Exit 0.
- **Exit:** 0
- **Source:** [016_current_account_awareness.md AC-05](../../../../docs/feature/016_current_account_awareness.md)

---

### IT-3: Account with missing accessToken shows `—` columns and error reason

- **Given:** One account whose credential file has no `accessToken` field (but has a future `expiresAt`).
- **When:** `clp .usage`
- **Then:** That account's row shows `—` for 5h Left and 7d Left; Status column shows an inline error reason. Expires column shows "in" (not "EXPIRED") because token has a future expiry. Exit 0.
- **Exit:** 0
- **Source:** [commands.md — .usage](../../../../docs/cli/commands.md#command--9-usage)

---

### IT-4: `format::json` produces valid JSON array with `expires_in_secs`, `is_current`, `is_active`

- **Given:** At least one saved account with a valid token.
- **When:** `clp .usage format::json`
- **Then:** Valid JSON array on stdout. Each element has `account` (string), `is_current` (boolean), `is_active` (boolean), and `expires_in_secs` (number). Successful elements have `session_5h_left_pct` and `weekly_7d_left_pct` (not `session_5h_pct` or `weekly_7d_pct`). No element has a top-level `active` key. Exit 0.
- **Exit:** 0
- **Source:** [016_current_account_awareness.md AC-08](../../../../docs/feature/016_current_account_awareness.md)

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

### IT-11: Best non-current account is marked with `→` in flag column

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

---

### IT-13: `*` marks `_active` account when it differs from current

- **Given:** Two saved accounts: `alice@acme.com` (stored as `_active`) and `work@acme.com`. Live `~/.claude/.credentials.json` `accessToken` matches `work@acme.com`'s stored token (not `alice`'s).
- **When:** `clp .usage`
- **Then:** A line contains `✓` and `work@acme.com`; a different line contains `*` and `alice@acme.com`. No line contains both `✓` and `alice`, or both `*` and `work`.
- **Exit:** 0
- **Source:** [016_current_account_awareness.md AC-06](../../../../docs/feature/016_current_account_awareness.md)

---

### IT-14: Credentials file unreadable — no `✓`; `*` still marks `_active`

- **Given:** Two saved accounts: `alice@acme.com` (stored as `_active`) and `work@acme.com`. `~/.claude/.credentials.json` is absent or unreadable.
- **When:** `clp .usage`
- **Then:** No line contains `✓`; a line contains `*` and `alice@acme.com`. All saved accounts are still shown. Exit 0.
- **Exit:** 0
- **Source:** [016_current_account_awareness.md AC-07](../../../../docs/feature/016_current_account_awareness.md)

---

### IT-15: When current = active, only `✓` appears; no `*` on any row

- **Given:** Two saved accounts: `alice@acme.com` (stored as `_active`) and `work@acme.com`. Live `~/.claude/.credentials.json` `accessToken` matches `alice@acme.com`'s stored token (current = active).
- **When:** `clp .usage`
- **Then:** A line contains `✓` and `alice@acme.com`; no line contains `*`.
- **Exit:** 0
- **Source:** [016_current_account_awareness.md AC-06](../../../../docs/feature/016_current_account_awareness.md)

---

### IT-16: JSON output uses `is_current` and `is_active`; no `active` key

- **Given:** Two saved accounts; live credentials match one of them; `_active` points to the other (divergence case).
- **When:** `clp .usage format::json`
- **Then:** Valid JSON array; the current account object has `"is_current":true` and `"is_active":false`; the `_active` account object has `"is_current":false` and `"is_active":true`; no object has a top-level `"active"` key.
- **Exit:** 0
- **Source:** [016_current_account_awareness.md AC-08](../../../../docs/feature/016_current_account_awareness.md)
