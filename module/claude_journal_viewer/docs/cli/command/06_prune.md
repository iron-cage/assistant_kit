# .prune

Delete old journal files by filename-date age.

-- **Parameters:** keep::, dry_run::, journal_dir::, no_color::
-- **Exit Codes:** 0 (success, including "dir not found" and "nothing to prune"), 1 (invalid or unknown param)

### Syntax

```
clj .prune [keep::RETENTION_SPEC] [dry_run::BOOL] [journal_dir::PATH] [no_color::BOOL]
```

### Parameters

| Parameter | Type | Default | Required | Purpose |
|-----------|------|---------|----------|---------|
| `keep` | RetentionSpec | `30d` | No | Age threshold — a duration, floored to whole days |
| `dry_run` | Boolean | 0 | No | Show what would be pruned without deleting |
| `journal_dir` | Path | ~/.clr/journal/ | No | Which journal is pruned — check it before a non-dry run |
| `no_color` | Boolean | 0 | No | Disable ANSI colors |

`.prune` accepts no event filters: it selects files by filename date, never by
event content, so `since::`/`dir::` and the rest are rejected rather than
quietly ignored on a command that deletes.

**Algorithm (4 steps):**

1. Parse `keep` as a duration and floor it to whole days (`keep_days`); error on invalid format
2. Delegate to `claude_journal::rotation::prune_by_age` — candidates are exactly the files named `YYYY-MM-DD.jsonl` whose filename date is strictly before `today - keep_days` (UTC); filesystem mtime is never consulted, and today's file is structurally never deleted
3. If `dry_run::1`, print each candidate as `Would delete: <path>` and exit 0 without deleting
4. Otherwise delete immediately (no confirmation prompt — `dry_run::1` is the preview mechanism), printing `Deleted: <path>` per file, `Warning: could not delete ...` on per-file failure (sweep continues), and a final count line

### Examples

```bash
clj .prune                          # Delete files older than 30 days (default)
clj .prune keep::7d                 # Delete files older than 7 days
clj .prune keep::4w dry_run::1      # Preview: what would be pruned
clj .prune keep::12h                # Sub-day floors to 0 days: keep only today's file
```

### Referenced User Stories

| # | User Story |
|---|-----------|
| 4 | [Capacity Planning](../user_story/004_capacity_planning.md) |
