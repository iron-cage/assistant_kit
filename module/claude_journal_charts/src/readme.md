# src/

Aggregates `claude_journal` `Command` events into a daily-usage SVG bar chart.

## Responsibility Table

| File | Responsibility |
|------|----------------|
| `lib.rs` | `generate_usage_chart()` entry point, `ClaudeJournalChartsError` |

### Scope

**In Scope:**
- Aggregating `Command` events by calendar day
- Rendering the result as one SVG bar chart via `svg_chart`

**Out of Scope:**
- CLI argument parsing, browser-opening (→ `claude_journal_viewer`)
- Redaction (journal content is already redacted at write time by `claude_journal`)
- Any aggregation granularity other than daily, any chart kind other than bar

See [`docs/api/001_journal_charts_api.md`](../docs/api/001_journal_charts_api.md) for the full behavioral contract.
