# tests/

### Scope

**Responsibilities:** Automated integration tests for the `claude_journal_charts` crate — journal aggregation and SVG chart generation.
**In Scope:** All crate functionality exercised via the public library API (`generate_usage_chart`).
**Out of Scope:** Manual testing, test planning documents.

### Domain Map

| Domain | File | Tests What |
|--------|------|------------|
| Usage chart generation (T01–T06) | `journal_charts_test.rs` | Empty journal placeholder, multi-day aggregation, same-day aggregation, non-Command event exclusion, output file validity, nonexistent journal directory error handling |
