# .serve

Start embedded web viewer on localhost.

-- **Parameters:** port::, bind::, open::, refresh::, journal_dir::, no_color::
-- **Exit Codes:** 0 (clean shutdown), 1 (bind failure, invalid refresh, unknown param)

### Syntax

```
clj .serve [port::PORT] [bind::ADDRESS] [open::BOOL] [refresh::SECONDS]
           [journal_dir::PATH] [no_color::BOOL]
```

### Parameters

| Parameter | Type | Default | Required | Purpose |
|-----------|------|---------|----------|---------|
| `port` | Port | `0` (OS-assigned) | No | HTTP server port; falls back to `CLJ_PORT`, then `0` |
| `bind` | String | 127.0.0.1 | No | Bind address; non-loopback values warn on stderr (INV-002) |
| `open` | Boolean | 0 | No | Open browser on start |
| `refresh` | Integer | 10 | No | Auto-refresh interval in seconds; `0` disables |
| `journal_dir` | Path | ~/.clr/journal/ | No | Which journal the server reads — launch-time only, never an HTTP query key |
| `no_color` | Boolean | 0 | No | Disable ANSI colors in startup/warning output |

`.serve` accepts no event filters on the command line — filtering happens
per-request via the query string. `dir::` in particular is rejected here rather
than being mistaken for the journal location.

**Algorithm (5 steps):**

1. Resolve `bind::` (default `127.0.0.1`), the port as `port::` → `CLJ_PORT` → `0`, and `refresh::` (default 10; a non-integer exits 1 with `Error: invalid refresh '{v}' …`)
2. Bind the HTTP server to `{bind}:{port}`; exit 1 on failure with `Error: could not start server on {addr}: {e}`
3. For a non-loopback bind, warn on stderr; then print `Listening on http://{host}:{actual_port}` to stdout and flush, so a piped reader sees the OS-assigned port immediately. `{host}` is `localhost` for a loopback bind and the literal `bind::` value otherwise. The warning is emitted *before* the startup line so that a reader synchronising on the startup line is guaranteed to already have the warning
4. If `open::1`, launch the default browser via the `open` crate (`xdg-open` on Linux, `open` on macOS); a failure warns on stderr and does not abort the server
5. Accept requests, matching the path (query string split off first): `/api/events`, `/api/stats`, `/api/health` route to JSON handlers reading `JournalReader`; any other `/api/*` path returns 404 JSON; everything else serves the embedded HTML

**Query parameters** on `/api/events` and `/api/stats` use the same names as
the CLI filter params — see [feature/002_web_viewing.md](../../feature/002_web_viewing.md).

### Examples

```bash
clj .serve                          # Serve on 127.0.0.1, OS-assigned port (read the startup line)
clj .serve port::9090               # Custom port
clj .serve bind::0.0.0.0 port::9090 # Network-accessible — warns on stderr
clj .serve open::1                  # Open browser on start
clj .serve refresh::30              # 30-second auto-refresh
clj .serve refresh::0               # Manual reload only
```

### Referenced User Stories

| # | User Story |
|---|-----------|
| 1 | [Cost Tracking](../user_story/001_cost_tracking.md) |
| 3 | [Automation Audit](../user_story/003_automation_audit.md) |
