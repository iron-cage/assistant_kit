# Feature: Usage Row Filtering and Extraction

### Scope

- **Purpose**: Provide row-level filters, count/offset pagination, and single-value extraction for `.usage` table output, enabling scripting and targeted monitoring.
- **Responsibility**: Documents the filtering parameters (`count::`, `offset::`, `only_active::`, `only_next::`, `min_5h::`, `min_7d::`, `only_valid::`, `exclude_exhausted::`), the `get::` single-value extraction shorthand, and the associated format extensions (`format::value`, `format::tsv`, `format::plain`, `abs::`, `no_color::`).
- **In Scope**: Row count limit, row offset, boolean row filters, percentage threshold filters, `get::` field extraction with `format::value` output, `abs::` for absolute values, `no_color::` for plain output.
- **Out of Scope**: Column visibility (→ 033_cols.md), sort order and `→` recommendation (→ 020_usage_sort_strategies.md), live monitor mode (→ 018_live_monitor.md).

### Design

`.usage` applies row filtering after sort but before rendering. Filters are composable — multiple filters combine with AND logic (a row must satisfy all active filters to appear). After filtering, `count::` and `offset::` apply as a window on the filtered result set.

**Filter evaluation order:**
1. Account list from filesystem (no HTTP) — `account::list()` reads `_active_{hostname}_{user}` marker to populate `is_active`
2. Request-Constraint pre-fetch gate: `only_active::` — filesystem-based; reduces account list to at most 1 entry before the HTTP fetch loop begins (Pipeline-Constraint rule: O(1) fetch when result is known to be ≤1)
3. Per-account quota fetch (HTTP — only for accounts not excluded in step 2)
4. Sort and tier grouping
5. Post-fetch boolean row filters: `only_next::`, `only_valid::`, `exclude_exhausted::` — predicates require quota data from step 3
6. Threshold filters: `min_5h::`, `min_7d::` — require per-account quota percentage data
7. Offset: skip first N rows from the filtered result
8. Count: truncate to at most N rows after offset

**Row filter parameters:**

| Parameter | Type | Default | Behavior |
|-----------|------|---------|----------|
| `count::` | `u64` | `0` | Maximum rows to display; `0` means show all remaining rows after offset |
| `offset::` | `u64` | `0` | Skip first N rows from the filtered result before display |
| `only_active::` | `bool` | `0` | Show only the row whose account matches the per-machine active marker; filesystem-based — gates HTTP fetch (Pipeline-Constraint) |
| `only_next::` | `bool` | `0` | Show only the row that received the `→` marker from the active `sort::` strategy |
| `min_5h::` | `f64` | `0` | Hide rows where `5h Left` is below this percentage (0–100); rows with `—` (no valid quota) are also hidden |
| `min_7d::` | `f64` | `0` | Hide rows where `7d Left` is below this percentage (0–100); rows with `—` are also hidden |
| `only_valid::` | `bool` | `0` | Hide rows where status is 🔴 (invalid or missing token) |
| `exclude_exhausted::` | `bool` | `0` | Hide rows where status is 🟡 (weekly or hourly exhausted) or 🔴 (invalid token) |

**`get::` single-value extraction:**

`get::field_id` extracts the value of one column for the first row in the current (filtered) result set and prints it as a bare string with no table headers, separator lines, or footer. Implies `format::value` output mode. Field IDs match the `cols::` column registry:

| Field ID | Output |
|----------|--------|
| `5h_left` | Percentage string, e.g. `88%` |
| `5h_reset` | Duration string, e.g. `in 3h 19m` or `—` |
| `7d_left` | Percentage string |
| `7d_son` | Percentage string |
| `7d_reset` | Duration string |
| `expires` | Duration or timestamp string |
| `renews` | Duration string, e.g. `~in 6d` or `in 3h 47m` |
| `next_event_type` | Event label string, e.g. `"+7d"` |
| `next_event_secs` | Seconds to next event, e.g. `10800` |
| `sub` | Subscription tier, e.g. `max` |
| `status` | Emoji: `🟢`, `🟡`, or `🔴` |
| `account` | Account name string |
| `host` | Host label string (from account profile metadata) |
| `role` | Role label string (from account profile metadata) |

`get::` combined with row filters allows extracting any single scalar value: `clp .usage only_next::1 get::7d_left` outputs the 7d Left percentage of the recommended next account.

**Output format extensions:**

| Format | Behavior |
|--------|----------|
| `format::value` | Bare value output — no headers, no separator lines, no footer; implied by `get::` |
| `format::tsv` | Tab-separated values with one header row; no emoji in status column (uses text labels: `ok`, `warn`, `err`) |
| `format::plain` | Same layout as `format::text` but with no emoji and no ANSI colors |

**`abs::` and `no_color::`:**

- `abs::1` replaces percentage values with absolute token counts where the API provides them.
- `no_color::1` is equivalent to `format::plain` for text output — strips all emoji and ANSI sequences from the output regardless of format.

### Acceptance Criteria

- **AC-01**: `clp .usage count::3` displays at most 3 rows (the first 3 after sort+tier+filter). Header and footer are still shown.
- **AC-02**: `clp .usage offset::2 count::3` skips the first 2 rows and displays at most the next 3 rows. `count::0` with any `offset::N` skips N rows and shows all remaining.
- **AC-03**: `clp .usage only_active::1` displays exactly one row — the active account row; exits 0 even when active account has no valid quota.
- **AC-04**: `clp .usage only_next::1` displays exactly one row — the account receiving `→` from the active `sort::` strategy; exits 0 with 0 rows when no eligible candidate exists.
- **AC-05**: `clp .usage min_5h::50` hides all rows where `5h Left < 50%` or where `5h Left` is `—`. Rows with `5h Left = 50%` are shown (inclusive boundary).
- **AC-06**: `clp .usage min_7d::20` hides all rows where `7d Left < 20%` or where `7d Left` is `—`. Rows with `7d Left = 20%` are shown (inclusive boundary).
- **AC-07**: `clp .usage only_valid::1` hides all 🔴 rows; shows 🟢 and 🟡 rows.
- **AC-08**: `clp .usage exclude_exhausted::1` hides all 🟡 and 🔴 rows; shows only 🟢 rows.
- **AC-09**: Multiple row filters combine with AND: `clp .usage only_valid::1 min_7d::30` shows only 🟢/🟡 rows where `7d Left ≥ 30%`.
- **AC-10**: `clp .usage get::7d_left` outputs the `7d Left` value of the first row (top of sorted, filtered result) as a bare string with no headers, separators, or footer. Exit 0. Implies `format::value`.
- **AC-11**: `clp .usage only_next::1 get::7d_left` outputs the `7d Left` value for the `→` account. Exit 0.
- **AC-12**: `clp .usage get::status` outputs one of `🟢`, `🟡`, or `🔴` for the first row.
- **AC-13**: `clp .usage format::tsv` produces tab-separated output with a header row; status column uses `ok`/`warn`/`err` text labels instead of emoji.
- **AC-14**: `clp .usage no_color::1` produces output with no emoji and no ANSI sequences; status column renders as plain text labels.
- **AC-15**: Invalid `get::` field ID exits 1 with an error listing the valid field IDs.
- **AC-16**: `count::`, `offset::`, filter params, and `get::` all work combined with `sort::`, `prefer::`, and `cols::`.
- **AC-17**: `clp .usage only_active::1 get::status` on an N-account store performs exactly 1 HTTP request to the OAuth usage API regardless of N. The active account is identified from the `_active_{hostname}_{user}` filesystem marker before any HTTP call; non-active accounts are excluded from the fetch set at step 2.

### Commands

| File | Relationship |
|------|--------------|
| [cli/command/006_usage.md](../cli/command/006_usage.md) | `.usage` command parameter table |

### Features

| File | Relationship |
|------|--------------|
| [009_token_usage.md](009_token_usage.md) | Base `.usage` rendering and column definitions |
| [020_usage_sort_strategies.md](020_usage_sort_strategies.md) | Sort applied before row filtering |
| [020_usage_sort_strategies.md](020_usage_sort_strategies.md) | `sort::` strategy drives `→` marker used by `only_next::1` |
| [029_account_host_metadata.md](029_account_host_metadata.md) | `get::host` and `get::role` field extraction |

### Parameters

| File | Relationship |
|------|--------------|
| [cli/param/002_format.md](../cli/param/002_format.md) | `format::` extensions (`value`, `tsv`, `plain`) |
| [cli/param/037_count.md](../cli/param/037_count.md) | `count::` parameter specification |
| [cli/param/038_offset.md](../cli/param/038_offset.md) | `offset::` parameter specification |
| [cli/param/039_only_active.md](../cli/param/039_only_active.md) | `only_active::` parameter specification |
| [cli/param/040_only_next.md](../cli/param/040_only_next.md) | `only_next::` parameter specification |
| [cli/param/041_min_5h.md](../cli/param/041_min_5h.md) | `min_5h::` parameter specification |
| [cli/param/042_min_7d.md](../cli/param/042_min_7d.md) | `min_7d::` parameter specification |
| [cli/param/043_only_valid.md](../cli/param/043_only_valid.md) | `only_valid::` parameter specification |
| [cli/param/044_exclude_exhausted.md](../cli/param/044_exclude_exhausted.md) | `exclude_exhausted::` parameter specification |
| [cli/param/045_get.md](../cli/param/045_get.md) | `get::` parameter specification |
| [cli/param/046_abs.md](../cli/param/046_abs.md) | `abs::` parameter specification |
| [cli/param/047_no_color.md](../cli/param/047_no_color.md) | `no_color::` parameter specification |

### Referenced Commands

| # | Command | Role |
|---|---------|------|
| 1 | [`.usage`](../cli/command/006_usage.md#command--9-usage) | CLI surface for this feature |

### Sources

| File | Relationship |
|------|--------------|
| `src/usage/mod.rs` | filter pipeline application and orchestration |
| `src/usage/render.rs` | `get::` field extraction, `format::value`/`tsv`/`plain` rendering |
