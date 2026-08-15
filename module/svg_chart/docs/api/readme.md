# API Doc Entity

### Scope

**Responsibilities:** Public API contracts for the `svg_chart` crate.
**In Scope:** ChartSpec construction, render_to_string in-memory rendering, render_to_file file-output rendering.
**Out of Scope:** Internal drawing helpers (`draw`, `draw_lines`, `draw_bars`), caller-specific chart content.

### Responsibility Table

| # | File | Responsibility |
|---|------|------|
| 001 | `001_chart_api.md` | ChartSpec, render_to_string, render_to_file contract |
