# Algorithm 011: Rounding-Boundary Classification Hazards

AC test cases for `docs/algorithm/011_rounding_boundary_classification_hazards.md`. Tests `pct_emoji` (closure inside `quota_text_cells(data: &claude_quota::OauthUsageData, now_secs: u64) -> [String; 5]`, `src/usage/format.rs:435-474`) and `apply_model_override(quota: &OauthUsageData, paths: &crate::ClaudePaths, trace: bool, label: &str, name: &str)` (`src/usage/api_switch.rs:225-298`).

**Type note:** Both functions derive a classification decision (emoji color, or sonnet↔opus branch) and a rounded display/log string from the same raw `f64` value. Post-BUG-331 fix, both round once (`.round()`) and reuse the rounded value for both the comparison and the display — this is the current, correct behavior these cases verify.

### AC Case Index

| AC | Short Name | Category | Status |
|----|------------|----------|--------|
| AC-1 | `pct_emoji` well inside threshold → 🟢 with matching rounded text | Nominal | ✅ |
| AC-2 | `pct_emoji` exact-integer threshold → 🟡 (boundary inclusive) | Boundary | ✅ |
| AC-3 | `apply_model_override` with `seven_day_sonnet = None` → conservative sonnet branch, no threshold compare | Error | ✅ |
| AC-4 | `pct_emoji` floating-point noise straddling `WEEKLY_EXHAUSTION_THRESHOLD` → identical rounded text, same color both sides (BUG-331) | Regression (BUG-331) | ✅ |

---

### AC-1: `pct_emoji` well inside threshold selects green with matching rounded text

- **Given:** `util = Some(56.0)` (44% left), `threshold = WEEKLY_EXHAUSTION_THRESHOLD (5.0)` — raw `left = 44.0`, far from the threshold and from any rounding boundary
- **When:** `pct_emoji(util, threshold)` is called
- **Then:** Returns `"🟢 44%"` — `left.round() = 44.0 > 5.0`, so 🟢 is selected, and the identical rounded `left` is reused for the `{left:.0}%` display text

### AC-2: `pct_emoji` at the exact-integer threshold selects yellow (boundary inclusive)

- **Given:** `util = Some(85.0)`, `threshold = H_EXHAUSTED_THRESHOLD (15.0)` — raw `left = 15.0` exactly, landing precisely on the threshold
- **When:** `pct_emoji(util, threshold)` is called
- **Then:** Returns `"🟡 15%"` — the branch is `if left > threshold { 🟢 } else { 🟡 }`; `15.0 > 15.0` is `false`, so the exact-boundary case is inclusive on the 🟡 side
- **Note:** Mirrors the existing exact-boundary coverage in `tests/usage/format_tests.rs:502-526` (`test_ft11_009_per_column_emoji_prefix_three_cases`, case "Pct C") — that test proves the closure is correct AT the threshold; this spec's AC-4 covers the separate near-threshold floating-point-noise gap BUG-331 exposed

### AC-3: `apply_model_override` with absent `seven_day_sonnet` takes the conservative branch, skipping the threshold compare

- **Given:** `quota.seven_day_sonnet = None` — no Sonnet tier present in the quota response
- **When:** `apply_model_override(&quota, paths, trace, label, name)` is called
- **Then:** Takes the conservative "sonnet tier absent" branch (`api_switch.rs:278-288`) — no `sonnet_left` is computed, no threshold comparison against `OPUS_OVERRIDE_THRESHOLD` occurs, model is written as `claude-sonnet-5` with effort `high`; this is the error/absent-data path, structurally distinct from both the sonnet→opus and opus→sonnet branches
- **Note:** This branch has no rounding-boundary hazard by construction — there is no raw value to compare or round when the tier itself is absent

### AC-4: `pct_emoji` with floating-point noise straddling the exact threshold produces consistent color for identical rounded text (BUG-331)

- **Given:** Two `util` values differing only in the 13th-14th significant decimal digit, both landing within the rounding half-interval `(4.5, 5.5)` around `WEEKLY_EXHAUSTION_THRESHOLD (5.0)`: `util_a = 94.999999999999716` (raw `left = 5.000000000000284`) and `util_b = 95.000000000000510` (raw `left = 4.999999999999488`)
- **When:** `pct_emoji(Some(util_a), 5.0)` and `pct_emoji(Some(util_b), 5.0)` are each called
- **Then:** Both return `"🟡 5%"` — `left` is rounded once (`(100.0 - u).round()`) before the comparison, collapsing both raw values to the identical rounded `left = 5.0`, which fails `5.0 > 5.0`, so both select 🟡; the raw sub-percent divergence between `util_a` and `util_b` no longer affects the color, since the same rounded value now drives both the comparison and the display text
- **Note:** BUG-331 regression — before the fix, `left` was compared raw and rounded only for display: `pct_emoji_buggy(94.999999999999716, 5.0)` returned `"🟢 5%"` while `pct_emoji_buggy(95.000000000000510, 5.0)` returned `"🟡 5%"` — identical displayed text, opposite colors, reproduced live as a 2-green/1-yellow split across three accounts with equivalent flat quota histories in the 7d-Left column. The same defect class and fix pattern applies to `apply_model_override`'s `sonnet_left` (`api_switch.rs:243-253`, now rounded once before both the branch comparison at line 253 and both trace `writeln!` calls at lines 264/278) — merged into BUG-331 as an additional instance (Step 6 Search More Instances) rather than filed separately.
