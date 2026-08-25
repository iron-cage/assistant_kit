# CLI Viewing

**Status**: Planned | **Since**: 1.3.0

### Scope

- **Purpose**: Provide CLI commands for exploring and managing journal data via unilang syntax.
- **Responsibility**: Documents the 9 `.command param::value` commands, their default output, and their dual standalone/super-app registration.
- **In Scope**: Command list and purpose, default `.list` table columns, and `clj`/`ast .journal.*` dual dispatch.
- **Out of Scope**: Web dashboard viewing (→ `docs/feature/002_web_viewing.md`), filter semantics shared across commands (→ `docs/feature/003_filtering.md`).

## Description

Nine CLI commands for exploring journal data using unilang `.command param::value` syntax. Commands are registered via `claude_journal.commands.yaml` and dispatched through the unilang `CommandRegistry`.

| # | Command | Purpose |
|---|---------|---------|
| 1 | `.list` | Display filtered event table with configurable sort and output format |
| 2 | `.tail` | Follow journal events in real-time (like `tail -f`) |
| 3 | `.stats` | Aggregate statistics grouped by day, model, dir, or agent |
| 4 | `.search` | Literal substring search across the prompt and the captured output |
| 5 | `.serve` | Start embedded HTTP server for web-based viewing |
| 6 | `.prune` | Delete journal files older than an age window (filename date) |
| 7 | `.status` | Show journal health: file count, total size, date range, config |
| 8 | `.export` | Export filtered events to file in table/json/csv/jsonl format |
| 9 | `.chart` | Render a usage SVG chart, optionally opened in the browser |

Default output (`.list`) renders a compact table with columns: TIME, CMD, MODEL, EXIT, COST, IN/OUT, DUR, TYPE. Cost data is extracted from the `cost_usd`, `input_tokens`, `output_tokens` fields. Duration is formatted as human-readable seconds.

The `clj` binary operates standalone and also registers its commands into the `assistant` super-app as `ast .journal.list`, `ast .journal.stats`, etc.

## Acceptance Criteria

- AC-001: All 9 commands are registered and dispatch correctly via unilang CommandRegistry
- AC-002: `.list` with no params shows the 50 most recent events in table format
- AC-003: `.list since::1h type::execution` applies time + type filter as AND conditions
- AC-004: `.tail` blocks and emits new events as they are appended to the journal
- AC-005: `.stats` without params shows daily aggregates for the last 7 days
- AC-006: `.search pattern::"rate limit"` matches the literal substring against six fields — `message`, `stdout`, `stderr`, `error_message`, `model`, `command`. There is no regex engine: metacharacters are literal, and a regex-shaped pattern exits 0 having matched nothing rather than reporting itself as unusable
- AC-007: `.prune keep::30d` deletes journal files whose `YYYY-MM-DD.jsonl` filename date is older than 30 days (today's file never deleted; `dry_run::1` previews; no confirmation prompt)
- AC-008: `.status` reports file count, total bytes, oldest/newest dates, and configured journal dir
- AC-009: `.export format::csv since::7d output::/tmp/events.csv` writes filtered events to file
- AC-010: Standalone `clj` binary and `ast .journal.*` super-app routing both work
- AC-011: `.stats by::dir` and `.stats by::agent` render count-ranked rows (descending); events missing the field aggregate under a visible `(no dir)` / `(no agent)` row; invalid `by` values exit 1 listing `day, model, dir, agent` (task 543)

## Sources

- `src/cli_main.rs` — `clj` binary: arg parsing, command dispatch, `.tail`/`.serve` loops, help text
- `src/output.rs` — shared command output logic (all command bodies)
- `src/routines.rs` — unilang routine adapters for `ast .journal.*`
- `claude_journal.commands.yaml` — unilang command definitions
