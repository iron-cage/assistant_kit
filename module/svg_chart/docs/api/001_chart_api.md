# Chart API

**Status**: Implemented | **Since**: 0.1.0

### Scope

- **Purpose**: Provide a domain-agnostic way to render line and bar charts to SVG from a small, declarative spec.
- **Responsibility**: Documents `ChartSpec`, `ChartKind`, `Series`, `SvgChartError`, `render_to_string()`, and `render_to_file()` — the crate's entire public surface — and their behavioral contract.
- **In Scope**: Time-series line charts, categorical bar charts, in-memory string output, file output.
- **Out of Scope**: PNG/raster output (SVG only), interactive/HTML charts, browser-opening or any file-system side effect beyond the one SVG file `render_to_file` is asked to write, user-configurable color palettes.

## Description

`ChartSpec` describes one chart: a title, axis labels, a `ChartKind` (`Line` or `Bar`), and a list of `Series`, each holding a name and `(x, y)` points. `render_to_string` renders the spec and returns the SVG document as a `String`; `render_to_file` renders it and writes the SVG document to a given path. Both entry points share the same drawing logic and differ only in the underlying `plotters` backend (in-memory string vs. file). Series are colored from a fixed internal 6-color palette, assigned by index. An empty `series` list (or a `series` list whose points are all empty) renders a labeled "No data" placeholder chart instead of failing.

## Interface

```rust
pub struct Series
{
  pub name : String,
  pub points : Vec< ( f64, f64 ) >,
}

pub enum ChartKind
{
  Line,
  Bar,
}

pub struct ChartSpec
{
  pub title : String,
  pub x_label : String,
  pub y_label : String,
  pub kind : ChartKind,
  pub series : Vec< Series >,
}

pub enum SvgChartError
{
  Render( String ),
}

/// Renders `spec` and returns the SVG document as a `String`.
pub fn render_to_string( spec : &ChartSpec ) -> Result< String, SvgChartError >;

/// Renders `spec` and writes the SVG document to `path`.
pub fn render_to_file( spec : &ChartSpec, path : &std::path::Path ) -> Result< (), SvgChartError >;
```

## Behavioral Contract

- `ChartKind::Line` draws one continuous line per series; `ChartKind::Bar` draws bars over the first series' points — two structurally distinct rendering paths, not a shared code path that ignores `kind`
- Series are colored from a fixed 6-color palette, assigned by index modulo palette length — no caller-configurable theming
- An empty `series` list, or a `series` list whose points are all empty, renders a "No data" placeholder SVG rather than panicking or returning `Err`
- `render_to_file` returns `Err(SvgChartError::Render(_))` — never panics — when the output path's parent directory does not exist or the file otherwise cannot be written
- Output is always well-formed SVG: parseable as XML, with a `<svg` root element and a matching `</svg>` close
- The crate has zero dependency on `claude_journal`, `claude_profile`, or `claude_journal_charts` — a leaf, reusable by any caller

## Sources

- `src/lib.rs` — implementation
- `tests/chart_test.rs` — Test Matrix T01-T07 coverage
- `task/svg_chart/completed/469_create_svg_chart_crate.md` — originating task, full Test Matrix and Acceptance Criteria
