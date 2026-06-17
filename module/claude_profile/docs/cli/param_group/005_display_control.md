# Group :: 5. Display Control

**Parameters:** `cols::`, `count::`, `offset::`, `only_active::`, `only_next::`, `min_5h::`, `min_7d::`, `only_valid::`, `exclude_exhausted::`, `abs::`, `no_color::`
**Pattern:** Column visibility, row filtering, and display modifiers for `.usage`
**Purpose:** Controls which columns appear, which rows survive filtering, pagination/truncation, and display rendering (absolute values, color stripping) for the `.usage` quota table.

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| [`cols::`](../param/033_cols.md) | `string` | (default set) | Comma-separated `+col_id` / `-col_id` modifiers relative to the default column set |
| [`count::`](../param/037_count.md) | `u64` | `0` (all) | Limit rows to N after all filtering; `0` = no limit |
| [`offset::`](../param/038_offset.md) | `u64` | `0` | Skip first N rows before applying `count::` |
| [`only_active::`](../param/039_only_active.md) | `bool` | `0` | Keep only the currently active account row |
| [`only_next::`](../param/040_only_next.md) | `bool` | `0` | Keep only the `→` (next-recommended) row |
| [`min_5h::`](../param/041_min_5h.md) | `u8` | `0` | Keep only rows with 5h quota ≥ N% |
| [`min_7d::`](../param/042_min_7d.md) | `u8` | `0` | Keep only rows with 7d quota ≥ N% |
| [`only_valid::`](../param/043_only_valid.md) | `bool` | `0` | Keep only 🟢 rows (non-exhausted, non-expired) |
| [`exclude_exhausted::`](../param/044_exclude_exhausted.md) | `bool` | `0` | Remove 🔴 exhausted rows |
| [`abs::`](../param/046_abs.md) | `bool` | `0` | Show absolute token counts instead of percentages |
| [`no_color::`](../param/047_no_color.md) | `bool` | `0` | Strip emoji and ANSI sequences from output |

### Referenced Commands

| # | Command | Role |
|---|---------|------|
| 1 | [`.usage`](../command/006_usage.md#command--9-usage) | All 11 display and filter params |

**Typical Patterns:**

```bash
# Show Sub column (off by default)
clp .usage cols::+sub

# Filter to healthy accounts only
clp .usage only_valid::1

# Scripting: get next recommended account's 7d quota
clp .usage only_next::1 get::7d_left

# Top 3 accounts by sort order
clp .usage count::3

# Paginate: skip first account, show next 2
clp .usage offset::1 count::2

# Non-TTY output (logs, CI)
clp .usage no_color::1

# Absolute token counts
clp .usage abs::1
```

**Semantic Coherence Test**

> "Does parameter X control **what rows appear, which columns are visible, or how values are rendered** in the `.usage` quota table?"

All 11 members pass:
- `cols::` — controls which columns are visible
- `count::` / `offset::` — control row window (pagination/truncation)
- `only_active::`, `only_next::`, `min_5h::`, `min_7d::`, `only_valid::`, `exclude_exhausted::` — filter which rows survive
- `abs::` — controls value rendering (percentages vs absolute counts)
- `no_color::` — controls symbol rendering (emoji/ANSI vs plain text)

Parameters that fail: `sort::`, `next::` (ordering strategy, not display); `format::` (serialization format, not row selection); `live::`, `refresh::` (fetch behavior, not display).

**Invariants**

- `cols::` cannot remove structural `flag` and `account` columns.
- `cols::` has no effect on `format::json` output — JSON always includes all schema fields.
- `Sub` and `7d Son Reset` columns are off by default; `host` and `role` columns are off by default; all other quota columns are on by default.
- Invalid `cols::` column IDs cause exit 1 with an error naming the valid column IDs.
- Row filters combine with AND logic — a row must pass ALL active filters to survive.
- `count::` and `offset::` are applied last (after all row filters and sorting).
- `abs::` and `no_color::` have no effect on `format::json` output.

**Cross-References**

- [../004_parameter_interactions.md](../004_parameter_interactions.md) — Interaction 8: `cols::` does not affect `format::json` output
- [../../feature/009_token_usage.md](../../feature/009_token_usage.md) — column definitions, AC-22, AC-23; four-group status partition
- [../../feature/028_usage_row_filtering.md](../../feature/028_usage_row_filtering.md) — row filter evaluation order and AND-composition rules
- [../param/033_cols.md](../param/033_cols.md) — complete column registry with IDs and defaults

### Referenced User Stories

| # | User Story | Persona |
|---|------------|---------|
| 1 | [Multi-Account Quota Monitoring](../user_story/003_quota_monitoring.md) | `cols::`, `count::`, `min_5h::` for quota table customization |
| 2 | [Scripted Pipeline Automation](../user_story/004_scripted_automation.md) | `only_next::`, `no_color::` for pipeline extraction |
| 3 | [Account Rotation](../user_story/001_account_rotation.md) | `only_next::`, `only_active::` for rotation targeting |
