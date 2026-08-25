# Web Viewing

**Status**: Partial | **Since**: 1.3.0

### Scope

- **Purpose**: Provide an embedded web dashboard for interactive journal viewing.
- **Responsibility**: Documents the `tiny-http`-served HTML dashboard, its JSON API endpoints, and its visualization components.
- **In Scope**: The `/`, `/api/events`, `/api/stats`, `/api/health` endpoints and the embedded, dependency-free HTML/JS app.
- **Out of Scope**: CLI command equivalents (→ `docs/feature/001_cli_viewing.md`), filter semantics shared across commands (→ `docs/feature/003_filtering.md`).

## Description

Embedded single-page web dashboard served by `tiny-http` (pure Rust, zero transitive C deps). The HTML is embedded in the binary via `include_str!()` — no external file dependencies. The dashboard provides the same data as the CLI commands but with interactive filtering and visualization.

The web server exposes three JSON API endpoints for dynamic data and one static endpoint for the HTML app:

| Path | Method | Response | Purpose | State |
|------|--------|----------|---------|-------|
| `/` | GET | HTML | Single-page dashboard application | Implemented |
| `/api/events` | GET | JSON array | Filtered event list (same query semantics as `.list`) | Implemented |
| `/api/stats` | GET | JSON object | Aggregated statistics (same semantics as `.stats`) | Implemented |
| `/api/health` | GET | JSON object | Journal health: file count, total bytes, oldest/newest dates | Implemented |

Routing splits the URL on `?` and matches the path exactly. Any other path
under `/api/` returns **404 with `{ "error" : "unknown endpoint '…'" }`**
rather than falling through to the dashboard — a typo'd endpoint fails loudly
instead of answering with a 200 and an HTML page. Every other path (including
unrecognised non-API ones) still serves the dashboard, so the catch-all is
scoped to `/api/`, not to the whole URL space.

**Shared computation, not parallel implementations.** The JSON endpoints and
the CLI commands are two renderings of the same data: `/api/events` and
`.list` both run `build_filter()`; `/api/stats` and `.stats` both run
`stats_data()`; `/api/health` and `.status` both run `health_data()`. The text
formatters take the same structs the JSON serializer does, so the web view and
the terminal view cannot drift apart in what they report.

**Query vocabulary.** `/api/events` and `/api/stats` accept the CLI filter
parameter names as query keys (`since`, `until`, `type`, `command`, `exit`,
`model`, `dir`, `creds`, `limit`; plus `by` for stats), percent-decoded with
`+` treated as space. An invalid value returns **400** with the same message
the CLI would print — an unparseable `since=banana` is a client error, not an
empty result set. An unrecognised *key* is likewise a **400**, for the same
reason the CLI rejects one: a filter key nothing reads returns the whole list
with HTTP 200, which is indistinguishable from a query that legitimately
matched everything. `/api/events` applies `limit=200` only when the query
supplies no limit of its own, so an empty query cannot stream an unbounded
journal.

The journal directory is deliberately *not* in this vocabulary. `journal_dir`
is a launch-time argument to the server process, never a per-request key — a
client that could repoint the reader could read any file tree the server user
can reach, which is the one thing `invariant/001_read_only.md` and the
loopback default exist to prevent. `dir` here means what it means everywhere
else: a substring filter over each event's own working directory.

Dashboard components:
- Filterable event table (columns match CLI `.list` output) — **partial**: the table ships with columns Time/Type/Cmd/Model/Exit/Cost/Dur and the API behind it accepts the full filter vocabulary, but there is no filter UI yet; the page requests `/api/events` with no query
- Daily cost bar chart (last 30 days) — **not implemented** (`.chart` renders an SVG to a file instead)
- Error class breakdown (pie/donut chart) — **not implemented**
- Auto-refresh toggle — **partial**: the interval is configurable via `refresh::` (default 10 s, `0` disables) and both the `setInterval` guard and the status-line label are rendered from that one value, but there is no in-page toggle control

The embedded HTML is vanilla JavaScript + CSS — no framework dependencies. Total embedded HTML target: under 20KB.

## Acceptance Criteria

- AC-001: `clj .serve` starts an HTTP server on `127.0.0.1` at an OS-assigned port (`port::` → `CLJ_PORT` → `0`) and prints `Listening on http://localhost:{port}` to stdout, flushed immediately so a piped reader can recover the chosen port
- AC-002: `clj .serve port::9090` overrides the port; `bind::` overrides the address (see INV-002). A non-loopback bind reports its real address in the startup line instead of `localhost` and warns on stderr
- AC-003: `GET /` returns the embedded HTML with Content-Type `text/html; charset=utf-8`
- AC-004: `GET /api/events?since=1h&type=execution` returns a filtered JSON array; an unparseable value returns 400 with the CLI's own error message
- AC-005: `GET /api/stats?by=model&since=7d` returns `{ "by", "column_label", "total_events", "groups" : [ { "key", "count", "cost_usd" } ] }`; an invalid `by` returns 400
- AC-006: `GET /api/health` returns `{ "files": N, "bytes": N, "oldest": "...", "newest": "..." }`, with `oldest`/`newest` as `null` (not a placeholder string) when the journal is empty
- AC-007: The HTML page renders correctly without external network access (no CDN dependencies)
- AC-008: Auto-refresh polls `/api/events` at the interval given by `refresh::` (default 10 s); `refresh::0` disables polling and the status line says so. A non-integer `refresh::` exits 1 at startup rather than silently falling back
- AC-009 (⏳ pending): Server shuts down cleanly on SIGTERM/SIGINT. `cmd_serve()` runs an unconditional `loop` with no signal handler, so shutdown today is the default signal disposition — the process terminates promptly but runs no cleanup. See "AC-009 blocker" below
- AC-010: `open::1` opens the default browser after the server starts. A failed launch (no browser, headless host) degrades to a stderr warning and never aborts the server — matching `.chart`'s treatment of the same failure

**AC-009 blocker.** Installing a SIGTERM/SIGINT handler needs either `libc::signal`,
which the workspace-wide `unsafe-code = "deny"` lint forbids, or a new
dependency (`ctrlc`/`signal-hook`) that the workspace does not currently carry.
Both are workspace policy decisions rather than crate-local implementation
choices, so AC-009 stays pending and this feature stays `Status: Partial`. The
practical gap is narrow: the accept loop holds no unflushed state, so the
default disposition already terminates without data loss — what is missing is
an explicit, testable graceful path.

## Sources

- `src/cli_main.rs` `cmd_serve()` — parameter resolution, bind/exposure reporting, and the `tiny_http` request loop
- `src/cli_main.rs` `respond_events`/`respond_stats`/`respond_health`/`respond_json` — the API route handlers
- `src/cli_main.rs` `INDEX_HTML_TEMPLATE` + `index_html()` — embedded dashboard HTML, a raw string literal with the refresh interval substituted in; there is no separate `src/web/` asset directory
- `src/output.rs` `build_filter`/`parse_query_string`/`stats_data`/`health_data` — the computation shared with the CLI commands
- `tests/serve_test.rs` — FT-1..FT-12 enforcement
