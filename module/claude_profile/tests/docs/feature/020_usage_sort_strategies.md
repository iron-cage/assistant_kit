# Test: Feature 020 — Usage Sort Strategies

### Scope

- **Purpose**: Test cases for usage table sort strategies.
- **Source**: `docs/feature/020_usage_sort_strategies.md`
- **Covers**: AC-01 through AC-12

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
| FT-11 | h-exhausted + `7d(Son)=5%` → HExhausted under `prefer::any` (BUG-299) | AC-12 | Group Boundary |
| FT-12 | `prefer::son` + absent Sonnet tier → `prefer_weekly = 0.0` (not 100.0) | AC-05 | Absent-Sonnet fix |
| FT-13 | `sort::` drives footer recommendation — top eligible shown in `Next (<strategy>)` line; footer uses `·`-delimited 2-line format | AC-09 | Recommendation + Footer |
| FT-14 | Green account with divergent `7d/7d_son` passes eligibility gate — model-agnostic `seven_day_left` (BUG-324) | AC-09 | Eligibility + BUG-324 |
| — | `sort::` + `live::1` stable within each cycle | AC-10 | Live-only (requires `live::1` + real credentials); no equivalent guarantee for ordinary invocations (BUG-330) |

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
| FT-11 | h-exhausted account with 7d_son=5% lands in HExhausted (not Red) under prefer::any (BUG-299) | AC-12 | Group Boundary |
| FT-12 | prefer::son + absent Sonnet tier → prefer_weekly = 0.0 (not 100.0) | AC-05 | Absent-Sonnet fix |
| FT-13 | sort:: drives footer recommendation — top eligible in Next line; `·`-delimited format | AC-09 | Recommendation + Footer |
| FT-14 | Green account with divergent 7d/7d_son passes eligibility gate (BUG-324) | AC-09 | Eligibility + BUG-324 |

**Total:** 14 FT cases

---

### FT-01: `sort::name` preserves alphabetical order

- **Given:** Three `AccountQuota` structs with names `c@x.com`, `a@x.com`, `b@x.com` in that order.
- **When:** `sort_indices(&accounts, SortStrategy::Name, None, PreferStrategy::Any, 0)`
- **Then:** Indices reordered to: `a@x.com`, `b@x.com`, `c@x.com`.
- **Exit:** n/a (unit test — function return assertion)
- **Source fn:** `test_sort_name_alphabetical` (in `sort_tests.rs`)
- **Source:** [feature/020_usage_sort_strategies.md AC-01](../../../docs/feature/020_usage_sort_strategies.md)

---

### FT-02: `sort::renew` sinks h-exhausted accounts to bottom; non-exhausted sorted by `7d Reset` ascending

- **Given:** Three `AccountQuota` structs: `soon` (`seven_day.resets_at=now+600s`, `5h_left=70%`), `late` (`seven_day.resets_at=now+7200s`, `5h_left=70%`), `exhausted` (`seven_day.resets_at=now+600s`, `5h_left=1%` — **h-exhausted**). All `result = Ok(...)`.
- **When:** `sort_indices(&accounts, SortStrategy::Renew, None, PreferStrategy::Any, now)`
- **Then:** Order: `soon`, `late`, then `exhausted` (sunk). Non-h-exhausted sorted by soonest `7d Reset` countdown first.
- **Exit:** n/a (unit test)
- **Source fn:** `test_sort_renew_soonest_first_exhausted_last` (in `sort_tests.rs`)
- **Note:** Corrected from a previously-cited four-account scenario (`A`/`B`/`C`/`D` with reset times 600s/2700s/7200s) — the cited test constructs exactly three accounts (`soon`/`late`/`exhausted`), with no account at `now+2700s`.
- **Source:** [feature/020_usage_sort_strategies.md AC-01](../../../docs/feature/020_usage_sort_strategies.md)

---

### FT-03: `format::json` output alphabetical regardless of `sort::`

- **Given:** Two `AccountQuota` structs `zzz@test.com` (70% left) and `aaa@test.com` (20% left) in that order (reverse-alphabetical input).
- **When:** `render_json(&accounts)` — no sort applied.
- **Then:** JSON output preserves input order — `zzz@test.com` appears before `aaa@test.com`, confirming `render_json` does not re-sort.
- **Exit:** n/a (unit test against `render_json`)
- **Source fn:** `test_json_unaffected_by_sort` (in `tests/usage/mod_tests.rs`)
- **Source:** [feature/020_usage_sort_strategies.md AC-11](../../../docs/feature/020_usage_sort_strategies.md)

---

### FT-04: Invalid `sort::` value exits 1 naming valid values

- **Given:** Any environment.
- **When:** `clp .usage sort::bogus`
- **Then:** Exits 1. Stderr names the three valid values: `name`, `renew`, `renews`.
- **Exit:** 1
- **Source fn:** `it057_sort_invalid_value_exit_1` (in `usage_sort_test.rs`); unit: `test_sort_strategy_parse_invalid_rejected` (in `tests/usage/mod_tests.rs`)
- **Source:** [feature/020_usage_sort_strategies.md AC-07](../../../docs/feature/020_usage_sort_strategies.md)

---

### FT-05: Invalid `prefer::` value exits 1 naming valid values

- **Given:** Any environment.
- **When:** `clp .usage prefer::bogus`
- **Then:** Exits 1. Stderr names the three valid values: `any`, `opus`, `sonnet`.
- **Exit:** 1
- **Source fn:** `it058_prefer_invalid_value_exit_1` (in `usage_sort_test.rs`); unit: `test_prefer_strategy_parse_invalid_rejected` (in `tests/usage/mod_tests.rs`)
- **Source:** [feature/020_usage_sort_strategies.md AC-08](../../../docs/feature/020_usage_sort_strategies.md)

---

### FT-06: Four-group status partition: 🟢 above 🟡 h-exhausted above 🟡 weekly-exhausted (incl. both-exhausted) above 🔴 Dead

- **Given:** Five `AccountQuota` structs (via `mk_aq_sort_weekly(name, five_hour_util, seven_day_util, seven_day_sonnet_util)` unless noted): `green@test.com` (`10.0, 10.0, 0.0` → 5h_left=90%, 7d_left=90% — both available, 🟢 G1), `h_exh@test.com` (`90.0, 10.0, 0.0` → 5h_left=10%, 7d_left=90% — 5h exhausted, 🟡 G2), `weekly_exh@test.com` (`10.0, 98.0, 0.0` → 5h_left=90%, 7d_left=2% — 7d exhausted, 🟡 G3), `both_exh@test.com` (`94.0, 98.0, 0.0` → 5h_left=6%, 7d_left=2% — both exhausted, 🟡 G3 weekly-exhausted — 7d is binding), `dead@test.com` (via `mk_aq_cancelled("dead@test.com", 50.0, 20.0)` → `result=Ok(...)` with `account.billing_type="none"` — the cancelled-subscription signal, not a fetch error — 🔴 G4). Any sort strategy.
- **When:** `sort_indices(&accounts, SortStrategy::Name, None, PreferStrategy::Any, 0)` — name sort would interleave groups alphabetically.
- **Then:** Output order: `green@test.com` (🟢 G1), then G2 h-exhausted before G3 weekly-exhausted (both `weekly_exh` and `both_exh` — alphabetical within G3), then `dead@test.com` (🔴 G4). Four-group partition overrides alphabetical sort. Fix(BUG-321): `both_exh@test.com` sorts to G3 weekly-exhausted (🟡), not G4 Dead (🔴).
- **Exit:** n/a (unit test)
- **Note:** Corrected — the previous Given used illustrative percentages (80/60/3/50/80/2/6/1) that didn't match the cited test's actual `mk_aq_sort_weekly`/`mk_aq_cancelled` fixture values, and stated `dead@test.com` has `result=Err`; the test's `mk_aq_cancelled` helper actually sets `result=Ok(...)` and signals "dead" via `account.billing_type="none"` (confirmed via `src/usage/test_support.rs:566-586` and the G4 classification check in `src/usage/sort.rs:38-47`, which fires on "Error result OR cancelled subscription (`billing_type=\"none\"`)" — this test exercises the latter, not the former).
- **Source fn:** `mre_bug321_four_group_partition_order` (in `tests/usage/sort_tests.rs`)
- **Source:** [feature/020_usage_sort_strategies.md AC-12](../../../docs/feature/020_usage_sort_strategies.md)

---

### FT-07: `sort::renew` is default when `sort::` omitted

- **Given:** Two `AccountQuota` structs: `early@test.com` (`seven_day.resets_at=now+3600s` — resets in 1h, `five_hour_util=30%` — 70% left), `late@test.com` (`seven_day.resets_at=now+86400s` — resets in 24h, `five_hour_util=30%` — 70% left). Both non-exhausted (identical 5h utilization — only the reset time differs).
- **When:** `sort_indices(&accounts, SortStrategy::Renew, None, PreferStrategy::Any, now_secs)` — default strategy is `renew`.
- **Then:** `early@test.com` ranks first (resets in 1h — soonest reset first), `late@test.com` second. Confirms renew default = `desc::0`.
- **Exit:** n/a (unit test)
- **Source fn:** `test_sort_renew_default_equals_desc0` (in `sort_tests.rs`); `it137_sort_default_is_renew_structural` (in `usage_model_test.rs`)
- **Note:** Corrected field — the cited test's second `mk_aq_with_7d_reset(...)` parameter is `five_hour_util`, not `seven_day.utilization` (which this helper hardcodes to `0.0` internally); both accounts pass `30.0` (identical), not the previously-claimed 20%/80% split.
- **Source:** [feature/020_usage_sort_strategies.md AC-01](../../../docs/feature/020_usage_sort_strategies.md)

---

### FT-08: Within 🟡 tier — h-exhausted before weekly-exhausted; `desc::` does not swap sub-groups

- **Given:** Unit test. Each variant uses four `AccountQuota` structs (via `mk_named_aq(name, five_hour_util, seven_day_util)`) — one 🟢 green plus the three 🟡 accounts:
  - When-A (`test_ft16_009_...`): `d@x.com` (`10.0, 10.0` → 5h_left=90%, 7d_left=90% — 🟢 green), `a@x.com` (`10.0, 98.0` → 5h_left=90%, 7d_left=2% — **weekly-exhausted**), `b@x.com` (`99.0, 30.0` → 5h_left=1%, 7d_left=70% — **h-exhausted**), `c@x.com` (`97.0, 50.0` → 5h_left=3%, 7d_left=50% — **h-exhausted**).
  - When-B (`test_ft15_020_...`): `c@x.com` (`10.0, 10.0` → 🟢 green), `a@x.com` (`99.0, 30.0` → **h-exhausted**), `b@x.com` (`97.0, 50.0` → **h-exhausted**), `z@x.com` (`10.0, 98.0` → **weekly-exhausted**).
- **When-A:** `render_text(...)` with `SortStrategy::Name` (default `desc::0`).
- **When-B:** `render_text(...)` with `SortStrategy::Name` and `desc::1` — reversed alpha within each sub-group.
- **Then-A (default):** Output order: `d@x.com` (🟢, first), then h-exhausted sub-group alphabetically (`b@x.com`, `c@x.com`), then `a@x.com` (weekly-exhausted, last despite being alpha-first among the 🟡 accounts).
- **Then-B (desc::1):** Output order: `c@x.com` (🟢, first), then h-exhausted sub-group reversed (`b@x.com`, `a@x.com`), then `z@x.com` (weekly-exhausted, still last — not moved to front by `desc::1`).
- **Exit:** n/a (unit test — position assertion via `output.find()`)
- **Note:** Corrected — the previous Given omitted the 🟢 green account present in both cited tests (`d@x.com`/`c@x.com`) and understated the count as "three, all 🟡" (actual: four accounts each, one green). The green-before-yellow ordering is part of what each test asserts (`pos_d < pos_b`, `pos_c < pos_b`). Also corrected one h-exhausted account's `seven_day.utilization` from a previously-claimed `40.0` (60% left) to the test's actual `50.0` (50% left).
- **Source fn:** `test_ft16_009_yellow_tier_session_before_weekly` (When-A), `test_ft15_020_yellow_sub_grouping_not_reversed_by_desc` (When-B) (in `tests/usage/mod_tests.rs`)
- **Source:** [feature/020_usage_sort_strategies.md AC-12](../../../docs/feature/020_usage_sort_strategies.md)

---

### FT-09: `sort::renew` alphabetical when all numeric sort keys tied (BUG-259)

- **Given:** Three `AccountQuota` structs inserted in **reverse** alphabetical order: `charlie@test.com`, `bravo@test.com`, `alpha@test.com`. All have identical `five_hour_util=50%`, `seven_day=None` (no weekly quota data at all), and `expires_at_ms=FAR_FUTURE_MS` — all sort keys are identical.
- **When:** `sort_indices(&accounts, SortStrategy::Renew, None, PreferStrategy::Any, 0)`
- **Then:** `alpha@test.com` ranks first (alphabetical winner when all numeric keys tie). Confirms the final name tiebreaker prevents filesystem-order-dependent non-determinism.
- **Exit:** n/a (unit test — name assertion on `accounts[idx[0]].name`)
- **Source fn:** `mre_bug259_sort_renew_alphabetical_when_all_keys_tied` (in `sort_tests.rs`)
- **Note:** Corrected — the cited test's `mk_aq_sort(name, five_hour_util, expires_at_ms)` helper sets `seven_day: None` unconditionally (not `utilization=50%`/`resets_at=FAR_FUTURE_MS`); `FAR_FUTURE_MS` is passed as `expires_at_ms` (credential expiry), a field unrelated to the `seven_day` weekly-reset data previously claimed here.
- **Source:** [feature/020_usage_sort_strategies.md AC-01](../../../docs/feature/020_usage_sort_strategies.md)

---

### FT-10: `sort::renews` sorts by renewal timer ascending; no renewal data placed last

- **Given:** Three `AccountQuota` structs: `soon_renew@test.com` (`renewal_at=now+3600s` — renews in 1h), `later_renew@test.com` (`renewal_at=now+86400s`), `no_renew@test.com` (no `renewal_at` — scores `u64::MAX`).
- **When:** `sort_indices(&accounts, SortStrategy::Renews, None, PreferStrategy::Any, now)`
- **Then:** Order: `soon_renew@test.com` (soonest renewal), `later_renew@test.com`, `no_renew@test.com` (no data, placed last). Default `desc::0`.
- **Exit:** n/a (unit test — index assertion)
- **Source fn:** `test_sort_renews_ascending` (in `sort_next_tests_b.rs`)
- **Source:** [feature/020_usage_sort_strategies.md AC-02](../../../docs/feature/020_usage_sort_strategies.md)

---

### FT-11: h-exhausted account with `7d(Son)=5%` lands in HExhausted (not Red) under `prefer::any` (BUG-299)

- **Given:** Two `AccountQuota` structs (via `mk_aq_sort_weekly(name, five_hour_util, seven_day_util, seven_day_sonnet_util)`):
  - `account-a`: `five_hour_util=100%` (5h_left=0%, h-exhausted), `seven_day_util=68%` (7d_left=32%), `seven_day_sonnet_util=95%` (7d_son_left=5%).
  - `weekly-exh` (test name `red-account`): `five_hour_util=100%`, `seven_day_util=98%` (7d_left=2%, weekly-exhausted), `seven_day_sonnet_util=98%`.
- **When:** `sort_indices(&accounts, SortStrategy::Renew, None, PreferStrategy::Any, 0)`
- **Then:** `account-a` appears before `red-account`. `account-a` is in HExhausted (group 2); `red-account` is in WeeklyExhausted (group 3). Under `prefer::any`, `prefer_weekly(account-a) = min(32%, 5%) = 5.0` — the bug's `prefer_weekly(aq, prefer) > 5.0` check evaluated `5.0 > 5.0` = false and placed `account-a` in Red; the fix uses `seven_day_left(aq) > 5.0` → `32.0 > 5.0` = true → HExhausted.
- **Exit:** n/a (unit test — position assertion)
- **Note:** Corrected — the doc previously cited `SortStrategy::Name` (the test actually uses `SortStrategy::Renew`) and `seven_day_sonnet_util=97%`/`7d_son_left=3%` (the test actually uses `95.0`/`5%`, confirmed via `mk_aq_sort_weekly`'s signature in `src/usage/test_support.rs:151-163` and the test's own doc comment, which states the boundary check is exactly `min(32%, 5%) = 5.0` and `5.0 > 5.0` is false). The second account's test-code name is `red-account`, not `weekly-exh`.
- **Source fn:** `mre_bug299_h_exhausted_misclassified_as_red_prefer_any` (in `sort_tests.rs`)
- **Source:** [feature/020_usage_sort_strategies.md AC-12](../../../docs/feature/020_usage_sort_strategies.md); BUG-299

---

### FT-12: `prefer::son` + absent Sonnet tier → `prefer_weekly = 0.0` (not 100.0)

- **Given:** An `AccountQuota` with `seven_day_sonnet = None` (no Sonnet tier) and `seven_day_util=30%` (7d_left=70%). `prefer::son` in effect.
- **When:** `prefer_weekly(aq, PreferStrategy::Sonnet)` is called (internally delegates to `relevant_quotas(aq, Sonnet).1`).
- **Then:** Returns `0.0`. Absent Sonnet tier under `prefer::son` = unknown Sonnet capacity, not 100%. `prefer_weekly = 0.0` causes the account to sort last in within-group tiebreak. Eligibility is model-agnostic: determined by raw `seven_day_left`, not `prefer_weekly` (Fix BUG-324).
- **Exit:** n/a (unit test — return value assertion)
- **Note:** Phase 2 fix from Plan 019. Old code: `map_or(0.0, |p| p.utilization)` returned `100.0 - 0.0 = 100.0`, treating absent tier as fully available. Fix: `if let Some(ref son)` guard returns `0.0` when `seven_day_sonnet = None`.
- **Source fn:** `test_relevant_quotas_son_no_sonnet` (in `tests/usage/format_tests.rs`)
- **Source:** [feature/020_usage_sort_strategies.md AC-05](../../../docs/feature/020_usage_sort_strategies.md)

---

### FT-13: `sort::` drives footer recommendation — top eligible account shown in `Next (<strategy>)` line; `·`-delimited 2-line format

- **Given (unit test):** Three `AccountQuota` structs: `current@x.com` (`is_current=true`, valid quota, `five_hour_util=20%`), `eligible@x.com` (valid quota, `five_hour_util=10%`, non-current, non-active), `exhausted@x.com` (valid quota, `five_hour_util=99%`, h-exhausted). `session_model = Some("sonnet")`, `session_effort = Some("low")`.
- **When:** `render_text(&accounts, SortStrategy::Renew, ...)` is called.
- **Then:** Footer line 1 contains `Current · current@x.com · sonnet/low · 2/3` — identifies the `✓` account with session model/effort (passed-in `session_effort`, displayed as-is on the Current line) and valid/total count. Footer line 2 contains `Next (renew) · eligible@x.com · sonnet/high` — model-derived effort always shown unconditionally on the Next line (TSK-335 H3: `"high"` for Sonnet regardless of `session_effort`). `exhausted@x.com` is skipped (h-exhausted → ineligible). Both lines use `·` delimiters with column alignment.
- **Exit:** n/a (unit test — string assertions on `render_text` output)
- **Source fn:** none exact (N/A — no single cited test builds this exact 3-account current/eligible/exhausted fixture with `session_model="sonnet"`/`session_effort="low"`). The individual claims are each covered by separate tests: `test_ft29_009_footer_session_effort_display` (`render_tests_b.rs:16`, Feature 009 FT-29) confirms the `Current · name · <model>/<effort> · N/M` line-1 format (using `claude-sonnet-5/low`, not `sonnet/low`); `ft05_footer_next_shows_model_and_effort_when_set` (`render_tests_b.rs:650`, Feature 062 FT-05) confirms the Next line shows `sonnet/high` when Sonnet is available; `test_ft08_020_footer_omits_recommendation_when_no_eligible_candidate` (`mod_tests.rs:388`) confirms exhausted/ineligible accounts are skipped for the Next slot. No single test asserts the full two-line `2/3` + `sonnet/low` + `sonnet/high` combination together. Live end-to-end smoke over the real binary: `it102`/`it103`/`it104` (`tests/cli/usage_touch_test.rs`) assert the CONDITIONAL contract — footer `Next (<strategy>)` line present when the sole candidate clears both quota floors (`5h > 15%`, `7d > 3%`), suppressed when it is exhausted — because their fixture caches mirror the host's live snapshot, so either state is legitimate at run time (audit-live-footer-fragile; previously they asserted the footer unconditionally and failed whenever the operator's live account crossed the weekly-exhaustion floor).
- **Source:** [feature/020_usage_sort_strategies.md AC-09](../../../docs/feature/020_usage_sort_strategies.md)

---

### FT-14: Green account with divergent `7d/7d_son` passes eligibility gate — model-agnostic `seven_day_left` used (BUG-324)

- **Given:** Two `AccountQuota` structs:
  - `aaa_target@test.com`: `five_hour_util=0%` (5h_left=100%), `seven_day_util=69%` (7d_left=31%), `seven_day_sonnet_util=100%` (7d_son_left=0%). Green (both quotas above status-group thresholds). Non-current, non-active.
  - `current@test.com`: `is_current=true` — forces selection of `aaa_target@test.com`.
- **When:** `find_next_for_strategy(&accounts, SortStrategy::Renew, PreferStrategy::Any, now, false, "anthropic")` — gate 7 evaluates eligibility.
- **Then:** Returns `Some(0)` — `aaa_target@test.com` is eligible. Gate 7 uses `seven_day_left(aq) = 31.0 > 3.0` (model-agnostic raw 7d quota). Before Fix(BUG-324): `prefer_weekly(aq, Any) = min(31.0, 0.0) = 0.0 ≤ 5.0` — gate would fire and block this green account.
- **Exit:** n/a (unit test — return value assertion)
- **Note:** Same class as BUG-299 (fixed in `sort.rs` status groups, left in `sort_next.rs` eligibility gate). Eligibility is model-agnostic; `apply_model_override()` handles model selection post-rotation. Corrected — `find_next_for_strategy`'s real signature (`src/usage/sort_next.rs:63-70`) takes a 6th `selected_provider: &str` argument, previously omitted from the When clause. The cited test also loops the assertion across all 3 `SortStrategy` variants × all 3 `PreferStrategy` variants (9 combinations total, all asserting `Some(0)`), not only the single `Renew`/`Any` combination shown above.
- **Source fn:** `mre_bug324_green_account_eligible_when_7d_son_exhausted` (in `sort_next_tests_b.rs`)
- **Source:** [feature/020_usage_sort_strategies.md AC-09](../../../docs/feature/020_usage_sort_strategies.md); BUG-324
