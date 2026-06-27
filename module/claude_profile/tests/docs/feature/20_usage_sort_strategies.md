# Test: Feature 020 — Usage Sort Strategies

Feature behavioral requirement test cases for `docs/feature/020_usage_sort_strategies.md`. Each FT case maps to one acceptance criterion. Command-level tests (IT-N) are in [cli/command/009_usage.md](../cli/command/09_usage.md).

### AC Coverage Index

| FT | Criterion | AC | Notes |
|----|-----------|-----|-------|
| FT-01 | `sort::name` preserves alphabetical order | AC-01 | Unit test |
| FT-02 | `sort::renew` sinks h-exhausted; non-exhausted sorted by `7d Reset` ascending | AC-01 | Unit test |
| FT-03 | `format::json` order unaffected by `sort::` | AC-11 | Integration |
| FT-04 | Invalid `sort::` value exits 1 naming valid values | AC-07 | Integration |
| FT-05 | Invalid `prefer::` value exits 1 naming valid values | AC-08 | Integration |
| FT-06 | Four-group status partition: 🟢 above 🟡h above 🟡w (incl. both-exhausted) above 🔴 Dead | AC-12 | Unit test |
| FT-07 | `sort::renew` is default when `sort::` omitted | AC-01 | Unit test |
| FT-08 | Within 🟡: h-exhausted before weekly-exhausted; `desc::` doesn't swap sub-groups | AC-12 | Unit test |
| FT-09 | `sort::renew` alphabetical when all numeric sort keys tied (BUG-259) | AC-01 | Tiebreaker |
| FT-10 | `sort::renews` sorts by renewal timer ascending; no renewal data placed last | AC-02 | Unit test |
| FT-11 | h-exhausted + `7d(Son) ≤ 5%` → HExhausted under `prefer::any` (BUG-299) | AC-12 | Group Boundary |
| FT-12 | `prefer::son` + absent Sonnet tier → `prefer_weekly = 0.0` (not 100.0) | AC-05 | Absent-Sonnet fix |
| FT-13 | `sort::` drives footer recommendation — top eligible shown in `Next (<strategy>)` line; footer uses `·`-delimited 2-line format | AC-09 | Recommendation + Footer |
| — | `sort::` + `live::1` stable within each cycle | AC-12 | Live-only (requires `live::1` + real credentials) |

### Test Case Index

| ID | Test Name | AC | Category |
|----|-----------|-----|----------|
| FT-01 | sort::name alphabetical | AC-01 | Sorting |
| FT-02 | sort::renew h-exhausted sunk | AC-01 | Sorting |
| FT-03 | JSON output alphabetical regardless of sort | AC-11 | JSON No-op |
| FT-04 | Invalid sort value rejected | AC-07 | Validation |
| FT-05 | Invalid prefer value rejected | AC-08 | Validation |
| FT-06 | Four-group partition: 🟢 above 🟡h above 🟡w (incl. both-exhausted) above 🔴 Dead | AC-12 | Tier Grouping |
| FT-07 | `sort::renew` is default when `sort::` omitted | AC-01 | Default |
| FT-08 | Within 🟡: h-exhausted before weekly-exhausted; sub-grouping not reversed by `desc::` | AC-12 | Yellow Sub-Grouping |
| FT-09 | sort::renew alphabetical tiebreaker when all numeric keys tied | AC-01 | Tiebreaker |
| FT-10 | sort::renews ascending; no renewal data last | AC-02 | Renews Sort |
| FT-11 | h-exhausted account with 7d_son ≤ 5% lands in HExhausted (not Red) under prefer::any (BUG-299) | AC-12 | Group Boundary |
| FT-12 | prefer::son + absent Sonnet tier → prefer_weekly = 0.0 (not 100.0) | AC-05 | Absent-Sonnet fix |
| FT-13 | sort:: drives footer recommendation — top eligible in Next line; `·`-delimited format | AC-09 | Recommendation + Footer |

**Total:** 13 FT cases

---

### FT-01: `sort::name` preserves alphabetical order

- **Given:** Three `AccountQuota` structs with names `c@x.com`, `a@x.com`, `b@x.com` in that order.
- **When:** `sort_indices(&accounts, SortStrategy::Name, None, PreferStrategy::Any, 0)`
- **Then:** Indices reordered to: `a@x.com`, `b@x.com`, `c@x.com`.
- **Exit:** n/a (unit test — function return assertion)
- **Source fn:** `test_sort_name_alphabetical` (in `src/usage/sort.rs`)
- **Source:** [feature/020_usage_sort_strategies.md AC-01](../../../docs/feature/020_usage_sort_strategies.md)

---

### FT-02: `sort::renew` sinks h-exhausted accounts to bottom; non-exhausted sorted by `7d Reset` ascending

- **Given:** Four `AccountQuota` structs: `A` (`seven_day.resets_at=now+600s`, `5h_left=50%`), `B` (`seven_day.resets_at=now+2700s`, `5h_left=50%`), `C` (`seven_day.resets_at=now+7200s`, `5h_left=50%`), `D` (`five_hour_util=99%` — **h-exhausted**). All `result = Ok(...)`.
- **When:** `sort_indices(&accounts, SortStrategy::Renew, None, PreferStrategy::Any, now_secs)`
- **Then:** Order: `A`, `B`, `C`, then `D` (sunk). Non-h-exhausted sorted by soonest `7d Reset` countdown first.
- **Exit:** n/a (unit test)
- **Source fn:** `test_sort_renew_soonest_first_exhausted_last` (in `src/usage/sort.rs`)
- **Source:** [feature/020_usage_sort_strategies.md AC-01](../../../docs/feature/020_usage_sort_strategies.md)

---

### FT-03: `format::json` output alphabetical regardless of `sort::`

- **Given:** Two `AccountQuota` structs `zzz@test.com` (70% left) and `aaa@test.com` (20% left) in that order (reverse-alphabetical input).
- **When:** `render_json(&accounts)` — no sort applied.
- **Then:** JSON output preserves input order — `zzz@test.com` appears before `aaa@test.com`, confirming `render_json` does not re-sort.
- **Exit:** n/a (unit test against `render_json`)
- **Source fn:** `test_json_unaffected_by_sort` (in `src/usage/mod.rs`)
- **Source:** [feature/020_usage_sort_strategies.md AC-11](../../../docs/feature/020_usage_sort_strategies.md)

---

### FT-04: Invalid `sort::` value exits 1 naming valid values

- **Given:** Any environment.
- **When:** `clp .usage sort::bogus`
- **Then:** Exits 1. Stderr names the three valid values: `name`, `renew`, `renews`.
- **Exit:** 1
- **Source fn:** `it057_sort_invalid_value_exit_1` (in `tests/cli/usage_test.rs`); unit: `test_sort_strategy_parse_invalid_rejected` (in `src/usage/mod.rs`)
- **Source:** [feature/020_usage_sort_strategies.md AC-07](../../../docs/feature/020_usage_sort_strategies.md)

---

### FT-05: Invalid `prefer::` value exits 1 naming valid values

- **Given:** Any environment.
- **When:** `clp .usage prefer::bogus`
- **Then:** Exits 1. Stderr names the three valid values: `any`, `opus`, `sonnet`.
- **Exit:** 1
- **Source fn:** `it058_prefer_invalid_value_exit_1` (in `tests/cli/usage_test.rs`); unit: `test_prefer_strategy_parse_invalid_rejected` (in `src/usage/mod.rs`)
- **Source:** [feature/020_usage_sort_strategies.md AC-08](../../../docs/feature/020_usage_sort_strategies.md)

---

### FT-06: Four-group status partition: 🟢 above 🟡 h-exhausted above 🟡 weekly-exhausted (incl. both-exhausted) above 🔴 Dead

- **Given:** Five `AccountQuota` structs: `green@test.com` (5h_left=80%, 7d_left=60% — both available, 🟢 G1), `h_exh@test.com` (5h_left=3%, 7d_left=50% — 5h exhausted, 🟡 G2), `weekly_exh@test.com` (5h_left=80%, 7d_left=2% — 7d exhausted, 🟡 G3), `both_exh@test.com` (5h_left=6%, 7d_left=4% — both exhausted, 🟡 G3 weekly-exhausted — 7d is binding), `dead@test.com` (result=Err — 🔴 G4). Any sort strategy.
- **When:** `sort_indices(&accounts, SortStrategy::Name, None, PreferStrategy::Any, 0)` — name sort would interleave groups alphabetically.
- **Then:** Output order: `green@test.com` (🟢 G1), then G2 h-exhausted before G3 weekly-exhausted (both `weekly_exh` and `both_exh` — alphabetical within G3), then `dead@test.com` (🔴 G4). Four-group partition overrides alphabetical sort. Fix(BUG-321): `both_exh@test.com` sorts to G3 weekly-exhausted (🟡), not G4 Dead (🔴).
- **Exit:** n/a (unit test)
- **Source fn:** `mre_bug321_four_group_partition_order` (in `src/usage/sort.rs` or `src/usage/mod.rs`)
- **Source:** [feature/020_usage_sort_strategies.md AC-12](../../../docs/feature/020_usage_sort_strategies.md)

---

### FT-07: `sort::renew` is default when `sort::` omitted

- **Given:** Two `AccountQuota` structs: `early@test.com` (`seven_day.resets_at=now+3600s` — resets in 1h, `seven_day.utilization=20%` — 80% left), `late@test.com` (`seven_day.resets_at=now+86400s` — resets in 24h, `seven_day.utilization=80%` — 20% left). Both non-exhausted.
- **When:** `sort_indices(&accounts, SortStrategy::Renew, None, PreferStrategy::Any, now_secs)` — default strategy is `renew`.
- **Then:** `early@test.com` ranks first (resets in 1h — soonest reset first), `late@test.com` second. Confirms renew default = `desc::0`.
- **Exit:** n/a (unit test)
- **Source fn:** `test_sort_renew_default_equals_desc0` (in `src/usage/sort.rs`); `it137_sort_default_is_renew_structural` (in `tests/cli/usage_test.rs`)
- **Source:** [feature/020_usage_sort_strategies.md AC-01](../../../docs/feature/020_usage_sort_strategies.md)

---

### FT-08: Within 🟡 tier — h-exhausted before weekly-exhausted; `desc::` does not swap sub-groups

- **Given:** Unit test. Three `AccountQuota` structs, all 🟡:
  - `weekly@x.com`: `five_hour.utilization=10.0` (90% left), `seven_day.utilization=98.0` (2% left) → **weekly-exhausted** sub-group (alpha first)
  - `sess_a@x.com`: `five_hour.utilization=99.0` (1% left), `seven_day.utilization=30.0` (70% left) → **h-exhausted** sub-group
  - `sess_b@x.com`: `five_hour.utilization=97.0` (3% left), `seven_day.utilization=40.0` (60% left) → **h-exhausted** sub-group
- **When-A:** `render_text(...)` with `SortStrategy::Name` (default `desc::0`) — alpha order is `sess_a -> sess_b -> weekly`.
- **When-B:** `render_text(...)` with `SortStrategy::Name` and `desc::1` — reversed alpha within each sub-group.
- **Then-A (default):** Output order: `sess_a@x.com` (h-exhausted sub-group), `sess_b@x.com` (h-exhausted sub-group), `weekly@x.com` (weekly sub-group). `weekly@x.com` is last despite being alpha-first.
- **Then-B (desc::1):** Output order: `sess_b@x.com`, `sess_a@x.com` (h-exhausted sub-group reversed), `weekly@x.com` (weekly sub-group last — not moved to front by `desc::1`).
- **Exit:** n/a (unit test — position assertion via `output.find()`)
- **Source fn:** `test_ft16_009_yellow_tier_session_before_weekly` (When-A), `test_ft15_020_yellow_sub_grouping_not_reversed_by_desc` (When-B) (in `src/usage/mod.rs`)
- **Source:** [feature/020_usage_sort_strategies.md AC-12](../../../docs/feature/020_usage_sort_strategies.md)

---

### FT-09: `sort::renew` alphabetical when all numeric sort keys tied (BUG-259)

- **Given:** Three `AccountQuota` structs inserted in **reverse** alphabetical order: `charlie@test.com`, `bravo@test.com`, `alpha@test.com`. All have identical `seven_day.utilization=50%` and `seven_day.resets_at` set to `FAR_FUTURE_MS` — all sort keys are identical.
- **When:** `sort_indices(&accounts, SortStrategy::Renew, None, PreferStrategy::Any, 0)`
- **Then:** `alpha@test.com` ranks first (alphabetical winner when all numeric keys tie). Confirms the final name tiebreaker prevents filesystem-order-dependent non-determinism.
- **Exit:** n/a (unit test — name assertion on `accounts[idx[0]].name`)
- **Source fn:** `mre_bug259_sort_renew_alphabetical_when_all_keys_tied` (in `src/usage/sort.rs`)
- **Source:** [feature/020_usage_sort_strategies.md AC-01](../../../docs/feature/020_usage_sort_strategies.md)

---

### FT-10: `sort::renews` sorts by renewal timer ascending; no renewal data placed last

- **Given:** Three `AccountQuota` structs: `soon_renew@test.com` (`renewal_at=now+3600s` — renews in 1h), `later_renew@test.com` (`renewal_at=now+86400s`), `no_renew@test.com` (no `renewal_at` — scores `u64::MAX`).
- **When:** `sort_indices(&accounts, SortStrategy::Renews, None, PreferStrategy::Any, now)`
- **Then:** Order: `soon_renew@test.com` (soonest renewal), `later_renew@test.com`, `no_renew@test.com` (no data, placed last). Default `desc::0`.
- **Exit:** n/a (unit test — index assertion)
- **Source fn:** `test_sort_renews_ascending` (in `src/usage/sort_next_tests.rs`)
- **Source:** [feature/020_usage_sort_strategies.md AC-02](../../../docs/feature/020_usage_sort_strategies.md)

---

### FT-11: h-exhausted account with `7d(Son) ≤ 5%` lands in HExhausted (not Red) under `prefer::any` (BUG-299)

- **Given:** Two `AccountQuota` structs:
  - `account-a`: `five_hour_util=100%` (5h_left=0%, h-exhausted), `seven_day_util=68%` (7d_left=32%), `seven_day_sonnet_util=95%` (7d_son_left=5%).
  - `weekly-exh`: `five_hour_util=10%` (5h_left=90%), `seven_day_util=96%` (7d_left=4%, weekly-exhausted).
- **When:** `sort_indices(&accounts, SortStrategy::Name, None, PreferStrategy::Any, 0)`
- **Then:** `account-a` appears before `weekly-exh`. `account-a` is in HExhausted (group 2); `weekly-exh` is in WeeklyExhausted (group 3). Under `prefer::any`, `prefer_weekly(account-a) = min(32%, 5%) = 5.0` — the bug used this value and placed `account-a` in Red; the fix uses `seven_day_left = 32% > 5.0%` → HExhausted.
- **Exit:** n/a (unit test — position assertion)
- **Source fn:** `mre_bug299_h_exhausted_misclassified_as_red_prefer_any` (in `src/usage/sort.rs`)
- **Source:** [feature/020_usage_sort_strategies.md AC-12](../../../docs/feature/020_usage_sort_strategies.md); BUG-299

---

### FT-12: `prefer::son` + absent Sonnet tier → `prefer_weekly = 0.0` (not 100.0)

- **Given:** An `AccountQuota` with `seven_day_sonnet = None` (no Sonnet tier) and `seven_day_util=30%` (7d_left=70%). `prefer::son` in effect.
- **When:** `prefer_weekly(aq, PreferStrategy::Sonnet)` is called (internally delegates to `relevant_quotas(aq, Sonnet).1`).
- **Then:** Returns `0.0`. Absent Sonnet tier under `prefer::son` = unknown Sonnet capacity, not 100%. The eligibility gate `prefer_weekly ≤ 5.0` fires (0.0 ≤ 5.0) → account is ineligible for next-account recommendation.
- **Exit:** n/a (unit test — return value assertion)
- **Note:** Phase 2 fix from Plan 019. Old code: `map_or(0.0, |p| p.utilization)` returned `100.0 - 0.0 = 100.0`, treating absent tier as fully available. Fix: `if let Some(ref son)` guard returns `0.0` when `seven_day_sonnet = None`.
- **Source fn:** `test_relevant_quotas_son_no_sonnet` (in `src/usage/format_tests.rs`)
- **Source:** [feature/020_usage_sort_strategies.md AC-05](../../../docs/feature/020_usage_sort_strategies.md)

---

### FT-13: `sort::` drives footer recommendation — top eligible account shown in `Next (<strategy>)` line; `·`-delimited 2-line format

- **Given (unit test):** Three `AccountQuota` structs: `current@x.com` (`is_current=true`, valid quota, `five_hour_util=20%`), `eligible@x.com` (valid quota, `five_hour_util=10%`, non-current, non-active), `exhausted@x.com` (valid quota, `five_hour_util=99%`, h-exhausted). `session_model = Some("sonnet")`, `session_effort = Some("low")`.
- **When:** `render_text(&accounts, SortStrategy::Renew, ...)` is called.
- **Then:** Footer line 1 contains `Current · current@x.com · sonnet/low · 2/3` — identifies the `✓` account with session model/effort and valid/total count. Footer line 2 contains `Next (renew) · eligible@x.com · ` — the top eligible (non-current, non-exhausted) account in renew sort order appears as the recommendation. `exhausted@x.com` is skipped (h-exhausted → ineligible). Both lines use `·` delimiters with column alignment.
- **Exit:** n/a (unit test — string assertions on `render_text` output)
- **Source:** [feature/020_usage_sort_strategies.md AC-09](../../../docs/feature/020_usage_sort_strategies.md)
