# State Machine: Quota Measurement Lifecycle

### Scope

- **Purpose**: Define the lifecycle states of the quota measurement ring buffer and approximation quality progression.
- **Responsibility**: Documents `empty`/`single`/`linear`/`quadratic`/`full` states and pre-fit filter behavior.
- **In Scope**: Ring buffer fill states; measurement append conditions; pre-fit filter and degree reduction at approximation time.
- **Out of Scope**: Approximation algorithm implementation (→ algorithm/006); cache fallback trigger (→ feature/033).

### Abstract

Track how the history ring buffer fills over time and how that enables progressively better quota approximation.

### States

| State | History entries | Approximation quality | Method |
|-------|----------------|----------------------|--------|
| `empty` | 0 | None (returns `None`) | — |
| `single` | 1 | Constant (last value) | Degree 0 |
| `linear` | 2 | Linear extrapolation | Degree 1 LS |
| `quadratic` | 3–10 | Polynomial fit | Degree 2 LS (Cramer 3×3) |
| `full` | 10 (ring full) | Same as quadratic | Oldest discarded on next write |

### Transitions

```
[empty]     --successful fetch_oauth_usage()--> [single]
[single]    --successful fetch_oauth_usage()--> [linear]
[linear]    --successful fetch_oauth_usage()--> [quadratic]
[quadratic] --successful fetch_oauth_usage()--> [quadratic]  (up to 9)
[quadratic] --successful fetch_oauth_usage()--> [full]       (at 10)
[full]      --successful fetch_oauth_usage()--> [full]       (ring: oldest dropped, newest appended)
```

Measurements are appended only on **successful** `fetch_oauth_usage()` calls (not on cache fallback or error). Each measurement stores: `ts`, `five_hour`, `seven_day`, `seven_day_sonnet`, `five_h_resets_at`, `seven_d_resets_at`.

### Pre-fit Filter (at approximation time)

Before fitting, measurements before `window_start = resets_at - window_duration` are discarded. A reset boundary in the history (measurement taken before the last `resets_at`) causes those older points to be excluded, potentially reducing effective degree:

```
after filter:  0 points → None
               1 point  → degree 0
               2 points → degree 1
               3+ points→ degree 2
```

### Ring Buffer Properties

- Maximum 10 entries per period per account
- Written to `{name}.json → history` array — see [schema/002](../schema/002_account_json.md)
- `write_quota_cache()` in `account.rs` preserves the `history` key when updating `_quota_cache`

### Behavioral Invariants

- Measurements are appended only on successful `fetch_oauth_usage()` calls — cache fallback and errors produce no measurement.
- The ring buffer caps at 10 entries; the oldest measurement is discarded when a new one would exceed the cap.
- Pre-fit filtering at approximation time may reduce the effective degree below what the buffer count implies.

### Features

| File | Relationship |
|------|-------------|
| [feature/040_quota_measurement_history.md](../feature/040_quota_measurement_history.md) | Storage and collection feature spec |
| [feature/033_quota_cache.md](../feature/033_quota_cache.md) | Cache fallback that triggers approximation |

### Algorithms

| File | Relationship |
|------|-------------|
| [algorithm/006](../algorithm/006_quota_approximation.md) | Approximation algorithm |

### Schema

| File | Relationship |
|------|-------------|
| [schema/002](../schema/002_account_json.md) | `history` field in `{name}.json` |
