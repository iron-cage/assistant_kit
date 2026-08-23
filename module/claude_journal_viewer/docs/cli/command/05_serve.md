# .serve

Start embedded web viewer on localhost.

-- **Parameters:** port::, open::, refresh:: (bind:: documented, not yet wired)
-- **Exit Codes:** 0 (clean shutdown), 1 (bind failure)

### Syntax

```
clj .serve [port::PORT] [open::BOOL] [refresh::SECONDS]
```

`bind::ADDRESS` is documented but not yet wired — see the note under
Parameters.

### Parameters

| Parameter | Type | Default | Required | Purpose |
|-----------|------|---------|----------|---------|
| `port` | Port | `0` (OS-assigned) | No | HTTP server port; falls back to `CLJ_PORT`, then `0` |
| `bind` | String | 127.0.0.1 | No | Bind address — **not implemented**, see below |
| `open` | Boolean | 0 | No | Open browser on start |
| `refresh` | Integer | 10 | No | Auto-refresh interval in seconds |

**Algorithm (4 steps):**

1. Resolve the port as `port::` → `CLJ_PORT` → `0`, then bind the HTTP server to `127.0.0.1:{port}`; exit 1 on failure with `Error: could not start server on {addr}: {e}`
2. Print `Listening on http://localhost:{actual_port}` to stdout and flush, so a piped reader sees the OS-assigned port immediately
3. If `open::1`, launch default browser via `xdg-open` (Linux) or `open` (macOS)
4. Accept requests: `/` serves embedded HTML; `/api/*` routes to JSON handlers reading `JournalReader`

**`bind::` is not implemented.** Step 1's host is the literal `127.0.0.1`
(`src/cli_main.rs:168`); no `bind` key is ever read from the parameter map, so
`.serve` is loopback-only whatever is passed. It is a Phase 2 deliverable —
see [param/16_bind.md](../param/16_bind.md) and INV-002
(`docs/invariant/002_localhost_only.md`, `Status: Planned`).

### Examples

```bash
clj .serve                          # Serve on 127.0.0.1, OS-assigned port (read the startup line)
clj .serve port::9090               # Custom port
clj .serve open::1                  # Open browser on start
clj .serve refresh::30             # 30-second auto-refresh
```

### Referenced User Stories

| # | User Story |
|---|-----------|
| 1 | [Cost Tracking](../user_story/001_cost_tracking.md) |
| 3 | [Automation Audit](../user_story/003_automation_audit.md) |
