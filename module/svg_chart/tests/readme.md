# tests/

### Scope

**Responsibilities:** Automated integration tests for the `svg_chart` crate — line/bar chart rendering to SVG string and file.
**In Scope:** All crate functionality exercised via the public library API (`render_to_string`, `render_to_file`, `ChartSpec`).
**Out of Scope:** Manual testing, test planning documents.

### Domain Map

| Domain | File | Tests What |
|--------|------|------------|
| Chart rendering (T01–T07) | `chart_test.rs` | Line chart single/multi series, bar chart categorical, empty-series placeholder, file output, title/axis label presence, unwritable output path error handling |
