# Test: Invariant 010 — Floating-Point Comparison vs. Display Consistency

Property assertion cases for `docs/invariant/010_floating_point_comparison_vs_display_consistency.md`.
Verifies that classification/branch decisions derived from a raw `f64` never diverge from what the
rounded display of that same value implies, at the two confirmed call sites (`pct_emoji`,
`apply_model_override`).

### Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| IN-1 | Two raw inputs rounding to the same displayed percentage produce the same `pct_emoji` classification | Invariant holds (normal) |
| IN-2 | Raw input inside the `(14.5, 15.5)` violation window around `H_EXHAUSTED_THRESHOLD` still classifies consistently with its rounded display | Invariant holds (boundary) |

**Total:** 2 IN cases

---

### IN-1: Two raw inputs rounding to the same displayed percentage produce the same `pct_emoji` classification

- **Given:** Two raw `f64` values, `util` and `util'`, both in `[0.0, 100.0]`, chosen so that
  `round(f(util)) == round(f(util'))` for `pct_emoji`'s value-deriving expression `f(u) = 100.0 - u`
  (e.g., `util = 20.2` and `util' = 20.4`, both yielding `left = 79.8` and `79.6`, which format to
  the same `{left:.0}%` text `"80%"`)
- **When:** `pct_emoji` (`src/usage/format.rs:443-451`) is called once with `util` and once with
  `util'`
- **Then:** Both calls produce the identical emoji (`🟢` or `🟡`) — the classification never
  diverges for two inputs that display identically, per the formal statement
  `round(f(util)) == round(f(util')) ⟹ classify(f(util)) == classify(f(util'))`
- **Source:** [docs/invariant/010_floating_point_comparison_vs_display_consistency.md](../../../docs/invariant/010_floating_point_comparison_vs_display_consistency.md)

---

### IN-2: Raw input inside the `(14.5, 15.5)` violation window around `H_EXHAUSTED_THRESHOLD` still classifies consistently with its rounded display

- **Given:** A raw `f64` input to `pct_emoji`'s 5h-Left column (`format.rs:460`) whose derived
  `left` value falls strictly inside the documented violation window `(14.5, 15.5)` around the
  `H_EXHAUSTED_THRESHOLD` constant `15.0` (`types.rs:408`) — e.g., `left = 14.999999999999716`
  (rounds to display `"15%"`, one ULP below the threshold) and `left = 15.000000000000510`
  (rounds to display `"15%"`, one ULP above the threshold)
- **When:** `pct_emoji` is called with both boundary-adjacent raw values, each of which rounds to
  the identical displayed text `"15%"`
- **Then:** Both calls select the same emoji branch — the classification is computed from (or
  provably consistent with) the rounded value `15`, not from the raw pre-rounding value that sits
  on opposite sides of the exact-integer threshold; a violation would manifest as one call
  producing `🟢` and the other `🟡` despite both displaying `"15%"`
- **Source:** [docs/invariant/010_floating_point_comparison_vs_display_consistency.md](../../../docs/invariant/010_floating_point_comparison_vs_display_consistency.md)
