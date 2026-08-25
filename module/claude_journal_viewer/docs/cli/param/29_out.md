# CLI Parameter: out

File path for the SVG `.chart` renders. Unlike
[`output`](23_output.md), it has a real default — `.chart` always writes a
file, because an SVG is not something to put on stdout — so omitting `out`
writes `usage.svg` into the current directory rather than streaming anywhere.

Parent directories must exist; `.chart` does not create them, and a write
failure exits 1. Nothing is written when a parameter is rejected: `open::` is
validated first, so a run that exits 1 on a bad value leaves no stale SVG
behind ([command/09_chart.md](../command/09_chart.md)).

`out` and `output` are deliberately different names for deliberately different
things and are not interchangeable — `.chart out::` is a destination that
always exists, `.export output::` is an opt-out of stdout. Passing either to
the other command exits 1 with the accepted list for that command.

- **Type:** [`Path`](../type/05_path.md)
- **Default:** `usage.svg` (current directory)
- **Required:** No

```bash
clj .chart                            # Writes ./usage.svg
clj .chart out::/tmp/usage.svg        # Custom path
clj .chart out::usage.svg open::1     # Render, then open in the browser
```

### Referenced Type

| Type | Kind | Fundamental | Key Constraint |
|------|------|-------------|----------------|
| [`Path`](../type/05_path.md) | Semantic | String | Writable file path |

### Referenced Parameter Groups

| # | Group | Membership |
|---|-------|------------|
| 2 | [Display](../param_group/02_display.md) | Partial (chart only) |

### Referenced Commands

| # | Command | Default | Notes |
|---|---------|---------|-------|
| 9 | [`.chart`](../command/09_chart.md) | `usage.svg` | Always writes a file; never stdout |

### Referenced User Stories

-- (none — `.chart` is not yet reached by a user story)
