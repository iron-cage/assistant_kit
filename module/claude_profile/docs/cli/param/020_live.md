# Parameter :: 20. `live::`

Enables continuous refresh mode for `.usage`. When `live::1`, the command enters a loop: fetch all accounts, clear the screen, render the table, display a countdown footer, wait `interval::` seconds (plus up to `jitter::` seconds), then repeat. Ctrl-C exits cleanly.

- **Type:** `bool`
- **Default:** `0` (single-shot — fetch once, render, exit)
- **Constraints:** Accepted values: `0`, `1`, `false`, `true`; incompatible with `format::json` (exits 1 before first fetch if combined); effective only under `#[cfg(feature = "enabled")]`
- **Commands:** [`.usage`](../command/006_usage.md#command--9-usage)
- **Purpose:** Provides an ambient monitoring dashboard showing live quota utilization for all accounts, refreshing automatically without re-invoking the command.
- **Group:** [Fetch Behavior](../param_group/003_fetch_behavior.md)

**Examples:**

```text
live::0   → single fetch, render, exit (default)
live::1   → continuous refresh loop until Ctrl-C
```

**Notes:**
- `live::1 format::json` exits 1 before any fetch with `"live monitor mode is incompatible with format::json"`.
- `interval::` and `jitter::` are only validated when `live::1` is present.
- See [feature/018_live_monitor.md](../../feature/018_live_monitor.md) for the full algorithm including screen-clear sequence and countdown footer format.
