# svg_chart

Minimal SVG line/bar chart rendering, wrapping `plotters`.

### Scope

Renders a small, domain-agnostic `ChartSpec` (title, axis labels, chart kind, data series) to an SVG file or in-memory string. Pure rendering, no I/O beyond the one file-write entry point, zero dependency on any `claude_*` crate.

### Responsibility Table

| File | Responsibility |
|------|----------------|
| `src/lib.rs` | ChartSpec, ChartKind, Series, render_to_string, render_to_file |
| `verb/` | Shell scripts implementing do-protocol verbs for this crate. |
| `docs/` | Public API contract |
| `tests/` | Test Matrix coverage for chart rendering behavior |
