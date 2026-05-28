# Test: Feature 020 — Usage Sort Strategies

Feature behavioral requirement test cases for `docs/feature/020_usage_sort_strategies.md`. Each FT case maps to one acceptance criterion. Command-level tests (IT-N) are in [cli/command/009_usage.md](../cli/command/009_usage.md).

### AC Coverage Index

| FT | Criterion | AC | Notes |
|----|-----------|-----|-------|
| FT-01 | `sort::name` preserves alphabetical order | AC-01 | Unit test |
| FT-02 | `sort::endurance` qualified accounts ranked first | AC-02 | Unit test |
| FT-03 | `sort::drain` sinks h-exhausted; non-exhausted sorted by `7d Left` ascending | AC-03 | Unit test |
| FT-04 | `sort::renew` sinks h-exhausted; non-exhausted sorted by `7d Reset` ascending | AC-04 | Unit test |
| FT-05 | `desc::1` reverses non-h-exhausted tier; h-exhausted floor unchanged | AC-05 | Unit test |
| FT-06 | Context-sensitive `desc::` defaults per strategy | AC-06 | Unit test |
| FT-07 | `prefer::sonnet` uses `7d(Son)` for endurance qualification | AC-07 | Unit test |
| FT-08 | `format::json` order unaffected by `sort::` | AC-13 | Integration |
| FT-09 | Invalid `sort::` value exits 1 naming valid values | AC-09 | Integration |
| FT-10 | Invalid `prefer::` value exits 1 naming valid values | AC-10 | Integration |
| FT-11 | `sort::` does not affect `next::` recommendation | AC-11 | Unit test |
| FT-12 | `prefer::` governs drain primary sort key (`7d Left` ascending, prefer-aware) | AC-08 | Unit test |
| FT-13 | Three-tier grouping: 🟢 above 🟡 above 🔴 | AC-14 | Unit test |
| FT-14 | `sort::renew` is default when `sort::` omitted | AC-01 | Unit test |
| FT-15 | Within 🟡: h-exhausted before weekly-exhausted; `desc::` doesn't swap sub-groups | AC-14 | Unit test |
| FT-16 | `sort::endurance` unqualified tiebreak by highest weekly when session quotas tied | AC-02 | Unit test |
| FT-17 | `sort::next` delegates to active `next::` strategy; `→` winner appears at row 1 | AC-15 | Integration |

### Test Case Index

| ID | Test Name | AC | Category |
|----|-----------|-----|----------|
| FT-01 | sort::name alphabetical | AC-01 | Sorting |
| FT-02 | sort::endurance qualified above unqualified | AC-02 | Sorting |
| FT-03 | sort::drain h-exhausted sunk | AC-03 | Sorting |
| FT-04 | sort::renew h-exhausted sunk | AC-04 | Sorting |
| FT-05 | desc::1 reversal preserves h-exhausted floor | AC-05 | Direction |
| FT-06 | Context-sensitive desc defaults | AC-06 | Direction |
| FT-07 | prefer::sonnet wires 7d(Son) into endurance | AC-07 | Prefer |
| FT-08 | JSON output alphabetical regardless of sort | AC-13 | JSON No-op |
| FT-09 | Invalid sort value rejected | AC-09 | Validation |
| FT-10 | Invalid prefer value rejected | AC-10 | Validation |
| FT-11 | Recommendation unaffected by sort | AC-11 | Independence |
| FT-12 | prefer:: drain primary key divergence | AC-08 | Primary Key |
| FT-13 | Three-tier grouping: 🟢 above 🟡 above 🔴 | AC-14 | Tier Grouping |
| FT-14 | `sort::renew` is default when `sort::` omitted | AC-01 | Default |
| FT-15 | Within 🟡: h-exhausted before weekly-exhausted; sub-grouping not reversed by `desc::` | AC-14 | Yellow Sub-Grouping |
| FT-16 | sort::endurance unqualified tiebreak by weekly | AC-02 | Tiebreak |
| FT-17 | sort::next delegates to active next:: strategy | AC-15 | Meta-Strategy |

**Total:** 17 FT cases

---

### FT-01: `sort::name` preserves alphabetical order

- **Given:** Three `AccountQuota` structs with names `c@x.com`, `a@x.com`, `b@x.com` in that order.
- **When:** `sort_indices(&accounts, SortStrategy::Name, None, PreferStrategy::Any, 0)`
- **Then:** Indices reordered to: `a@x.com`, `b@x.com`, `c@x.com`.
- **Exit:** n/a (unit test — function return assertion)
- **Source fn:** `test_sort_name_alphabetical` (in `src/usage.rs`)
- **Source:** [feature/020_usage_sort_strategies.md AC-01](../../../../docs/feature/020_usage_sort_strategies.md)

---

### FT-02: `sort::endurance` qualified accounts ranked above unqualified

- **Given:** Three `AccountQuota` structs: `A` (5h_reset in 30 min, weekly=40% — qualified), `B` (5h_reset in 3h, weekly=80% — unqualified: reset not in 15–60 min range), `C` (5h_reset in 50 min, weekly=20% — unqualified: weekly < 30%). All `result = Ok(...)`.
- **When:** `sort_indices(&accounts, SortStrategy::Endurance, None, PreferStrategy::Any, now_secs)`
- **Then:** `A` ranks first (only qualified account); `B` and `C` follow (unqualified).
- **Exit:** n/a (unit test)
- **Source fn:** `test_sort_endurance_default_equals_desc1` (in `src/usage.rs`)
- **Source:** [feature/020_usage_sort_strategies.md AC-02](../../../../docs/feature/020_usage_sort_strategies.md)

---

### FT-03: `sort::drain` sinks h-exhausted accounts to bottom; non-exhausted sorted by `7d Left` ascending

- **Given:** Three `AccountQuota` structs: `A` (`five_hour_util=99%` — **h-exhausted**, `seven_day_util=40%` → 60% 7d Left), `B` (`five_hour_util=30%`, `seven_day_util=70%` → 30% 7d Left — lowest weekly), `C` (`five_hour_util=30%`, `seven_day_util=0%` → 100% 7d Left — most weekly). All `result = Ok(...)`.
- **When:** `sort_indices(&accounts, SortStrategy::Drain, None, PreferStrategy::Any, 0)` (desc=false is the default for drain)
- **Then:** Order: `B` (30% 7d Left — lowest non-exhausted weekly), `C` (100% 7d Left), then `A` (h-exhausted, sunk). h-exhausted floor at bottom; non-h-exhausted sorted by `7d Left` ascending; tiebreak `5h Left`.
- **Exit:** n/a (unit test)
- **Source fn:** `test_sort_drain_exhausted_sunk_rest_ascending` (in `src/usage.rs`)
- **Source:** [feature/020_usage_sort_strategies.md AC-03](../../../../docs/feature/020_usage_sort_strategies.md)

---

### FT-04: `sort::renew` sinks h-exhausted accounts to bottom; non-exhausted sorted by `7d Reset` ascending

- **Given:** Four `AccountQuota` structs: `A` (`seven_day.resets_at=now+600s`, `5h_left=50%`), `B` (`seven_day.resets_at=now+2700s`, `5h_left=50%`), `C` (`seven_day.resets_at=now+7200s`, `5h_left=50%`), `D` (`five_hour_util=99%` — **h-exhausted**). All `result = Ok(...)`.
- **When:** `sort_indices(&accounts, SortStrategy::Renew, None, PreferStrategy::Any, now_secs)`
- **Then:** Order: `A`, `B`, `C`, then `D` (sunk). Non-h-exhausted sorted by soonest `7d Reset` countdown first.
- **Exit:** n/a (unit test)
- **Source fn:** `test_sort_renew_soonest_first_exhausted_last` (in `src/usage.rs`)
- **Source:** [feature/020_usage_sort_strategies.md AC-04](../../../../docs/feature/020_usage_sort_strategies.md)

---

### FT-05: `desc::1` reversal preserves h-exhausted floor

- **Given:** Three `AccountQuota` structs: `A` (5h_left=70%, not h-exhausted), `B` (5h_left=25%, not h-exhausted), `C` (utilization=99%, **h-exhausted**). Strategy: `sort::drain`.
- **When-A:** `sort_indices(..., SortStrategy::Drain, None, ...)` → natural: `B`, `A`, `C`. (No weekly data → 7d Left=100% tied; tiebreak `5h Left` asc: B=25% < A=70%.)
- **When-B:** `sort_indices(..., SortStrategy::Drain, Some(true), ...)` → reversed non-h-exhausted: `A`, `B`, then `C` still last.
- **Then-A:** Order: `B` (25% 5h Left, 7d Left tied at 100% — tiebreak wins), `A` (70%), `C` (1%, sunk).
- **Then-B:** Order: `A` (70%), `B` (25%), `C` (1%, still sunk — h-exhausted floor is not reversed).
- **Exit:** n/a (unit test)
- **Source fn:** `test_sort_drain_desc_reverses_non_exhausted_only` (in `src/usage.rs`)
- **Source:** [feature/020_usage_sort_strategies.md AC-05](../../../../docs/feature/020_usage_sort_strategies.md)

---

### FT-06: Context-sensitive `desc::` defaults per strategy

- **Given:** Two `AccountQuota` structs: `A` (5h_left=80%), `B` (5h_left=20%).
- **When-A:** `sort_indices(..., SortStrategy::Endurance, None, ...)` — no explicit desc = endurance default (desc=true).
- **When-B:** `sort_indices(..., SortStrategy::Drain, None, ...)` — no explicit desc = drain default (desc=false).
- **Then-A (endurance no desc):** Same as `desc::1` — best on top.
- **Then-B (drain no desc):** Same as `desc::0` — drain targets on top (7d Left=100% tied; tiebreak `5h_left` asc: B=20% < A=80%).
- **Exit:** n/a (unit test)
- **Source fn:** `test_sort_endurance_default_equals_desc1`, `test_sort_drain_default_equals_desc0` (in `src/usage.rs`)
- **Source:** [feature/020_usage_sort_strategies.md AC-06](../../../../docs/feature/020_usage_sort_strategies.md)

---

### FT-07: `prefer::sonnet` uses `7d(Son)` for endurance qualification

- **Given:** One `AccountQuota` struct: `seven_day.utilization=90%` (10% left), `seven_day_sonnet.utilization=65%` (35% left). 5h_reset within 30 min.
- **When-A:** `prefer_weekly(aq, PreferStrategy::Any)` → `min(10%, 35%)` = 10% < 30% → **not qualified**.
- **When-B:** `prefer_weekly(aq, PreferStrategy::Sonnet)` → 35% ≥ 30% → **qualified**.
- **Then-A:** `prefer_weekly` returns ~10.0 (below qualification threshold).
- **Then-B:** `prefer_weekly` returns ~35.0 (above qualification threshold).
- **Exit:** n/a (unit test)
- **Source fn:** `test_prefer_sonnet_qualifies_by_sonnet_quota` (in `src/usage.rs`)
- **Source:** [feature/020_usage_sort_strategies.md AC-07](../../../../docs/feature/020_usage_sort_strategies.md)

---

### FT-08: `format::json` output alphabetical regardless of `sort::`

- **Given:** Two `AccountQuota` structs `zzz@test.com` (70% left) and `aaa@test.com` (20% left) in that order (reverse-alphabetical input).
- **When:** `render_json(&accounts)` — no sort applied.
- **Then:** JSON output preserves input order — `zzz@test.com` appears before `aaa@test.com`, confirming `render_json` does not re-sort.
- **Exit:** n/a (unit test against `render_json`)
- **Source fn:** `test_json_unaffected_by_sort` (in `src/usage.rs`)
- **Source:** [feature/020_usage_sort_strategies.md AC-13](../../../../docs/feature/020_usage_sort_strategies.md)

---

### FT-09: Invalid `sort::` value exits 1 naming valid values

- **Given:** Any environment.
- **When:** `clp .usage sort::bogus`
- **Then:** Exits 1. Stderr names the five valid values: `name`, `endurance`, `drain`, `renew`, `next`.
- **Exit:** 1
- **Source fn:** `it047_sort_invalid_value_exit_1` (in `tests/cli/usage_test.rs`); unit: `test_sort_strategy_parse_invalid_rejected` (in `src/usage.rs`)
- **Source:** [feature/020_usage_sort_strategies.md AC-09](../../../../docs/feature/020_usage_sort_strategies.md)

---

### FT-10: Invalid `prefer::` value exits 1 naming valid values

- **Given:** Any environment.
- **When:** `clp .usage prefer::bogus`
- **Then:** Exits 1. Stderr names the three valid values: `any`, `opus`, `sonnet`.
- **Exit:** 1
- **Source fn:** `it048_prefer_invalid_value_exit_1` (in `tests/cli/usage_test.rs`); unit: `test_prefer_strategy_parse_invalid_rejected` (in `src/usage.rs`)
- **Source:** [feature/020_usage_sort_strategies.md AC-10](../../../../docs/feature/020_usage_sort_strategies.md)

---

### FT-11: `sort::` does not affect `→ Next` recommendation in footer

- **Given:** Two `AccountQuota` structs: `a@x.com` (5h_left=80%), `b@x.com` (5h_left=25%). Neither is current.
- **When:** `render_text(&accounts, SortStrategy::Drain, None, PreferStrategy::Any)` — drain order puts `b@x.com` (25%) first.
- **Then:** Footer still shows `→ a@x.com` (AC-11: `find_recommendation` always runs on the original alphabetical slice).
- **Exit:** n/a (unit test against `render_text` + `find_recommendation`)
- **Source fn:** `test_sort_recommendation_unaffected_by_sort_strategy` (in `src/usage.rs`)
- **Source:** [feature/020_usage_sort_strategies.md AC-11](../../../../docs/feature/020_usage_sort_strategies.md)

---

### FT-12: `prefer::` governs drain primary sort key (`7d Left` ascending, prefer-aware)

- **Given:** Two `AccountQuota` structs with identical `five_hour.utilization` (50% left): `high_son@test.com` (`seven_day.utilization=80%` → 20% 7d Left, `seven_day_sonnet.utilization=20%` → 80% 7d(Son)) and `high_any@test.com` (`seven_day.utilization=40%` → 60% 7d Left, `seven_day_sonnet.utilization=70%` → 30% 7d(Son)).
- **When-A:** `sort_indices(..., SortStrategy::Drain, None, PreferStrategy::Sonnet, 0)` — prefer weekly selects `7d(Son)`.
- **When-B:** `sort_indices(..., SortStrategy::Drain, None, PreferStrategy::Opus, 0)` — prefer weekly selects `7d Left`.
- **Then-A:** `high_any@test.com` ranks first (30% `7d(Son)` < 80% → ascending → lower weekly first under `prefer::sonnet`).
- **Then-B:** `high_son@test.com` ranks first (20% `7d Left` < 60% → ascending → lower weekly first under `prefer::opus`).
- **Exit:** n/a (unit test — function return assertion)
- **Source fn:** `test_sort_drain_prefer_sonnet_primary`, `test_prefer_opus_primary_in_drain` (in `src/usage.rs`)
- **Source:** [feature/020_usage_sort_strategies.md AC-08](../../../../docs/feature/020_usage_sort_strategies.md)

---

### FT-13: Three-tier grouping: 🟢 above 🟡 above 🔴

- **Given:** Three `AccountQuota` structs: `green@test.com` (5h_left=80%, 7d_left=60% — 5h >15% and 7d >5%, tier 🟢), `yellow@test.com` (5h_left=3%, 7d_left=50% — 5h ≤15%, tier 🟡), `red@test.com` (result=Err — tier 🔴). Any sort strategy.
- **When:** `sort_indices(&accounts, SortStrategy::Name, None, PreferStrategy::Any, 0)` — name sort would place red before yellow alphabetically.
- **Then:** Output order: `green@test.com` (🟢), `yellow@test.com` (🟡), `red@test.com` (🔴). Three-tier grouping overrides alphabetical sort.
- **Exit:** n/a (unit test)
- **Source fn:** `test_three_tier_grouping_green_before_yellow_before_red`
- **Source:** [feature/020_usage_sort_strategies.md AC-14](../../../../docs/feature/020_usage_sort_strategies.md)

---

### FT-14: `sort::renew` is default when `sort::` omitted

- **Given:** Two `AccountQuota` structs: `early@test.com` (`seven_day.resets_at=now+3600s` — resets in 1h, `seven_day.utilization=20%` — 80% left), `late@test.com` (`seven_day.resets_at=now+86400s` — resets in 24h, `seven_day.utilization=80%` — 20% left). Both non-exhausted.
- **When:** `sort_indices(&accounts, SortStrategy::Renew, None, PreferStrategy::Any, now_secs)` — default strategy is `renew`.
- **Then:** `early@test.com` ranks first (resets in 1h — soonest reset first), `late@test.com` second. Confirms renew default = `desc::0`.
- **Exit:** n/a (unit test)
- **Source fn:** `test_sort_renew_default_equals_desc0` (in `src/usage.rs`); `it127_sort_default_is_renew_structural` (in `tests/cli/usage_test.rs`)
- **Source:** [feature/020_usage_sort_strategies.md AC-01](../../../../docs/feature/020_usage_sort_strategies.md)

---

### FT-15: Within 🟡 tier — h-exhausted before weekly-exhausted; `desc::` does not swap sub-groups

- **Given:** Unit test. Three `AccountQuota` structs, all 🟡:
  - `weekly@x.com`: `five_hour.utilization=10.0` (90% left), `seven_day.utilization=98.0` (2% left) → **weekly-exhausted** sub-group (alpha first)
  - `sess_a@x.com`: `five_hour.utilization=99.0` (1% left), `seven_day.utilization=30.0` (70% left) → **h-exhausted** sub-group
  - `sess_b@x.com`: `five_hour.utilization=97.0` (3% left), `seven_day.utilization=40.0` (60% left) → **h-exhausted** sub-group
- **When-A:** `render_text(...)` with `SortStrategy::Name` (default `desc::0`) — alpha order is `sess_a → sess_b → weekly`.
- **When-B:** `render_text(...)` with `SortStrategy::Name` and `desc::1` — reversed alpha within each sub-group.
- **Then-A (default):** Output order: `sess_a@x.com` (h-exhausted sub-group), `sess_b@x.com` (h-exhausted sub-group), `weekly@x.com` (weekly sub-group). `weekly@x.com` is last despite being alpha-first.
- **Then-B (desc::1):** Output order: `sess_b@x.com`, `sess_a@x.com` (h-exhausted sub-group reversed), `weekly@x.com` (weekly sub-group last — not moved to front by `desc::1`).
- **Exit:** n/a (unit test — position assertion via `output.find()`)
- **Source fn:** `test_ft16_009_yellow_tier_session_before_weekly` (When-A), `test_ft15_020_yellow_sub_grouping_not_reversed_by_desc` (When-B) (in `src/usage.rs`)
- **Source:** [feature/020_usage_sort_strategies.md AC-14](../../../../docs/feature/020_usage_sort_strategies.md)

---

### FT-16: `sort::endurance` unqualified bucket tiebreak by highest weekly

- **Given:** Three `AccountQuota` structs, all in the unqualified bucket (none qualify for endurance): equal `five_hour.utilization=50%` (50% left), `seven_day` utilization = [98%, 0%, 73%] (2%, 100%, 27% left). Alphabetical name order: A (2% weekly), B (100% weekly), C (73% weekly).
- **When:** `sort_indices(&accounts, SortStrategy::Endurance, None, PreferStrategy::Any, now_secs)` — endurance default `desc::1`, all unqualified.
- **Then:** Order: B (100% weekly), C (73% weekly), A (2% weekly). When session quotas are equal, highest `weekly(prefer)` wins the tiebreak — not alphabetical insertion order.
- **Exit:** n/a (unit test — index assertion)
- **Source fn:** `test_bug173_mre_endurance_unqualified_prefers_highest_weekly` (in `src/usage.rs`)
- **Source:** [feature/020_usage_sort_strategies.md AC-02](../../../../docs/feature/020_usage_sort_strategies.md)

---

### FT-17: `sort::next` delegates to active `next::` strategy

- **Given:** Empty credential store.
- **When-A:** `clp .usage sort::next` (with default `next::drain`)
- **When-B:** `clp .usage sort::next next::endurance`
- **Then-A:** Exits 0 with "(no accounts configured)". `sort::next` resolves to `sort::drain` and is accepted without error.
- **Then-B:** Exits 0 with "(no accounts configured)". `sort::next` resolves to `sort::endurance` and is accepted without error.
- **Exit:** 0 both cases
- **Source fn:** `it111_sort_next_accepted`, `it128_sort_next_resolves_to_drain_structural`, `it129_sort_next_resolves_to_endurance_structural` (in `tests/cli/usage_test.rs`)
- **Source:** [feature/020_usage_sort_strategies.md AC-15](../../../../docs/feature/020_usage_sort_strategies.md)
