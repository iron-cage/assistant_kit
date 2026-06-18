# Test: Sort Control Parameter Group

Interaction tests for Group 4 (Sort Control: `sort::`, `desc::`, `prefer::`). See [param_group/004_sort_control.md](../../../../docs/cli/param_group/004_sort_control.md) for specification.

### Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| CC-1 | `sort::` and `desc::` have no effect on `format::json` output | JSON No-op |
| CC-2 | `prefer::` selects weekly column for status partition and `sort::renew` tiebreak | Prefer x Renew |
| CC-3 | `sort::` drives `-> Next` recommendation in footer | Sort x Recommendation |

---

### CC-1: `sort::` and `desc::` have no effect on `format::json` output

- **Behavioral Divergence:** `sort::renew format::json` vs `sort::name format::json` -- JSON array order is alphabetical in both cases, while text output would differ.
- **Given:** Two saved accounts: `b@x.com` and `a@x.com`; both with credential files missing accessToken (will appear with error in JSON but in same order as text accounts).
- **When-A:** `clp .usage sort::name format::json`
- **When-B:** `clp .usage sort::renew format::json`
- **Then-A:** JSON array with `a@x.com` at index 0, `b@x.com` at index 1.
- **Then-B:** Same order as Then-A -- `sort::renew` does not reorder JSON output.
- **Exit:** 0 both cases
- **Source:** [feature/020_usage_sort_strategies.md AC-13](../../../../docs/feature/020_usage_sort_strategies.md)

---

### CC-2: `prefer::` selects weekly column for status partition and `sort::renew` tiebreak

- **Behavioral Divergence:** `sort::renew prefer::sonnet` vs `sort::renew prefer::any` can produce different status group membership when an account's `7d(Son)` > 5% but `min(7d Left, 7d(Son))` <= 5%. The account lands in Green with `prefer::sonnet` but in weekly-exhausted with `prefer::any`.
- **Given:** Unit-level test. `AccountQuota` vectors with `seven_day_sonnet.utilization = 90%` (10% left) and `seven_day.utilization = 97%` (3% left). 5h Left > 15%.
- **When-A:** `sort::renew prefer::any` -> `prefer_weekly` = min(3%, 10%) = 3% <= 5% -> **weekly-exhausted** group.
- **When-B:** `sort::renew prefer::sonnet` -> `prefer_weekly` = 10% > 5% -> **Green** group.
- **Then-A:** Account in weekly-exhausted group (below Green accounts).
- **Then-B:** Account in Green group.
- **Exit:** 0 both cases
- **Source:** [feature/020_usage_sort_strategies.md AC-07](../../../../docs/feature/020_usage_sort_strategies.md)

---

### CC-3: `sort::` drives `-> Next` recommendation in footer

- **Behavioral Divergence:** The `-> Next` footer recommendation is now determined by the active `sort::` strategy. Different strategies can recommend different accounts. `sort::name` recommends the first alphabetically; `sort::renew` recommends the account with the soonest renewal event.
- **Given:** Unit-level or integration test. Two non-exhausted accounts: `a@x.com` (7d reset in 2h) and `b@x.com` (7d reset in 10min).
- **When-A:** `clp .usage sort::name`
- **When-B:** `clp .usage sort::renew`
- **Then-A:** Footer: "Next (name): a@x.com" (first alphabetically).
- **Then-B:** Footer: "Next (renew): b@x.com" (soonest renewal event).
- **Exit:** 0 both cases
- **Source:** [feature/020_usage_sort_strategies.md AC-10](../../../../docs/feature/020_usage_sort_strategies.md)
