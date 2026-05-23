# Group :: 3. Fetch Behavior

**Parameters:** `refresh::`, `live::`, `interval::`, `jitter::`, `trace::`
**Pattern:** Per-invocation fetch control
**Purpose:** Controls how `.usage` fetches and re-fetches quota data — whether to refresh expired tokens on auth errors and whether to run as a continuous monitor loop.

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| [`refresh::`](../param/019_refresh.md) | `bool` | `1` | On 401/403 auth error, refresh token via isolated subprocess and retry once per account |
| [`live::`](../param/020_live.md) | `bool` | `0` | Enable continuous refresh loop (Ctrl-C to exit cleanly) |
| [`interval::`](../param/021_interval.md) | `u64` | `30` | Seconds between refresh cycles (≥ 30; validated only when `live::1`) |
| [`jitter::`](../param/022_jitter.md) | `u64` | `0` | Max random seconds added to each cycle delay (0 ≤ jitter ≤ interval; validated only when `live::1`) |
| [`trace::`](../param/023_trace.md) | `bool` | `0` | Print `[trace]` lines to stderr: credential reads, API calls, and refresh steps |

**Used By (1 command):** [`.usage`](../command/006_usage.md#command--9-usage)

**Typical Patterns:**

```bash
# Refresh expired tokens automatically before showing quota
clp .usage refresh::1

# Continuous live monitor, refresh every 60 seconds with up to 10s jitter
clp .usage live::1 interval::60 jitter::10

# Combine: live monitor that also auto-refreshes expired tokens
clp .usage live::1 refresh::1 interval::60

# Default: single fetch, no loop
clp .usage
```

**Semantic Coherence Test**

> "Does parameter X control **how `.usage` fetches quota data** (retry strategy or iteration mode)?"

All 5 members pass: `refresh::` (retry strategy on auth error), `live::` (iteration mode), `interval::` (loop cycle duration), `jitter::` (loop timing variance), `trace::` (diagnostic output during fetch operations). `format::` fails (output serialisation, not fetch strategy) and is correctly excluded.

**Invariants**

- `interval::` and `jitter::` are only validated when `live::1`; their values have no effect when `live::0`.
- `refresh::` is orthogonal to `live::` — both may be set simultaneously without conflict.
- `live::1 format::json` is rejected before any fetch (see [../004_parameter_interactions.md](../004_parameter_interactions.md#interaction--4-live1-is-incompatible-with-formatjson)).

**Cross-References**

- [../004_parameter_interactions.md](../004_parameter_interactions.md) — `live::1 format::json` incompatibility rule
- [../../feature/017_token_refresh.md](../../feature/017_token_refresh.md) — `refresh::` feature design
- [../../feature/018_live_monitor.md](../../feature/018_live_monitor.md) — `live::` / `interval::` / `jitter::` feature design
