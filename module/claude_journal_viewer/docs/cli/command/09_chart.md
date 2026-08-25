# .chart

Render a usage SVG chart from journal events, optionally opened in the
default browser.

-- **Parameters:** out::, open::, journal_dir::, no_color::
-- **Exit Codes:** 0 (success), 1 (chart rendering or write failure)

### Syntax

```
clj .chart [out::PATH] [open::0|1] [journal_dir::PATH] [no_color::BOOL]
```

### Parameters

| Parameter | Type | Default | Required | Purpose |
|-----------|------|---------|----------|---------|
| `out` | Path | usage.svg | No | Output SVG file path |
| `open` | Bool | 0 | No | Open the rendered file in the default browser |
| `journal_dir` | Path | ~/.clr/journal/ | No | Journal directory override |
| `no_color` | Boolean | 0 | No | Disable ANSI colors in the confirmation message |

`.chart` renders every event in the journal; it accepts no event filters, so
`dir::` (the event working-directory filter) is not valid here.

**Algorithm (3 steps):**

1. Resolve the journal directory (`journal_dir::` > `CLR_JOURNAL_DIR` > default)
2. Render the usage chart via `claude_journal_charts::generate_usage_chart` and write it to `out::`
3. If `open::1`, open the file in the default browser — a failure to open is a non-fatal warning appended to the success message, never a command failure

### Examples

```bash
clj .chart                            # Write usage.svg in the current directory
clj .chart out::/tmp/usage.svg        # Custom output path
clj .chart open::1                    # Render and open in browser
```
