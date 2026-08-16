# API Doc Entity

### Scope

**Responsibilities:** Public API contracts for the `claude_journal_charts` crate.
**In Scope:** `generate_usage_chart` aggregation-and-render contract, `ClaudeJournalChartsError` variants.
**Out of Scope:** `claude_journal` read API (-> `claude_journal/docs/api/`), `svg_chart` render API (-> `svg_chart/docs/api/`), CLI/browser wiring (-> `claude_journal_viewer`).

### Responsibility Table

| # | File | Responsibility |
|---|------|------|
| 001 | `001_journal_charts_api.md` | generate_usage_chart, ClaudeJournalChartsError contract |
