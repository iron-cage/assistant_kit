# Test: Feature 016 — Current Account Awareness

Feature behavioral requirement test cases for `docs/feature/016_current_account_awareness.md`. Each FT case maps to one or more acceptance criteria.

### AC Coverage Index

| FT | Criterion | AC |
|----|-----------|-----|
| FT-01 | `.accounts` shows `Current: yes` on token-matched account | AC-01 |
| FT-02 | `.accounts` suppresses `Current:` entirely when credentials unreadable | AC-02 |
| FT-03 | `current::0` suppresses the `Current:` line | AC-03 |
| FT-04 | `.accounts` JSON includes `is_current` boolean per account | AC-04 |
| FT-05 | `.usage` `✓` marks current account (token match, not active marker) | AC-05 |
| FT-06 | `.usage` `*` marks active account when it differs from current | AC-06 |
| FT-07 | `.usage` credentials unreadable → no `✓`; `*` still marks active | AC-07 |
| FT-08 | `.usage` JSON uses `is_current` + `is_active`; no `active` field | AC-08 |
| FT-09 | Synthetic current-session row injected when no stored account matches live token | AC-09 |
| FT-10 | Injection suppressed when derived name collides with existing stored account | AC-10 |
| FT-11 | Quota table contains at most one row per unique account name | AC-11 |

### Test Case Index

| ID | Test Name | AC | Category |
|----|-----------|-----|----------|
| FT-01 | `.accounts` shows Current: yes/no per token match | AC-01 | Accounts |
| FT-02 | `.accounts` suppresses Current: when creds absent | AC-02 | Accounts |
| FT-03 | `current::0` suppresses Current: line | AC-03 | Field Toggle |
| FT-04 | `.accounts format::json` has `is_current` boolean | AC-04 | JSON Format |
| FT-05 | `.usage` `✓` on current, not just active | AC-05 | Usage Flags |
| FT-06 | `.usage` `*` on active when diverges from current | AC-06 | Usage Flags |
| FT-07 | `.usage` no creds → no `✓`; `*` still on active | AC-07 | Usage Flags |
| FT-08 | `.usage format::json` uses `is_current` + `is_active` | AC-08 | JSON Format |
| FT-09 | Synthetic row injected for unknown live token | AC-09 | Synthetic Row |
| FT-10 | Synthetic row suppressed on name collision | AC-10 | Synthetic Row |
| FT-11 | At most one row per account name in quota table | AC-11 | Deduplication |

**Total:** 11 FT cases

---

### FT-01: `.accounts` shows `Current: yes/no` per token match

- **Given:** Two accounts saved. `alice@acme.com`'s `accessToken` matches `~/.claude/.credentials.json`. `work@acme.com` does not match.
- **When:** `clp .accounts`
- **Then:** `alice@acme.com`'s block shows `Current:  yes`; `work@acme.com`'s block shows `Current:  no`.
- **Exit:** 0
- **Source fn:** `acc31_accounts_shows_current_yes_no`
- **Source:** [016_current_account_awareness.md AC-01](../../../../docs/feature/016_current_account_awareness.md)

---

### FT-02: `.accounts` suppresses `Current:` entirely when credentials unreadable

- **Given:** `~/.claude/.credentials.json` is absent or unreadable. Two accounts saved.
- **When:** `clp .accounts`
- **Then:** Neither account shows a `Current:` line at all. No misleading `Current: no` on any block.
- **Exit:** 0
- **Source fn:** `acc32_accounts_suppresses_current_when_creds_absent`
- **Source:** [016_current_account_awareness.md AC-02](../../../../docs/feature/016_current_account_awareness.md)

---

### FT-03: `current::0` suppresses `Current:` line

- **Given:** Valid credentials; `~/.claude/.credentials.json` readable.
- **When:** `clp .accounts current::0`
- **Then:** No `Current:` line appears in any account block.
- **Exit:** 0
- **Source fn:** `acc33_accounts_current_param_and_json`
- **Source:** [016_current_account_awareness.md AC-03](../../../../docs/feature/016_current_account_awareness.md)

---

### FT-04: `.accounts format::json` has `is_current` boolean

- **Given:** Two accounts saved; one matches live credentials.
- **When:** `clp .accounts format::json`
- **Then:** Each account object in the JSON array has an `is_current` boolean field. The matching account has `"is_current": true`; others have `"is_current": false`.
- **Exit:** 0
- **Source fn:** `acc33_accounts_current_param_and_json`
- **Source:** [016_current_account_awareness.md AC-04](../../../../docs/feature/016_current_account_awareness.md)

---

### FT-05: `.usage` `✓` on current, not just active

- **Given:** `work@acme.com`'s token matches `~/.claude/.credentials.json` (current). `alice@acme.com` has the per-machine active marker (active). These are different accounts.
- **When:** `clp .usage`
- **Then:** `work@acme.com`'s row shows `✓` flag. `alice@acme.com`'s row shows `*` flag. Other rows show ` ` (space).
- **Exit:** 0
- **Source fn:** `it013_active_divergence_shows_star`
- **Source:** [016_current_account_awareness.md AC-05](../../../../docs/feature/016_current_account_awareness.md)

---

### FT-06: `.usage` `*` on active when diverges from current

- **Given:** Same divergence as FT-05 — current ≠ active.
- **When:** `clp .usage`
- **Then:** Only the active account (not current) shows `*`. When current = active (normal case), no `*` appears on any row.
- **Exit:** 0
- **Source fn:** `it013_active_divergence_shows_star`, `it015_current_equals_active_no_star`
- **Source:** [016_current_account_awareness.md AC-06](../../../../docs/feature/016_current_account_awareness.md)

---

### FT-07: `.usage` no creds → no `✓`; `*` still on active

- **Given:** `~/.claude/.credentials.json` is absent or unreadable. One account has the per-machine active marker.
- **When:** `clp .usage`
- **Then:** No `✓` on any row. The active account still shows `*`. No error.
- **Exit:** 0
- **Source fn:** `it014_creds_unreadable_no_checkmark_star_shown`
- **Source:** [016_current_account_awareness.md AC-07](../../../../docs/feature/016_current_account_awareness.md)

---

### FT-08: `.usage format::json` uses `is_current` + `is_active`

- **Given:** Two accounts; current ≠ active.
- **When:** `clp .usage format::json`
- **Then:** Each object has `"is_current"` and `"is_active"` boolean fields. No `"active"` field (old name). Current account has `"is_current": true`; active has `"is_active": true`.
- **Exit:** 0
- **Source fn:** `it016_json_is_current_is_active`
- **Source:** [016_current_account_awareness.md AC-08](../../../../docs/feature/016_current_account_awareness.md)

---

### FT-09: Synthetic row injected for unknown live token

- **Given:** `~/.claude/.credentials.json` contains a token belonging to `stranger@example.com`, which is NOT in the credential store. `~/.claude.json` has `oauthAccount.emailAddress = "stranger@example.com"`.
- **When:** `clp .usage`
- **Then:** A synthetic row for `stranger@example.com` is prepended to the table with `✓`. Other stored accounts have no `✓`.
- **Exit:** 0
- **Source fn:** `it018_synthetic_row_when_no_saved_match`, `it025_synthetic_row_uses_claude_json_email`
- **Source:** [016_current_account_awareness.md AC-09](../../../../docs/feature/016_current_account_awareness.md)

---

### FT-10: Synthetic row suppressed on name collision

- **Given:** `~/.claude/.credentials.json` belongs to `alice@acme.com`. `alice@acme.com` IS in the credential store. The live token would otherwise trigger synthetic-row injection.
- **When:** `clp .usage`
- **Then:** No duplicate row for `alice@acme.com`. The stored row carries the quota data. `✓` appears on the stored row.
- **Exit:** 0
- **Source fn:** `it247_synthetic_row_suppressed_name_collision`
- **Source:** [016_current_account_awareness.md AC-10](../../../../docs/feature/016_current_account_awareness.md)

---

### FT-11: At most one row per account name in quota table

- **Given:** Any account setup (with or without live credential match).
- **When:** `clp .usage`
- **Then:** No account name appears more than once in the output table. The quota table produced by `fetch_all_quota()` enforces deduplication via lookup-then-insert.
- **Exit:** 0
- **Source fn:** `it018_synthetic_row_when_no_saved_match` (at-most-one invariant enforced by fixture design — no duplicate injected when stored account already has the matching token)
- **Source:** [016_current_account_awareness.md AC-11](../../../../docs/feature/016_current_account_awareness.md)
