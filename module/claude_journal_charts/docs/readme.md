# docs/

### Scope

**Responsibilities:** API contract for the `claude_journal_charts` crate.
**In Scope:** Public aggregation/chart-generation API (`generate_usage_chart`, `ClaudeJournalChartsError`).
**Out of Scope:** Source code (-> `src/`), automated tests (-> `tests/`), CLI wiring or browser-opening (-> `claude_journal_viewer`).

### Responsibility Table

| File | Responsibility |
|------|----------------|
| `api/` | Public library API contract: generate_usage_chart |
