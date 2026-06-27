# .serve

Start embedded web viewer on localhost.

-- **Parameters:** port::, bind::, open::, refresh::
-- **Exit Codes:** 0 (clean shutdown), 1 (bind failure)

### Syntax

```
clj .serve [port::PORT] [bind::ADDRESS] [open::BOOL] [refresh::SECONDS]
```

### Parameters

| Parameter | Type | Default | Required | Purpose |
|-----------|------|---------|----------|---------|
| `port` | Port | 8411 | No | HTTP server port |
| `bind` | String | 127.0.0.1 | No | Bind address |
| `open` | Boolean | 0 | No | Open browser on start |
| `refresh` | Integer | 10 | No | Auto-refresh interval in seconds |

**Algorithm (4 steps):**

1. Bind HTTP server to `bind::address:port::port`; exit 1 on failure
2. Print `Serving journal viewer at http://{bind}:{port}` to stdout
3. If `open::1`, launch default browser via `xdg-open` (Linux) or `open` (macOS)
4. Accept requests: `/` serves embedded HTML; `/api/*` routes to JSON handlers reading `JournalReader`

### Examples

```bash
clj .serve                          # Serve on 127.0.0.1:8411
clj .serve port::9090               # Custom port
clj .serve bind::0.0.0.0 open::1   # Network-accessible, open browser
clj .serve refresh::30             # 30-second auto-refresh
```

### Referenced User Stories

| # | User Story |
|---|-----------|
| 1 | [Cost Tracking](../user_story/01_cost_tracking.md) |
| 3 | [Automation Audit](../user_story/03_automation_audit.md) |
