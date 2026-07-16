# Invariant: Floating-Point Comparison vs. Display Consistency

### Scope

- **Purpose**: Guarantee that any classification/branch decision derived from a floating-point value never diverges from what the rounded display of that same value implies.
- **Responsibility**: Documents the raw-vs-rounded consistency constraint for all closures/functions that both (a) compare a floating-point value against a threshold to select a classification, color, or branch, and (b) separately format/round that same value for display or logging.
- **In Scope**: Any code path computing a classification, emoji/color selection, or branch decision from a raw `f64` AND separately rounding the identical value for display text or trace/log output, for all workspace crates. Confirmed instances: `pct_emoji` (`src/usage/format.rs:443-451`), `apply_model_override` (`src/usage/api_switch.rs:244-270`).
- **Out of Scope**: The upstream floating-point noise source itself (→ algorithm/006_quota_approximation.md, `quadratic_fit()`); the classification algorithms' branch tables and entry points (→ algorithm/011_rounding_boundary_classification_hazards.md); JSON/TSV output paths that emit raw numeric fields or recompute bare percentages independently without any threshold comparison (`render_json.rs`, `render_tsv.rs`'s `pct_bare` — not affected, no branch logic present).

### Invariant Statement

For any two raw `f64` inputs to the same classification function, if both inputs round to the identical displayed percentage text, the function MUST also produce the identical classification (color/emoji/branch) for both. Equivalently: classification MUST be computed from the rounded value, or otherwise be provably guaranteed consistent with the rounded value that is displayed — never computed from the raw, pre-rounding value while display text is computed from the rounded value in the same code path.

**Formal statement:**

```
∀ util, util' : f64 in the function's valid input domain (typically [0.0, 100.0]):
  round( f( util ) ) == round( f( util' ) )  ⟹  classify( f( util ) ) == classify( f( util' ) )
```

Where `f()` is the value-deriving expression (e.g., `100.0 - u`), `round()` is the display rounding applied for the format string (e.g., `{:.0}`), and `classify()` is the branch/color/model-selection decision made from that same value.

**Measurable threshold:** Zero occurrences of a classification function producing two different `classify()` outputs for two inputs that produce textually identical `round()` output, across the function's full valid input domain. Concretely for the two confirmed instances:

- `pct_emoji`: for all `util : f64` in `[0.0, 100.0]`, if two calls produce the same `{left:.0}%` text, they MUST produce the same emoji (`🟢` or `🟡`).
- `apply_model_override`: for all `sonnet_left : f64` in `[0.0, 100.0]`, if two calls would log the same `{sonnet_left:.0}%` text, they MUST select the same override branch (sonnet→opus vs. opus→sonnet).

**Violation window:** The invariant is violated whenever a raw value falls within the half-interval `(threshold - 0.5, threshold + 0.5)` around an exact-integer classification threshold, AND the classification comparison uses the raw (unrounded) value instead of the rounded value. Confirmed violation windows in this codebase (BUG-331):

| Threshold | Constant | Violation window | Call site |
|-----------|----------|-------------------|-----------|
| `5.0` | `WEEKLY_EXHAUSTION_THRESHOLD` (`types.rs:414`) | `(4.5, 5.5)` | `pct_emoji`, 7d-Left column (`format.rs:462`) |
| `15.0` | `H_EXHAUSTED_THRESHOLD` (`types.rs:408`) | `(14.5, 15.5)` | `pct_emoji`, 5h-Left column (`format.rs:460`) |
| `10.0` | `OPUS_OVERRIDE_THRESHOLD` (`types.rs:399`) | `(9.5, 10.5)` | `apply_model_override` (`api_switch.rs:245`) |

### Detection Signal

A code path is a candidate violation of this invariant when the following pattern is present:

```rust
let left = <raw f64 expression>;                 // no rounding applied
let decision = if left > threshold { A } else { B };   // ← compares RAW value
format!( "{decision} {left:.0}%" )                     // ← rounds SAME value for display
```

The defect is the absence of a shared rounding step between the comparison and the format call — both derive their output from `left`, but at two different precisions, with no intermediate `.round()` linking them.

**Detection command/pattern:** grep for `if <raw_float_var> > threshold { ... } else { ... }` (or equivalent `<`/`>=`/`<=` comparisons against a named threshold constant) immediately followed by `{<same_float_var>:.N}` formatting on the same variable, with no `.round()` call appearing between the comparison and the format expression.

### Enforcement

**Current status:** Enforced in code since 2026-07-08 — BUG-331 (`task/claude_profile/bug/331_pct_raw_vs_rounded_mismatch.md`, state 🎯 Verified, fix applied 2026-07-08) documented this invariant's violation at both confirmed call sites; the fix (round once, derive both `classify()` and the display string from the single rounded value) is now live at both sites (see algorithm/011 § Rounding-Boundary Hazard for the applied fix).

**Required fix pattern (once applied):**

```rust
let left  = ( 100.0 - u ).round();               // round ONCE
let emoji = if left > threshold { "🟢" } else { "🟡" };  // compare the ROUNDED value
format!( "{emoji} {left:.0}%" )                    // display the SAME rounded value
```

**Detection layer:** No automated lint currently exists for this pattern in this codebase. Prevention is currently procedural: any new closure/function that both branches on and formats the same floating-point value must be reviewed against this invariant before merge (see algorithm/011 § Rounding-Boundary Hazard : Prevention for the full sweep recommendation).

**Test coverage:** `tests/usage/format_tests.rs:502-526` (`test_ft11_009_per_column_emoji_prefix_three_cases`) covers the exact-integer-boundary case (`util=85.0` → `left=15.0` exactly) but does not cover values merely near a boundary without landing on it exactly — this is the specific gap BUG-331 exposes. The focused regression case from BUG-331's Minimum Reproducible Example (`util=94.999999999999716` vs. `util=95.000000000000510`) is implemented as `mre_bug331_pct_emoji_color_matches_rounded_display_at_threshold_boundary` (`tests/usage/format_tests.rs:563`); the broader property-style test asserting the formal statement above remains a recommended addition.

### Related Invariants

| File | Relationship |
|------|--------------|
| [009_container_only_test_execution.md](009_container_only_test_execution.md) | Structural peer — both invariants govern correctness constraints enforced across the same `src/usage/` call sites |

### Sources

| File | Relationship |
|------|--------------|
| `src/usage/format.rs:443-451` | `pct_emoji` closure — confirmed violation site (color from raw `left`, display from rounded `left`) |
| `src/usage/api_switch.rs:244-270` | `apply_model_override` — confirmed violation site (branch from raw `sonnet_left`, trace log from rounded `sonnet_left`) |
| `src/usage/types.rs:399,408,414` | `OPUS_OVERRIDE_THRESHOLD`, `H_EXHAUSTED_THRESHOLD`, `WEEKLY_EXHAUSTION_THRESHOLD` — the three exact-integer thresholds whose half-intervals define the violation windows |

### Tests

| File | Relationship |
|------|--------------|
| `tests/usage/format_tests.rs` | `test_ft11_009_per_column_emoji_prefix_three_cases` covers the exact-boundary case only; the near-boundary regression test described in BUG-331 § Prevention is implemented as `mre_bug331_pct_emoji_color_matches_rounded_display_at_threshold_boundary` (line 563); the broader property-style test remains a recommended addition |
