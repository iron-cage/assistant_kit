# Test: `.serve`

### Scope

- **Purpose**: Verify `.serve` starts the embedded web viewer on loopback at the resolved port, and pin the accepted-and-ignored behavior of its three unwired parameters.
- **Responsibility**: Test case coverage for the one `.serve` parameter that is read (`port`), the three that are not (`bind`, `open`, `refresh`), and bind-failure handling.
- **In Scope**: Port resolution, loopback bind, unwired-parameter tolerance, bind failure on a pinned port.
- **Out of Scope**: Dashboard content rendering (-> `../../feature/002_web_viewing.md`), API route behavior (-> same).

Test case planning for [command/05_serve.md](../../../../docs/cli/command/05_serve.md).

`cmd_serve()` reads exactly one parameter. `bind::`, `open::`, and `refresh::`
are documented Phase 2 deliverables that are never queried from the parameter
map, so they are silently ignored rather than rejected. IT-3 pins that
tolerance deliberately: the value in asserting it is that a future wiring
commit must consciously update this case rather than discover the gap in
production.

## Test Case Index

| ID | Test Name | Category | Status |
|----|-----------|----------|--------|
| IT-1 | No args -> serves on 127.0.0.1 at an OS-assigned port | Default | ✅ |
| IT-2 | `port::9090` -> serves on custom port | Custom Port | ✅ |
| IT-3 | `bind::`/`open::`/`refresh::` -> accepted and ignored, exit 0 | Unwired Params | ✅ |
| IT-4 | Pinned port already in use -> exit 1, bind failure message | Error Handling | ✅ |
| IT-5 | `refresh::30` -> 30-second auto-refresh interval | Refresh Interval | ⏳ Phase 2 |
| IT-6 | `open::1` -> default browser launches | Browser Open | ⏳ Phase 2 |

## Test Coverage Summary

- Default: 1 test (IT-1)
- Custom Port: 1 test (IT-2)
- Unwired Params: 1 test (IT-3)
- Error Handling: 1 test (IT-4)
- Refresh Interval: 1 test (IT-5, deferred)
- Browser Open: 1 test (IT-6, deferred)

**Total:** 6 tests (4 executable, 2 blocked on Phase 2)

---

### IT-1: No args -> serves on 127.0.0.1 at an OS-assigned port

- **Given:** temp journal dir; `CLJ_PORT` unset in the child environment
- **When:** `clj .serve`
- **Then:** exit 0 on shutdown; stdout prints `Listening on http://localhost:{port}` with a nonzero OS-assigned port, flushed immediately so the harness can read it before the server blocks in its accept loop; `GET http://127.0.0.1:{port}/` returns 200
- **Exit:** 0
- **Source:** [command/05_serve.md](../../../../docs/cli/command/05_serve.md) — Algorithm steps 1-2

---

### IT-2: `port::9090` -> serves on custom port

- **Given:** port 9090 is free on localhost
- **When:** `clj .serve port::9090`
- **Then:** exit 0 on shutdown; startup line reports 9090; server binds there instead of taking an ephemeral port
- **Exit:** 0
- **Source:** [command/05_serve.md](../../../../docs/cli/command/05_serve.md), [param/15_port.md](../../../../docs/cli/param/15_port.md)

---

### IT-3: `bind::`/`open::`/`refresh::` -> accepted and ignored, exit 0

- **Given:** port 9093 is free on localhost
- **When:** `clj .serve bind::0.0.0.0 open::1 refresh::30 port::9093`
- **Then:** exit 0 on shutdown — none of the three unwired parameters causes a rejection. The server is still loopback-only on 9093, no browser process is spawned, and the served page still reports `auto-refresh 5s`. Assert all three negatives, not just the exit code, or the case degenerates into a smoke test
- **Exit:** 0
- **Source:** [command/05_serve.md](../../../../docs/cli/command/05_serve.md) — `bind::` is not implemented

---

### IT-4: Pinned port already in use -> exit 1, bind failure message

- **Given:** the test harness holds a listener on port 9094
- **When:** `clj .serve port::9094`
- **Then:** exit 1; stderr contains `Error: could not start server on 127.0.0.1:9094:`. The port must be pinned — with the ephemeral default a no-arg `.serve` can never collide, so the pre-existing form of this case could not fail
- **Exit:** 1
- **Source:** [command/05_serve.md](../../../../docs/cli/command/05_serve.md) — Algorithm step 1

---

### IT-5: `refresh::30` -> 30-second auto-refresh interval — ⏳ Phase 2

- **Given:** port 9095 is free
- **When:** `clj .serve refresh::30 port::9095`
- **Then:** the served dashboard auto-refreshes every 30 seconds
- **Blocked on:** `refresh::` being read at all. `INDEX_HTML` hardcodes `setInterval( load, 5000 )` (`src/cli_main.rs:63`), so the interval is neither 30 s nor the 10 s the feature doc targets
- **Source:** [command/05_serve.md](../../../../docs/cli/command/05_serve.md), [param/27_refresh.md](../../../../docs/cli/param/27_refresh.md)

---

### IT-6: `open::1` -> default browser launches — ⏳ Phase 2

- **Given:** port 9096 is free; a default browser is configured
- **When:** `clj .serve open::1 port::9096`
- **Then:** the default browser launches via `xdg-open` (Linux) or `open` (macOS) after the server starts
- **Blocked on:** `cmd_serve()` never reading an `open` key and never spawning a child process
- **Source:** [command/05_serve.md](../../../../docs/cli/command/05_serve.md), [param/17_open.md](../../../../docs/cli/param/17_open.md)
