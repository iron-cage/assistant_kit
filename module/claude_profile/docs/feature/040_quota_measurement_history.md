# Feature: Quota Measurement History and Polynomial Approximation

### Scope

- **Purpose**: Store up to 10 timestamped real server measurements per account and use quadratic least-squares regression to approximate current quota levels when the API is unavailable.
- **Responsibility**: Documents the measurement history ring buffer in `{name}.json` cache, the approximation algorithm, reset boundary handling, and display integration.
- **In Scope**: History storage format (`cache.history[]`), write-on-success append, polynomial degree selection (0/1/2), quadratic LS solver (3x3 Cramer), time normalization, reset boundary detection and filtering, value clamping [0,100], extrapolation safety (tangent-line continuation), singular matrix fallback, backward compatibility with old cache format, duplicate-timestamp deduplication.
- **Out of Scope**: Approximated values stored in history (only real server values), cache invalidation by time, third-party math dependencies (invariant 001), higher-order polynomials (>2), separate approximation display indicator (reuses `~` from Feature 033).

### Design

**Measurement History Ring Buffer**

Each successful `fetch_oauth_usage` call appends a timestamped measurement to a `"history"` array in the `"cache"` object of `{name}.json`. The array is capped at 10 entries (FIFO — oldest evicted when full). Only real server-returned values are stored; approximated values are never persisted.

**Storage format** (extends the existing `"cache"` object from Feature 033):

```json
{
  "cache": {
    "fetched_at": "2026-06-21T12:00:00Z",
    "status": "ok",
    "five_hour": { "utilization": 42.0, "resets_at": "2026-06-21T14:00:00+00:00" },
    "seven_day": { "utilization": 35.0, "resets_at": "2026-06-25T00:00:00+00:00" },
    "seven_day_sonnet": { "utilization": 20.0, "resets_at": "2026-06-25T00:00:00+00:00" },
    "model_override": "opus",
    "last_touch_at": "2026-06-21T06:30:00Z",
    "touch_idle": true,
    "history": [
      { "t": 1750520000, "h5": [38.0, "2026-06-21T14:00:00+00:00"], "d7": [33.0, "2026-06-25T00:00:00+00:00"], "sn": [18.0, "2026-06-25T00:00:00+00:00"] },
      { "t": 1750521800, "h5": [42.0, "2026-06-21T14:00:00+00:00"], "d7": [35.0, "2026-06-25T00:00:00+00:00"], "sn": [20.0, "2026-06-25T00:00:00+00:00"] }
    ]
  }
}
```

Fields per entry:

| Field | Type | Meaning |
|-------|------|---------|
| `t` | integer | Unix timestamp (seconds) when the measurement was taken |
| `h5` | `[f64, string]` or `null` | 5h period: `[utilization, resets_at]`; `null` when period absent |
| `d7` | `[f64, string]` or `null` | 7d period: `[utilization, resets_at]` |
| `sn` | `[f64, string]` or `null` | 7d-sonnet period: `[utilization, resets_at]` |

Short keys (`h5`, `d7`, `sn`) minimize JSON size in a 10-entry array.

**Approximation Algorithm**

When the server is unavailable and history measurements exist, `approximate_utilization()` fits a polynomial to the stored measurements for each period independently:

| Measurements (post-filter) | Degree | Method |
|---|---|---|
| 0 | — | `None` — no data |
| 1 | 0 (constant) | Return last value |
| 2 | 1 (linear) | Extrapolate from slope between 2 points |
| 3–10 | 2 (quadratic) | Least-squares regression via 3x3 Cramer |

**Quadratic Least-Squares**: Fit `y = a2*t^2 + a1*t + a0` by solving the normal equations:

```
| S4  S3  S2 | | a2 |   | T2 |
| S3  S2  S1 | | a1 | = | T1 |
| S2  S1  S0 | | a0 |   | T0 |

where Sk = sum(ti^k), Tk = sum(ti^k * yi)
```

Solved via Cramer's rule (3x3 determinant). No external dependencies.

**Time Normalization**: Subtract `t_values[0]` from all timestamps before computing sums. This maps Unix timestamps (~1.75e9) to small positive offsets, preventing f64 precision loss in power sums.

**Reset Boundary Detection**: Before fitting, filter out measurements from before the current window:

| Period | Window duration | Window start |
|--------|-----------------|-------------|
| 5h | 18000s | `latest_resets_at - 18000` |
| 7d / 7d-sonnet | 604800s | `latest_resets_at - 604800` |

Measurements with `t < window_start` belong to a previous window and are discarded. If `resets_at` has elapsed (`now_secs > resets_at_secs`), utilization is `0.0` (window reset — no active session).

**Extrapolation Safety**: When `t_now` exceeds 2x the measurement span beyond the last measurement, the polynomial is evaluated at `t_max` and the tangent line at `t_max` is used for linear continuation:

```
slope_at_tmax = 2 * a2 * t_max + a1
y = y(t_max) + slope_at_tmax * (t_now - t_max)
```

This prevents quadratic divergence (a2 > 0 shooting to infinity).

**Value Clamping**: All approximated values are clamped to `[0.0, 100.0]`.

**Singular Matrix Fallback**: If `|det| < 1e-12` during Cramer's rule, fall back to linear (degree 1). If linear is also degenerate (identical timestamps), return the last measurement value.

**Display Integration**: Approximated values reuse the existing `~` prefix and `cached` flag from Feature 033. The `cache_age_secs` is computed from the most recent measurement's timestamp, consistent with the current single-point behavior.

**Duplicate Timestamp Handling**: If a new measurement has the same `t` (Unix second) as the last entry in `history[]`, the existing entry is overwritten instead of appended. This prevents fast-cycle callers from filling the buffer with near-identical measurements.

### Acceptance Criteria

- **AC-01**: On successful `fetch_oauth_usage`, the measurement is appended to `cache.history[]` in `{name}.json` with `t` (current Unix seconds), `h5`, `d7`, `sn` fields matching the fetched quota data.
- **AC-02**: `cache.history[]` contains at most 10 entries; when the 11th is appended, the oldest (index 0) is evicted (FIFO).
- **AC-03**: Only real server-returned values are stored in `cache.history[]` — approximated values, cached fallback values, and error results are never appended.
- **AC-04**: When the server is unavailable (transient error) and `cache.history[]` has 3+ measurements in the current window, quota columns display quadratic-LS-approximated values with `~` prefix.
- **AC-05**: Each period (5h, 7d, 7d-sonnet) is approximated independently — absence of one period does not affect the others.
- **AC-06**: Measurements from a previous window (before `window_start = latest_resets_at - window_duration`) are excluded from the polynomial fit.
- **AC-07**: If `resets_at` has elapsed (`now_secs > resets_at_secs`), approximated utilization is `0.0` (window reset).
- **AC-08**: Approximated values are clamped to `[0.0, 100.0]` — polynomial extrapolation never produces out-of-range values.
- **AC-09**: When `t_now` exceeds 2x the measurement span beyond the last measurement, extrapolation uses the tangent line at the last measurement instead of the raw polynomial.
- **AC-10**: When the 3x3 normal equation matrix is near-singular (`|det| < 1e-12`), the algorithm falls back to linear extrapolation (degree 1); if linear is also degenerate, returns the last measurement value.
- **AC-11**: Old cache format (no `"history"` key) is backward-compatible: treated as 0 measurements — approximation not available, current single-point fallback behavior preserved.
- **AC-12**: Non-owned accounts do not append to history (only real server fetches qualify; non-owned accounts use cache-only path from Feature 036 G1 gate).
- **AC-13**: Duplicate timestamps (same Unix second) overwrite the existing entry in `history[]` instead of appending.

### Features

| File | Relationship |
|------|-------------|
| [033_quota_cache.md](033_quota_cache.md) | Extends single-point cache with multi-point history; uses same cache object and display path |
| [009_token_usage.md](009_token_usage.md) | Approximated values displayed in `.usage` table |
| [036_account_ownership.md](036_account_ownership.md) | Non-owned accounts skip history append (G1 gate) |
| [039_decision_algorithms.md](039_decision_algorithms.md) | Approximation algorithm documented as Table 6 |

### Sources

| File | Relationship |
|------|-------------|
| `src/usage/approx.rs` | Approximation algorithm — polynomial fit, reset filtering, clamping, tangent-line continuation |
| `src/usage/fetch.rs` | History append on success; approximation call on cache-fallback path |
| `claude_profile_core/src/account.rs` | Storage layer — `write_history_entry()`, `read_history()`, ring buffer management |
