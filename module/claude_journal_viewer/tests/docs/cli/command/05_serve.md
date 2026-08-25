# Test: `.serve`

### Scope

- **Purpose**: Verify `.serve` starts the embedded web viewer at the resolved address and port, and that each of its four parameters is genuinely read.
- **Responsibility**: Test case coverage for `port`, `bind`, `open`, and `refresh`, plus bind-failure and invalid-value handling.
- **In Scope**: Port resolution, bind selection, browser-open tolerance, refresh substitution and rejection, bind failure on a pinned port.
- **Out of Scope**: Dashboard content rendering (-> `../../feature/002_web_viewing.md`), API route behavior (-> same), the localhost-only invariant (-> `../../invariant/002_localhost_only.md`).

Test case planning for [command/05_serve.md](../../../../docs/cli/command/05_serve.md).

`cmd_serve()` now reads all four documented parameters. The cases below are
implemented in `tests/serve_test.rs`; where a case is the same assertion as a
feature-level or invariant-level case, the cross-reference names the shared
test rather than duplicating it, so one behavior is never asserted by two
tests that could drift apart.

## Test Case Index

| ID | Test Name | Category | Status | Implemented as |
|----|-----------|----------|--------|----------------|
| IT-1 | No `port::` -> serves on 127.0.0.1 at an OS-assigned port | Default | ✅ | FT-1 / IN-1 |
| IT-2 | `port::N` -> serves on custom port | Custom Port | ✅ | FT-5 |
| IT-3 | `bind::` -> selects the bound interface | Bind | ✅ | IN-2 / IN-3 |
| IT-4 | Pinned port already in use -> exit 1, bind failure message | Error Handling | ✅ | `it4_busy_pinned_port_exits_1` |
| IT-5 | `refresh::30` -> 30-second auto-refresh interval | Refresh Interval | ✅ | FT-11 |
| IT-6 | `open::1` -> browser launch attempted, failure non-fatal | Browser Open | ✅ | FT-12 |

## Test Coverage Summary

- Default: 1 test (IT-1)
- Custom Port: 1 test (IT-2)
- Bind: 1 test (IT-3)
- Error Handling: 1 test (IT-4)
- Refresh Interval: 1 test (IT-5)
- Browser Open: 1 test (IT-6)

**Total:** 6 tests (all executable; 5 shared with the feature/invariant plans, 1 owned here)

---

### IT-1: No `port::` -> serves on 127.0.0.1 at an OS-assigned port

- **Given:** temp journal dir; `CLJ_PORT` unset in the child environment
- **When:** `clj .serve journal_dir::<dir> port::0`
- **Then:** stdout prints `Listening on http://localhost:{port}` with a nonzero OS-assigned port, flushed immediately so the harness can read it before the server blocks in its accept loop; `GET http://127.0.0.1:{port}/` returns 200
- **Implemented as:** FT-1 / IN-1 (`ft1_in1_serve_starts_on_loopback_and_prints_url`)
- **Source:** [command/05_serve.md](../../../../docs/cli/command/05_serve.md) — Algorithm steps 1-3

---

### IT-2: `port::N` -> serves on custom port

- **Given:** port 19090 is free on localhost
- **When:** `clj .serve port::19090`
- **Then:** startup line reports 19090; server binds there instead of taking an ephemeral port
- **Implemented as:** FT-5 (`ft5_port_override_binds_requested_port`)
- **Source:** [command/05_serve.md](../../../../docs/cli/command/05_serve.md), [param/15_port.md](../../../../docs/cli/param/15_port.md)

---

### IT-3: `bind::` -> selects the bound interface

- **Given:** temp journal dir; a pinned free port
- **When:** `clj .serve bind::0.0.0.0 port::19412`, and separately `clj .serve bind::127.0.0.2 port::19413`
- **Then:** the startup line reports the configured address rather than `localhost`, a non-loopback bind warns on stderr, and a `127.0.0.2` bind leaves `127.0.0.1` refused on the same port
- **Implemented as:** IN-2 / IN-3 — see [invariant/002_localhost_only.md](../../invariant/002_localhost_only.md) for why `0.0.0.0` alone cannot prove the parameter took effect
- **Source:** [command/05_serve.md](../../../../docs/cli/command/05_serve.md), [param/16_bind.md](../../../../docs/cli/param/16_bind.md)

---

### IT-4: Pinned port already in use -> exit 1, bind failure message

- **Given:** the test harness holds a listener on an OS-assigned port and passes that port to the child
- **When:** `clj .serve port::<held>`
- **Then:** exit 1; stderr contains `could not start server on 127.0.0.1:<held>`. The port must be pinned — with the ephemeral default a no-arg `.serve` can never collide, so the case could not fail. The harness takes the port from its own listener rather than a hardcoded constant, so the case cannot flake on a machine where that constant happens to be free
- **Exit:** 1
- **Implemented as:** `it4_busy_pinned_port_exits_1`
- **Source:** [command/05_serve.md](../../../../docs/cli/command/05_serve.md) — Algorithm step 2

---

### IT-5: `refresh::30` -> 30-second auto-refresh interval

- **Given:** temp journal dir
- **When:** `clj .serve refresh::30`, and separately with no `refresh::` and with `refresh::0`
- **Then:** the served dashboard carries `setInterval(load,30000)` and the label `auto-refresh 30s`; the default carries 10000/`auto-refresh 10s`; `refresh::0` leaves the interval unarmed and labels itself `auto-refresh off`. A non-integer value exits 1
- **Implemented as:** FT-11 / FT-11b
- **Source:** [command/05_serve.md](../../../../docs/cli/command/05_serve.md), [param/27_refresh.md](../../../../docs/cli/param/27_refresh.md)

---

### IT-6: `open::1` -> browser launch attempted, failure non-fatal

- **Given:** temp journal dir; a container with no desktop environment
- **When:** `clj .serve open::1 port::0`
- **Then:** the browser launch fails, warns on stderr, and the server still answers `GET /` with 200
- **Coverage limit:** the successful-launch path cannot be asserted headlessly. What is testable — and what matters — is that a failed launch never takes the server down; the success path is the same `open::that` call `.chart open::1` already uses
- **Implemented as:** FT-12 (`ft12_open_failure_is_non_fatal`)
- **Source:** [command/05_serve.md](../../../../docs/cli/command/05_serve.md), [param/17_open.md](../../../../docs/cli/param/17_open.md)
