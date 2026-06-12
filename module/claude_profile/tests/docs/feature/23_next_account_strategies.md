# Test: Feature 023 — Next Account Recommendation Strategies

Feature behavioral requirement test cases for `docs/feature/023_next_account_strategies.md`. Each FT case maps to one acceptance criterion. Parameter edge cases are in [cli/param/032_next.md](../cli/param/32_next.md). Command-level tests (IT-N) are in [cli/command/009_usage.md](../cli/command/09_usage.md).

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
| FT-12 | All strategies skip `is_occupied_elsewhere` accounts | AC-11 | Unit test |
| FT-13 | All strategies skip h-exhausted accounts (5h Left ≤ 15%) | AC-12 | Unit test |
| FT-14 | Endurance footer shows `session + 5h_reset` instead of `7d left + expires` | AC-13 | Unit test |
| FT-15 | `next::renew` prefers lower `5h_left` account on equal renewal time (BUG-243) | AC-10 | Unit test |
| FT-16 | renew deterministic: alphabetical winner when all numeric keys tied (BUG-260) | AC-10 | Unit test |
| FT-17 | endurance skips `prefer_weekly ≤ 5.0` accounts in unqualified tier (BUG-287) | AC-03 | Unit test |

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
| FT-12 | All strategies skip `is_occupied_elsewhere` accounts | AC-11 | Eligibility |
| FT-13 | All strategies skip h-exhausted accounts (5h Left ≤ 15%) | AC-12 | Eligibility |
| FT-14 | Endurance footer shows `session + 5h_reset` not `7d left + expires` | AC-13 | Footer |
| FT-15 | renew tiebreaker: prefers lower `5h_left` on equal renewal time | AC-10 | Tiebreaker |
| FT-16 | renew deterministic: alphabetical winner when all numeric keys tied (BUG-260) | AC-10 | Tiebreaker |
| FT-17 | endurance never recommends `prefer_weekly ≤ 5.0` accounts (BUG-287) | AC-03 | BUG-287 |

**Total:** 17 FT cases

---

### FT-01: Footer always shows all three strategy lines when ≥2 valid accounts

- **Given:** Two accounts with valid quota data; `next::drain` (not the default strategy).
- **When:** `clp .usage next::drain`
- **Then:** Footer contains "Next by strategy:" followed by three lines — one starting "renew", one starting "endurance", and one starting "drain". All three lines appear regardless of which `next::` value is active.
- **Exit:** 0
- **Live:** yes (requires ≥2 accounts with live quota)
- **Source fn:** `it104_lim_it_footer_always_shows_both_strategy_lines` (in `tests/cli/usage_test.rs`)
- **Source:** [feature/023_next_account_strategies.md AC-01](../../../docs/feature/023_next_account_strategies.md)

---

### FT-02: `→` placed on active strategy winner; absent when no eligible candidate

- **Given:** Three `AccountQuota` structs: `A` (is_current=true — ineligible), `B` (result=Ok, eligible), `C` (result=Ok, eligible). `next::endurance`.
- **When-A:** `find_next_for_strategy(&accounts, NextStrategy::Endurance, PreferStrategy::Any, now_secs)` with B and C as eligible candidates.
- **When-B:** All accounts are `is_current=true` — no eligible candidates.
- **Then-A:** Returns `Some(index_of_endurance_winner)`.
- **Then-B:** Returns `None` — no `→` placed.
- **Exit:** n/a (unit test)
- **Note:** TSK-184 deleted `find_recommendation()`; this case now calls `find_next_for_strategy()` directly.
- **Source fn:** `test_ft02_023_find_next_for_strategy_some_when_eligible_none_when_all_current` (in `src/usage/sort_next.rs`)
- **Source:** [feature/023_next_account_strategies.md AC-02](../../../docs/feature/023_next_account_strategies.md)

---

### FT-03: `next::endurance` places `→` on endurance top candidate

- **Given:** Two accounts with valid quota: `end_winner@test.com` (5h_reset in 30m, weekly=40% — endurance-qualified), `drain_winner@test.com` (5h_reset in 3h — not qualified, weekly=50%). `next::endurance`.
- **When:** `clp .usage next::endurance`
- **Then:** The row for `end_winner@test.com` contains `→` in the flag column. `drain_winner@test.com` does NOT have `→`.
- **Exit:** 0
- **Live:** yes (requires live quota data)
- **Source fn:** `it102_lim_it_next_endurance_places_arrow_on_winner` (in `tests/cli/usage_test.rs`)
- **Source:** [feature/023_next_account_strategies.md AC-03](../../../docs/feature/023_next_account_strategies.md)

---

### FT-04: `next::drain` places `→` on drain top candidate

- **Given:** Two accounts with valid quota: `high_weekly@test.com` (7d_left=80%, non-exhausted), `low_weekly@test.com` (7d_left=20%, non-exhausted). `next::drain` selects the account with the lowest non-exhausted `prefer_weekly` (7d Left) first.
- **When:** `clp .usage next::drain`
- **Then:** The row for `low_weekly@test.com` contains `→` (lowest non-exhausted `prefer_weekly`). `high_weekly@test.com` does NOT have `→`.
- **Exit:** 0
- **Live:** yes (requires live quota data)
- **Source fn:** `it103_lim_it_next_drain_places_arrow_on_winner` (in `tests/cli/usage_test.rs`)
- **Source:** [feature/023_next_account_strategies.md AC-04](../../../docs/feature/023_next_account_strategies.md)

---

### FT-05: Invalid `next::` value exits 1 naming valid values

- **Given:** Any environment (empty credential store).
- **When:** `clp .usage next::bogus`
- **Then:** Exits 1. Stderr contains "renew", "endurance", and "drain" (the three valid values). Does NOT contain "all", "session", or "reset".
- **Exit:** 1
- **Source fn:** `it092_next_all_rejected_exit_1`, `it094_next_session_rejected_exit_1`
- **Source:** [feature/023_next_account_strategies.md AC-05](../../../docs/feature/023_next_account_strategies.md)

---

### FT-06: `next::` does not affect `format::json` output

- **Given:** Two accounts with valid quota data.
- **When-A:** `clp .usage format::json`
- **When-B:** `clp .usage next::drain format::json`
- **Then-A and Then-B:** Identical JSON arrays. No `"->"` marker. JSON account order is alphabetical. `next::` has no effect on JSON output.
- **Exit:** 0 both cases
- **Source fn:** `it091_next_json_output_unchanged_by_next_param`, `it096_next_drain_json_output_unchanged`
- **Source:** [feature/023_next_account_strategies.md AC-06](../../../docs/feature/023_next_account_strategies.md)

---

### FT-07: Footer suppressed when valid_count < 2

- **Given:** One saved account whose credential file has no `accessToken` (quota fetch fails; `valid_count = 0`).
- **When:** `clp .usage next::endurance`
- **Then:** Exits 0. Stdout does NOT contain "Next by strategy:". Footer suppressed when fewer than 2 accounts have valid quota data.
- **Exit:** 0
- **Source fn:** `it090_next_footer_absent_when_no_valid_accounts`
- **Source:** [feature/023_next_account_strategies.md AC-07](../../../docs/feature/023_next_account_strategies.md)

---

### FT-08: Footer omits strategy line when no eligible candidate exists

- **Given:** Two `AccountQuota` structs: both `is_current=true` (no eligible candidates for either strategy).
- **When:** Unit test of footer rendering with zero eligible candidates.
- **Then:** Neither "endurance" nor "drain" strategy lines appear in the footer output (both omitted — no eligible candidate for either).
- **Exit:** n/a (unit test)
- **Source fn:** `test_ft08_023_footer_omits_strategy_lines_when_no_eligible_candidate` (in `src/usage/mod.rs`)
- **Source:** [feature/023_next_account_strategies.md AC-08](../../../docs/feature/023_next_account_strategies.md)

---

### FT-10: drain footer label and reset source reflect binding weekly dimension (BUG-216)

- **Given-A:** `AccountQuota` with `seven_day_left = 61.0`, `seven_day_sonnet_left = 39.0` (Sonnet is binding — `prefer_weekly(Any) = min(61, 39) = 39`). `seven_day.resets_at` = some timestamp T1; `seven_day_sonnet.resets_at` = some timestamp T2 (T1 ≠ T2).
- **Given-B:** `AccountQuota` with `seven_day_left = 39.0`, `seven_day_sonnet_left = 61.0` (overall 7d is binding — `prefer_weekly(Any) = min(39, 61) = 39`). Same T1/T2 values.
- **When-A:** Unit test calls `strategy_metric(drain, any, aq_a, now_secs)`.
- **When-B:** Unit test calls `strategy_metric(drain, any, aq_b, now_secs)`.
- **Then-A:** Returns a string containing `"39% 7d(Son) left"` (not `"7d left"`). The reset countdown is derived from T2 (`seven_day_sonnet.resets_at`), not T1.
- **Then-B:** Returns a string containing `"39% 7d left"` (not `"7d(Son) left"`). The reset countdown is derived from T1 (`seven_day.resets_at`), not T2.
- **Exit:** n/a (unit test)
- **Source fn:** `mre_bug_216_drain_footer_label_sonnet_binding`, `mre_bug_216_drain_footer_label_7d_binding` (in `src/usage/sort_next.rs`)
- **Source:** [feature/023_next_account_strategies.md AC-09](../../../docs/feature/023_next_account_strategies.md)

---

### FT-09: drain never recommends `prefer_weekly ≤ 5.0` accounts (BUG-206)

- **Given:** Three accounts: `weekly_zero` (`prefer_weekly(Any) = min(4%, 0%) = 0%` — Sonnet fully exhausted), `weekly_one` (`prefer_weekly(Any) = 1%` — 🟡 weekly-exhausted tier, BUG-206 reopen case), and `weekly_ten` (`prefer_weekly(Any) = min(15%, 10%) = 10%`). Drain sort places `weekly_zero` first, `weekly_one` second (ascending `prefer_weekly`).
- **When-A:** `find_next_for_strategy(&accounts, Drain, Any, now)` with all three accounts eligible.
- **When-B:** Same call with only `prefer_weekly ≤ 5.0` accounts (weekly_zero + weekly_one).
- **Then-A:** Returns `Some(index_of_weekly_ten)` — both `weekly_zero` (0%) and `weekly_one` (1%) skipped despite ranking first and second in drain sort; threshold is `> 5.0`.
- **Then-B:** Returns `None` — all candidates are weekly-exhausted, nothing meaningful to drain.
- **Exit:** n/a (unit test)
- **Source fn:** `mre_bug_206_drain_skips_prefer_weekly_zero_accounts` (in `src/usage/sort_next.rs`)
- **Source:** [feature/023_next_account_strategies.md AC-04](../../../docs/feature/023_next_account_strategies.md)

---

### FT-11: `next::renew` places `→` on account with soonest quota refill

- **Given:** Two accounts with valid quota: `soon@test.com` (5h_reset in 20m), `later@test.com` (5h_reset in 3h). `next::renew` (default).
- **When:** `clp .usage next::renew`
- **Then:** The row for `soon@test.com` contains `→` in the flag column. `later@test.com` does NOT have `→`. Footer "renew" line shows `soon@test.com` with `5h resets in 20m`.
- **Exit:** 0
- **Live:** yes (requires live quota data with active 5h timers)
- **Source fn:** `it145_lim_it_next_renew_places_arrow_on_soonest_refill` (in `tests/cli/usage_test.rs`)
- **Source:** [feature/023_next_account_strategies.md AC-10](../../../docs/feature/023_next_account_strategies.md)

---

### FT-12: All strategies skip `is_occupied_elsewhere` accounts

- **Given:** Three `AccountQuota` structs: `A` (is_current=false, is_active=false, is_occupied_elsewhere=true, result=Ok — occupied by another machine), `B` (is_current=false, is_active=false, is_occupied_elsewhere=false, result=Ok — free), `C` (is_current=true — ineligible).
- **When-A:** `find_next_for_strategy(&accounts, NextStrategy::Renew, PreferStrategy::Any, now)`.
- **When-B:** `find_next_for_strategy(&accounts, NextStrategy::Endurance, PreferStrategy::Any, now)`.
- **When-C:** `find_next_for_strategy(&accounts, NextStrategy::Drain, PreferStrategy::Any, now)`.
- **Then-A/B/C:** All return `Some(index_of_B)` — account `A` is skipped because `is_occupied_elsewhere == true`; only `B` is eligible.
- **When-D:** Same three strategies with only `A` and `C` (A occupied, C current — no free candidate).
- **Then-D:** All return `None` — no eligible candidate exists.
- **Exit:** n/a (unit test)
- **Source fn:** `test_ft12_023_all_strategies_skip_occupied_elsewhere` (in `src/usage/sort_next.rs`)
- **Source:** [feature/023_next_account_strategies.md AC-11](../../../docs/feature/023_next_account_strategies.md)

---

### FT-13: All strategies skip h-exhausted accounts (5h Left ≤ 15%)

- **Given:** Three `AccountQuota` structs: `A` (is_current=false, is_active=false, is_occupied_elsewhere=false, `five_hour.utilization=92.0` → 5h_left=8% — h-exhausted, result=Ok), `B` (same flags, `five_hour.utilization=70.0` → 5h_left=30% — healthy, result=Ok), `C` (is_current=true — ineligible).
- **When-A:** `find_next_for_strategy(&accounts, NextStrategy::Renew, PreferStrategy::Any, now)`.
- **When-B:** `find_next_for_strategy(&accounts, NextStrategy::Endurance, PreferStrategy::Any, now)`.
- **When-C:** `find_next_for_strategy(&accounts, NextStrategy::Drain, PreferStrategy::Any, now)`.
- **Then-A/B/C:** All return `Some(index_of_B)` — account `A` is skipped because `five_hour.utilization ≥ 85.0` (h-exhausted); only `B` is eligible.
- **When-D:** Same three strategies with only `A` (h-exhausted) and `C` (current) — no healthy candidate.
- **Then-D:** All return `None` — no eligible candidate.
- **Exit:** n/a (unit test)
- **Source fn:** `test_ft13_023_all_strategies_skip_h_exhausted` (in `src/usage/sort_next.rs`)
- **Source:** [feature/023_next_account_strategies.md AC-12](../../../docs/feature/023_next_account_strategies.md)

---

### FT-14: Endurance footer shows `session + 5h_reset` not `7d left + expires`

- **Given:** An `AccountQuota` with `five_hour.utilization=20.0` (5h_left=80%), `five_hour.resets_at` = ISO timestamp T1 (2h 30m from now), `seven_day.utilization=10.0` (7d_left=90%), `expires_at_ms` = 5h from now.
- **When:** Unit test calls `strategy_metric(&aq, NextStrategy::Endurance, PreferStrategy::Any, now_secs)`.
- **Then:** Returns a string containing `"80% session"` and `"5h resets in 2h 30m"`. Does NOT contain `"7d left"`, `"expires"`, or `"90%"`.
- **Exit:** n/a (unit test)
- **Source fn:** `test_ft14_023_endurance_footer_shows_5h_reset` (in `src/usage/sort_next.rs`)
- **Source:** [feature/023_next_account_strategies.md AC-13](../../../docs/feature/023_next_account_strategies.md)

---

### FT-15: `next::renew` prefers lower `5h_left` account on equal renewal time (BUG-243)

- **Given:** Two `AccountQuota` structs: `A` (is_current=false, is_active=false, is_occupied_elsewhere=false, result=Ok, `five_hour.utilization=77.0` → 5h_left=23%, `seven_day.resets_at` = T (same for both), `renewal_at` = R (same for both)); `B` (same flags, `five_hour.utilization=0.0` → 5h_left=100%, `seven_day.resets_at` = T, `renewal_at` = R). Both accounts have identical `renewal_event_secs`.
- **When:** Unit test calls `find_next_for_strategy(&[A, B], NextStrategy::Renew, PreferStrategy::Any, now_secs)`.
- **Then:** Returns `Some(index_of_A)` — `A` wins because `5h_left=23% < 100%` (more depleted, benefits more from the same renewal event). `B` is not selected despite equal renewal time.
- **Exit:** n/a (unit test)
- **Source fn:** `test_ft15_023_renew_tiebreaker_prefers_lower_5h_left` (in `src/usage/sort_next.rs`)
- **Source:** [feature/023_next_account_strategies.md AC-10](../../../docs/feature/023_next_account_strategies.md)

---

### FT-16: `next::renew` deterministic alphabetical winner when all numeric keys tied (BUG-260)

- **Given:** Two `AccountQuota` structs in reverse-alphabetical slice order: `zorro@test` at index 0 (is_current=false, is_active=false, is_occupied_elsewhere=false, result=Ok, `five_hour.utilization=0.0` → 5h_left=100%), `alice@test` at index 1 (same flags, same `five_hour.utilization=0.0` → 5h_left=100%). Both have identical `renewal_event_secs` (same `seven_day.resets_at` and same `renewal_at`). Both have identical `five_hour_left` (100%).
- **When:** Unit test calls `find_next_for_strategy(&[zorro, alice], NextStrategy::Renew, PreferStrategy::Any, now_secs)`.
- **Then:** Returns `Some(1)` (index of `alice@test`) — alphabetically first name wins when all numeric keys are fully tied. Without a name tiebreaker, `min_by` would return index 0 (`zorro@test`) — input-slice order.
- **Exit:** n/a (unit test)
- **Source fn:** `mre_bug260_renew_nondeterministic_when_fully_tied` (in `src/usage/sort_next.rs`)
- **Source:** [feature/023_next_account_strategies.md AC-10](../../../docs/feature/023_next_account_strategies.md)

---

### FT-17: endurance never recommends `prefer_weekly ≤ 5.0` accounts (BUG-287)

- **Given-A:** Three `AccountQuota` structs: `weekly_zero` (is_current=false, is_active=false, is_occupied_elsewhere=false, result=Ok, `prefer_weekly(Any)=0.0` — fully weekly-exhausted 🟡), `weekly_three` (same flags, `prefer_weekly(Any)=3.0` — 🟡 range), `weekly_green` (same flags, `prefer_weekly(Any)=60.0` — healthy 🟢). Endurance sort places `weekly_green` first in unqualified tier (no qualified accounts available).
- **Given-B:** Only `weekly_zero` and `weekly_three` present (all candidates have `prefer_weekly ≤ 5.0`).
- **Given-C:** `weekly_boundary` with `prefer_weekly(Any)=5.0` exactly — boundary is exclusive (`> 5.0`, not `≥ 5.0`).
- **When-A:** `find_next_for_strategy(&accounts, Endurance, Any, now)` with `weekly_zero`, `weekly_three`, `weekly_green`.
- **When-B:** Same call with only `weekly_zero` and `weekly_three`.
- **When-C:** Same call with only `weekly_boundary`.
- **Then-A:** Returns `Some(index_of_weekly_green)` — `weekly_zero` (0%) and `weekly_three` (3%) skipped despite being present; threshold is `> 5.0`.
- **Then-B:** Returns `None` — all candidates are weekly-exhausted (≤ 5.0); no eligible account in unqualified tier.
- **Then-C:** Returns `None` — `prefer_weekly=5.0` is at the boundary; `> 5.0` is exclusive, so `5.0` is skipped.
- **Exit:** n/a (unit test)
- **Source fn:** `mre_bug287_endurance_skips_weekly_exhausted_unqualified` (in `src/usage/sort_next.rs`)
- **Source:** [feature/023_next_account_strategies.md AC-03](../../../docs/feature/023_next_account_strategies.md)
