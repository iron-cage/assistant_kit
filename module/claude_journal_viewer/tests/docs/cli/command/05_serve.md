# Test: `.serve`

### Scope

- **Purpose**: Verify `.serve` starts the embedded web viewer with correct bind/port/browser behavior.
- **Responsibility**: Test case coverage for all 4 `.serve` parameters and bind-failure handling.
- **In Scope**: Default bind/port, custom port/bind address, browser auto-open, refresh interval, bind failure.
- **Out of Scope**: Dashboard content rendering (-> user_story specs), API route behavior (implementation-internal).

Test case planning for [command/05_serve.md](../../../../docs/cli/command/05_serve.md).

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| IT-1 | No args -> serves on 127.0.0.1:8411 | Default |
| IT-2 | `port::9090` -> serves on custom port | Custom Port |
| IT-3 | `bind::0.0.0.0 open::1` -> network-accessible, opens browser | Combined |
| IT-4 | `refresh::30` -> 30-second auto-refresh interval | Refresh Interval |
| IT-5 | Port already in use -> exit 1, bind failure message | Error Handling |

## Test Coverage Summary

- Default: 1 test (IT-1)
- Custom Port: 1 test (IT-2)
- Combined: 1 test (IT-3)
- Refresh Interval: 1 test (IT-4)
- Error Handling: 1 test (IT-5)

**Total:** 5 tests

---

### IT-1: No args -> serves on 127.0.0.1:8411

- **Given:** port 8411 is free on localhost
- **When:** `clj .serve`
- **Then:** exit 0 on shutdown; stdout prints `Serving journal viewer at http://127.0.0.1:8411`
- **Exit:** 0
- **Source:** [command/05_serve.md](../../../../docs/cli/command/05_serve.md)

---

### IT-2: `port::9090` -> serves on custom port

- **Given:** port 9090 is free on localhost
- **When:** `clj .serve port::9090`
- **Then:** exit 0 on shutdown; server binds to port 9090 instead of the default
- **Exit:** 0
- **Source:** [command/05_serve.md](../../../../docs/cli/command/05_serve.md), [param/15_port.md](../../../../docs/cli/param/15_port.md)

---

### IT-3: `bind::0.0.0.0 open::1` -> network-accessible, opens browser

- **Given:** port 8411 is free; a default browser is configured
- **When:** `clj .serve bind::0.0.0.0 open::1`
- **Then:** exit 0 on shutdown; server binds to all interfaces; default browser launches automatically
- **Exit:** 0
- **Source:** [command/05_serve.md](../../../../docs/cli/command/05_serve.md), [param/16_bind.md](../../../../docs/cli/param/16_bind.md), [param/17_open.md](../../../../docs/cli/param/17_open.md)

---

### IT-4: `refresh::30` -> 30-second auto-refresh interval

- **Given:** port 8411 is free
- **When:** `clj .serve refresh::30`
- **Then:** exit 0 on shutdown; served dashboard page auto-refreshes every 30 seconds instead of the 10-second default
- **Exit:** 0
- **Source:** [command/05_serve.md](../../../../docs/cli/command/05_serve.md), [param/27_refresh.md](../../../../docs/cli/param/27_refresh.md)

---

### IT-5: Port already in use -> exit 1, bind failure message

- **Given:** another process is already bound to port 8411
- **When:** `clj .serve`
- **Then:** exit 1; stderr contains a bind failure message
- **Exit:** 1
- **Source:** [command/05_serve.md](../../../../docs/cli/command/05_serve.md)
