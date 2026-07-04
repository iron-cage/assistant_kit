# .prune

Delete old journal files by age or size.

-- **Parameters:** keep::, dry_run::, confirm::
-- **Exit Codes:** 0 (success), 1 (invalid param or I/O error)

### Syntax

```
clj .prune keep::RETENTION_SPEC [dry_run::BOOL] [confirm::BOOL]
```

### Parameters

| Parameter | Type | Default | Required | Purpose |
|-----------|------|---------|----------|---------|
| `keep` | RetentionSpec | -- | Yes | Retention: age (`30d`, `4w`, `3m`) or size (`100mb`, `1gb`) |
| `dry_run` | Boolean | 0 | No | Show what would be pruned without deleting |
| `confirm` | Boolean | 0 | No | Skip interactive confirmation prompt |

**Algorithm (4 steps):**

1. Parse `keep` as age (duration) or size (bytes); error on invalid format
2. List journal files, identify candidates for deletion
3. If `dry_run::1`, print candidate list and exit 0 without deleting
4. If `confirm::0` (default), prompt user for confirmation; on yes or `confirm::1`, delete files and report count

### Examples

```bash
clj .prune keep::30d                # Delete files older than 30 days
clj .prune keep::100mb              # Delete oldest until under 100MB
clj .prune keep::4w dry_run::1     # Preview: what would be pruned
clj .prune keep::7d confirm::1     # Delete without confirmation
```

### Referenced User Stories

| # | User Story |
|---|-----------|
| 4 | [Capacity Planning](../user_story/004_capacity_planning.md) |
