# Test: Sort Control Parameter Group

Interaction tests for Group 4 (Sort Control: `sort::`, `desc::`, `prefer::`). See [param_group/004_sort_control.md](../../../../docs/cli/param_group/004_sort_control.md) for specification.

### Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| CC-1 | `sort::` and `desc::` have no effect on `format::json` output | JSON No-op |
| CC-2 | `prefer::` selects weekly column for `sort::renew` tiebreak; group membership unchanged | Prefer x Renew |
| CC-3 | `sort::` drives `-> Next` recommendation in footer | Sort x Recommendation |
| CC-4 | `sort::name` and `sort::renew` produce different TEXT row ordering for same accounts | Sort Strategy Divergence |

---

### CC-1: `sort::` and `desc::` have no effect on `format::json` output

- **Behavioral Divergence:** `sort::renew format::json` vs `sort::name format::json` — JSON array order is alphabetical in both cases, while text output would differ.
- **Given:** Two saved accounts: `b@x.com` and `a@x.com`; both with credential files missing accessToken (will appear with error in JSON but in same order as text accounts).
- **When-A:** `clp .usage sort::name format::json`
- **When-B:** `clp .usage sort::renew format::json`
- **Then-A:** JSON array with `a@x.com` at index 0, `b@x.com` at index 1.
- **Then-B:** Same order as Then-A -- `sort::renew` does not reorder JSON output.
- **Exit:** 0 both cases
- **Source:** [feature/020_usage_sort_strategies.md AC-11](../../../../docs/feature/020_usage_sort_strategies.md)

---

### CC-2: `prefer::` selects weekly column for `sort::renew` tiebreak; group membership unchanged

- **Behavioral Divergence:** `sort::renew prefer::any` vs `sort::renew prefer::opus` can produce different **within-group ordering** when two accounts in the same status group have the same renewal event time but different `7d Left` / `7d(Son)` values. `prefer::` does **not** change which group an account belongs to — the four-group status partition always uses raw `7d Left` for the weekly boundary (AC-12).
- **Given:** Unit-level test. Two Green accounts (`5h Left > 15%`, `7d Left > 5%`) with identical `renewal_event_secs`. Account P: `7d_left=60%, 7d_son=20%`. Account Q: `7d_left=40%, 7d_son=80%`. Same renewal event time → falls to tiebreak.
- **When-A:** `sort::renew prefer::any` → `prefer_weekly(P) = min(60%, 20%) = 20%`, `prefer_weekly(Q) = min(40%, 80%) = 40%` → P (20%) < Q (40%) → **P first**.
- **When-B:** `sort::renew prefer::opus` → `prefer_weekly(P) = 60%`, `prefer_weekly(Q) = 40%` → Q (40%) < P (60%) → **Q first**.
- **Then-A:** Within the Green group: P ranks before Q.
- **Then-B:** Within the Green group: Q ranks before P.
- **Note:** Both accounts remain in the Green group in both cases — `prefer::` changes tiebreak order only.
- **Exit:** 0 both cases (unit-level assertion)
- **Source:** [feature/020_usage_sort_strategies.md AC-05/AC-06](../../../../docs/feature/020_usage_sort_strategies.md)

---

### CC-3: `sort::` drives `-> Next` recommendation in footer

- **Behavioral Divergence:** The `-> Next` footer recommendation is now determined by the active `sort::` strategy. Different strategies can recommend different accounts. `sort::name` recommends the first alphabetically; `sort::renew` recommends the account with the soonest renewal event.
- **Given:** Unit-level or integration test. Two non-exhausted accounts: `a@x.com` (7d reset in 2h) and `b@x.com` (7d reset in 10min).
- **When-A:** `clp .usage sort::name`
- **When-B:** `clp .usage sort::renew`
- **Then-A:** Footer: "Next (name): a@x.com" (first alphabetically).
- **Then-B:** Footer: "Next (renew): b@x.com" (soonest renewal event).
- **Exit:** 0 both cases
- **Source:** [feature/020_usage_sort_strategies.md AC-09](../../../../docs/feature/020_usage_sort_strategies.md)

---

### CC-4: `sort::name` and `sort::renew` produce different TEXT row ordering

- **Behavioral Divergence:** Unlike `format::json` (where row order is always alphabetical — CC-1), TEXT output row ordering is determined by `sort::`. `sort::name` uses alphabetical order; `sort::renew` uses next renewal event time within each status group.
- **Given:** Two owned accounts: `z@x.com` (renews in 10 min) and `a@x.com` (renews in 10 hours). Both in the same status group (Green). Valid cached quota data.
- **When-A:** `clp .usage sort::name`
- **When-B:** `clp .usage sort::renew`
- **Then-A:** Table row order: `a@x.com` first (alphabetical ascending).
- **Then-B:** Table row order: `z@x.com` first (soonest renewal within the group).
- **Note:** Both use text format. JSON format would show `a@x.com` first in both cases (CC-1).
- **Exit:** 0 both cases
- **Source:** [feature/020_usage_sort_strategies.md AC-11, AC-13](../../../../docs/feature/020_usage_sort_strategies.md)
