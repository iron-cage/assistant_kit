# Test: `.account.limits`

Integration test specifications for the `.account.limits` command. See [commands.md](../../../../docs/cli/commands.md#command--11-accountlimits) and [feature/013_account_limits.md](../../../../docs/feature/013_account_limits.md) for specification.

### Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| IT-1 | Active account — default output shows session, weekly-all, weekly-sonnet | Happy Path |
| IT-3 | Active account — `format::json` returns parseable JSON | Format |
| IT-4 | Named account — `name::work@acme.com` shows limits for that account | Named Account |
| IT-5 | Named account — `name::ghost@example.com` unknown account exits 2 | Not Found |
| IT-6 | No active account set — exits 2 with actionable error | Error Handling |
| IT-7 | Data unavailable — exits 2 with actionable error (not silent 0) | Error Handling |
| IT-8 | `name::` with non-email value — exits 1 (usage error, not 2) | Parameter Validation |
| IT-9 | Positional bare arg `alice@acme.com` (no `name::`) shows limits | Positional Syntax |
| IT-10 | Prefix `alice` resolves to `alice@acme.com` limits | Prefix Resolution |

### Test Coverage Summary

- Happy Path: 1 test (IT-1)
- Format: 1 test (IT-3)
- Named Account: 1 test (IT-4)
- Not Found: 1 test (IT-5)
- Error Handling: 2 tests (IT-6, IT-7)
- Parameter Validation: 1 test (IT-8)
- Positional Syntax: 1 test (IT-9)
- Prefix Resolution: 1 test (IT-10)

**Total:** 9 integration tests

**Requirement:** FR-18 (feature/013_account_limits.md)

---

### IT-1: Happy Path — Default Output

- **Given:** Active account configured; rate-limit data available.
- **When:** `clp .account.limits`
- **Then:** Exit 0; output contains three utilization lines for session (5h), weekly (all), and weekly (sonnet) with percentage and reset time.; all three utilization categories visible
- **Exit:** 0
- **Source:** [commands.md — .account.limits](../../../../docs/cli/commands.md#command--11-accountlimits) (FR-18)

---

### IT-3: Format — `format::json`

- **Given:** Active account configured; rate-limit data available.
- **When:** `clp .account.limits format::json`
- **Then:** Exit 0; stdout is valid JSON containing utilization percentage fields.; valid JSON output
- **Exit:** 0
- **Source:** [params.md — format::](../../../../docs/cli/params.md#parameter--2-format)

---

### IT-4: Named Account — `name::work@acme.com`

- **Given:** Two accounts configured: active is `personal@home.com`, named `work@acme.com` exists.
- **When:** `clp .account.limits name::work@acme.com`
- **Then:** Exit 0; output reflects `work` account limits.; named account limits displayed
- **Exit:** 0
- **Source:** [commands.md — .account.limits](../../../../docs/cli/commands.md#command--11-accountlimits) (FR-18)

---

### IT-5: Not Found — Unknown Named Account

- **Given:** `ghost@example.com` account does not exist in `~/.persistent/claude/credential/`.
- **When:** `clp .account.limits name::ghost@example.com`
- **Then:** Exit 2; stderr contains `not found` or `ghost@example.com`.; not-found is a runtime error (2), not a usage error (1)
- **Exit:** 2
- **Source:** [commands.md — .account.limits](../../../../docs/cli/commands.md#command--11-accountlimits)

---

### IT-6: Error Handling — No Active Account

- **Given:** No `_active` marker set, no active credentials.
- **When:** `clp .account.limits`
- **Then:** Exit 2; stderr contains actionable message.; actionable error message shown
- **Exit:** 2
- **Source:** [invariant/003_clear_errors.md](../../../../docs/invariant/003_clear_errors.md)

---

### IT-7: Error Handling — Data Unavailable

- **Given:** Active account configured but rate-limit data source unavailable.
- **When:** `clp .account.limits`
- **Then:** Exit 2; stderr contains actionable error naming the missing data source.; explicit error, never silent zero
- **Exit:** 2
- **Source:** [feature/013_account_limits.md](../../../../docs/feature/013_account_limits.md) AC-04

---

### IT-8: Parameter Validation — Non-email `name::` Value

- **Given:** Any environment.
- **When:** `clp .account.limits name::notanemail`
- **Then:** Exit 1; stderr contains `email address`.; email validation is a usage error
- **Exit:** 1
- **Source:** [params.md — name::](../../../../docs/cli/params.md#parameter--1-name)

---

### IT-9: Positional Bare Arg Shows Named Account Limits

- **Given:** Two accounts exist: `work@acme.com` (active) and `alice@acme.com`. Rate-limit data available for `alice@acme.com`.
- **When:** `clp .account.limits alice@acme.com` (no `name::` prefix)
- **Then:** Exits 0; output reflects `alice@acme.com` limits; identical to `clp .account.limits name::alice@acme.com`.
- **Exit:** 0
- **Source:** [015_name_shortcut_syntax.md AC-04](../../../../docs/feature/015_name_shortcut_syntax.md)

---

### IT-10: Prefix Resolves to Named Account Limits

- **Given:** Two accounts saved: `alice@acme.com` and `work@acme.com` (active). Rate-limit data available for `alice@acme.com`.
- **When:** `clp .account.limits alice` (prefix form, no `@`)
- **Then:** Exits 0; output reflects `alice@acme.com` limits.
- **Exit:** 0
- **Source:** [015_name_shortcut_syntax.md AC-05](../../../../docs/feature/015_name_shortcut_syntax.md)
