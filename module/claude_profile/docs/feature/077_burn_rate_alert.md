# Feature: Burn-Rate Alert (Time-to-Exhaustion)

### Scope

- **Responsibility**: Forecast each account's 5h-window time-to-exhaustion from its quota measurement ring (Feature 040) and warn under the `.usage` table when any account is predicted to exhaust within a configurable horizon — so a fast burn (e.g. a parallel agent fleet) is visible *before* the window hits 100%, not only as retry errors after the fact.
- **In Scope**: in-window sample selection from the history ring; recent-slope time-to-exhaustion estimate; `⚠ 5h burn` footer warning lines on `format::text`/`format::plain`; `alert::` horizon parameter (task 544).
- **Out of Scope**: 7d-window forecasting (5h is the operational bottleneck); push/exec notification channels (the watchdog layer consumes `.usage` output); history ring capture itself (Feature 040); live-mode (`live::1`) per-cycle alerting.

### Design

Motivating incident (2026-08-20): a review fleet burned one account's 5h window 0→100% in ≤31 minutes; exhaustion surfaced only as downstream retry errors. The ring already held everything needed to predict it.

**Sample selection** (`h5_in_window_samples`): the anchor is the newest ring entry carrying a 5h measurement with a parseable `resets_at`. Entries qualify when their `resets_at` is within ±300s of the anchor's (`WINDOW_IDENTITY_TOLERANCE_S` — same-window `resets_at` jitters by ~1s between fetches, so equality fails) and their timestamp is within the anchor's 18000s window span. An anchor already in the past (window elapsed) yields no samples — utilization resets at the boundary, so there is nothing to forecast.

**Estimate** (`time_to_exhaustion`):

- **Confidence gate**: fewer than 3 in-window samples → no estimate (one watchdog tick is noise, not a trend).
- **Recent slope, not whole-window regression**: the slope comes from the last two samples. The captured incident ring held ~2h of in-window flat zeros before the 31-minute ramp — a whole-window least-squares fit averages the idle prefix in and would have alerted only after exhaustion.
- **Prediction only**: extrapolated utilization ≥ 100 → no estimate. Actual exhaustion is a fact shown on the row; this also self-suppresses stale rings (extrapolation past a dead account's last climb quickly exceeds 100).
- Non-positive slope → no estimate; result carries `tte_secs` and `rate_pct_per_min`.

**Surface** (`burn_warnings`): after the quota table, one line per sub-horizon account — `⚠ 5h burn · {name} · ~{duration} to exhaustion (≈{rate:.1}%/min)`. Every number is marked as an estimate (`~` duration, `≈` rate). Rendered for `format::text` and `format::plain` only (`no_color` strips `⚠` to `!`); `json`/`tsv`/`value` output stays byte-stable for scripts. Horizon: `alert::` minutes, default 15, `0` disables.

### Acceptance Criteria

- **AC-01**: Samples from a previous 5h window (different `resets_at` beyond tolerance, or older than the window span) never enter the slope; a rollover mid-ring cannot produce a spurious estimate.
- **AC-02**: Replaying the captured 2026-08-20 burn at the watchdog's ~3-minute cadence produces the warning line at an intermediate sample — before the window reaches 100% — under the default 15-minute horizon.
- **AC-03**: Idle accounts (flat ring), rings with <3 in-window samples, negative slopes, and `alert::0` produce no warning line; `json`/`tsv`/`value` output never contains one.
- **AC-04**: Warning lines label all numbers as estimates and identify the account by name.

### Parameters

| # | Parameter | Role |
|---|-----------|------|
| 1 | [alert::](../cli/param/088_alert.md) | Horizon in minutes (default 15; `0` = off) |

### Commands

| # | Command | Role |
|---|---------|------|
| 1 | [`.usage`](../cli/command/006_usage.md#command-9-usage) | Warning lines rendered under the quota table |

### Features

| File | Relationship |
|------|--------------|
| [040_quota_measurement_history.md](040_quota_measurement_history.md) | Data source — the per-account measurement ring this feature forecasts from |
| [009_token_usage.md](009_token_usage.md) | Host surface — `.usage` table whose footer carries the warning lines |

### Sources

| File | Relationship |
|------|--------------|
| `src/usage/forecast.rs` | Sample selection, time-to-exhaustion estimate, warning-line builder |
| `src/usage/api.rs` | Computes warnings for `Text`/`Plain` and appends them after the rendered table |
| `src/usage/params.rs` | `alert::` parsing (non-negative integer minutes; default 15) |
| `claude_profile_core/src/account/history.rs` | `read_history()` — ring read the forecast consumes |

### Tests

| File | Relationship |
|------|--------------|
| `tests/usage/forecast_tests.rs` | FC-01–FC-09: captured-burn replay (mid-burn alert), rollover discard, flat/short/negative/elapsed/exhausted suppression, `burn_warnings` rendering, threshold boundary, `alert::0` disable |
