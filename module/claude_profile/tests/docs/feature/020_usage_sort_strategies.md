# Test: Feature 020 — Usage Sort Strategies

Feature behavioral requirement test cases for `docs/feature/020_usage_sort_strategies.md`. Each FT case maps to one acceptance criterion. Command-level tests (IT-N) are in [cli/command/009_usage.md](../cli/command/009_usage.md).

### AC Coverage Index

| FT | Criterion | AC | Notes |
|----|-----------|-----|-------|
| FT-01 | `sort::name` preserves alphabetical order | AC-01 | Unit test |
| FT-02 | `sort::endurance` qualified accounts ranked first | AC-02 | Unit test |
| FT-03 | `sort::drain` sinks exhausted (≤ 5%) accounts to bottom | AC-03 | Unit test |
| FT-04 | `sort::reset` sinks exhausted (≤ 5%) accounts to bottom | AC-04 | Unit test |
| FT-05 | `desc::1` reverses non-exhausted tier; exhausted floor unchanged | AC-05 | Unit test |
| FT-06 | Context-sensitive `desc::` defaults per strategy | AC-06 | Unit test |
| FT-07 | `prefer::sonnet` uses `7d(Son)` for endurance qualification | AC-07 | Unit test |
| FT-08 | `format::json` order unaffected by `sort::` | AC-13 | Integration |
| FT-09 | Invalid `sort::` value exits 1 naming valid values | AC-09 | Integration |
| FT-10 | Invalid `prefer::` value exits 1 naming valid values | AC-10 | Integration |
| FT-11 | `sort::` does not affect `→ Next` recommendation | AC-11 | Unit test |

### Test Case Index

| ID | Test Name | AC | Category |
|----|-----------|-----|----------|
| FT-01 | sort::name alphabetical | AC-01 | Sorting |
| FT-02 | sort::endurance qualified above unqualified | AC-02 | Sorting |
| FT-03 | sort::drain exhausted sunk | AC-03 | Sorting |
| FT-04 | sort::reset exhausted sunk | AC-04 | Sorting |
| FT-05 | desc::1 reversal preserves exhausted floor | AC-05 | Direction |
| FT-06 | Context-sensitive desc defaults | AC-06 | Direction |
| FT-07 | prefer::sonnet wires 7d(Son) into endurance | AC-07 | Prefer |
| FT-08 | JSON output alphabetical regardless of sort | AC-13 | JSON No-op |
| FT-09 | Invalid sort value rejected | AC-09 | Validation |
| FT-10 | Invalid prefer value rejected | AC-10 | Validation |
| FT-11 | Recommendation unaffected by sort | AC-11 | Independence |

**Total:** 11 FT cases

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

### FT-03: `sort::drain` sinks exhausted accounts to bottom

- **Given:** Three `AccountQuota` structs: `A` (five_hour.utilization=99%, 1% left — exhausted), `B` (five_hour.utilization=75%, 25% left), `C` (five_hour.utilization=30%, 70% left). All `result = Ok(...)`.
- **When:** `sort_indices(&accounts, SortStrategy::Drain, None, PreferStrategy::Any, 0)` (desc=false is the default for drain)
- **Then:** Order: `B` (25%), `C` (70%), then `A` (1%, sunk). Exhausted floor is at bottom; non-exhausted sorted by 5h Left ascending.
- **Exit:** n/a (unit test)
- **Source fn:** `test_sort_drain_exhausted_sunk_rest_ascending` (in `src/usage.rs`)
- **Source:** [feature/020_usage_sort_strategies.md AC-03](../../../../docs/feature/020_usage_sort_strategies.md)

---

### FT-04: `sort::reset` sinks exhausted accounts to bottom

- **Given:** Four `AccountQuota` structs: `A` (5h_reset=now+600s, 5h_left=50%), `B` (5h_reset=now+2700s, 5h_left=50%), `C` (5h_reset=now+7200s, 5h_left=50%), `D` (utilization=99% — exhausted). All `result = Ok(...)`.
- **When:** `sort_indices(&accounts, SortStrategy::Reset, None, PreferStrategy::Any, now_secs)`
- **Then:** Order: `A`, `B`, `C`, then `D` (sunk). Non-exhausted sorted by soonest reset first.
- **Exit:** n/a (unit test)
- **Source fn:** `test_sort_reset_soonest_first_exhausted_last` (in `src/usage.rs`)
- **Source:** [feature/020_usage_sort_strategies.md AC-04](../../../../docs/feature/020_usage_sort_strategies.md)

---

### FT-05: `desc::1` reversal preserves exhausted floor

- **Given:** Three `AccountQuota` structs: `A` (5h_left=70%, non-exhausted), `B` (5h_left=25%, non-exhausted), `C` (utilization=99%, exhausted). Strategy: `sort::drain`.
- **When-A:** `sort_indices(..., SortStrategy::Drain, None, ...)` → natural: `B`, `A`, `C`.
- **When-B:** `sort_indices(..., SortStrategy::Drain, Some(true), ...)` → reversed non-exhausted: `A`, `B`, then `C` still last.
- **Then-A:** Order: `B` (25%), `A` (70%), `C` (1%, sunk).
- **Then-B:** Order: `A` (70%), `B` (25%), `C` (1%, still sunk — exhausted floor is not reversed).
- **Exit:** n/a (unit test)
- **Source fn:** `test_sort_drain_desc_reverses_non_exhausted_only` (in `src/usage.rs`)
- **Source:** [feature/020_usage_sort_strategies.md AC-05](../../../../docs/feature/020_usage_sort_strategies.md)

---

### FT-06: Context-sensitive `desc::` defaults per strategy

- **Given:** Two `AccountQuota` structs: `A` (5h_left=80%), `B` (5h_left=20%).
- **When-A:** `sort_indices(..., SortStrategy::Endurance, None, ...)` — no explicit desc = endurance default (desc=true).
- **When-B:** `sort_indices(..., SortStrategy::Drain, None, ...)` — no explicit desc = drain default (desc=false).
- **Then-A (endurance no desc):** Same as `desc::1` — best on top.
- **Then-B (drain no desc):** Same as `desc::0` — drain targets (lowest 5h_left) on top.
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
- **Then:** Exits 1. Stderr names the four valid values: `name`, `endurance`, `drain`, `reset`.
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
