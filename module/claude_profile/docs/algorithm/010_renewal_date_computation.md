# Algorithm: Renewal Date Computation

### Scope

- **Purpose**: Define how `.usage` computes the time remaining until the next billing renewal, from either a manually-set exact timestamp or an organization-creation-date estimate.
- **Responsibility**: Documents `renewal_secs()`'s two computation branches, the shared Gregorian calendar helpers they call, and the day-of-month preservation requirement for the exact branch's auto-advance step.
- **In Scope**: `renewal_secs()` exact-branch auto-advance logic; estimate-branch next-occurrence logic; `unix_to_date()`/`date_to_unix()` calendar helpers; `parse_iso_secs()`; day-of-month preservation invariant (AC-10).
- **Out of Scope**: `~Renews` string rendering and `~` estimate-prefix rules (→ feature/009 AC-27/AC-28/AC-29, feature/030 Design); quota utilization approximation (→ algorithm/006); renewal cache persistence (→ feature/033).

### Abstract

`renewal_secs()` answers one question: how many seconds remain until the next billing renewal, and is that figure exact or an estimate? Two mutually-exclusive input branches produce the answer, selected by priority — a manually-set `_renewal_at` timestamp (exact) takes precedence over an `org_created_at`-derived billing day (estimate). Both branches are intended to preserve the same day-of-month across renewal cycles; only the estimate branch currently does so correctly, using calendar arithmetic (`date_to_unix`/`unix_to_date`). The exact branch's auto-advance step instead adds a flat 30-day (`2_592_000`s) increment per iteration, which drifts the day-of-month on any month that is not exactly 30 days long — an open defect (BUG-329).

### Algorithm

#### Entry Point

`claude_profile/src/usage/format.rs` — `renewal_secs(renewal_at_opt: Option<&str>, org_created_at_opt: Option<&str>, now_secs: u64) -> Option<(u64, bool)>`

Returns `(seconds_until_renewal, is_estimate)`. `is_estimate = false` for the exact branch, `true` for the estimate branch.

#### Branch Priority

| Priority | Branch | Trigger | Returns |
|----------|--------|---------|---------|
| 1 | Exact | `renewal_at_opt` is `Some` | `(secs, false)` |
| 2 | Estimate | `renewal_at_opt` is `None`, `org_created_at_opt` is `Some` | `(secs, true)` |
| 3 | Absent | Both `None`, or parse failure in either branch | `None` |

The exact branch is checked first and returns unconditionally once entered — an estimate is never computed when an exact `_renewal_at` is present (feature/030 AC-10 context).

#### Exact Branch (`format.rs:134-138`)

```
if let Some( renewal_at ) = renewal_at_opt:
  ts = parse_iso_secs( renewal_at )?          // ISO-8601 UTC -> Unix seconds
  while ts < now_secs:
    ts = ts.saturating_add( 2_592_000 )       // flat 30-day step
  return Some( ( ts.saturating_sub( now_secs ), false ) )
```

When `_renewal_at` is a past timestamp, the loop advances it in fixed 2,592,000-second (30-day) increments until it lands in the future. Intended behavior (feature/030 AC-10): the auto-advanced timestamp preserves the same day-of-month as the original `_renewal_at`.

**Known Defect (BUG-329, open):** the flat 30-day step does not preserve day-of-month across months whose length is not exactly 30 days (Jan/Mar/May/Jul/Aug/Oct/Dec = 31 days; Feb = 28 or 29 days). An original day-of-month of 29, 30, or 31 drifts backward by 1-3 days per step through any 31-day or February month it crosses. See BUG-329 (`task/claude_profile/bug/329_auto_advance_flat_step_drifts_day_of_month.md`) for the full evidence trail. Fix location: replace the flat-step loop with calendar arithmetic mirroring the Estimate Branch below — decompose `ts` via `unix_to_date()`, advance to the next calendar month (carrying year), and re-encode via `date_to_unix()` using the original day-of-month.

#### Estimate Branch (`format.rs:140-159`)

```
if let Some( org_created_at ) = org_created_at_opt:
  if org_created_at.len() < 10: return None
  billing_day = org_created_at[ 8..10 ].parse()?      // day-of-month from ISO date substring
  if billing_day == 0 or billing_day > 31: return None
  ( year, month, day ) = unix_to_date( now_secs )
  ( renewal_year, renewal_month ) =
    if billing_day > day:        ( year, month )          // renewal still upcoming this month
    else if month == 12:         ( year + 1, 1 )          // wrap December -> January
    else:                        ( year, month + 1 )      // next calendar month
  renewal_ts = date_to_unix( renewal_year, renewal_month, billing_day )
  return Some( ( renewal_ts.saturating_sub( now_secs ), true ) )
```

The billing day-of-month is read once from `org_created_at` and never recomputed — every renewal estimate reuses the same `billing_day` value, projected onto the next occurring month via calendar-correct `date_to_unix()`. This is the pattern the Exact Branch's fix (BUG-329) is expected to mirror.

**Caveat (not a defect):** `date_to_unix(renewal_year, renewal_month, billing_day)` does not clamp `billing_day` to the target month's actual length — e.g. `billing_day = 31` projected onto April (30 days) overflows into May 1st via the flat day-count arithmetic in `date_to_unix()`. This caveat is inherited by whatever day-of-month resolution the BUG-329 fix adopts for the Exact Branch.

#### Calendar Helpers

| Function | Signature | Purpose |
|----------|-----------|---------|
| `unix_to_date()` | `(unix_secs: u64) -> (u64, u64, u64)` | Decompose Unix seconds into `(year, month, day)`, 1-based month/day. Hand-rolled Gregorian arithmetic, no external crate. |
| `date_to_unix()` | `(year: u64, month: u64, day: u64) -> u64` (private) | Encode `(year, month, day)` into Unix seconds at UTC midnight. Inverse of `unix_to_date()`. Assumes `year >= 1970`. |
| `parse_iso_secs()` | `(s: &str) -> Option<u64>` (private) | Parse `"YYYY-MM-DDTHH:MM:SSZ"` into Unix seconds via `date_to_unix()` plus time-of-day offset. `None` on malformed input or year < 1970. |

Both `unix_to_date()` and `date_to_unix()` share an inline `is_leap` closure: `(y % 4 == 0 && y % 100 != 0) || y % 400 == 0` — standard Gregorian leap-year rule.

### Features

| File | Relationship |
|------|--------------|
| [feature/030_account_renewal_override.md](../feature/030_account_renewal_override.md) | Defines AC-10 (exact-branch auto-advance day-of-month preservation) and the `_renewal_at`/`org_created_at` override precedence this algorithm implements |
| [feature/009_token_usage.md](../feature/009_token_usage.md) | `~Renews` column consumes `renewal_secs()`'s `(secs, is_estimate)` output via `renews_label()`; AC-27/AC-28/AC-29 |

### Sources

| File | Relationship |
|------|--------------|
| `src/usage/format.rs` | `renewal_secs()`, `unix_to_date()`, `date_to_unix()`, `parse_iso_secs()`, `renews_label()` |

### Tests

| File | Relationship |
|------|--------------|
| `tests/usage/format_tests.rs` | `rl_auto_advance_past_renewal_at` (exact-branch auto-advance, exercises the BUG-329 defect path); estimate-branch coverage |
