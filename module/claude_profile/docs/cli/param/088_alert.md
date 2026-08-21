# Parameter: 88. `alert::`

### Scope

- **Purpose**: Set the burn-rate alert horizon for `.usage` — when any account's 5h window is forecast (Feature 077) to exhaust within N minutes, a `⚠ 5h burn` warning line is rendered under the quota table.
- **Responsibility**: Documents the `alert::` parameter accepted by `.usage`.

### Design

After the quota table renders, each account's measurement ring (Feature 040) is checked for an in-window burn trend. An account whose extrapolated time-to-exhaustion falls under the horizon produces one warning line:

```
⚠ 5h burn · mykola.nn@wbox.pro · ~13m to exhaustion (≈3.2%/min)
```

Every number is labeled as an estimate (`~` duration, `≈` rate). Lines appear on `format::text` and `format::plain` only (`no_color::1` strips `⚠` to `!`); `json`/`tsv`/`value` output is never modified, so scripted consumers stay byte-stable.

| Value | Behavior |
|-------|----------|
| *(omit)* | Default horizon: 15 minutes |
| `N ≥ 1` | Warn when exhaustion is forecast within N minutes |
| `0` | Alerts disabled — no forecast computed |

Accounts with an idle (flat) ring, fewer than 3 same-window samples, a shrinking utilization, or an already-exhausted window never produce a line — the forecast is prediction-only (see [feature/077](../../feature/077_burn_rate_alert.md) § Design).

### Specification

| Attribute | Value |
|-----------|-------|
| Parameter | `alert::` |
| Type | `u64` (minutes) |
| Default | `15` |
| Valid Values | `0` (off), any non-negative integer |
| Commands | `.usage` |
| Pipeline Stage | Display |
| Group | [Display Control](../param_group/005_display_control.md) |

### Validation

| Input | Result |
|-------|--------|
| Negative or non-integer | Exit 1 — `alert:: must be a non-negative integer (minutes)` |

### Acceptance Criteria

- **AC-01**: `alert::0` suppresses all burn warnings regardless of ring state.
- **AC-02**: With `alert::` omitted, an account forecast to exhaust within 15 minutes produces a warning line; one forecast beyond 15 minutes does not.
- **AC-03**: Warning lines never appear in `format::json`, `format::tsv`, or `get::` value output.

### Examples

```bash
clp .usage                       # default: warn when exhaustion forecast within 15m
clp .usage alert::60             # widen the horizon to 1 hour
clp .usage alert::0              # disable burn warnings
clp .usage alert::15 no_color::1 # warning marker rendered as "!" instead of "⚠"
```

### Referenced Type

- **Fundamental Type:** `u64`

### Referenced Parameter Groups

| # | Parameter Group | Role |
|---|-----------------|------|
| 1 | [Display Control](../param_group/005_display_control.md) | Member parameter |

### Referenced Commands

| # | Command | Role |
|---|---------|------|
| 1 | [`.usage`](../command/006_usage.md#command-9-usage) | Burn-rate warning horizon for the quota table footer |

### Referenced User Stories

| # | User Story | Persona |
|---|------------|---------|
| 1 | [Multi-Account Quota Monitoring](../user_story/003_quota_monitoring.md) | Early warning before a fast burn exhausts a 5h window |

### Referenced Features

| File | Relationship |
|------|--------------|
| [feature/077_burn_rate_alert.md](../../feature/077_burn_rate_alert.md) | Forecast algorithm, warning-line format, suppression rules |
| [feature/040_quota_measurement_history.md](../../feature/040_quota_measurement_history.md) | Measurement ring the forecast reads |
