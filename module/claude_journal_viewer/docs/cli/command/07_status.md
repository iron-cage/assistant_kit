# .status

Show journal health, size, and configuration.

-- **Parameters:** verbosity::, journal_dir::
-- **Exit Codes:** 0 (success)

### Syntax

```
clj .status [verbosity::LEVEL] [journal_dir::PATH]
```

### Parameters

| Parameter | Type | Default | Required | Purpose |
|-----------|------|---------|----------|---------|
| `verbosity` | Integer | 1 | No | Detail level (0=compact, 1=standard, 2=per-file) |
| `journal_dir` | Path | ~/.clr/journal/ | No | Journal directory override |

**Algorithm (2 steps):**

1. Open journal directory, count files, sum total bytes, extract oldest/newest dates
2. Render health report at requested verbosity

**Output (verbosity 1):**

```
Journal directory: ~/.clr/journal/
Files: 42
Total size: 12.3 MB
Date range: 2026-05-16 to 2026-06-27
Journal level: full (CLR_JOURNAL=full)
```

### Examples

```bash
clj .status                        # Standard health report
clj .status verbosity::0          # Compact one-line summary
clj .status verbosity::2          # Per-file breakdown
```

### Referenced User Stories

| # | User Story |
|---|-----------|
| 4 | [Capacity Planning](../user_story/04_capacity_planning.md) |
