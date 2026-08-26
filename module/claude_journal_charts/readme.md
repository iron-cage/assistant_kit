# claude_journal_charts

Aggregates `claude_journal` `Command` events into a daily-usage SVG bar chart.

### Scope

Reads `Command` events from a `claude_journal` event log, groups them by calendar day, and renders the daily counts as an SVG bar chart via `svg_chart`. Exposes exactly one entry point, `generate_usage_chart`, invoked only by an explicit caller — never as a side effect of any other operation.

### Responsibility Table

| File | Responsibility |
|------|----------------|
| `Cargo.toml` | Crate manifest and dependency configuration |
| `src/lib.rs` | generate_usage_chart, ClaudeJournalChartsError |
| `verb/` | Shell scripts implementing do-protocol verbs for this crate. |
| `docs/` | Public API contract |
| `tests/` | Test Matrix coverage for aggregation and chart-generation behavior |
