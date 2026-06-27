# .stats

Aggregate statistics over journal events.

-- **Parameters:** since::, until::, type::, by::, verbosity::, journal_dir::
-- **Exit Codes:** 0 (success), 1 (invalid param)

### Syntax

```
clj .stats [since::DURATION] [until::DURATION] [type::EVENT_TYPE] [by::GROUP_BY]
           [verbosity::LEVEL] [journal_dir::PATH]
```

### Parameters

| Parameter | Type | Default | Required | Purpose |
|-----------|------|---------|----------|---------|
| `since` | Duration | 7d | No | Time window start |
| `until` | Duration | -- | No | Time window end |
| `type` | EventType | execution | No | Event type to aggregate |
| `by` | GroupBy | day | No | Grouping dimension |
| `verbosity` | Integer | 1 | No | Output detail level |
| `journal_dir` | Path | ~/.clr/journal/ | No | Journal directory override |

**Algorithm (4 steps):**

1. Construct filter with `since` (default 7d), `until`, and `type` (default execution)
2. Query all matching events via `JournalReader`
3. Group events by the `by` dimension and compute aggregates: count, ok/fail, retries, total cost, total in/out tokens, average duration
4. Render summary table with totals row and success rate

### Examples

```bash
clj .stats                           # Daily stats for last 7 days
clj .stats by::model since::30d      # By model, last 30 days
clj .stats by::error since::7d       # Error class distribution
clj .stats by::command               # Run vs ask vs isolated breakdown
clj .stats by::hour since::1d        # Hourly activity today
```

### Referenced User Stories

| # | User Story |
|---|-----------|
| 1 | [Cost Tracking](../user_story/01_cost_tracking.md) |
| 4 | [Capacity Planning](../user_story/04_capacity_planning.md) |
