# Test: Feature 039 — Decision Algorithm Reference

Feature behavioral requirement test cases for `docs/feature/039_decision_algorithms.md`. Tests verify the six core decision algorithms: touch model selection (Table 1), session model override (Table 2), quota status groups (Table 3), next-account eligibility gates (Table 4), next-account positive selection (Table 5), and quota approximation (Table 6).

### AC Coverage Index

| FT | Algorithm | Table | Notes |
|----|-----------|-------|-------|
| FT-01 | Touch model: idle Sonnet window → Sonnet | Table 1 | Unit test |
| FT-02 | Touch model: absent Sonnet tier → Haiku | Table 1 | Unit test |
| FT-09 | Touch model: active Sonnet window, 40% remaining → Sonnet | Table 1 | Unit test — Fix BUG-301 |
| FT-03 | Session model override: exactly 10% Sonnet left → writes Sonnet (Fix BUG-311) | Table 2 | Boundary — Phase 1 fix |
| FT-04 | Session model override: below 10% Sonnet left → Opus | Table 2 | Boundary — Phase 1 fix |
| FT-05 | Status groups: four-group partition order (both-exhausted in G3) | Table 3 | Unit test |
| FT-06 | Eligibility gate G7: `seven_day_left ≤ WEEKLY_EXHAUSTION_THRESHOLD` → account skipped | Table 4 | Phase 2 |
| FT-07 | Eligibility gate G7: `seven_day_left > WEEKLY_EXHAUSTION_THRESHOLD` → account eligible | Table 4 | Phase 2 |
| FT-08 | Positive selection: first eligible Green account wins | Table 5 | Unit test |
| FT-10 | Quadratic extrapolation from 3+ measurements | Table 6 | Cross-ref F040 FT-04 |
| FT-11 | Expired window returns 0.0 | Table 6 | Cross-ref F040 FT-07 |
| FT-12 | Singular matrix falls back to constant | Table 6 | Cross-ref F040 FT-10 |
| FT-13 | Eligibility gate G7: divergent `7d/7d_son` — model-agnostic `seven_day_left` used (BUG-324) | Table 4 | BUG-324 fix |

### Test Case Index

| ID | Test Name | Table | Category |
|----|-----------|-------|----------|
| FT-01 | Touch model idle Sonnet → Sonnet | Table 1 | Touch Model |
| FT-02 | Touch model absent Sonnet → Haiku | Table 1 | Touch Model |
| FT-09 | Touch model active Sonnet, 40% remaining → Sonnet | Table 1 | Touch Model |
| FT-03 | Session override at 10% boundary → no-op | Table 2 | Session Override |
| FT-04 | Session override below 10% → Opus | Table 2 | Session Override |
| FT-05 | Status group four-partition order (both-exhausted in G3) | Table 3 | Status Groups |
| FT-06 | Gate 7 seven_day_left ≤ WEEKLY_EXHAUSTION_THRESHOLD skips account | Table 4 | Eligibility |
| FT-07 | Gate 7 seven_day_left > WEEKLY_EXHAUSTION_THRESHOLD passes account | Table 4 | Eligibility |
| FT-08 | Positive selection first eligible wins | Table 5 | Selection |
| FT-10 | Quadratic extrapolation from 3+ measurements | Table 6 | Approximation |
| FT-11 | Expired window returns 0.0 | Table 6 | Approximation |
| FT-12 | Singular matrix falls back to constant | Table 6 | Approximation |
| FT-13 | Gate 7 divergent 7d/7d_son — model-agnostic seven_day_left (BUG-324) | Table 4 | Eligibility |

**Total:** 13 FT cases

---

### FT-01: Touch model — idle Sonnet window selects Sonnet

- **Given:** An `AccountQuota` with `seven_day_sonnet = Some(PeriodUsage { resets_at: None, utilization: 40.0 })` — Sonnet tier present, `resets_at = None` (no active window → `son_idle = true`).
- **When:** `resolve_model(aq)` is called (entry point: `subprocess.rs:29-59`).
- **Then:** Returns `Sonnet`. `son_idle = son.resets_at.is_none() = true` (no active window). Gate `son_idle || son_available`: `son_idle=true` → Sonnet selected to activate the idle window.
- **Note:** Table 1 row 2. This is the warm-up case: no active Sonnet session → using Sonnet keeps it warm.
- **Source fn:** `it_imodel_auto_selects_sonnet_when_son_idle` (in `src/usage/subprocess.rs`)
- **Source:** [feature/039_decision_algorithms.md Table 1](../../../docs/feature/039_decision_algorithms.md)

---

### FT-02: Touch model — absent Sonnet tier selects Haiku

- **Given:** An `AccountQuota` with `seven_day_sonnet = None` — no Sonnet tier on this account.
- **When:** `resolve_model(aq)` is called.
- **Then:** Returns `Haiku`. `if let Some(ref son) = data.seven_day_sonnet` does not match (tier absent). Falls through to Haiku. No Sonnet tier → conserve quota.
- **Note:** Table 1 row 1.
- **Source fn:** `it_imodel_auto_selects_haiku_when_son_tier_absent` (in `src/usage/subprocess.rs`)
- **Source:** [feature/039_decision_algorithms.md Table 1](../../../docs/feature/039_decision_algorithms.md)

---

### FT-09: Touch model — active Sonnet window with remaining quota selects Sonnet

- **Given:** An `AccountQuota` with `seven_day_sonnet = Some(PeriodUsage { resets_at: Some("2026-06-20T..."), utilization: 60.0 })` — Sonnet window active (`resets_at=Some`), 40% quota remaining (`100.0 - 60.0 = 40.0 > 20.0` → `son_available=true`).
- **When:** `resolve_model(aq)` is called (entry point: `subprocess.rs:29-59`).
- **Then:** Returns `Sonnet`. `son_idle=false` (window running), but `son_available=true` (40% > 20% threshold). Gate `son_idle || son_available = true`. Remaining Sonnet quota must not expire unused.
- **Note:** Table 1 row 3. Fix BUG-301 (TSK-311): old binary `son_idle` gate returned Haiku in this case; extended gate adds `son_available` check so quota is consumed before window expires.
- **Source fn:** `mre_bug301_son_active_with_remaining_quota_selects_sonnet` (in `src/usage/subprocess.rs`)
- **Source:** [feature/039_decision_algorithms.md Table 1](../../../docs/feature/039_decision_algorithms.md)

---

### FT-03: Session model override — exactly 10% Sonnet left → writes Sonnet (Fix BUG-311)

- **Given:** An `AccountQuota` with `seven_day_sonnet = Some(PeriodUsage { utilization: 90.0, .. })` → `sonnet_left = 100.0 - 90.0 = 10.0%`.
- **When:** `apply_model_override(&quota, &paths, false, "usage", "test-account")` is called (canonical path: `api.rs:259-290`; uses `recommended_model(aq)` from `format.rs` — Feature 062).
- **Then:** Writes `"sonnet"` to `settings.json`. Opus override does NOT fire (`sonnet_left < OPUS_OVERRIDE_THRESHOLD (10.0)` evaluates to `10.0 < 10.0 = false`). Sonnet-restore path fires via `override_session_model_to_sonnet()` (Fix BUG-311).
- **Note:** Table 2 boundary. Exactly at threshold = sufficient capacity. Threshold canonicalized as `OPUS_OVERRIDE_THRESHOLD` in `format.rs` (Feature 062).
- **Source fn:** `t07_model_override_writes_sonnet_at_10pct_boundary` (in `tests/usage/api_tests_a.rs`); `test_render_footer_model_label_at_10pct_no_override` (in `tests/usage/render_tests_a.rs`)
- **Source:** [feature/039_decision_algorithms.md Table 2](../../../docs/feature/039_decision_algorithms.md)

---

### FT-04: Session model override — below 10% Sonnet left → Opus

- **Given:** An `AccountQuota` with `seven_day_sonnet = Some(PeriodUsage { utilization: 90.1, .. })` → `sonnet_left ≈ 9.9%`. Current model is Sonnet.
- **When:** `recommended_model(aq)` is called (canonical entry point: `format.rs`; called by `apply_model_override` at `api.rs:259-290` — Feature 062).
- **Then:** Returns `"opus"`. Override fires. The gate `sonnet_left < OPUS_OVERRIDE_THRESHOLD (10.0)` evaluates to `9.9 < 10.0 = true`. Sonnet near-exhausted — preserve remaining tokens.
- **Note:** Table 2 boundary. Threshold canonicalized as `OPUS_OVERRIDE_THRESHOLD` in `format.rs` (Feature 062).
- **Source fn:** `test_render_footer_model_label_below_10pct_opus` (in `tests/usage/render_tests_a.rs`)
- **Source:** [feature/039_decision_algorithms.md Table 2](../../../docs/feature/039_decision_algorithms.md)

---

### FT-05: Status groups — four-group partition: Green → h-exhausted → weekly-exhausted → Dead

- **Given:** Five `AccountQuota` structs fed to `sort_indices` under `sort::name` (which would interleave alphabetically without group partitioning):
  - `green@x.com`: `five_hour_util=10%` (5h_left=90%), `seven_day_util=10%` (7d_left=90%) — both thresholds above → 🟢 G1 Green.
  - `h_exh@x.com`: `five_hour_util=90%` (5h_left=10% ≤ 15%), `seven_day_util=10%` (7d_left=90%) — 5h exhausted → 🟡 G2 h-exhausted.
  - `weekly@x.com`: `five_hour_util=10%` (5h_left=90%), `seven_day_util=98%` (7d_left=2% ≤ 5%) — 7d exhausted → 🟡 G3 weekly-exhausted.
  - `both@x.com`: `five_hour_util=94%` (5h_left=6% ≤ 15%), `seven_day_util=96%` (7d_left=4% ≤ 5%) — both exhausted → 🟡 G3 weekly-exhausted (7d is binding; Fix BUG-321).
  - `dead@x.com`: `result = Err(...)` → 🔴 G4 Dead.
- **When:** `status_group_of(aq)` is evaluated per account via `sort_indices` (entry point: `sort.rs:31-48`).
- **Then:** Group assignment: `green@x.com` → Green; `h_exh@x.com` → HExhausted; `weekly@x.com` → WeeklyExhausted; `both@x.com` → WeeklyExhausted (same G3 — `(false,false)` maps to `StatusGroup::WeeklyExhausted`); `dead@x.com` → Dead. Output row order: Green (G1) → h-exhausted (G2) → weekly-exhausted (G3, including `both@x.com`) → Dead (G4 🔴). `sort::name` alpha order is overridden by group partition.
- **Source fn:** `mre_bug321_four_group_partition_order` (in `src/usage/sort.rs` or `src/usage/mod.rs`)
- **Source:** [feature/039_decision_algorithms.md Table 3](../../../docs/feature/039_decision_algorithms.md)

---

### FT-06: Eligibility gate G7 — `seven_day_left ≤ WEEKLY_EXHAUSTION_THRESHOLD` skips account

- **Given:** An `AccountQuota` with `seven_day_util=96%` (7d_left=4%) and `seven_day_sonnet = None`. `prefer::any` in effect.
- **When:** The `extra` predicate in `find_next_for_strategy()` evaluates Gate 7 (`sort_next.rs:59`): `seven_day_left(aq) <= WEEKLY_EXHAUSTION_THRESHOLD`.
- **Then:** `seven_day_left` returns `4.0` (raw 7d_left). `4.0 ≤ 5.0` → gate fires → account is skipped. No `->` marker assigned.
- **Note:** Table 4 gate 7. Eligibility uses `seven_day_left` (model-agnostic raw 7d quota). `prefer_weekly` is used only for sort tiebreak (Fix BUG-324).
- **Source fn:** `test_relevant_quotas_son_no_sonnet` (in `tests/usage/format_tests.rs`); `mre_bug292_renew_skips_weekly_exhausted_even_with_soonest_renewal` (in `tests/usage/sort_next_tests.rs`)
- **Source:** [feature/039_decision_algorithms.md Table 4](../../../docs/feature/039_decision_algorithms.md)

---

### FT-07: Eligibility gate G7 — `seven_day_left > WEEKLY_EXHAUSTION_THRESHOLD` passes account

- **Given:** An `AccountQuota` with `seven_day_util=90%` (7d_left=10%) and `seven_day_sonnet = None`. `prefer::any` in effect. All other gates (G1–G6, G8) do not fire.
- **When:** Gate 7 evaluates `seven_day_left(aq)`.
- **Then:** Returns `10.0`. `10.0 > WEEKLY_EXHAUSTION_THRESHOLD (5.0)` → gate does NOT fire. Account remains eligible. `->` marker may be assigned if first in sorted order.
- **Source fn:** `test_relevant_quotas_son_with_sonnet` (in `tests/usage/format_tests.rs`); `test_find_next_for_strategy_some_when_eligible_none_when_all_current` (in `tests/usage/sort_next_tests.rs`)
- **Source:** [feature/039_decision_algorithms.md Table 4](../../../docs/feature/039_decision_algorithms.md)

---

### FT-08: Positive selection — first eligible Green account wins

- **Given:** Three non-current, non-active, non-occupied Green `AccountQuota` structs inserted in reverse-alphabetical order: `c@x.com`, `a@x.com`, `b@x.com`. All pass all 8 eligibility gates. Sort strategy: `sort::name`.
- **When:** `find_next_for_strategy()` walks the sorted list (entry point: `sort_next.rs:46-83`). `sort_indices` produces order: `a@x.com`, `b@x.com`, `c@x.com`.
- **Then:** `a@x.com` is selected as the winner (position 0 in sorted order, first to pass all 8 gates). `->` marker assigned to `a@x.com`.
- **Note:** Table 5 Step 3: first account in the within-group sorted order that passes all gates wins. Input insertion order does not determine the winner.
- **Source fn:** `test_sort_name_alphabetical` (in `src/usage/sort.rs`); `test_find_next_for_strategy_some_when_eligible_none_when_all_current` (in `tests/usage/sort_next_tests.rs`)
- **Source:** [feature/039_decision_algorithms.md Table 5](../../../docs/feature/039_decision_algorithms.md)

---

### FT-10: Quadratic extrapolation from 3+ measurements

- **Given:** A history of 3 non-collinear measurements with accelerating utilization: `[(t0, 20.0), (t0+100, 50.0), (t0+200, 90.0)]`. Window active (`resets_at > now`). `now = t0 + 300`.
- **When:** `approximate_utilization()` is called with the 3 measurements, `resets_at`, window duration, and `now`.
- **Then:** Returns `Some(value)` where `value > 90.0` (extrapolated beyond last measurement due to accelerating quadratic fit). Value is clamped to [0.0, 100.0].
- **Note:** Table 6 row "3–10 measurements → degree 2 quadratic LS (Cramer 3x3)". Detailed algorithm testing in F040 FT-04.
- **Source fn:** `approx_quadratic_three_points_extrapolates` (in `src/usage/approx.rs`)
- **Source:** [feature/039_decision_algorithms.md Table 6](../../../docs/feature/039_decision_algorithms.md)

---

### FT-11: Expired window returns 0.0

- **Given:** Measurements exist in history, but the window has expired: `now > resets_at`.
- **When:** `approximate_utilization()` is called with `resets_at_secs = Some(r)` where `now > r`.
- **Then:** Returns `Some(0.0)`. Window expired → new window starts at 0% utilization regardless of historical measurements.
- **Note:** Table 6 pre-fit: "If `now > resets_at` → return 0.0 (window expired)". Detailed testing in F040 FT-07.
- **Source fn:** `approx_expired_window_returns_zero` (in `src/usage/approx.rs`)
- **Source:** [feature/039_decision_algorithms.md Table 6](../../../docs/feature/039_decision_algorithms.md)

---

### FT-12: Singular matrix falls back to constant

- **Given:** 3+ measurements that are perfectly collinear or have identical timestamps, producing a singular normal-equations matrix (`|det| < 1e-12`).
- **When:** `approximate_utilization()` attempts quadratic LS fit.
- **Then:** Falls back to linear; if linear also singular, returns last measurement value (constant). Result is clamped to [0.0, 100.0].
- **Note:** Table 6 fallback column: "linear if singular". Detailed testing in F040 FT-10.
- **Source fn:** `approx_singular_matrix_falls_back_to_constant` (in `src/usage/approx.rs`)
- **Source:** [feature/039_decision_algorithms.md Table 6](../../../docs/feature/039_decision_algorithms.md)

---

### FT-13: Eligibility gate G7 — divergent `7d/7d_son` with `seven_day_sonnet` present; model-agnostic `seven_day_left` used (BUG-324)

- **Given:** Two `AccountQuota` structs:
  - `aaa_target@test.com`: `five_hour_util=0%`, `seven_day_util=69%` (7d_left=31%), `seven_day_sonnet_util=100%` (7d_son_left=0%). Non-current, non-active, non-occupied. `prefer::any` in effect.
  - `current@test.com`: `is_current=true` — forces selection.
- **When:** Gate 7 evaluates eligibility in `find_next_for_strategy()` (entry: `sort_next.rs:59`).
- **Then:** Account passes gate 7. Fix(BUG-324): gate uses `seven_day_left(aq) = 31.0 > WEEKLY_EXHAUSTION_THRESHOLD (5.0)` — model-agnostic. Before fix: `prefer_weekly(aq, Any) = min(31.0, 0.0) = 0.0 ≤ 5.0` — model-aware value blocked a green account from rotation.
- **Note:** Table 4 gate 7. Existing FT-06/FT-07 test with `seven_day_sonnet = None` (no divergence possible). This case exercises the divergence path where `seven_day_sonnet` is present and differs from `seven_day`. Same class as BUG-299.
- **Source fn:** `mre_bug324_green_account_eligible_when_7d_son_exhausted` (in `tests/usage/sort_next_tests.rs`)
- **Source:** [feature/039_decision_algorithms.md Table 4](../../../docs/feature/039_decision_algorithms.md)

---

## Source Reference

| Algorithm | Unit Tests | Implementation |
|-----------|-----------|----------------|
| Touch model selection (Table 1) | `it_imodel_auto_selects_sonnet_when_son_idle`, `it_imodel_auto_selects_haiku_when_son_tier_absent`, `mre_bug301_son_active_with_remaining_quota_selects_sonnet` in `src/usage/subprocess.rs` | `subprocess.rs:29-59` |
| Session model override (Table 2) | `ft01..ft04_recommended_model_*` in `tests/usage/format_tests.rs` (Feature 062); `t07_model_override_writes_sonnet_at_10pct_boundary` in `tests/usage/api_tests_a.rs`; `test_render_footer_model_label_at_10pct_no_override`, `test_render_footer_model_label_below_10pct_opus` in `tests/usage/render_tests_a.rs` | `format.rs` (`recommended_model`, `OPUS_OVERRIDE_THRESHOLD`), `api.rs:259-290` (`apply_model_override`), `render.rs` (footer) |
| Quota status groups (Table 3) | `test_three_tier_grouping_*` in `src/usage/mod.rs` | `sort.rs:31-48` |
| Eligibility gates (Table 4) | `test_relevant_quotas_*` in `tests/usage/format_tests.rs` | `sort_next.rs:24-35, 59` |
| Positive selection (Table 5) | `test_sort_name_alphabetical` in `src/usage/sort.rs` | `sort_next.rs:46-83` |
| Quota approximation (Table 6) | `approx_quadratic_three_points_extrapolates`, `approx_expired_window_returns_zero`, `approx_singular_matrix_falls_back_to_constant` in `src/usage/approx.rs` | `approx.rs` |
