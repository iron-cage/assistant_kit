# .stats

Aggregate statistics over journal events.

-- **Parameters:** since::, until::, type::, by::, dir::
-- **Exit Codes:** 0 (success), 1 (invalid param)

### Syntax

```
clj .stats [since::DURATION] [until::DURATION] [type::EVENT_TYPE] [by::GROUP_BY]
           [dir::PATH]
```

### Parameters

| Parameter | Type | Default | Required | Purpose |
|-----------|------|---------|----------|---------|
| `since` | Duration | 7d | No | Time window start |
| `until` | Duration | -- | No | Time window end |
| `type` | EventType | execution | No | Event type to aggregate |
| `by` | GroupBy | day | No | Grouping dimension |
| `dir` | Path | ~/.clr/journal/ | No | Journal directory override (falls back to `CLR_JOURNAL_DIR` env, then the default) |

**Algorithm (4 steps):**

1. Construct filter with `since` (default 7d), `until`, and `type` (default execution)
2. Query all matching events via `JournalReader`
3. Group events by the `by` dimension and compute per-bucket aggregates: event count and total cost
4. Render the bucket table plus a `Total: N event(s)` footer — `day`/`model` rows ordered by key, `dir`/`agent` rows ranked by descending count (task 543); events missing the grouping field aggregate under a visible `(no dir)` / `(no agent)` row

### Examples

```bash
clj .stats                           # Daily stats for last 7 days
clj .stats by::model since::30d      # By model, last 30 days
clj .stats by::dir since::1d         # Top working directories by activity today
clj .stats by::agent since::7d       # Top agents (user@host+dir identity) by activity
```

### Referenced User Stories

| # | User Story |
|---|-----------|
| 1 | [Cost Tracking](../user_story/001_cost_tracking.md) |
| 4 | [Capacity Planning](../user_story/004_capacity_planning.md) |
