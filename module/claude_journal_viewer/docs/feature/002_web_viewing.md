# Web Viewing

**Status**: Planned | **Since**: 1.3.0

### Scope

- **Purpose**: Provide an embedded web dashboard for interactive journal viewing.
- **Responsibility**: Documents the `tiny-http`-served HTML dashboard, its JSON API endpoints, and its visualization components.
- **In Scope**: The `/`, `/api/events`, `/api/stats`, `/api/health` endpoints and the embedded, dependency-free HTML/JS app.
- **Out of Scope**: CLI command equivalents (→ `docs/feature/001_cli_viewing.md`), filter semantics shared across commands (→ `docs/feature/003_filtering.md`).

## Description

Embedded single-page web dashboard served by `tiny-http` (pure Rust, zero transitive C deps). The HTML is embedded in the binary via `include_str!()` — no external file dependencies. The dashboard provides the same data as the CLI commands but with interactive filtering and visualization.

The web server is targeted to expose three JSON API endpoints for dynamic data and one static endpoint for the HTML app:

| Path | Method | Response | Purpose | State |
|------|--------|----------|---------|-------|
| `/` | GET | HTML | Single-page dashboard application | Implemented |
| `/api/events` | GET | JSON array | Filtered event list (same query semantics as `.list`) | Partial — route exists, query string ignored |
| `/api/stats` | GET | JSON object | Aggregated statistics (same semantics as `.stats`) | Not implemented |
| `/api/health` | GET | JSON object | Journal health: file count, total bytes, oldest/newest dates | Not implemented |

Routing is a single `if url.starts_with( "/api/events" )` with an `else` that
serves the HTML (`src/cli_main.rs:185`), so `/api/stats` and `/api/health` do
not 404 — they silently return the dashboard page with
`Content-Type: text/html`. Any test asserting those endpoints must assert on
the body or content type, never on the status code alone.

Dashboard components:
- Filterable event table (columns match CLI `.list` output) — **partial**: the table ships with columns Time/Type/Cmd/Model/Exit/Cost/Dur, but there is no filter UI; `/api/events` returns the newest 200 events unconditionally (`JournalFilter { limit : Some( 200 ), ..default() }`)
- Daily cost bar chart (last 30 days) — **not implemented** (`.chart` renders an SVG to a file instead)
- Error class breakdown (pie/donut chart) — **not implemented**
- Auto-refresh toggle — **not implemented**: `setInterval( load, 5000 )` is hardcoded in `INDEX_HTML`; there is no toggle, the interval is 5 s rather than the targeted 10 s, and `refresh::` is not read by `cmd_serve()`

The embedded HTML is vanilla JavaScript + CSS — no framework dependencies. Total embedded HTML target: under 20KB.

## Acceptance Criteria

- AC-001: `clj .serve` starts an HTTP server on `127.0.0.1` at an OS-assigned port (`port::` → `CLJ_PORT` → `0`) and prints `Listening on http://localhost:{port}` to stdout, flushed immediately so a piped reader can recover the chosen port
- AC-002: `clj .serve port::9090` overrides the port. Overriding the bind address is deferred — `bind::` is a Phase 2 deliverable that `cmd_serve()` does not yet read (see INV-002, `Status: Planned`), so `.serve` is loopback-only today
- AC-003: `GET /` returns the embedded HTML with Content-Type `text/html; charset=utf-8`
- AC-004 (⏳ pending): `GET /api/events?since=1h&type=execution` returns a filtered JSON array. Today the route returns the newest 200 events with the query string discarded — the filter arguments are not parsed at all
- AC-005 (⏳ pending): `GET /api/stats?by=model&since=7d` returns grouped statistics JSON. No such route exists; the request falls through to the HTML branch
- AC-006 (⏳ pending): `GET /api/health` returns `{ "files": N, "bytes": N, "oldest": "...", "newest": "..." }`. No such route exists; the request falls through to the HTML branch
- AC-007: The HTML page renders correctly without external network access (no CDN dependencies)
- AC-008 (⏳ pending): Auto-refresh polls `/api/events` at the *configured* interval. It currently polls at a hardcoded 5 s and `refresh::` is not read
- AC-009 (⏳ pending): Server shuts down cleanly on SIGTERM/SIGINT. `cmd_serve()` runs an unconditional `loop` with no signal handler and no break condition, so shutdown today is whatever the default signal disposition does
- AC-010 (⏳ pending): `open::1` opens the default browser after server starts. `cmd_serve()` never reads an `open` key and never spawns a browser

**Implementation state:** `cmd_serve()` reads exactly one parameter — `port`.
`bind::`, `open::`, and `refresh::` are documented but unread, so they are
accepted-and-ignored rather than rejected. AC-001, AC-002 (port half), AC-003,
and AC-007 hold today; the six marked ⏳ are the remaining Phase 2 scope and
are why this feature is `Status: Planned`.

## Sources

- `src/cli_main.rs` `cmd_serve()` — HTTP server implementation (inline `tiny_http` request loop, no separate serve module)
- `src/cli_main.rs` `INDEX_HTML` (line 18) — embedded dashboard HTML, a raw string literal in-source; there is no separate `src/web/` asset directory
