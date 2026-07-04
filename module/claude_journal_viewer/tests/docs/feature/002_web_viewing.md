# Test: Feature — Web Viewing

Test case planning for [feature/002_web_viewing.md](../../../docs/feature/002_web_viewing.md). Tests validate HTTP server startup, default bind address and port, HTML embedding, `/api/events` filtering, `/api/health` response structure, and SIGTERM shutdown.

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| FT-1 | `.serve` starts server on `127.0.0.1:8411` and prints URL to stdout | Server Start |
| FT-2 | `GET /` returns 200 with `Content-Type: text/html` and non-empty body | HTML Serve |
| FT-3 | `GET /api/events` returns 200 with JSON array | Events API |
| FT-4 | `GET /api/health` returns expected JSON structure | Health API |
| FT-5 | `.serve port::9090` starts server on port 9090 | Port Override |
| FT-6 | HTML renders without external network requests (no CDN links in source) | Self-Contained |
| FT-7 | Server shuts down cleanly on SIGTERM (process exits; no zombie) | Shutdown |

## Test Coverage Summary

- Server Start: 1 test (FT-1)
- HTML Serve: 1 test (FT-2)
- Events API: 1 test (FT-3)
- Health API: 1 test (FT-4)
- Port Override: 1 test (FT-5)
- Self-Contained: 1 test (FT-6)
- Shutdown: 1 test (FT-7)

**Total:** 7 tests

## Architectural Constraint

FT-1, FT-2, FT-3, FT-4, FT-5, FT-7 require spawning `clj .serve --journal-dir <tmpdir>` as a subprocess, waiting for the startup line on stdout, then making HTTP requests with a short timeout, and finally killing the process.

FT-6 is a structural test: inspect the embedded HTML source (`src/web/index.html`) and assert it contains no references to external CDN domains (e.g., `cdn.jsdelivr.net`, `unpkg.com`, `cdnjs.cloudflare.com`).

---

### FT-1: `.serve` starts on `127.0.0.1:8411` and prints URL

- **Given:** temp journal dir
- **When:** `clj .serve --journal-dir <dir>` started as background process
- **Then:** within 3s, stdout contains `127.0.0.1:8411` or `http://127.0.0.1:8411`; `GET http://127.0.0.1:8411/` returns 200
- **Source:** [feature/002_web_viewing.md](../../../docs/feature/002_web_viewing.md) AC-001

---

### FT-2: `GET /` returns embedded HTML

- **Given:** `.serve` running on port 8411
- **When:** `GET http://127.0.0.1:8411/`
- **Then:** HTTP 200; `Content-Type` header contains `text/html`; response body length > 0
- **Source:** [feature/002_web_viewing.md](../../../docs/feature/002_web_viewing.md) AC-003

---

### FT-3: `GET /api/events` returns JSON array

- **Given:** journal dir with 3 events; `.serve` running
- **When:** `GET http://127.0.0.1:8411/api/events`
- **Then:** HTTP 200; response body parses as JSON array; array length == 3
- **Source:** [feature/002_web_viewing.md](../../../docs/feature/002_web_viewing.md) AC-004

---

### FT-4: `GET /api/health` returns expected structure

- **Given:** journal dir with 2 files; `.serve` running
- **When:** `GET http://127.0.0.1:8411/api/health`
- **Then:** HTTP 200; JSON object; contains keys `"files"` (number), `"bytes"` (number), `"oldest"` (string or null), `"newest"` (string or null)
- **Source:** [feature/002_web_viewing.md](../../../docs/feature/002_web_viewing.md) AC-006

---

### FT-5: `.serve port::9090` starts on port 9090

- **Given:** temp journal dir; port 9090 available
- **When:** `clj .serve port::9090 --journal-dir <dir>`
- **Then:** `GET http://127.0.0.1:9090/` returns 200; port 8411 not used
- **Source:** [feature/002_web_viewing.md](../../../docs/feature/002_web_viewing.md) AC-002

---

### FT-6: Embedded HTML has no external CDN dependencies

- **Given:** `src/web/index.html` read as a string
- **When:** scan for external CDN domains: `cdn.jsdelivr.net`, `unpkg.com`, `cdnjs.cloudflare.com`, `ajax.googleapis.com`
- **Then:** none of the CDN patterns appear in the HTML source
- **Source:** [feature/002_web_viewing.md](../../../docs/feature/002_web_viewing.md) AC-007

---

### FT-7: Server shuts down cleanly on SIGTERM

- **Given:** `.serve` running as a background subprocess; PID known
- **When:** `kill -TERM <pid>`
- **Then:** process exits within 5s; no zombie process remains; exit status is 0 or signal-terminated (not hung)
- **Source:** [feature/002_web_viewing.md](../../../docs/feature/002_web_viewing.md) AC-009
