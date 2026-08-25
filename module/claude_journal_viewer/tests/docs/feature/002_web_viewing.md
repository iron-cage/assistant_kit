# Test: Feature — Web Viewing

### Scope

- **Purpose**: FT- test cases verifying the HTTP server's startup, HTML/API responses, routing, parameter handling, and shutdown behavior.
- **Responsibility**: Acceptance criteria confirming `.serve` starts correctly, serves embedded HTML and all three JSON routes, rejects bad input with 400/404 rather than a misleading 200, and honors `port::`/`refresh::`/`open::`.
- **In Scope**: `.serve` loopback bind and port resolution, `GET /` HTML response, `/api/events`, `/api/stats`, `/api/health`, unknown-`/api/*` 404, query-string filtering and its error path, `refresh::` substitution, `open::` failure tolerance, CDN-free HTML, shutdown behavior.
- **Out of Scope**: CLI command behaviors (-> `001_cli_viewing.md`), detailed filter semantics (-> `003_filtering.md`), localhost-only invariant (-> `../invariant/002_localhost_only.md`).

Test case planning for [feature/002_web_viewing.md](../../../docs/feature/002_web_viewing.md).

All cases live in `tests/serve_test.rs`, which owns every `.serve` case in the
crate. Routing splits the URL on `?` and matches the path exactly, so
`/api/stats` and `/api/health` are real routes and an unknown `/api/*` path is
a 404 with a JSON body. Cases may therefore assert on status codes — but each
one below still asserts content type as well, because a status-only assertion
would have passed against the pre-Phase-2 catch-all that answered every path
with the dashboard page.

## Test Case Index

| ID | Test Name | Category | Status |
|----|-----------|----------|--------|
| FT-1 | `.serve` starts on 127.0.0.1 at an OS-assigned port and prints the URL | Server Start | ✅ |
| FT-2 | `GET /` returns 200 with `Content-Type: text/html` and non-empty body | HTML Serve | ✅ |
| FT-3 | `GET /api/events` returns 200 with a JSON array | Events API | ✅ |
| FT-4 | `GET /api/health` returns the documented JSON structure | Health API | ✅ |
| FT-4b | `GET /api/health` on an empty journal reports `files: 0` and null dates | Health API | ✅ |
| FT-5 | `.serve port::N` starts server on the pinned port | Port Override | ✅ |
| FT-6 | Embedded HTML has no external CDN dependencies | Self-Contained | ✅ |
| FT-7 | `GET /api/events?since=1h&type=execution` honors the query string | Events Filtering | ✅ |
| FT-7b | `GET /api/events?since=banana` returns 400 naming the bad value | Events Filtering | ✅ |
| FT-7c | An unrecognised query *key* returns 400 naming the key | Events Filtering | ✅ |
| FT-8 | Server shuts down cleanly on SIGTERM (process exits; no zombie) | Shutdown | ✅ |
| FT-9 | `GET /api/stats?by=model&since=7d` returns grouped statistics JSON | Stats API | ✅ |
| FT-9b | `GET /api/stats?by=banana` returns 400 naming the bad grouping | Stats API | ✅ |
| FT-10 | Unknown `/api/*` returns 404 JSON while non-API paths still serve HTML | Routing | ✅ |
| FT-11 | `refresh::` drives both the poll interval and the status-line label | Refresh | ✅ |
| FT-11b | A non-integer `refresh::` exits 1 with an explanatory stderr message | Refresh | ✅ |
| FT-12 | `open::1` warns but does not abort when no browser is available | Browser Open | ✅ |
| FT-13 | SIGTERM and SIGINT each exit 0 with `Shutting down`, from an idle server | Shutdown | ✅ |
| FT-14 | `port::` and `open::` are validated before the socket is bound | Param Validation | ✅ |

## Test Coverage Summary

- Server Start: 1 test (FT-1)
- HTML Serve: 1 test (FT-2)
- Events API: 1 test (FT-3)
- Health API: 2 tests (FT-4, FT-4b)
- Port Override: 1 test (FT-5)
- Self-Contained: 1 test (FT-6)
- Events Filtering: 3 tests (FT-7, FT-7b, FT-7c)
- Shutdown: 2 tests (FT-8, FT-13)
- Stats API: 2 tests (FT-9, FT-9b)
- Routing: 1 test (FT-10)
- Refresh: 2 tests (FT-11, FT-11b)
- Param Validation: 1 test (FT-14)
- Browser Open: 1 test (FT-12)

**Total:** 19 tests (all executable)

FT-8 and FT-13 are deliberately separate and must stay that way. FT-8 asserts
only that the process *terminates* — it would pass against a `kill -9` and
against the default signal disposition. FT-13 asserts it terminates *cleanly*,
and the two claims fail differently: collapsing them into one case would lose
the ability to tell "the server hung" from "the server died without running its
exit path".

FT-13 kills an idle server on purpose. A `recv`-based accept loop blocks inside
the syscall until the next connection arrives, so issuing a request first would
unstick that broken implementation and the case would pass against code that
ignores the signal on a quiet server — which is every real deployment waiting
for its first visitor.

## Architectural Constraint

Every case spawns `clj .serve journal_dir::<tmpdir>` as a subprocess via the
shared `serve()` harness, reads the startup line from stdout, then speaks
HTTP/1.0 over a plain `TcpStream`. Note the parameter syntax: this is a unilang
CLI, so the journal directory is passed as `journal_dir::<dir>`, not as a
GNU-style `--journal-dir` flag. `dir::` is a different parameter entirely — a
substring filter over each event's own working directory (see
`docs/cli/param/07_dir.md`) — and is not accepted by `.serve` at all.

Because the port is OS-assigned unless pinned, the harness parses it out of the
`Listening on http://{host}:{port}` line rather than assuming a well-known
number. That line is explicitly flushed before the accept loop begins so a
piped reader can recover it without racing. The `Serve` harness kills its child
on `Drop`, so a panicking assertion cannot leak a listening server into the
rest of the suite.

The harness redirects the child's stderr to a file rather than a pipe: the
cases that assert on warnings read it after the fact, and an unread pipe would
deadlock the child once its buffer filled.

FT-6 is a structural test: it scans the *served* body for external CDN domains
rather than reading the Rust source, so it stays valid if the HTML ever moves
out of the `INDEX_HTML_TEMPLATE` literal.

FT-11 asserts on the rendered `setInterval(load,NNNNN)` call and the
status-line label together. Asserting only one would let the page advertise a
cadence it is not running.

---

### FT-1: `.serve` starts on 127.0.0.1 at an OS-assigned port and prints the URL

- **Given:** temp journal dir; `CLJ_PORT` unset in the child environment
- **When:** `clj .serve journal_dir::<dir>` started as a background process
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

### FT-4: `GET /api/health` returns the documented JSON structure

- **Given:** journal dir with fixture events; `.serve` running
- **When:** `GET http://127.0.0.1:{port}/api/health`
- **Then:** HTTP 200; `Content-Type: application/json`; JSON object with `"files"` ≥ 1 (number), `"bytes"` > 0 (number), `"oldest"` (string), `"newest"` (string)
- **Source:** [feature/002_web_viewing.md](../../../docs/feature/002_web_viewing.md) AC-006

---

### FT-4b: `GET /api/health` on an empty journal reports null dates

- **Given:** an empty temp journal dir (no fixture written); `.serve` running
- **When:** `GET http://127.0.0.1:{port}/api/health`
- **Then:** HTTP 200; `"files"` is `0`; `"oldest"` and `"newest"` are JSON `null`, not a `"(none)"` placeholder — so a consumer distinguishes empty from populated without string-matching
- **Source:** [feature/002_web_viewing.md](../../../docs/feature/002_web_viewing.md) AC-006

---

### FT-5: `.serve port::N` starts on the pinned port

- **Given:** temp journal dir; port 19090 available
- **When:** `clj .serve port::19090 journal_dir::<dir>`
- **Then:** startup line reports 19090; `GET http://127.0.0.1:19090/` returns 200
- **Note:** the port is deliberately in the high ephemeral range and distinct from the ports IN-2/IN-3 pin, so cases running concurrently under nextest cannot collide
- **Source:** [feature/002_web_viewing.md](../../../docs/feature/002_web_viewing.md) AC-002

---

### FT-6: Embedded HTML has no external CDN dependencies

- **Given:** the body returned by `GET /`
- **When:** scan for external CDN domains: `cdn.jsdelivr.net`, `unpkg.com`, `cdnjs.cloudflare.com`, `ajax.googleapis.com`, `//fonts.googleapis.com`
- **Then:** none of the CDN patterns appear; all styling and script content is inline
- **Source:** [feature/002_web_viewing.md](../../../docs/feature/002_web_viewing.md) AC-007

---

### FT-7: `GET /api/events?…` honors the query string

- **Given:** journal dir with 4 fixture events — 2 `execution`, 1 `credential`, 1 `retry`; `.serve` running
- **When:** `GET /api/events?since=1h&type=execution`, then `GET /api/events?limit=1`
- **Then:** the first returns exactly the 2 `execution` events and every element's `"type"` is `"execution"`; the second returns exactly 1 element
- **Note:** the `limit=1` half is what proves the whole query map reaches `build_filter` — a route that read only `type` would pass the first assertion alone
- **Source:** [feature/002_web_viewing.md](../../../docs/feature/002_web_viewing.md) AC-004

---

### FT-7b: `GET /api/events?since=banana` returns 400

- **Given:** `.serve` running
- **When:** `GET /api/events?since=banana`
- **Then:** HTTP 400; `Content-Type: application/json`; body is `{ "error" : "…" }` and the message names `banana` — an unparseable filter is a client error, never a silently empty array
- **Source:** [feature/002_web_viewing.md](../../../docs/feature/002_web_viewing.md) AC-004

---

### FT-7c: An unrecognised query key returns 400

- **Given:** `.serve` running against a populated journal
- **When:** `GET /api/events?exit_code=2`, `GET /api/events?journal_dir=/etc`, and `GET /api/stats?by=model&bogus=1`
- **Then:** each is HTTP 400 with a JSON body naming the offending key. `journal_dir` is checked explicitly because it must never be settable per-request — a client able to repoint the reader could reach any tree the server process can. The same case then asserts `?exit=0&limit=5` and `?by=model` still return 200, so the guard rejects keys rather than queries
- **Source:** [feature/002_web_viewing.md](../../../docs/feature/002_web_viewing.md) AC-004, [invariant/001_read_only.md](../../../docs/invariant/001_read_only.md)

An unread filter key would return HTTP 200 with the *entire* list. That is the
same failure the CLI's parameter rejection closes, and it is the reason this
case asserts on a specific key name rather than merely on the status code.

---

### FT-8: Server terminates on SIGTERM

- **Given:** `.serve` running as a background subprocess; PID known
- **When:** `kill -TERM <pid>`
- **Then:** process exits within 5s; no zombie process remains
- **Note:** this case asserts termination only — deliberately weaker than FT-13, and kept that way so a hang and an ungraceful death remain distinguishable failures. Do not add an exit-status assertion here; that is FT-13's job
- **Source:** [feature/002_web_viewing.md](../../../docs/feature/002_web_viewing.md) AC-009

---

### FT-13: SIGTERM and SIGINT shut the server down gracefully

- **Given:** `.serve` running as a background subprocess, idle — no request has ever been issued to it
- **When:** `kill -TERM <pid>`, and separately `kill -INT <pid>`, against a fresh server each time
- **Then:** each exits within 5s with exit status exactly `0`, having written `Shutting down` to stderr
- **Note:** `Some( 0 )` is the discriminator. A signal-killed process carries no exit code at all, so `ExitStatus::code()` is `None` — this assertion cannot pass under the default disposition FT-8 tolerates. Verified against `kill -KILL`, which no handler can intercept and which yields 137 with no message
- **Note:** the server is idle by design; see the reasoning under Test Coverage Summary
- **Source:** [feature/002_web_viewing.md](../../../docs/feature/002_web_viewing.md) AC-009

---

### FT-9: `GET /api/stats?by=model&since=7d` returns grouped statistics

- **Given:** journal dir with fixture events across 2 models; `.serve` running
- **When:** `GET /api/stats?by=model&since=7d`, then `GET /api/stats` with no query
- **Then:** the first returns HTTP 200 JSON with `"by": "model"`, `"column_label": "MODEL"`, `"total_events": 4`, and a `"groups"` array whose `claude-sonnet-5` entry has `"count": 1` and `"cost_usd": 0.012`; the second returns `"by": "day"`
- **Note:** the no-query request is the discriminator — its different default grouping means an ignored query string could not produce the first response
- **Source:** [feature/002_web_viewing.md](../../../docs/feature/002_web_viewing.md) AC-005

---

### FT-9b: `GET /api/stats?by=banana` returns 400

- **Given:** `.serve` running
- **When:** `GET /api/stats?by=banana`
- **Then:** HTTP 400 JSON; the error message names `banana` and matches what `.stats by::banana` prints
- **Source:** [feature/002_web_viewing.md](../../../docs/feature/002_web_viewing.md) AC-005

---

### FT-10: Unknown `/api/*` returns 404 JSON; non-API paths still serve HTML

- **Given:** `.serve` running
- **When:** `GET /api/nonsense`, then `GET /whatever`
- **Then:** the first returns HTTP 404 with `Content-Type: application/json` and an `"error"` naming `/api/nonsense`; the second returns HTTP 200 with the dashboard body
- **Note:** the second half pins the scope of the catch-all. Making *every* unrecognised path a 404 would break the dashboard's own routing story; making none of them 404 is the pre-Phase-2 bug this case exists to prevent regressing
- **Source:** [feature/002_web_viewing.md](../../../docs/feature/002_web_viewing.md) — routing description

---

### FT-11: `refresh::` drives the poll interval and its label

- **Given:** `.serve` started three times — no `refresh::`, `refresh::30`, and `refresh::0`
- **When:** `GET /` for each
- **Then:** the bodies contain `auto-refresh 10s` + `setInterval(load,10000)`, `auto-refresh 30s` + `setInterval(load,30000)`, and `auto-refresh off` + an unarmed `if(0>0)` guard respectively
- **Source:** [feature/002_web_viewing.md](../../../docs/feature/002_web_viewing.md) AC-008

---

### FT-11b: A non-integer `refresh::` exits 1

- **Given:** temp journal dir
- **When:** `clj .serve refresh::soon journal_dir::<dir> port::0` run to completion
- **Then:** exit code 1; stderr contains `Error: invalid integer 'soon' for parameter 'refresh'`. The server never binds — a typo'd interval fails at startup rather than silently reverting to the default
- **Source:** [feature/002_web_viewing.md](../../../docs/feature/002_web_viewing.md) AC-008

---

### FT-12: `open::1` warns but does not abort when no browser is available

- **Given:** temp journal dir; a container with no desktop environment, so the browser launch necessarily fails
- **When:** `clj .serve open::1 journal_dir::<dir> port::0`, then `GET /`
- **Then:** the server still starts and answers 200 — the failed launch degrades to a stderr warning, matching how `.chart open::1` treats the same failure
- **Note:** the successful-launch path cannot be asserted headlessly. This case covers the failure path, which is the one that could take the server down
- **Source:** [feature/002_web_viewing.md](../../../docs/feature/002_web_viewing.md) AC-010

---

### FT-14: `port::` and `open::` are validated before the socket is bound

- **Given:** temp journal dir; `CLJ_PORT` cleared from the environment so it cannot supply a competing value
- **When:** `clj .serve` is run once per bad value — `port::` in `99999`, `abc`, `-1`, `65536`, and `open::` in `true`, `banana`, `2`
- **Then:** every run exits 1 with the documented message for its type — `Error: invalid integer '<v>' for parameter 'port'` and `invalid boolean '<v>' for parameter 'open' — expected 0 or 1` — and nothing binds
- **And:** `port::65535 bind::not-an-address` exits 1 at the *bind* with `could not start server on not-an-address:65535`, which is what proves the ceiling passed validation rather than being rejected as a value
- **Note:** the positive control pairs the ceiling with an unresolvable `bind::` deliberately. Handing `port::65535` to a real bind would start a server, and the case would then hang on its own success — `Command::output()` waits for a process that is doing exactly what it was asked to do. This is not hypothetical; it is the shape the case had first, and it stalled the suite for seven minutes before the pairing replaced it
- **Note:** before this case, `port::` resolved through `.unwrap_or( 0 )`. `port::99999` did not fail — it bound an OS-assigned port and reported that port on the startup line, so the server came up on the wrong port and said so in a line most callers do not re-read
- **Source:** [feature/002_web_viewing.md](../../../docs/feature/002_web_viewing.md) AC-008, AC-010; [param/15_port.md](../../../docs/cli/param/15_port.md), [type/08_boolean.md](../../../docs/cli/type/08_boolean.md)
