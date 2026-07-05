# CLI Viewing

**Status**: Planned | **Since**: 1.3.0

### Scope

- **Purpose**: Provide CLI commands for exploring and managing journal data via unilang syntax.
- **Responsibility**: Documents the 8 `.command param::value` commands, their default output, and their dual standalone/super-app registration.
- **In Scope**: Command list and purpose, default `.list` table columns, and `clj`/`ast .journal.*` dual dispatch.
- **Out of Scope**: Web dashboard viewing (→ `docs/feature/002_web_viewing.md`), filter semantics shared across commands (→ `docs/feature/003_filtering.md`).

## Description

Eight CLI commands for exploring journal data using unilang `.command param::value` syntax. Commands are registered via `claude_journal.commands.yaml` and dispatched through the unilang `CommandRegistry`.

| # | Command | Purpose |
|---|---------|---------|
| 1 | `.list` | Display filtered event table with configurable columns and sort |
| 2 | `.tail` | Follow journal events in real-time (like `tail -f`) |
| 3 | `.stats` | Aggregate statistics: cost/day, success rate, token usage, error distribution |
| 4 | `.search` | Full-text regex search across event messages and optionally stdout |
| 5 | `.serve` | Start embedded HTTP server for web-based viewing |
| 6 | `.prune` | Delete old journal files by age or total size |
| 7 | `.status` | Show journal health: file count, total size, date range, config |
| 8 | `.export` | Export filtered events to file in table/json/csv/jsonl format |

Default output (`.list`) renders a compact table with columns: TIME, CMD, MODEL, EXIT, COST, IN/OUT, DUR, TYPE. Cost data is extracted from the `cost_usd`, `input_tokens`, `output_tokens` fields. Duration is formatted as human-readable seconds.

The `clj` binary operates standalone and also registers its commands into the `assistant` super-app as `ast .journal.list`, `ast .journal.stats`, etc.

## Acceptance Criteria

- AC-001: All 8 commands are registered and dispatch correctly via unilang CommandRegistry
- AC-002: `.list` with no params shows the 50 most recent events in table format
- AC-003: `.list since::1h type::execution` applies time + type filter as AND conditions
- AC-004: `.tail` blocks and emits new events as they are appended to the journal
- AC-005: `.stats` without params shows daily aggregates for the last 7 days
- AC-006: `.search pattern::"rate limit"` searches event messages using regex
- AC-007: `.prune keep::30d` deletes journal files older than 30 days with confirmation prompt
- AC-008: `.status` reports file count, total bytes, oldest/newest dates, and configured journal dir
- AC-009: `.export format::csv since::7d output::/tmp/events.csv` writes filtered events to file
- AC-010: Standalone `clj` binary and `ast .journal.*` super-app routing both work

## Sources

- `src/cli/mod.rs` — command dispatch
- `src/cli/list.rs` through `src/cli/export.rs` — individual command implementations
- `claude_journal.commands.yaml` — unilang command definitions
