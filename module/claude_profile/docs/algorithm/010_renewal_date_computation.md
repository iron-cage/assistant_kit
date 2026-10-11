# Algorithm: Renewal Date Computation

### Scope

- **Purpose**: Define how `.usage` computes the time remaining until the next billing renewal, from either a manually-set exact timestamp or an organization-creation-date estimate.
- **Responsibility**: Documents `renewal_secs()`'s two computation branches, the shared Gregorian calendar helpers they call,
  and the day-of-month and time-of-day preservation requirements for the exact branch's auto-advance step.
- **In Scope**: `renewal_secs()` exact-branch auto-advance logic; estimate-branch next-occurrence logic; `unix_to_date()`/`date_to_unix()` calendar helpers; `parse_iso_secs()`;
  day-of-month and time-of-day preservation invariant (AC-10).
- **Out of Scope**: `~Renews` string rendering and `~` estimate-prefix rules (→ feature/009 AC-27/AC-28/AC-29, feature/030 Design); quota utilization approximation (→ algorithm/006); renewal cache persistence (→ feature/033).

### Abstract

`renewal_secs()` answers one question: how many seconds remain until the next billing renewal, and is that figure exact or an estimate?
Two mutually-exclusive input branches produce the answer, selected by priority
— a manually-set `_renewal_at` timestamp (exact) takes precedence over an `org_created_at`-derived billing day (estimate).
Both branches preserve the same day-of-month across renewal cycles using calendar arithmetic (`date_to_unix`/`unix_to_date`),
clamping the day-of-month to each target month's actual length via a shared `days_in_month()` helper; the exact branch also keeps the anchor's time of day across those steps.
Prior to the BUG-329 fix, the exact branch instead added a flat 30-day (`2_592_000`s) increment per iteration, drifting the day-of-month on any month not exactly 30 days long;
separately, neither branch clamped the day-of-month at month-length boundaries, so a day-29/30/31 anchor could silently overflow into the following month.
Both are now fixed.
The calendar rebuild that fixed them then dropped the exact anchor's time of day, re-encoding every advanced month at 00:00 UTC (BUG-589); that's fixed as well.

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

#### Exact Branch (`format.rs:260-277`)

```
if let Some( renewal_at ) = renewal_at_opt:
  ts = parse_iso_secs( renewal_at )?                    // ISO-8601 UTC -> Unix seconds
  ( cur_year, cur_month, orig_day ) = unix_to_date( ts )
  time_of_day = ts % 86_400                             // anchor's seconds past 00:00 UTC
  while ts < now_secs:
    cur_month += 1
    if cur_month > 12: cur_month = 1; cur_year += 1
    day = orig_day.min( days_in_month( cur_year, cur_month ) )
    ts  = date_to_unix( cur_year, cur_month, day ) + time_of_day
  return Some( ( ts.saturating_sub( now_secs ), false ) )
```

When `_renewal_at` is a past timestamp, the loop decomposes it once via `unix_to_date()`, then advances month-by-month (carrying year on December→January wraparound),
clamping the preserved day-of-month to `min(orig_day, days_in_month(cur_year, cur_month))` at each step before re-encoding via `date_to_unix()` and adding back the anchor's time of day,
since `date_to_unix()` encodes 00:00 UTC.
This satisfies feature/030 AC-10: the auto-advanced timestamp preserves the same day-of-month and time of day as the original `_renewal_at`,
except where the target month is shorter than the original day — there, the clamp lands on that month's last valid day, at the same time of day, instead of overflowing into the next month.
On the renewal day itself, before the stored time, that day's candidate is still in the future, so the loop stops there rather than a month later.

**Fixed defect (BUG-329):** the exact branch previously added a flat 30-day (`2_592_000`s) increment per loop iteration instead of calendar-correct stepping, which drifted the day-of-month on any month whose length is not exactly 30 days (Jan/Mar/May/Jul/Aug/Oct/Dec = 31 days; Feb = 28 or 29 days) by 1-3 days per step. See BUG-329 (`task/claude_profile/bug/329_auto_advance_flat_step_drifts_day_of_month.md`, closed) for the full evidence trail and fix history — the underlying defect is fixed and verified in code, and the bug-tracking file's own lifecycle closure is complete.

**Fixed defect (BUG-589):** the BUG-329 fix re-encoded each advanced month with `date_to_unix()` alone, which returns 00:00 UTC, so the first advance dropped the anchor's time of day.
The shown renewal came early by the stored time of day (up to 23h 59m 59s), and on the renewal day before the stored time it came a whole month late: that day's midnight candidate was already past,
so the loop stepped once more.
The exact branch now carries `ts % 86_400` through the loop.
<!-- BUG-589 — each advanced month was rebuilt at 00:00 UTC, dropping the stored time of day -->

#### Estimate Branch (`format.rs:279-298`)

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
  renewal_ts = date_to_unix( renewal_year, renewal_month, billing_day.min( days_in_month( renewal_year, renewal_month ) ) )
  return Some( ( renewal_ts.saturating_sub( now_secs ), true ) )
```

The billing day-of-month is read once from `org_created_at` and never recomputed — every renewal estimate reuses the same `billing_day` value, projected onto the next occurring month via calendar-correct `date_to_unix()`, clamped to that month's actual length the same way as the Exact Branch.

**Fixed defect (BUG-329):** this branch previously passed `billing_day` to `date_to_unix()` unclamped — e.g. `billing_day = 31` projected onto April (30 days) silently overflowed into May 1st via the flat day-count arithmetic in `date_to_unix()`. Originally catalogued here as a non-blocking "Caveat" inherited from the Exact Branch's then-open defect; once clamping became the established pattern for this function, the same gap in this branch was closed as part of the same fix rather than left as an accepted caveat. The underlying defect is fixed and verified in code, and the bug-tracking file's own lifecycle closure is complete.

#### Calendar Helpers

| Function | Signature | Purpose |
|----------|-----------|---------|
| `unix_to_date()` | `(unix_secs: u64) -> (u64, u64, u64)` | Decompose Unix seconds into `(year, month, day)`, 1-based month/day. Hand-rolled Gregorian arithmetic, no external crate. |
| `date_to_unix()` | `(year: u64, month: u64, day: u64) -> u64` (private) | Encode `(year, month, day)` into Unix seconds at UTC midnight. Inverse of `unix_to_date()`. Assumes `year >= 1970`. The exact branch adds the anchor's time of day back onto this midnight. |
| `parse_iso_secs()` | `(s: &str) -> Option<u64>` (private) | Parse `"YYYY-MM-DDTHH:MM:SSZ"` into Unix seconds via `date_to_unix()` plus time-of-day offset. `None` on malformed input or year < 1970. |

Both `unix_to_date()` and `date_to_unix()` share an inline `is_leap` closure: `(y % 4 == 0 && y % 100 != 0) || y % 400 == 0` — standard Gregorian leap-year rule.

### Features

| File | Relationship |
|------|--------------|
| [feature/030_account_renewal_override.md](../feature/030_account_renewal_override.md) | Defines AC-10 (exact-branch auto-advance day-of-month and time-of-day preservation) and the `_renewal_at`/`org_created_at` override precedence this algorithm implements |
| [feature/009_token_usage.md](../feature/009_token_usage.md) | `~Renews` column consumes `renewal_secs()`'s `(secs, is_estimate)` output via `renews_label()`; AC-27/AC-28/AC-29 |

### Sources

| File | Relationship |
|------|--------------|
| `src/usage/format.rs` | `renewal_secs()`, `unix_to_date()`, `date_to_unix()`, `parse_iso_secs()`, `renews_label()` |

### Tests

| File | Relationship |
|------|--------------|
| `tests/usage/format_tests.rs` | Exact-branch clamping regressions (BUG-329): `rl_auto_advance_past_renewal_at`, `rl_auto_advance_single_step_preserves_day_across_31_day_month` |
| `tests/usage/format_tests.rs` | Exact-branch clamping regressions (BUG-329): `rl_auto_advance_multi_year_preserves_day_of_month`, `rl_auto_advance_clamps_day_31_anchor_at_shorter_month_end` |
| `tests/usage/format_tests.rs` | Exact-branch clamping regressions (BUG-329): `rl_auto_advance_day29_clamps_in_common_february_then_recovers` |
| `tests/usage/format_tests.rs` | Exact-branch time of day (BUG-589): `rl_auto_advance_keeps_anchor_time_of_day`, `rl_auto_advance_on_renewal_day_before_stored_time_stays_same_day` |
| `tests/usage/format_tests.rs` | Exact-branch time of day (BUG-589): the day-31 clamping test above also asserts its full instant |
| `tests/usage/format_tests.rs` | Estimate branch, including BUG-329's clamping gap: `rl_estimate_from_org_created_at`, `rl_estimate_clamps_day31_billing_anchor_at_shorter_month_end` |
