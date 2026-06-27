# Web Viewing

**Status**: Planned | **Since**: 1.3.0

## Description

Embedded single-page web dashboard served by `tiny-http` (pure Rust, zero transitive C deps). The HTML is embedded in the binary via `include_str!()` — no external file dependencies. The dashboard provides the same data as the CLI commands but with interactive filtering and visualization.

The web server exposes three JSON API endpoints for dynamic data and one static endpoint for the HTML app:

| Path | Method | Response | Purpose |
|------|--------|----------|---------|
| `/` | GET | HTML | Single-page dashboard application |
| `/api/events` | GET | JSON array | Filtered event list (same query semantics as `.list`) |
| `/api/stats` | GET | JSON object | Aggregated statistics (same semantics as `.stats`) |
| `/api/health` | GET | JSON object | Journal health: file count, total bytes, oldest/newest dates |

Dashboard components:
- Filterable event table (columns match CLI `.list` output)
- Daily cost bar chart (last 30 days)
- Error class breakdown (pie/donut chart)
- Auto-refresh toggle (default 10-second interval, configurable via `refresh::` param)

The embedded HTML is vanilla JavaScript + CSS — no framework dependencies. Total embedded HTML target: under 20KB.

## Acceptance Criteria

- AC-001: `clj .serve` starts HTTP server on `127.0.0.1:8411` and prints the URL to stdout
- AC-002: `clj .serve port::9090 bind::0.0.0.0` overrides port and bind address
- AC-003: `GET /` returns the embedded HTML with Content-Type `text/html`
- AC-004: `GET /api/events?since=1h&type=execution` returns filtered JSON array
- AC-005: `GET /api/stats?by=model&since=7d` returns grouped statistics JSON
- AC-006: `GET /api/health` returns `{ "files": N, "bytes": N, "oldest": "...", "newest": "..." }`
- AC-007: The HTML page renders correctly without external network access (no CDN dependencies)
- AC-008: Auto-refresh polls `/api/events` at the configured interval
- AC-009: Server shuts down cleanly on SIGTERM/SIGINT (no zombie processes)
- AC-010: `open::1` opens the default browser after server starts

## Sources

- `src/cli/serve.rs` — HTTP server implementation
- `src/web/index.html` — embedded dashboard HTML
