# docs/

### Scope

**Responsibilities:** API contract for the `svg_chart` crate.
**In Scope:** Public chart-rendering API (`ChartSpec`, `ChartKind`, `Series`, `render_to_string`, `render_to_file`).
**Out of Scope:** Source code (-> `src/`), automated tests (-> `tests/`), caller-specific chart content or labeling (-> consuming crates' own docs).

### Responsibility Table

| File | Responsibility |
|------|----------------|
| `api/` | Public library API contract: ChartSpec, render_to_string, render_to_file |
