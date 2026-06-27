# User Story: Capacity Planning

**As a** developer managing CLR usage at scale,
**I want to** understand usage patterns and manage journal storage,
**so that** I can plan API capacity and prevent disk space issues.

### Persona

Developer running high-volume CLR automation needing resource management.

### Primary Commands

| # | Command | Role in Story |
|---|---------|---------------|
| 3 | [`.stats`](../command/03_stats.md) | Usage trends and aggregates |
| 6 | [`.prune`](../command/06_prune.md) | Manage journal file retention |
| 7 | [`.status`](../command/07_status.md) | Journal health and size |

### Acceptance Criteria

| # | Criterion |
|---|-----------|
| AC-01 | `clj .status` shows file count, total size, and date range |
| AC-02 | `clj .status verbosity::2` shows per-file size breakdown |
| AC-03 | `clj .stats by::day since::30d` shows daily invocation volume trend |
| AC-04 | `clj .stats by::hour since::1d` shows hourly distribution (peak detection) |
| AC-05 | `clj .prune keep::30d dry_run::1` previews what would be pruned |
| AC-06 | `clj .prune keep::100mb` maintains journal under 100MB |
| AC-07 | `clj .prune keep::7d confirm::1` prunes without confirmation |

### Workflow

```bash
# Health check: how big is the journal?
clj .status

# Trend analysis: daily volume
clj .stats by::day since::30d

# Peak detection: hourly distribution
clj .stats by::hour since::1d

# Preview cleanup
clj .prune keep::30d dry_run::1

# Execute cleanup
clj .prune keep::30d confirm::1
```

### Referenced Parameters

| # | Parameter | Usage |
|---|-----------|-------|
| 13 | [`by`](../param/13_by.md) | Group by day/hour for trends |
| 18 | [`keep`](../param/18_keep.md) | Retention threshold |
| 19 | [`dry_run`](../param/19_dry_run.md) | Preview mode |
| 22 | [`verbosity`](../param/22_verbosity.md) | Status detail level |
