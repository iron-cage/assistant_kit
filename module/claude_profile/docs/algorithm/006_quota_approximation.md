# Algorithm: Quota Polynomial Approximation

### Scope

- **Purpose**: Define the quota utilization approximation algorithm for offline estimation when the API is unavailable.
- **Responsibility**: Documents the polynomial regression method, pre-fit filter, Cramer 3×3 solver, and post-fit rules for `approximate_utilization()`.
- **In Scope**: `approximate_utilization()` logic; degree selection by measurement count; Cramer solver; tangent continuation; time normalization.
- **Out of Scope**: Measurement storage format (→ schema/002); cache fallback trigger (→ feature/033).

### Abstract

Estimate quota utilization percentages when the server is unavailable (rate-limited, timeout, network error) using historical measurements stored in `{name}.json`.

### Algorithm

#### Entry Point

`src/usage/approx.rs` — `approximate_utilization(period, history, now_secs)`

#### Decision Table (by measurement count after filtering)

| Post-filter measurements | Degree | Method | Fallback |
|---|---|---|---|
| 0 | — | `None` (no data) | — |
| 1 | 0 | Constant (last value) | — |
| 2 | 1 | Linear extrapolation (LS) | — |
| 3–10 | 2 | Quadratic LS (Cramer 3×3) | linear if Cramer singular |

#### Pre-fit Filter

Discard measurements before `window_start`:
```
window_start = resets_at - window_duration
window_duration: 18000s for 5h period, 604800s for 7d and 7d_sonnet periods
```

If `now > resets_at` → return `0.0` (window has expired; quota has reset).

#### Cramer 3×3 Solver

Fits quadratic `u = a0 + a1*t + a2*t^2` via normal equations. Uses Cramer's rule on the 3×3 system formed by power sums `s0, s1, s2, s3, s4` and cross-products `r0, r1, r2`.

**Bug BUG-307 (Fix 2026-06-22):** Cofactor `det0` must use `s2*r1` (not `s1*r2`) — col-3 replacement minor uses RHS element `r1`, not power-sum `r2`. Wrong formula clamped linear data to 100.0 (masked by broad test ranges).

#### Post-fit Rules

- **Clamp:** Result clamped to `[0.0, 100.0]`.
- **Tangent continuation:** If extrapolation beyond 2× measurement span, evaluate derivative at `t_max` and extend linearly (prevents explosive extrapolation from quadratic curves).
- **Time normalization:** Subtract `t_values[0]` before computing power sums to avoid f64 precision loss on large Unix timestamps.

#### Window Durations

| Period | Duration |
|--------|----------|
| `five_hour` | 18 000 s |
| `seven_day` | 604 800 s |
| `seven_day_sonnet` | 604 800 s |

#### Storage

Up to 10 measurements per account stored in `{name}.json → history` array. New measurements appended; oldest discarded when count exceeds 10. See [schema/002](../schema/002_account_json.md).

#### Downstream Rounding-Boundary Hazard (BUG-331)

The raw `f64` percentage this algorithm produces — particularly via `quadratic_fit()`'s least-squares arithmetic, which can differ from the "true" flat value by as little as the 13th-14th significant decimal digit — feeds directly into downstream classification logic (`pct_emoji`, `apply_model_override`) that compares the raw value against an exact-integer threshold while separately rounding that same value for display or trace logging. When the raw noise lands within the rounding half-interval of such a threshold, two accounts with visually identical rounded percentages can receive different classifications (colors, override branches). This algorithm is not itself defective — the noise is an expected consequence of least-squares fitting — but it is the confirmed noise source for BUG-331. See [011_rounding_boundary_classification_hazards.md](011_rounding_boundary_classification_hazards.md) for the affected downstream functions, and [invariant/010_floating_point_comparison_vs_display_consistency.md](../invariant/010_floating_point_comparison_vs_display_consistency.md) for the formal consistency invariant they must satisfy.

### Features

| File | Relationship |
|------|-------------|
| [feature/040_quota_measurement_history.md](../feature/040_quota_measurement_history.md) | Storage format, measurement collection |
| [feature/033_quota_cache.md](../feature/033_quota_cache.md) | Cache fallback that triggers approximation |
| [feature/061_solo_token_conservation.md](../feature/061_solo_token_conservation.md) | `approximate_quota()` in `fetch.rs` calls this for non-current accounts |
| [feature/039_decision_algorithms.md](../feature/039_decision_algorithms.md) | Table 6 (legacy reference) |

### Schema

| File | Relationship |
|------|-------------|
| [schema/002](../schema/002_account_json.md) | `history` field schema |
