# Test: Feature 023 — Next Account Recommendation Strategies

Feature behavioral requirement test cases for `docs/feature/023_next_account_strategies.md`. Each FT case maps to one acceptance criterion. Parameter edge cases are in [cli/param/032_next.md](../cli/param/032_next.md). Command-level tests (IT-N) are in [cli/command/009_usage.md](../cli/command/009_usage.md).

### AC Coverage Index

| FT | Criterion | AC | Notes |
|----|-----------|-----|-------|
| FT-01 | Footer always shows all three strategy lines when ≥2 valid accounts | AC-01 | Integration |
| FT-02 | `→` placed on active strategy winner; omitted when no eligible candidate | AC-02 | Unit test |
| FT-03 | `next::endurance` places `→` on endurance winner | AC-03 | Integration |
| FT-04 | `next::drain` places `→` on drain winner | AC-04 | Integration |
| FT-05 | Invalid `next::` value exits 1 naming valid values | AC-05 | Integration |
| FT-06 | `next::` does not affect `format::json` output | AC-06 | Integration |
| FT-07 | Footer omitted when 0 or 1 accounts have valid quota data | AC-07 | Integration |
| FT-08 | Footer omits strategy line when no eligible candidate exists | AC-08 | Unit test |
| FT-09 | drain skips `prefer_weekly ≤ 5.0` accounts (BUG-206) | AC-04 | Unit test |
| FT-10 | drain footer label and reset source reflect binding weekly dimension (BUG-216) | AC-09 | Unit test |
| FT-11 | `next::renew` places `→` on account with soonest quota refill | AC-10 | Integration |

### Test Case Index

| ID | Test Name | AC | Category |
|----|-----------|-----|----------|
| FT-01 | Footer always visible with three strategy lines | AC-01 | Footer |
| FT-02 | `→` on active strategy winner; absent when no eligible | AC-02 | Marker |
| FT-03 | next::endurance selects endurance top candidate | AC-03 | Strategy |
| FT-04 | next::drain selects drain top candidate | AC-04 | Strategy |
| FT-05 | Invalid next:: value rejected | AC-05 | Validation |
| FT-06 | JSON unaffected by next:: | AC-06 | JSON No-op |
| FT-07 | Footer suppressed when valid_count < 2 | AC-07 | Footer Threshold |
| FT-08 | No-eligible-candidate strategy line omitted | AC-08 | Footer |
| FT-09 | drain never recommends `prefer_weekly ≤ 5.0` accounts | AC-04 | BUG-206 |
| FT-10 | drain footer label and reset source reflect binding weekly dimension | AC-09 | BUG-216 |
| FT-11 | next::renew places `→` on soonest-refill account | AC-10 | Strategy |

**Total:** 11 FT cases

---

### FT-01: Footer always shows all three strategy lines when ≥2 valid accounts

- **Given:** Two accounts with valid quota data; `next::drain` (not the default strategy).
- **When:** `clp .usage next::drain`
- **Then:** Footer contains "Next by strategy:" followed by three lines — one starting "renew", one starting "endurance", and one starting "drain". All three lines appear regardless of which `next::` value is active.
- **Exit:** 0
- **Live:** yes (requires ≥2 accounts with live quota)
- **Source fn:** `it104_lim_it_footer_always_shows_both_strategy_lines` (in `tests/cli/usage_test.rs`)
- **Source:** [feature/023_next_account_strategies.md AC-01](../../../../docs/feature/023_next_account_strategies.md)

---

### FT-02: `→` placed on active strategy winner; absent when no eligible candidate

- **Given:** Three `AccountQuota` structs: `A` (is_current=true — ineligible), `B` (result=Ok, eligible), `C` (result=Ok, eligible). `next::endurance`.
- **When-A:** `find_next_for_strategy(&accounts, NextStrategy::Endurance, PreferStrategy::Any, now_secs)` with B and C as eligible candidates.
- **When-B:** All accounts are `is_current=true` — no eligible candidates.
- **Then-A:** Returns `Some(index_of_endurance_winner)`.
- **Then-B:** Returns `None` — no `→` placed.
- **Exit:** n/a (unit test)
- **Note:** TSK-184 deleted `find_recommendation()`; this case now calls `find_next_for_strategy()` directly.
- **Source fn:** `test_ft02_023_find_next_for_strategy_some_when_eligible_none_when_all_current` (in `src/usage/sort.rs`)
- **Source:** [feature/023_next_account_strategies.md AC-02](../../../../docs/feature/023_next_account_strategies.md)

---

### FT-03: `next::endurance` places `→` on endurance top candidate

- **Given:** Two accounts with valid quota: `end_winner@test.com` (5h_reset in 30m, weekly=40% — endurance-qualified), `drain_winner@test.com` (5h_reset in 3h — not qualified, weekly=50%). `next::endurance`.
- **When:** `clp .usage next::endurance`
- **Then:** The row for `end_winner@test.com` contains `→` in the flag column. `drain_winner@test.com` does NOT have `→`.
- **Exit:** 0
- **Live:** yes (requires live quota data)
- **Source fn:** `it102_lim_it_next_endurance_places_arrow_on_winner` (in `tests/cli/usage_test.rs`)
- **Source:** [feature/023_next_account_strategies.md AC-03](../../../../docs/feature/023_next_account_strategies.md)

---

### FT-04: `next::drain` places `→` on drain top candidate

- **Given:** Two accounts with valid quota: `high_weekly@test.com` (7d_left=80%, non-exhausted), `low_weekly@test.com` (7d_left=20%, non-exhausted). `next::drain` selects the account with the lowest non-exhausted `prefer_weekly` (7d Left) first.
- **When:** `clp .usage next::drain`
- **Then:** The row for `low_weekly@test.com` contains `→` (lowest non-exhausted `prefer_weekly`). `high_weekly@test.com` does NOT have `→`.
- **Exit:** 0
- **Live:** yes (requires live quota data)
- **Source fn:** `it103_lim_it_next_drain_places_arrow_on_winner` (in `tests/cli/usage_test.rs`)
- **Source:** [feature/023_next_account_strategies.md AC-04](../../../../docs/feature/023_next_account_strategies.md)

---

### FT-05: Invalid `next::` value exits 1 naming valid values

- **Given:** Any environment (empty credential store).
- **When:** `clp .usage next::bogus`
- **Then:** Exits 1. Stderr contains "renew", "endurance", and "drain" (the three valid values). Does NOT contain "all", "session", or "reset".
- **Exit:** 1
- **Source fn:** `it092_next_all_rejected_exit_1`, `it094_next_session_rejected_exit_1`
- **Source:** [feature/023_next_account_strategies.md AC-05](../../../../docs/feature/023_next_account_strategies.md)

---

### FT-06: `next::` does not affect `format::json` output

- **Given:** Two accounts with valid quota data.
- **When-A:** `clp .usage format::json`
- **When-B:** `clp .usage next::drain format::json`
- **Then-A and Then-B:** Identical JSON arrays. No `"->"` marker. JSON account order is alphabetical. `next::` has no effect on JSON output.
- **Exit:** 0 both cases
- **Source fn:** `it091_next_json_output_unchanged_by_next_param`, `it096_next_drain_json_output_unchanged`
- **Source:** [feature/023_next_account_strategies.md AC-06](../../../../docs/feature/023_next_account_strategies.md)

---

### FT-07: Footer suppressed when valid_count < 2

- **Given:** One saved account whose credential file has no `accessToken` (quota fetch fails; `valid_count = 0`).
- **When:** `clp .usage next::endurance`
- **Then:** Exits 0. Stdout does NOT contain "Next by strategy:". Footer suppressed when fewer than 2 accounts have valid quota data.
- **Exit:** 0
- **Source fn:** `it090_next_footer_absent_when_no_valid_accounts`
- **Source:** [feature/023_next_account_strategies.md AC-07](../../../../docs/feature/023_next_account_strategies.md)

---

### FT-08: Footer omits strategy line when no eligible candidate exists

- **Given:** Two `AccountQuota` structs: both `is_current=true` (no eligible candidates for either strategy).
- **When:** Unit test of footer rendering with zero eligible candidates.
- **Then:** Neither "endurance" nor "drain" strategy lines appear in the footer output (both omitted — no eligible candidate for either).
- **Exit:** n/a (unit test)
- **Source fn:** `test_ft08_023_footer_omits_strategy_lines_when_no_eligible_candidate` (in `src/usage/mod.rs`)
- **Source:** [feature/023_next_account_strategies.md AC-08](../../../../docs/feature/023_next_account_strategies.md)

---

### FT-10: drain footer label and reset source reflect binding weekly dimension (BUG-216)

- **Given-A:** `AccountQuota` with `seven_day_left = 61.0`, `seven_day_sonnet_left = 39.0` (Sonnet is binding — `prefer_weekly(Any) = min(61, 39) = 39`). `seven_day.resets_at` = some timestamp T1; `seven_day_sonnet.resets_at` = some timestamp T2 (T1 ≠ T2).
- **Given-B:** `AccountQuota` with `seven_day_left = 39.0`, `seven_day_sonnet_left = 61.0` (overall 7d is binding — `prefer_weekly(Any) = min(39, 61) = 39`). Same T1/T2 values.
- **When-A:** Unit test calls `strategy_metric(drain, any, aq_a, now_secs)`.
- **When-B:** Unit test calls `strategy_metric(drain, any, aq_b, now_secs)`.
- **Then-A:** Returns a string containing `"39% 7d(Son) left"` (not `"7d left"`). The reset countdown is derived from T2 (`seven_day_sonnet.resets_at`), not T1.
- **Then-B:** Returns a string containing `"39% 7d left"` (not `"7d(Son) left"`). The reset countdown is derived from T1 (`seven_day.resets_at`), not T2.
- **Exit:** n/a (unit test)
- **Source fn:** `mre_bug_216_drain_footer_label_sonnet_binding`, `mre_bug_216_drain_footer_label_7d_binding` (in `src/usage/sort.rs`)
- **Source:** [feature/023_next_account_strategies.md AC-09](../../../../docs/feature/023_next_account_strategies.md)

---

### FT-09: drain never recommends `prefer_weekly ≤ 5.0` accounts (BUG-206)

- **Given:** Three accounts: `weekly_zero` (`prefer_weekly(Any) = min(4%, 0%) = 0%` — Sonnet fully exhausted), `weekly_one` (`prefer_weekly(Any) = 1%` — 🟡 weekly-exhausted tier, BUG-206 reopen case), and `weekly_ten` (`prefer_weekly(Any) = min(15%, 10%) = 10%`). Drain sort places `weekly_zero` first, `weekly_one` second (ascending `prefer_weekly`).
- **When-A:** `find_next_for_strategy(&accounts, Drain, Any, now)` with all three accounts eligible.
- **When-B:** Same call with only `prefer_weekly ≤ 5.0` accounts (weekly_zero + weekly_one).
- **Then-A:** Returns `Some(index_of_weekly_ten)` — both `weekly_zero` (0%) and `weekly_one` (1%) skipped despite ranking first and second in drain sort; threshold is `> 5.0`.
- **Then-B:** Returns `None` — all candidates are weekly-exhausted, nothing meaningful to drain.
- **Exit:** n/a (unit test)
- **Source fn:** `mre_bug_206_drain_skips_prefer_weekly_zero_accounts` (in `src/usage/sort.rs`)
- **Source:** [feature/023_next_account_strategies.md AC-04](../../../../docs/feature/023_next_account_strategies.md)

---

### FT-11: `next::renew` places `→` on account with soonest quota refill

- **Given:** Two accounts with valid quota: `soon@test.com` (5h_reset in 20m), `later@test.com` (5h_reset in 3h). `next::renew` (default).
- **When:** `clp .usage next::renew`
- **Then:** The row for `soon@test.com` contains `→` in the flag column. `later@test.com` does NOT have `→`. Footer "renew" line shows `soon@test.com` with `5h resets in 20m`.
- **Exit:** 0
- **Live:** yes (requires live quota data with active 5h timers)
- **Source fn:** ⏳ `it145_lim_it_next_renew_places_arrow_on_soonest_refill` (in `tests/cli/usage_test.rs`)
- **Source:** [feature/023_next_account_strategies.md AC-10](../../../../docs/feature/023_next_account_strategies.md)
