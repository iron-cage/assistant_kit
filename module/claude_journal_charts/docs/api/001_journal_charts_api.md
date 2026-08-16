# Journal Charts API

**Status**: Implemented | **Since**: 0.1.0

### Scope

- **Purpose**: Combine journal-read with chart-render into a usage-specific daily-invocation-count aggregation.
- **Responsibility**: Documents `generate_usage_chart()` and `ClaudeJournalChartsError` — the crate's entire public surface — and their behavioral contract.
- **In Scope**: Aggregating `Command` events by calendar day; rendering the result as one SVG bar chart.
- **Out of Scope**: CLI argument parsing, browser-opening (both -> `claude_journal_viewer`), redaction (journal content is already redacted at write time), any aggregation granularity other than daily, any chart kind other than bar.

## Description

`generate_usage_chart(journal_dir, out_path)` opens `journal_dir` via `claude_journal::JournalReader`, queries it for `EventType::Command` events only, buckets the results by the first 10 characters of each event's ISO-8601 `ts` field (the calendar day), and counts invocations per day. The day → count map is turned into one `svg_chart::Series` (day order becomes point order) and rendered as a `ChartKind::Bar` chart via `svg_chart::render_to_file`. An empty or `Command`-event-free journal produces a placeholder chart — `svg_chart`'s own empty-series handling — rather than an error. The function never runs implicitly: it has no internal caller, timer, or hook anywhere in this crate.

## Interface

```rust
pub enum ClaudeJournalChartsError
{
  JournalDirNotFound( std::path::PathBuf ),
  Chart( svg_chart::SvgChartError ),
}

/// Reads Command events from `journal_dir`, aggregates them into daily
/// invocation counts, and renders the result as an SVG bar chart at `out_path`.
pub fn generate_usage_chart( journal_dir : &std::path::Path, out_path : &std::path::Path ) -> Result< (), ClaudeJournalChartsError >;
```

## Behavioral Contract

- Groups `Command` events by calendar day, extracted from each event's `ts` field (first 10 characters, `YYYY-MM-DD`)
- Non-`Command` events (`Execution`, `Credential`, etc.) are excluded from the count — enforced by `claude_journal`'s own `JournalFilter{ event_type: Some(EventType::Command), .. }` exact-type match
- An empty or `Command`-event-free journal renders a "No data" placeholder chart rather than returning `Err`
- Returns `Err(ClaudeJournalChartsError::JournalDirNotFound)` — never panics — when `journal_dir` does not exist (`claude_journal::JournalReader::open` is itself infallible, so this crate performs its own existence check first)
- Returns `Err(ClaudeJournalChartsError::Chart)` if the underlying SVG rendering fails
- Renders via `svg_chart::render_to_file` — never reimplements drawing
- Has exactly one entry point; no code path in this crate calls it on its own initiative

## Sources

- `src/lib.rs` — implementation
- `tests/journal_charts_test.rs` — Test Matrix T01-T06 coverage
- `task/claude_journal_charts/completed/471_create_claude_journal_charts_crate.md` — originating task, full Test Matrix and Acceptance Criteria
