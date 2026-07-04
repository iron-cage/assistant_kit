# Test: user story 3 — Multi-Account Quota Monitoring

User acceptance tests for the "Multi-Account Quota Monitoring" story. Each UA-N case maps
to one Acceptance Criterion from
[docs/cli/user_story/003_quota_monitoring.md](../../../../docs/cli/user_story/003_quota_monitoring.md).

**Persona:** Power user managing multiple Claude accounts to maximize available quota.

### Test Case Index

| ID | Test Name | Acceptance Criterion |
|----|-----------|---------------------|
| UA-1 | `.usage` shows all saved accounts with quota, expiry, and renewal in one table | AC-1 |
| UA-2 | `sort::renew` ranks by soonest renewal event; `sort::renews` ranks by soonest billing renewal | AC-2 |
| UA-3 | `live::1` continuously refreshes the table at `interval::` seconds | AC-3 |
| UA-4 | Footer `Next:` line recommends the top eligible account per the active `sort::` strategy | AC-4 |
| UA-5 | `min_5h::X` and `min_7d::X` filter to accounts meeting minimum quota thresholds | AC-5 |

### Test Coverage Summary

- Table display: 1 test
- Sort strategies: 1 test
- Live mode: 1 test
- Next recommendation: 1 test
- Row filtering: 1 test

**Total:** 5 user acceptance tests

---

### UA-1: `.usage` shows all saved accounts with 5h/7d quota, expiry, and renewal in one table

- **Given:** Three saved accounts with differing 5h/7d quota, `expiresAt`, and `_renewal_at` values.
- **When:** `clp .usage`
- **Then:** Exit 0. Output table contains one row per saved account. Each row includes 5h Left and 7d Left quota columns, token expiry, and renewal date. All three accounts visible in a single view.
- **Exit:** 0
- **Source:** [003_quota_monitoring.md — AC-1](../../../../docs/cli/user_story/003_quota_monitoring.md)

---

### UA-2: `sort::renew` ranks by soonest renewal event; `sort::renews` ranks by soonest billing renewal

- **Given:** Two saved accounts with different renewal dates.
- **When (a):** `clp .usage sort::renew`
- **When (b):** `clp .usage sort::renews`
- **Then (a):** Exit 0. Rows ordered with soonest quota renewal account first.
- **Then (b):** Exit 0. Rows ordered with soonest billing renewal account first.
- **Exit:** 0
- **Source:** [003_quota_monitoring.md — AC-2](../../../../docs/cli/user_story/003_quota_monitoring.md)

---

### UA-3: `live::1` continuously refreshes the table at `interval::` seconds

- **Given:** Multiple saved accounts with quota data. TTY attached.
- **When:** `clp .usage live::1 interval::5` (run for at least 2 refresh cycles, then Ctrl-C)
- **Then:** Table refreshes on screen at approximately 5-second intervals. Each cycle shows current quota snapshot. Exit 0 on interrupt (or graceful exit code).
- **Exit:** 0
- **Source:** [003_quota_monitoring.md — AC-3](../../../../docs/cli/user_story/003_quota_monitoring.md)

---

### UA-4: Footer `Next:` line recommends the top eligible account per the active `sort::` strategy

- **Given:** Two saved accounts with differing expiry; one is current.
- **When:** `clp .usage sort::renew`
- **Then:** Exit 0. Stdout contains a footer line matching `Next` and the non-current account name. No table data row contains a `→` marker in the flag column.
- **Exit:** 0
- **Source:** [003_quota_monitoring.md — AC-4](../../../../docs/cli/user_story/003_quota_monitoring.md)

---

### UA-5: `min_5h::X` and `min_7d::X` filter to accounts meeting minimum quota thresholds

- **Given:** Three accounts: alice (5h=80%, 7d=70%), bob (5h=30%, 7d=20%), carol (5h=50%, 7d=90%).
- **When:** `clp .usage min_5h::40`
- **Then:** Exit 0. Only alice and carol appear in output (both have ≥ 40% 5h quota). Bob excluded.
- **Exit:** 0
- **Source:** [003_quota_monitoring.md — AC-5](../../../../docs/cli/user_story/003_quota_monitoring.md)
