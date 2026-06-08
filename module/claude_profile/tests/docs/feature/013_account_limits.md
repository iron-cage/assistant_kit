# Test: Feature 013 — Account Rate-Limit Utilization

Feature behavioral requirement test cases for `docs/feature/013_account_limits.md` (FR-18). Each FT case maps to one or more acceptance criteria.

### AC Coverage Index

| FT | Criterion | AC |
|----|-----------|-----|
| FT-01 | Shows 5h/7d utilization and rate-limit status for active account | AC-01 |
| FT-02 | `name::` selects a named account's limits | AC-02 |
| FT-03 | `format::json` returns structured utilization JSON | AC-03 |
| FT-04 | Missing data source exits 2 with actionable error | AC-04 |

### Test Case Index

| ID | Test Name | AC | Category |
|----|-----------|-----|----------|
| FT-01 | Active account shows 5h/7d utilization and status (live) | AC-01 | Happy Path |
| FT-02 | `name::alice@acme.com` shows limits for named account | AC-02 | Named Account |
| FT-03 | `format::json` returns valid JSON with utilization fields | AC-03 | JSON Format |
| FT-04 | Data unavailable / account not found → exit 2, not silent | AC-04 | Error Handling |

**Total:** 4 FT cases

---

### FT-01: Active account shows 5h/7d utilization and status (live)

- **Given:** A valid active account with a live token.
- **When:** `clp .account.limits` (live API call)
- **Then:** Output contains session (5h) utilization, weekly (7d) utilization, and rate-limit status (`allowed` / `allowed_warning` / `rejected`). Exit 0.
- **Exit:** 0
- **Source fn:** `lim_it1_active_account_exits_0_with_utilization_text`
- **Source:** [013_account_limits.md AC-01](../../../../docs/feature/013_account_limits.md)

---

### FT-02: `name::alice@acme.com` shows limits for named account

- **Given:** `alice@acme.com` is saved in the credential store (but not the active account).
- **When:** `clp .account.limits name::alice@acme.com`
- **Then:** Limits are shown for `alice@acme.com`, not the active account. Exit 0 or 2 depending on data availability.
- **Exit:** 0
- **Source fn:** manual — IT-4 in `tests/manual/readme.md`
- **Source:** [013_account_limits.md AC-02](../../../../docs/feature/013_account_limits.md)

---

### FT-03: `format::json` returns valid JSON with utilization fields

- **Given:** A valid active account with a live token.
- **When:** `clp .account.limits format::json` (live API call)
- **Then:** Output is valid JSON containing utilization fields (5h, 7d percentages and reset timestamps).
- **Exit:** 0
- **Source fn:** `lim_it3_json_format_exits_0_with_valid_json`
- **Source:** [013_account_limits.md AC-03](../../../../docs/feature/013_account_limits.md)

---

### FT-04: Data unavailable / account not found → exit 2, not silent

- **Given (case A):** Named account does not exist in the store.
- **Given (case B):** No credentials file — data source unavailable.
- **Given (case C):** Account exists but live API call fails (data unavailable).
- **When:** `clp .account.limits` in any of the above cases.
- **Then:** Exits 2 with an actionable error. Never silently exits 0 with missing data.
- **Exit:** 2
- **Source fn:** `lim01_unknown_named_account_exits_2`, `lim02_no_active_credentials_exits_2`, `lim03_data_unavailable_exits_2_not_silent`, `lim05_existing_named_account_exits_2_with_data_unavailable`
- **Source:** [013_account_limits.md AC-04](../../../../docs/feature/013_account_limits.md)
