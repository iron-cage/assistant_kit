# .status

Show journal health, size, and configuration.

-- **Parameters:** journal_dir::, no_color::
-- **Exit Codes:** 0 (success), 1 (unknown or unimplemented param)

### Syntax

```
clj .status [journal_dir::PATH] [no_color::BOOL]
```

### Parameters

| Parameter | Type | Default | Required | Purpose |
|-----------|------|---------|----------|---------|
| `journal_dir` | Path | ~/.clr/journal/ | No | Journal directory override |
| `no_color` | Boolean | 0 | No | Disable ANSI colors |

**Not yet implemented:** `verbosity::`. The report is always rendered at the
standard level shown below; passing `verbosity::` exits 1 rather than being
silently ignored.

**Algorithm (2 steps):**

1. Open journal directory, count files, sum total bytes, extract oldest/newest dates
2. Render the health report

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
| 4 | [Capacity Planning](../user_story/004_capacity_planning.md) |
