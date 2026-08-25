# .status

Show journal health, size, and configuration.

-- **Parameters:** verbosity::, journal_dir::, no_color::
-- **Exit Codes:** 0 (success), 1 (unknown param, or non-integer `verbosity::`)

### Syntax

```
clj .status [verbosity::0|1|2] [journal_dir::PATH] [no_color::BOOL]
```

### Parameters

| Parameter | Type | Default | Required | Purpose |
|-----------|------|---------|----------|---------|
| `verbosity` | Integer | 1 | No | Detail level: 0 one line, 1 report, 2 per-file |
| `journal_dir` | Path | ~/.clr/journal/ | No | Journal directory override |
| `no_color` | Boolean | 0 | No | Disable ANSI colors |

**Algorithm (3 steps):**

1. Parse `verbosity::`, clamping values above 2 (see [param/22_verbosity.md](../param/22_verbosity.md))
2. List the journal directory once — count files, sum bytes, take the date range from the first and last
3. Render at the selected level, appending the per-file breakdown when it is 2

Every figure in the report comes from that single listing, so no two lines can
describe the directory at different moments.

`Total size:` is rounded for reading. A consumer that needs the exact byte count
should read `/api/health`, which reports it unrounded
([feature/002_web_viewing.md](../../feature/002_web_viewing.md)).

`Date range:` reports a single date when the journal holds one day's file, and
`no events` when it is empty — never a placeholder pair that could be misread as
a real range.

**Output (verbosity 1):**

```
Journal directory: /home/dev/.clr/journal
Files: 42
Total size: 12.3 MB
Date range: 2026-05-16 to 2026-06-27
Journal level: full (CLR_JOURNAL=full)
```

`Journal level` reports `clr`'s own `CLR_JOURNAL` setting verbatim, naming the
env var when it is set and `full (default)` when it is not. `.status` does not
validate the value — `clr` is what rejects an invalid level, and normalizing it
here would hide the very setting the report exists to surface.

**Output (verbosity 0):**

```
42 files, 12.3 MB, 2026-05-16 to 2026-06-27
```

**Output (verbosity 2):** the verbosity-1 report, then a blank line, then one
row per file:

```
DATE          SIZE
2026-05-16    1.2 MB
2026-05-17    980.4 KB
```

On an empty journal the breakdown is `(no journal files)` with no column header —
a `DATE`/`SIZE` heading above zero rows would announce a table that is not there.

### Examples

```bash
clj .status                        # Standard health report
clj .status verbosity::0           # Compact one-line summary
clj .status verbosity::2           # Per-file breakdown
```

### Referenced User Stories

| # | User Story |
|---|-----------|
| 4 | [Capacity Planning](../user_story/004_capacity_planning.md) |
