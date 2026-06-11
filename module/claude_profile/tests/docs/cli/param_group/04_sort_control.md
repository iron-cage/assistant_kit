# Test: Sort Control Parameter Group

Interaction tests for Group 4 (Sort Control: `sort::`, `desc::`, `prefer::`). See [param_group/004_sort_control.md](../../../../docs/cli/param_group/004_sort_control.md) for specification.

### Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| CC-1 | `sort::` and `desc::` have no effect on `format::json` output | JSON No-op |
| CC-2 | `prefer::` interacts with `sort::endurance` qualification | Prefer × Endurance |
| CC-3 | `sort::` does not affect `→ Next` recommendation in footer | Sort × Recommendation |

---

### CC-1: `sort::` and `desc::` have no effect on `format::json` output

- **Behavioral Divergence:** `sort::endurance format::json` vs `sort::name format::json` — JSON array order is alphabetical in both cases, while text output would differ.
- **Given:** Two saved accounts: `b@x.com` and `a@x.com`; both with credential files missing accessToken (will appear with error in JSON but in same order as text accounts).
- **When-A:** `clp .usage sort::name format::json`
- **When-B:** `clp .usage sort::endurance format::json`
- **Then-A:** JSON array with `a@x.com` at index 0, `b@x.com` at index 1.
- **Then-B:** Same order as Then-A — `sort::endurance` does not reorder JSON output.
- **Exit:** 0 both cases
- **Source:** [feature/020_usage_sort_strategies.md AC-13](../../../../docs/feature/020_usage_sort_strategies.md)

---

### CC-2: `prefer::` interacts with `sort::endurance` qualification

- **Behavioral Divergence:** `sort::endurance prefer::sonnet` vs `sort::endurance prefer::any` can produce different row rankings when an account's `7d(Son)` ≥ 30% but `min(7d Left, 7d(Son))` < 30%. The account qualifies with `prefer::sonnet` but not with `prefer::any`.
- **Given:** Unit-level test. `AccountQuota` vectors with `seven_day_sonnet.utilization = 65%` (35% left) and `seven_day.utilization = 90%` (10% left). 5h Reset within 15–60 min.
- **When-A:** `sort::endurance prefer::any` → `prefer_weekly` = min(10%, 35%) = 10% < 30% → **unqualified**.
- **When-B:** `sort::endurance prefer::sonnet` → `prefer_weekly` = 35% ≥ 30% → **qualified** (ranked first).
- **Then-A:** Account not in qualified tier.
- **Then-B:** Account in qualified tier, ranked above unqualified accounts.
- **Exit:** 0 both cases
- **Source:** [feature/020_usage_sort_strategies.md AC-07](../../../../docs/feature/020_usage_sort_strategies.md)

---

### CC-3: `sort::` does not affect `→ Next` recommendation in footer

- **Behavioral Divergence:** The `→ Next` footer recommendation is determined by `find_recommendation()` algorithm (highest `5h Left` → highest expiry → highest `7d Left` → alpha), not by `sort::`. Sort changes display order but not the recommended account. An account ranked 3rd by `sort::drain` can still be the `→ Next` recommendation.
- **Given:** Unit-level or integration test. Three accounts: `a@x.com` (5h Left=80%, non-exhausted), `b@x.com` (5h Left=50%, non-exhausted), `c@x.com` (5h Left=3%, exhausted by drain/reset floor at ≤15%). `sort::drain` order: `b@x.com`, `a@x.com`, then `c@x.com` sunk.
- **When-A:** `clp .usage sort::name`
- **When-B:** `clp .usage sort::drain`
- **Then-A:** Row order: a, b, c. Footer: "Next: a@x.com" (highest 5h Left).
- **Then-B:** Row order: b (50%), a (80%), c (3% sunk). Footer: still "Next: a@x.com" — same recommendation regardless of sort.
- **Exit:** 0 both cases
- **Source:** [feature/020_usage_sort_strategies.md AC-11](../../../../docs/feature/020_usage_sort_strategies.md)

---

> **Note:** CC-4 removed — unit test of `sort_indices()` function return not directly observable via clp output — behavior only verifiable at unit-test level. Unit tests live in `src/usage/sort.rs` as `test_sort_drain_prefer_sonnet_primary` and `test_sort_drain_prefer_any_primary`.
