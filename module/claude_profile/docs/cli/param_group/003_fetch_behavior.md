# Group :: 3. Fetch Behavior

**Parameters:** `refresh::`, `live::`, `interval::`, `jitter::`, `trace::`, `touch::`, `imodel::`, `effort::`
**Pattern:** Per-invocation fetch control
**Purpose:** Controls fetch behavior across quota, inspection, and switch commands — token refresh on auth error or locally-expired credentials, continuous monitor loop configuration, and isolated subprocess setup.

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| [`refresh::`](../param/019_refresh.md) | `bool` | `1` | Refresh expired OAuth token via isolated subprocess and retry once; trigger is 401/403 auth error (`.usage`) or locally-expired `expiresAt` before endpoint calls (`.account.inspect`) — see [param 019](../param/019_refresh.md) |
| [`live::`](../param/020_live.md) | `bool` | `0` | Enable continuous refresh loop (Ctrl-C to exit cleanly) |
| [`interval::`](../param/021_interval.md) | `u64` | `30` | Seconds between refresh cycles (≥ 30; validated only when `live::1`) |
| [`jitter::`](../param/022_jitter.md) | `u64` | `0` | Max random seconds added to each cycle delay (0 ≤ jitter ≤ interval; validated only when `live::1`) |
| [`trace::`](../param/023_trace.md) | `bool` | `0` | Print `[trace]` lines to stderr: credential reads, API calls, and refresh steps |
| [`touch::`](../param/034_touch.md) | `bool` | `1` | Activate idle 5h windows via isolated subprocess |
| [`imodel::`](../param/035_imodel.md) | `enum` | `auto` | Model for isolated subprocesses: `auto` (haiku by default; sonnet when `son_idle=true`), `sonnet`, `opus`, `haiku`, `keep` |
| [`effort::`](../param/036_effort.md) | `enum` | `auto` | Effort level for isolated subprocesses: `auto` (`low` for any model; no flag for haiku/keep), `low`, `normal`, `high`, `max` |

### Referenced Commands

| # | Command | Role |
|---|---------|------|
| 1 | [`.usage`](../command/006_usage.md#command--9-usage) | All 8 params |
| 2 | [`.account.use`](../command/001_account.md#command--5-accountuse) | `trace::`, `touch::`, `imodel::`, `effort::` |
| 3 | [`.account.inspect`](../command/001_account.md#command--15-accountinspect) | `refresh::`, `trace::` |

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

> "Does parameter X control **how commands fetch remote data** (retry strategy, iteration mode, or isolated subprocess configuration)?"

All 8 members pass: `refresh::` (retry strategy on auth error or locally-expired token), `live::` (iteration mode), `interval::` (loop cycle duration), `jitter::` (loop timing variance), `trace::` (diagnostic output during fetch operations), `touch::` (active-window extension strategy), `imodel::` (subprocess model configuration), `effort::` (subprocess effort configuration). `format::` fails (output serialisation, not fetch strategy) and is correctly excluded.

**Invariants**

- `interval::` and `jitter::` are only validated when `live::1`; their values have no effect when `live::0`.
- `refresh::` is orthogonal to `live::` — both may be set simultaneously without conflict.
- `live::1 format::json` is rejected before any fetch (see [../004_parameter_interactions.md](../004_parameter_interactions.md#interaction--4-live1-is-incompatible-with-formatjson)).

**Cross-References**

- [../004_parameter_interactions.md](../004_parameter_interactions.md) — `live::1 format::json` incompatibility rule
- [../../feature/017_token_refresh.md](../../feature/017_token_refresh.md) — `refresh::` feature design
- [../../feature/018_live_monitor.md](../../feature/018_live_monitor.md) — `live::` / `interval::` / `jitter::` feature design
- [../../feature/024_session_touch.md](../../feature/024_session_touch.md) — `touch::` feature design
- [../../feature/026_subprocess_model_effort.md](../../feature/026_subprocess_model_effort.md) — `imodel::` / `effort::` feature design
- [../../feature/027_account_use_post_switch_touch.md](../../feature/027_account_use_post_switch_touch.md) — `touch::`, `imodel::`, `effort::` on `.account.use`

### Referenced User Stories

| # | User Story | Persona |
|---|------------|---------|
| 1 | [Multi-Account Quota Monitoring](../user_story/003_quota_monitoring.md) | `live::`, `interval::`, `jitter::` for continuous monitoring |
| 2 | [Account Rotation](../user_story/001_account_rotation.md) | `touch::`, `refresh::` for post-switch activation |
| 3 | [Credential Diagnostics](../user_story/005_credential_diagnostics.md) | `trace::`, `refresh::` for deep diagnostic depth |
