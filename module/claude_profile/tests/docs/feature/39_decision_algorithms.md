# Test: Feature 039 — Decision Algorithm Reference

Feature behavioral requirement test cases for `docs/feature/039_decision_algorithms.md`. Tests verify the five core decision algorithms: touch model selection (Table 1), session model override (Table 2), quota status groups (Table 3), next-account eligibility gates (Table 4), and next-account positive selection (Table 5).

### AC Coverage Index

| FT | Algorithm | Table | Notes |
|----|-----------|-------|-------|
| FT-01 | Touch model: idle Sonnet window → Sonnet | Table 1 | Unit test |
| FT-02 | Touch model: absent Sonnet tier → Haiku | Table 1 | Unit test |
| FT-09 | Touch model: active Sonnet window, 40% remaining → Sonnet | Table 1 | Unit test — Fix BUG-301 |
| FT-03 | Session model override: exactly 15% Sonnet left → no override | Table 2 | Boundary — Phase 1 fix |
| FT-04 | Session model override: below 15% Sonnet left → Opus | Table 2 | Boundary — Phase 1 fix |
| FT-05 | Status groups: four-group partition order | Table 3 | Unit test |
| FT-06 | Eligibility gate G7: `prefer_weekly ≤ 5.0` → account skipped | Table 4 | Phase 2 |
| FT-07 | Eligibility gate G7: `prefer_weekly > 5.0` → account eligible | Table 4 | Phase 2 |
| FT-08 | Positive selection: first eligible Green account wins | Table 5 | Unit test |
| FT-10 | Quadratic extrapolation from 3+ measurements | Table 6 | Cross-ref F040 FT-04 |
| FT-11 | Expired window returns 0.0 | Table 6 | Cross-ref F040 FT-07 |
| FT-12 | Singular matrix falls back to constant | Table 6 | Cross-ref F040 FT-10 |

### Test Case Index

| ID | Test Name | Table | Category |
|----|-----------|-------|----------|
| FT-01 | Touch model idle Sonnet → Sonnet | Table 1 | Touch Model |
| FT-02 | Touch model absent Sonnet → Haiku | Table 1 | Touch Model |
| FT-09 | Touch model active Sonnet, 40% remaining → Sonnet | Table 1 | Touch Model |
| FT-03 | Session override at 15% boundary → no-op | Table 2 | Session Override |
| FT-04 | Session override below 15% → Opus | Table 2 | Session Override |
| FT-05 | Status group four-partition order | Table 3 | Status Groups |
| FT-06 | Gate 7 prefer_weekly ≤ 5.0 skips account | Table 4 | Eligibility |
| FT-07 | Gate 7 prefer_weekly > 5.0 passes account | Table 4 | Eligibility |
| FT-08 | Positive selection first eligible wins | Table 5 | Selection |
| FT-10 | Quadratic extrapolation from 3+ measurements | Table 6 | Approximation |
| FT-11 | Expired window returns 0.0 | Table 6 | Approximation |
| FT-12 | Singular matrix falls back to constant | Table 6 | Approximation |

**Total:** 12 FT cases

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

### FT-03: Session model override — exactly 15% Sonnet left → no override

- **Given:** An `AccountQuota` with `seven_day_sonnet = Some(PeriodUsage { utilization: 85.0, .. })` → `sonnet_left = 100.0 - 85.0 = 15.0%`. Current model is Sonnet.
- **When:** `apply_model_override(aq, params)` is called (entry point: `api.rs:259-290`).
- **Then:** Override does NOT fire. Model remains Sonnet. The gate `sonnet_left < 15.0` evaluates to `15.0 < 15.0 = false`.
- **Note:** Table 2 boundary. Exactly at threshold = sufficient capacity. Phase 1 fix: threshold changed from `20.0` to `15.0` at `api.rs:274`.
- **Source fn:** `t07_model_override_skips_at_and_above_15pct_boundary` (in `src/usage/api_tests.rs`); `test_render_footer_model_label_at_15pct_no_override` (in `src/usage/render_tests.rs`)
- **Source:** [feature/039_decision_algorithms.md Table 2](../../../docs/feature/039_decision_algorithms.md)

---

### FT-04: Session model override — below 15% Sonnet left → Opus

- **Given:** An `AccountQuota` with `seven_day_sonnet = Some(PeriodUsage { utilization: 85.1, .. })` → `sonnet_left ≈ 14.9%`. Current model is Sonnet.
- **When:** `apply_model_override(aq, params)` is called.
- **Then:** Override fires. Model written as `"opus"`. The gate `sonnet_left < 15.0` evaluates to `14.9 < 15.0 = true`. Sonnet near-exhausted — preserve remaining tokens.
- **Note:** Table 2 boundary. Phase 1 fix.
- **Source fn:** `test_render_footer_model_label_below_15pct_opus` (in `src/usage/render_tests.rs`)
- **Source:** [feature/039_decision_algorithms.md Table 2](../../../docs/feature/039_decision_algorithms.md)

---

### FT-05: Status groups — four-group partition: Green → h-exhausted → weekly-exhausted → Red

- **Given:** Four `AccountQuota` structs fed to `sort_indices` under `sort::name` (which would interleave alphabetically without group partitioning):
  - `green@x.com`: `five_hour_util=10%` (5h_left=90%), `seven_day_util=10%` (7d_left=90%) — both thresholds above → 🟢 Green.
  - `h_exh@x.com`: `five_hour_util=90%` (5h_left=10% ≤ 15%), `seven_day_util=10%` (7d_left=90%) — 5h exhausted → 🟡 h-exhausted.
  - `weekly@x.com`: `five_hour_util=10%` (5h_left=90%), `seven_day_util=98%` (7d_left=2% ≤ 5%) — 7d exhausted → 🟡 weekly-exhausted.
  - `red@x.com`: `result = Err(...)` → 🔴 Red.
- **When:** `status_group_of(aq)` is evaluated per account via `sort_indices` (entry point: `sort.rs:31-48`).
- **Then:** Group assignment: `green@x.com` → Green; `h_exh@x.com` → HExhausted; `weekly@x.com` → WeeklyExhausted; `red@x.com` → Red. Output row order: Green → h-exhausted → weekly-exhausted → Red. `sort::name` alpha order is overridden by group partition.
- **Source fn:** `test_three_tier_grouping_green_before_yellow_before_red` (in `src/usage/mod.rs`)
- **Source:** [feature/039_decision_algorithms.md Table 3](../../../docs/feature/039_decision_algorithms.md)

---

### FT-06: Eligibility gate G7 — `prefer_weekly ≤ 5.0` skips account

- **Given:** An `AccountQuota` with `seven_day_util=96%` (7d_left=4%) and `seven_day_sonnet = None`. `prefer::any` in effect.
- **When:** The `extra` predicate in `find_next_for_strategy()` evaluates Gate 7 (`sort_next.rs:59`): `prefer_weekly(aq, PreferStrategy::Any) <= 5.0`.
- **Then:** `prefer_weekly` returns `4.0` (raw 7d_left when `seven_day_sonnet = None` under `prefer::any`). `4.0 ≤ 5.0` → gate fires → account is skipped. No `->` marker assigned.
- **Note:** Table 4 gate 7. `prefer_weekly` is computed via `relevant_quotas(aq, prefer).1` (Phase 2 extraction).
- **Source fn:** `test_relevant_quotas_son_no_sonnet` (in `src/usage/format_tests.rs`); `mre_bug292_renew_skips_weekly_exhausted_even_with_soonest_renewal` (in `src/usage/sort_next_tests.rs`)
- **Source:** [feature/039_decision_algorithms.md Table 4](../../../docs/feature/039_decision_algorithms.md)

---

### FT-07: Eligibility gate G7 — `prefer_weekly > 5.0` passes account

- **Given:** An `AccountQuota` with `seven_day_util=90%` (7d_left=10%) and `seven_day_sonnet = None`. `prefer::any` in effect. All other gates (G1–G6, G8) do not fire.
- **When:** Gate 7 evaluates `prefer_weekly(aq, PreferStrategy::Any)`.
- **Then:** Returns `10.0`. `10.0 > 5.0` → gate does NOT fire. Account remains eligible. `->` marker may be assigned if first in sorted order.
- **Source fn:** `test_relevant_quotas_son_with_sonnet` (in `src/usage/format_tests.rs`); `test_find_next_for_strategy_some_when_eligible_none_when_all_current` (in `src/usage/sort_next_tests.rs`)
- **Source:** [feature/039_decision_algorithms.md Table 4](../../../docs/feature/039_decision_algorithms.md)

---

### FT-08: Positive selection — first eligible Green account wins

- **Given:** Three non-current, non-active, non-occupied Green `AccountQuota` structs inserted in reverse-alphabetical order: `c@x.com`, `a@x.com`, `b@x.com`. All pass all 8 eligibility gates. Sort strategy: `sort::name`.
- **When:** `find_next_for_strategy()` walks the sorted list (entry point: `sort_next.rs:46-83`). `sort_indices` produces order: `a@x.com`, `b@x.com`, `c@x.com`.
- **Then:** `a@x.com` is selected as the winner (position 0 in sorted order, first to pass all 8 gates). `->` marker assigned to `a@x.com`.
- **Note:** Table 5 Step 3: first account in the within-group sorted order that passes all gates wins. Input insertion order does not determine the winner.
- **Source fn:** `test_sort_name_alphabetical` (in `src/usage/sort.rs`); `test_find_next_for_strategy_some_when_eligible_none_when_all_current` (in `src/usage/sort_next_tests.rs`)
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

## Source Reference

| Algorithm | Unit Tests | Implementation |
|-----------|-----------|----------------|
| Touch model selection (Table 1) | `it_imodel_auto_selects_sonnet_when_son_idle`, `it_imodel_auto_selects_haiku_when_son_tier_absent`, `mre_bug301_son_active_with_remaining_quota_selects_sonnet` in `src/usage/subprocess.rs` | `subprocess.rs:29-59` |
| Session model override (Table 2) | `t07_model_override_skips_at_and_above_15pct_boundary` in `src/usage/api_tests.rs`; `test_render_footer_model_label_at_15pct_no_override`, `test_render_footer_model_label_below_15pct_opus` in `src/usage/render_tests.rs` | `api.rs:259-290`, `render.rs:258` |
| Quota status groups (Table 3) | `test_three_tier_grouping_*` in `src/usage/mod.rs` | `sort.rs:31-48` |
| Eligibility gates (Table 4) | `test_relevant_quotas_*` in `src/usage/format_tests.rs` | `sort_next.rs:24-35, 59` |
| Positive selection (Table 5) | `test_sort_name_alphabetical` in `src/usage/sort.rs` | `sort_next.rs:46-83` |
| Quota approximation (Table 6) | `approx_quadratic_three_points_extrapolates`, `approx_expired_window_returns_zero`, `approx_singular_matrix_falls_back_to_constant` in `src/usage/approx.rs` | `approx.rs` |
