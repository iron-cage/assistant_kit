# CLI Parameter: open

Auto-open the default browser on what the command produced — the web viewer for
[`.serve`](../command/05_serve.md), the rendered SVG for
[`.chart`](../command/09_chart.md). Uses `xdg-open` on Linux and `open` on
macOS.

A failed launch is a warning, never a failure: on a headless box both commands
still succeed, `.serve` still serves and `.chart` still leaves the SVG on disk.
An *invalid* value is the opposite — `open::` is validated before either
command has an effect, so a rejected value exits 1 with nothing bound and
nothing written.

- **Type:** [`Boolean`](../type/08_boolean.md)
- **Default:** 0
- **Required:** No

```bash
clj .serve open::1                    # Start and open browser
clj .serve open::1 port::9090         # Custom port, auto-open
clj .chart open::1                    # Render usage.svg and open it
clj .chart open::banana; echo $?      # 1 — and no usage.svg written
```

### Referenced Type

| Type | Kind | Fundamental | Key Constraint |
|------|------|-------------|----------------|
| [`Boolean`](../type/08_boolean.md) | Fundamental | Integer | 0 or 1 |

### Referenced Parameter Groups

| # | Group | Membership |
|---|-------|------------|
| 5 | [Global](../param_group/05_global.md) | Partial (serve, chart) |

### Referenced Commands

| # | Command | Default | Notes |
|---|---------|---------|-------|
| 5 | [`.serve`](../command/05_serve.md) | 0 | No auto-open |
| 9 | [`.chart`](../command/09_chart.md) | 0 | Opens the file written to `out::` |

### Referenced User Stories

| # | User Story | Persona |
|---|------------|---------|
| 1 | [Cost Tracking](../user_story/001_cost_tracking.md) | Developer |
