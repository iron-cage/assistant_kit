# User Story: Cost Tracking

**As a** developer using CLR for automation,
**I want to** track API costs over time by model and project,
**so that** I can optimize usage and budget API spend.

### Persona

Developer running `clr run` and `clr ask` daily for development tasks.

### Primary Commands

| # | Command | Role in Story |
|---|---------|---------------|
| 1 | [`.list`](../command/01_list.md) | View recent events with cost column |
| 3 | [`.stats`](../command/03_stats.md) | Aggregate costs by day/model/command |
| 5 | [`.serve`](../command/05_serve.md) | Visual cost dashboard |

### Acceptance Criteria

| # | Criterion |
|---|-----------|
| AC-01 | `clj .list sort::cost reverse::1 since::7d` shows most expensive invocations |
| AC-02 | `clj .stats by::model since::30d` shows per-model cost breakdown with totals |
| AC-03 | `clj .stats by::day since::7d` shows daily cost trend |
| AC-04 | `clj .stats by::command since::30d` shows run vs ask cost split |
| AC-05 | `clj .serve` dashboard displays cost chart over time |
| AC-06 | Cost values are displayed in USD with 4 decimal places |
| AC-07 | Token counts (in/out) are displayed alongside costs |

### Workflow

```bash
# Daily: quick cost check
clj .list since::1d sort::cost reverse::1

# Weekly: cost breakdown by model
clj .stats by::model since::7d

# Monthly: full cost report
clj .stats by::day since::30d verbosity::2

# Visual: open dashboard
clj .serve open::1
```

### Referenced Parameters

| # | Parameter | Usage |
|---|-----------|-------|
| 01 | [`since`](../param/01_since.md) | Time window for cost analysis |
| 10 | [`format`](../param/10_format.md) | Output format for reports |
| 11 | [`sort`](../param/11_sort.md) | Sort by cost |
| 12 | [`reverse`](../param/12_reverse.md) | Most expensive first |
| 13 | [`by`](../param/13_by.md) | Group by model/day/command |
