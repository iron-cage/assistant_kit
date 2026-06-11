# User Story :: 3. Multi-Account Quota Monitoring

**Persona:** Power user managing multiple Claude accounts to maximize available quota
**Goal:** See all accounts' remaining session and weekly quota in one view to decide which to use next
**Benefit:** Prevents quota exhaustion by identifying the best account before starting resource-intensive work
**Priority:** Medium

### Acceptance Criteria

- [ ] `clp .usage` shows all saved accounts with 5h/7d quota, expiry, and renewal in a single table
- [ ] `sort::endurance` ranks by most remaining quota; `sort::renew` by soonest renewal
- [ ] `live::1` continuously refreshes the table at `interval::` seconds
- [ ] `→ Next` column recommends the best account per the selected `next::` strategy
- [ ] `min_5h::X` and `min_7d::X` filter to accounts meeting minimum quota thresholds

### Referenced Commands

| # | Command | Role |
|---|---------|------|
| 1 | [`.usage`](../command/006_usage.md#command--9-usage) | Primary: multi-account quota table with sort/filter/live |
| 2 | [`.account.limits`](../command/001_account.md#command--11-accountlimits) | Secondary: per-account rate-limit header detail |

### Referenced Parameters

| # | Parameter | Role |
|---|-----------|------|
| 1 | [`sort::`](../param/025_sort.md) | Row ordering strategy |
| 2 | [`desc::`](../param/026_desc.md) | Sort direction override |
| 3 | [`prefer::`](../param/027_prefer.md) | Weekly quota column for sort heuristics |
| 4 | [`next::`](../param/032_next.md) | Recommendation strategy for `→ Next` column |
| 5 | [`live::`](../param/020_live.md) | Continuous quota refresh loop |
| 6 | [`interval::`](../param/021_interval.md) | Live mode cycle duration |
| 7 | [`jitter::`](../param/022_jitter.md) | Live mode cycle timing variance |
| 8 | [`min_5h::`](../param/041_min_5h.md) | Minimum 5h Left % filter |
| 9 | [`min_7d::`](../param/042_min_7d.md) | Minimum 7d Left % filter |
| 10 | [`abs::`](../param/046_abs.md) | Show absolute token counts instead of percentages |
| 11 | [`cols::`](../param/033_cols.md) | Column visibility modifiers |
| 12 | [`format::`](../param/002_format.md) | Output format (text default; json for processing) |

### Referenced Parameter Groups

| # | Parameter Group | Role |
|---|-----------------|------|
| 1 | [Sort Control](../param_group/004_sort_control.md) | `sort::`, `desc::`, `prefer::`, `next::` |
| 2 | [Fetch Behavior](../param_group/003_fetch_behavior.md) | `live::`, `interval::`, `jitter::` |
| 3 | [Display Control](../param_group/005_display_control.md) | Row filtering and column visibility |

### Referenced Formats

| # | Format | Role |
|---|--------|------|
| 1 | [`text`](../format/001_text.md) | Default human-readable quota table |
| 2 | [`json`](../format/002_json.md) | Structured output for downstream processing |
