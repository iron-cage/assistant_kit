# Test: Feature — Web Viewing

### Scope

- **Purpose**: FT- test cases verifying the HTTP server's startup, HTML/API responses, port override, and shutdown behavior.
- **Responsibility**: Acceptance criteria confirming `.serve` starts correctly, serves embedded HTML and the one implemented JSON route, and marking the Phase 2 routes as untestable rather than claiming coverage.
- **In Scope**: `.serve` loopback bind and port resolution, `GET /` HTML response, `/api/events`, port override, CDN-free HTML, shutdown behavior.
- **Out of Scope**: CLI command behaviors (-> `001_cli_viewing.md`), detailed filter semantics (-> `003_filtering.md`), localhost-only invariant (-> `../invariant/002_localhost_only.md`).

Test case planning for [feature/002_web_viewing.md](../../../docs/feature/002_web_viewing.md).

The feature is `Status: Planned` and `cmd_serve()` implements a subset:
routing is a single `if url.starts_with( "/api/events" )` with an `else` that
serves the dashboard HTML (`src/cli_main.rs:185`). Consequently `/api/stats`
and `/api/health` return **HTTP 200 with `Content-Type: text/html`**, not a
404 and not JSON. Any case for those routes that asserts only on the status
code would pass against a page that is not the endpoint at all — which is why
FT-4 below asserts content type and is marked deferred rather than written.

## Test Case Index

| ID | Test Name | Category | Status |
|----|-----------|----------|--------|
| FT-1 | `.serve` starts on 127.0.0.1 at an OS-assigned port and prints the URL | Server Start | ✅ |
| FT-2 | `GET /` returns 200 with `Content-Type: text/html` and non-empty body | HTML Serve | ✅ |
| FT-3 | `GET /api/events` returns 200 with a JSON array | Events API | ✅ |
| FT-4 | `GET /api/health` returns the documented JSON structure | Health API | ⏳ Phase 2 |
| FT-5 | `.serve port::9090` starts server on port 9090 | Port Override | ✅ |
| FT-6 | Embedded HTML has no external CDN dependencies | Self-Contained | ✅ |
| FT-7 | `GET /api/events?since=1h&type=execution` honors the query string | Events Filtering | ⏳ Phase 2 |
| FT-8 | Server shuts down cleanly on SIGTERM (process exits; no zombie) | Shutdown | ✅ |

## Test Coverage Summary

- Server Start: 1 test (FT-1)
- HTML Serve: 1 test (FT-2)
- Events API: 1 test (FT-3)
- Health API: 1 test (FT-4, deferred)
- Port Override: 1 test (FT-5)
- Self-Contained: 1 test (FT-6)
- Events Filtering: 1 test (FT-7, deferred)
- Shutdown: 1 test (FT-8)

**Total:** 8 tests (6 executable, 2 blocked on Phase 2)

## Architectural Constraint

FT-1, FT-2, FT-3, FT-5, FT-8 require spawning `clj .serve journal_dir::<tmpdir>`
as a subprocess, reading the startup line from stdout, then making HTTP requests
with a short timeout, and finally killing the process. Note the parameter
syntax: this is a unilang CLI, so the journal directory is passed as
`dir::<dir>` — the form `run_clj` uses at
`tests/viewer_integration_test.rs:81` — not as a GNU-style `--journal-dir`
flag. Note that `docs/cli/param/21_journal_dir.md` documents a `journal_dir::`
spelling that no code reads; `dir::` is what `resolve_journal_dir` actually
consumes.

Because the port is OS-assigned unless pinned, the harness must parse it out of
the `Listening on http://localhost:{port}` line rather than assume a well-known
number. That line is explicitly flushed before the accept loop begins
(`src/cli_main.rs:177`) so a piped reader can recover it without racing.

FT-6 is a structural test: inspect the embedded HTML and assert it contains no
references to external CDN domains. The HTML is the `INDEX_HTML` raw string
literal at `src/cli_main.rs:18` — there is no `src/web/` asset directory, so the
test must read the Rust source (or assert against the served response body),
not a standalone `.html` file.

---

### FT-1: `.serve` starts on 127.0.0.1 at an OS-assigned port and prints the URL

- **Given:** temp journal dir; `CLJ_PORT` unset in the child environment
- **When:** `clj .serve dir::<dir>` started as a background process
- **Then:** within 3s, stdout contains `Listening on http://localhost:{port}` with a nonzero port; `GET http://127.0.0.1:{port}/` returns 200
- **Source:** [feature/002_web_viewing.md](../../../docs/feature/002_web_viewing.md) AC-001

---

### FT-2: `GET /` returns embedded HTML

- **Given:** `.serve` running on a known port
- **When:** `GET http://127.0.0.1:{port}/`
- **Then:** HTTP 200; `Content-Type` header is `text/html; charset=utf-8`; response body length > 0
- **Source:** [feature/002_web_viewing.md](../../../docs/feature/002_web_viewing.md) AC-003

---

### FT-3: `GET /api/events` returns JSON array

- **Given:** journal dir with 3 events; `.serve` running
- **When:** `GET http://127.0.0.1:{port}/api/events`
- **Then:** HTTP 200; `Content-Type: application/json`; body parses as a JSON array; array length == 3. The route applies `JournalFilter { limit : Some( 200 ), ..default() }` unconditionally, so this holds for any fixture under 200 events
- **Source:** [feature/002_web_viewing.md](../../../docs/feature/002_web_viewing.md) AC-004

---

### FT-4: `GET /api/health` returns the documented JSON structure — ⏳ Phase 2

- **Given:** journal dir with 2 files; `.serve` running
- **When:** `GET http://127.0.0.1:{port}/api/health`
- **Then:** HTTP 200; JSON object containing keys `"files"` (number), `"bytes"` (number), `"oldest"` (string or null), `"newest"` (string or null)
- **Blocked on:** the route not existing. The request falls through to the `else` branch and returns the dashboard HTML with HTTP 200, so a status-code-only assertion would pass against the wrong response entirely. Same applies to `/api/stats` (AC-005)
- **Source:** [feature/002_web_viewing.md](../../../docs/feature/002_web_viewing.md) AC-006

---

### FT-5: `.serve port::9090` starts on port 9090

- **Given:** temp journal dir; port 9090 available
- **When:** `clj .serve port::9090 dir::<dir>`
- **Then:** startup line reports 9090; `GET http://127.0.0.1:9090/` returns 200
- **Source:** [feature/002_web_viewing.md](../../../docs/feature/002_web_viewing.md) AC-002

---

### FT-6: Embedded HTML has no external CDN dependencies

- **Given:** the `INDEX_HTML` literal (`src/cli_main.rs:18`), or the body returned by `GET /`
- **When:** scan for external CDN domains: `cdn.jsdelivr.net`, `unpkg.com`, `cdnjs.cloudflare.com`, `ajax.googleapis.com`
- **Then:** none of the CDN patterns appear; all styling and script content is inline
- **Source:** [feature/002_web_viewing.md](../../../docs/feature/002_web_viewing.md) AC-007

---

### FT-7: `GET /api/events?since=1h&type=execution` honors the query string — ⏳ Phase 2

- **Given:** journal dir with a mix of event types and timestamps; `.serve` running
- **When:** `GET http://127.0.0.1:{port}/api/events?since=1h&type=execution`
- **Then:** the returned array contains only `execution` events from the last hour
- **Blocked on:** query-string parsing not existing. `url` is matched with `starts_with`, so the query is retained in the string but never parsed — the response is the same unfiltered newest-200 array regardless of what is passed
- **Source:** [feature/002_web_viewing.md](../../../docs/feature/002_web_viewing.md) AC-004

---

### FT-8: Server shuts down cleanly on SIGTERM

- **Given:** `.serve` running as a background subprocess; PID known
- **When:** `kill -TERM <pid>`
- **Then:** process exits within 5s; no zombie process remains; exit status is signal-terminated, not hung
- **Note:** `cmd_serve()` runs an unconditional `loop` with no signal handler, so this passes via the default SIGTERM disposition rather than via graceful shutdown logic. AC-009's "cleanly" is therefore only satisfied in the weak sense of "does not hang" — assert termination, not a zero exit status
- **Source:** [feature/002_web_viewing.md](../../../docs/feature/002_web_viewing.md) AC-009
