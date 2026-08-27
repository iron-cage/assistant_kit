# src/

Domain-agnostic line/bar chart rendering to SVG from a small, declarative spec.

## Responsibility Table

| File | Responsibility |
|------|----------------|
| `lib.rs` | `ChartSpec`, `ChartKind`, `Series`, `SvgChartError`, `render_to_string()`, `render_to_file()` |

### Scope

**In Scope:**
- Time-series line charts, categorical bar charts
- In-memory string output and file output, sharing one drawing implementation

**Out of Scope:**
- PNG/raster output (SVG only), interactive/HTML charts
- Browser-opening or any file-system side effect beyond the one SVG file `render_to_file` writes
- User-configurable color palettes (fixed internal 6-color palette)

See [`docs/api/001_chart_api.md`](../docs/api/001_chart_api.md) for the full behavioral contract.
