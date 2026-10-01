# Algorithm: Rounding-Boundary Classification Hazards

### Scope

- **Purpose**: Document the classification algorithms (`pct_emoji`, `apply_model_override`) that derive a color/branch decision and a rounded display string from the same raw floating-point value, and the rounding-boundary hazard this shared-input pattern creates.
- **Responsibility**: Documents `pct_emoji`'s per-column emoji/text cell computation, `apply_model_override`'s sonnet↔opus branch selection, their entry points, branch logic, and the specific raw-vs-rounded divergence identified by BUG-331.
- **In Scope**: `pct_emoji` closure logic (both call sites: 5h-Left, 7d-Left); `apply_model_override` branch selection and trace logging; the rounding-boundary hazard mechanism; the fix pattern required to close it.
- **Out of Scope**: The upstream floating-point noise source that feeds these functions (→ algorithm/006_quota_approximation.md, `quadratic_fit()`); the formal invariant these functions must satisfy (→ invariant/010_floating_point_comparison_vs_display_consistency.md); `status_emoji()`/`status_group_of()` (structurally similar threshold consumers but confirmed NOT affected — see § Related Functions Checked and Ruled Out).

### Abstract

Two functions in `src/usage/` each derive a classification decision (an emoji color, or a session-model override branch) from a raw `f64` utilization value, and separately format that identical value as a rounded percentage for on-screen or trace-log display. Neither function inserts a shared rounding step between the comparison and the format call. When the raw value lands within the rounding half-interval of the function's exact-integer threshold — which happens whenever the value originates from the polynomial-approximation noise documented in algorithm/006 — two inputs that are visually indistinguishable (identical rounded text) can select opposite classifications. This is BUG-331: three accounts with flat, equivalent quota histories displayed identical "5%" text with a 2-green/1-yellow color split in the same table.

### Algorithm

#### Entry Point 1 — `pct_emoji`

`claude_profile/src/usage/format.rs:470-478` — closure defined inside `quota_text_cells(data: &claude_quota::OauthUsageData, now_secs: u64) -> [String; 5]` (`format.rs:454`). Shown below in its current, post-BUG-331-fix form — see § Rounding-Boundary Hazard below for the defect this fix closed.

```rust
let pct_emoji = |util : Option< f64 >, threshold : f64| -> String
{
  util.map_or_else( || dash.clone(), |u|
  {
    let left  = ( 100.0 - u ).round();
    let emoji = if left > threshold { "🟢" } else { "🟡" };
    format!( "{emoji} {left:.0}%" )
  } )
};
```

Called twice, once per quota period column:

| Call site | Line | Threshold arg | Column |
|-----------|------|----------------|--------|
| `pct_emoji( data.five_hour...utilization, H_EXHAUSTED_THRESHOLD )` | `format.rs:487` | `15.0` | 5h Left |
| `pct_emoji( data.seven_day...utilization, WEEKLY_EXHAUSTION_THRESHOLD )` | `format.rs:489` | `3.0` | 7d Left |

#### `pct_emoji` Branch Logic

| Step | Expression | Precision |
|------|------------|-----------|
| 1 | `left = ( 100.0 - u ).round()` | rounded once (Fix BUG-331) |
| 2 | `emoji = if left > threshold { 🟢 } else { 🟡 }` | compares the **rounded**, step-1 `left` |
| 3 | `format!("{emoji} {left:.0}%")` | formats the **same** step-1 `left` — already rounded, `.0` is a no-op |

Rounding happens once, at step 1, and both the comparison and the display consume the identical rounded value — this is the fix (BUG-331, applied 2026-07-08) for the raw-vs-rounded split described in § Rounding-Boundary Hazard below.

#### Entry Point 2 — `apply_model_override`

`claude_profile/src/usage/api_switch.rs:289-375` — `pub fn apply_model_override(quota: &OauthUsageData, paths: &crate::ClaudePaths, trace: bool, label: &str, name: &str, backend: AccountBackend)`. Called from `usage_routine()` (`api.rs:176,372`) and from `apply_post_switch_touch()` (`api_switch.rs:406`, itself invoked by `account_use_routine()` in `commands/account_ops.rs` during `.account.use`) for the current/winning account after a successful quota fetch. Shown below in its current, post-BUG-331-fix form.

```rust
if let Some( ref sonnet ) = quota.seven_day_sonnet
{
  let sonnet_left = sonnet_left_pct( sonnet );                   // api_switch.rs:321 — ( 100.0 - u ).round() at :121-124, rounded once (Fix BUG-331)
  if sonnet_left < OPUS_OVERRIDE_THRESHOLD                       // api_switch.rs:322, compares the ROUNDED value
  {
    // sonnet→opus branch (api_switch.rs:323-340)
    // ... trace log: format!( "...sonnet→opus (7d(Son) left={sonnet_left:.0}%)..." )   // api_switch.rs:333, formats SAME rounded value
  }
  else
  {
    // opus→sonnet branch (api_switch.rs:342-353)
    // ... trace log: format!( "...opus→sonnet (7d(Son) left={sonnet_left:.0}%)..." )   // api_switch.rs:347, formats SAME rounded value
  }
}
else
{
  // sonnet tier absent — conservative "sonnet" branch (api_switch.rs:356-365), no threshold comparison
}
```

#### `apply_model_override` Branch Table

| Condition | Branch | Model write | Effort write | Trace log (when `trace::1`) |
|-----------|--------|--------------|----------------|-------------------------------|
| `seven_day_sonnet` present AND `sonnet_left < OPUS_OVERRIDE_THRESHOLD` (rounded compare, `api_switch.rs:322`) | sonnet→opus | `"opus"` shorthand (via `override_session_model_to_opus`) | `max` | `model override: sonnet→opus (7d(Son) left={sonnet_left:.0}%)` (`api_switch.rs:333`, formats same rounded value) |
| `seven_day_sonnet` present AND `sonnet_left >= OPUS_OVERRIDE_THRESHOLD` | opus→sonnet | `"sonnet"` shorthand (via `override_session_model_to_sonnet`) | `high` | `model override: opus→sonnet (7d(Son) left={sonnet_left:.0}%)` (`api_switch.rs:347`, formats same rounded value) |
| `seven_day_sonnet` absent (`None`) | conservative sonnet | `"sonnet"` shorthand (via `override_session_model_to_sonnet`) | `high` | none (no threshold comparison, not affected) |

Each model write happens only when the session model passes that function's gate (`claude_profile_core/src/account/session_settings.rs`) — e.g. the opus write skips a model that is already `"opus"`; the effort write is unconditional in every row (TSK-335).

`OPUS_OVERRIDE_THRESHOLD : f64 = 10.0` (`types.rs:567`). Both entry points now round once and reuse the rounded value for both comparison and display/trace — the BUG-331 fix (see § Rounding-Boundary Hazard : Fix pattern below).

### Rounding-Boundary Hazard (BUG-331)

Both functions above share one defect class: a classification decision and a display/log string are derived from the same `f64` value at two different precisions, with no shared rounding step between them. When the raw value falls inside the open half-interval `(threshold - 0.5, threshold + 0.5)` around the function's exact-integer threshold, the rounded display text is identical across the interval, but the classification silently flips depending on which side of the exact (unrounded) threshold the raw value happens to land on.

**Confirmed violation windows:**

| Function | Threshold | Window | Symptom |
|----------|-----------|--------|---------|
| `pct_emoji` (7d Left) | `WEEKLY_EXHAUSTION_THRESHOLD = 3.0` | `(2.5, 3.5)` | Identical "3%" text, 🟢/🟡 split across rows (same defect class as BUG-331, which originally exposed this at the prior 5.0 threshold) |
| `pct_emoji` (5h Left) | `H_EXHAUSTED_THRESHOLD = 15.0` | `(14.5, 15.5)` | Same defect class, by generalization |
| `apply_model_override` | `OPUS_OVERRIDE_THRESHOLD = 10.0` | `(9.5, 10.5)` | Same rounded trace percentage logged regardless of which override branch actually fired |

**Root cause chain** (see BUG-331 § Root Cause for the full trace):

```
approx.rs:49-76 approximate_utilization()  (dispatcher)
  → [3+ history points] approx.rs:100-176 quadratic_fit()   (algorithm/006 — source of 13th-digit float noise)
→ format.rs:435 quota_text_cells()
  → format.rs:443-451 pct_emoji( util, threshold )
      let left  = 100.0 - u;                                    // raw, full precision
      let emoji = if left > threshold { 🟢 } else { 🟡 };         // ← compares RAW left
      format!( "{emoji} {left:.0}%" )                            // ← rounds SAME left for display
→ render.rs:125 assembles cells
```

The raw-value noise is not produced by `pct_emoji` or `apply_model_override` themselves — it is inherited from `quadratic_fit()`'s least-squares arithmetic (algorithm/006), which can differ from the "true" flat value by as little as the 13th-14th significant decimal digit. Neither classification function controls for this noise before comparing against its exact-integer threshold.

**Fix pattern** (described in BUG-331 § Fix Location; fix applied 2026-07-08 — see BUG-331 for current status):

```rust
// pct_emoji, format.rs:470-478 — round once, derive both outputs from the rounded value:
let left  = ( 100.0 - u ).round();
let emoji = if left > threshold { "🟢" } else { "🟡" };
format!( "{emoji} {left:.0}%" )
```

```rust
// apply_model_override, api_switch.rs:321-322 — same pattern, applied before BOTH the
// branch comparison (line 322) and both trace writeln! calls (lines 333, 347):
let sonnet_left = sonnet_left_pct( sonnet ); // ( 100.0 - sonnet.utilization ).round()
if sonnet_left < OPUS_OVERRIDE_THRESHOLD { /* ... */ } else { /* ... */ }
```

Increasing display precision (e.g., one decimal place) does NOT resolve the hazard — BUG-331's incident-level divergence was 13 decimal places deep, far below any reasonable display precision. Rounding once and reusing the rounded value for both the comparison and the display is the only verified remediation.

**Governing invariant:** invariant/010_floating_point_comparison_vs_display_consistency.md states the formal constraint both functions must satisfy: identical rounded display text must imply identical classification.

#### Related Functions Checked and Ruled Out

Per BUG-331 § History (Step 6 — Search More Instances), the following threshold-consuming functions were audited and confirmed to NOT share this defect class:

| Function | Location | Why not affected |
|----------|----------|-------------------|
| `status_emoji()` | `format.rs:483-510` | Compares raw `h5_left`/`d7_left` against thresholds but does not separately format/round either value for display in the same function — no raw-vs-rounded split present |
| `recommended_model()` | `format.rs:415-426` | Compares raw utilization against `OPUS_OVERRIDE_THRESHOLD` but produces only a `&'static str` model name, never a rounded numeric display of the compared value |
| `sort.rs:57`, `sort_next.rs:67/80/85/90` | — | Gate account-rotation eligibility on the same raw threshold comparison — internally self-consistent (no display/log of the compared value in the same function), flagged as a related but unconfirmed risk in BUG-331 § Impact, not a second instance of this defect |
| `render_tsv.rs` `pct_bare` (`render_tsv.rs:95-98`) | — | `format!("{:.0}%", 100.0 - u)` — pure display formatting, no threshold comparison or branch logic at all |
| `render_json.rs` | — | Emits raw numeric fields only; never calls `pct_emoji` or performs threshold classification |

### Features

| File | Relationship |
|------|--------------|
| [feature/009_token_usage.md](../feature/009_token_usage.md) | AC-32 documents `apply_model_override()`'s branch behavior and trace output; the 5h Left / 7d Left table cells produced by `pct_emoji` are the display surface for this algorithm |

### Sources

| File | Relationship |
|------|--------------|
| `src/usage/format.rs:470-478` | `pct_emoji` closure — both call sites (`format.rs:487,489`) |
| `src/usage/api_switch.rs:289-375` | `apply_model_override` — branch selection (`321-322`, rounding in `sonnet_left_pct` at `121-124`) and trace logging (`333,347`) |
| `src/usage/types.rs:567,576,582` | `OPUS_OVERRIDE_THRESHOLD`, `H_EXHAUSTED_THRESHOLD`, `WEEKLY_EXHAUSTION_THRESHOLD` constant definitions |
| `src/usage/approx.rs:100-176` | `quadratic_fit()` — upstream source of the floating-point noise that triggers this hazard (see algorithm/006) |

### Invariants

| File | Relationship |
|------|--------------|
| [invariant/010_floating_point_comparison_vs_display_consistency.md](../invariant/010_floating_point_comparison_vs_display_consistency.md) | Formal invariant both `pct_emoji` and `apply_model_override` must satisfy; satisfied by both functions since the BUG-331 fix (2026-07-08) — previously violated, see § Rounding-Boundary Hazard : Fix pattern |

### Algorithms

| File | Relationship |
|------|--------------|
| [006_quota_approximation.md](006_quota_approximation.md) | Source of the raw-value floating-point noise (`quadratic_fit()`) that lands in this algorithm's rounding-boundary violation windows |

### Tests

| File | Relationship |
|------|--------------|
| `tests/usage/format_tests.rs:502-526` | `test_ft11_009_per_column_emoji_prefix_three_cases` — covers the exact-integer-boundary case (`util=85.0` → `left=15.0` exactly) for `pct_emoji`; does not cover near-boundary floating-point-noise cases (the gap BUG-331 exposes). The near-boundary regression case described in BUG-331 § Prevention is implemented as `mre_bug331_pct_emoji_color_matches_rounded_display_at_threshold_boundary` (`tests/usage/format_tests.rs:563`). |
