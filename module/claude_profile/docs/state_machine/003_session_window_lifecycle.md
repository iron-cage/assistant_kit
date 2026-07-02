# State Machine: Session Window Lifecycle

### Scope

- **Purpose**: Define the lifecycle states for the three per-account quota windows (5h, 7d, 7d-Sonnet).
- **Responsibility**: Documents `idle`/`active`/`exhausted`/`reset` states, model-capability constraints, and touch trigger conditions.
- **In Scope**: Window state transitions; Haiku/Sonnet model asymmetry for the 7d-Sonnet window; touch trigger conditions; `resets_at = None` display behavior.
- **Out of Scope**: Status group partitioning (→ algorithm/003); touch invocation mechanics (→ subprocess/004).

### Applies To

Each of the three quota windows per account: **5h**, **7d**, **7d-Sonnet**.

### States

| State | `resets_at` | `utilization` | Description |
|-------|-------------|---------------|-------------|
| `idle` | `None` | 0% | Window not started; no session activity yet |
| `active` | `Some(timestamp)` | 0%–100% | Session running; timer counting down |
| `exhausted` | `Some(timestamp)` | ~100% | Quota consumed; timer still running |
| `reset` | New `Some(timestamp)` | 0% | Window expired and server reset it (new cycle) |

### Transitions

```
[idle]      --any API call (model-specific; see below)--> [active]
[active]    --usage accumulates--> [exhausted]
[exhausted] --resets_at reaches now--> [reset] → [active]  (new window, utilization=0%)
[active]    --resets_at reaches now--> [reset] → [active]   (or [idle] if no call in new window)
```

### Model-Capability Constraint

| Window | Started by |
|--------|-----------|
| `five_hour` | Any model API call (Haiku, Sonnet, Opus) |
| `seven_day` | Any model API call |
| `seven_day_sonnet` | **Sonnet-family API calls only** — Haiku cannot start this window |

This asymmetry is why `resolve_model(Auto)` selects Sonnet when `son_idle=true` (7d-Sonnet window absent with `resets_at=None`): a Haiku touch subprocess cannot open the 7d-Sonnet window, causing an infinite per-call no-op loop (BUG-289).

### Touch Trigger

An account qualifies for touch when any timer is in `idle` state (`resets_at = None`). Touch sends a `["--print", "."]` subprocess which makes an API call, transitioning the idle window(s) to `active`. See [subprocess/004](../subprocess/004_session_touch_invocation.md).

### `5h Reset = —` in Display

When `resets_at = None` for the 5h window, `.usage` shows `—` in the `5h Reset` column. This is expected behavior — the server returns `null` when no active 5h session window exists. It is NOT an error or missing data.

### Behavioral Invariants

- Only Sonnet-family API calls can start the `seven_day_sonnet` window; Haiku cannot open it.
- When `resets_at = None` for the 5h window, `.usage` displays `—` — this is not an error condition.
- A touch subprocess using Haiku cannot transition `seven_day_sonnet` from `idle` to `active`.

### Features

| File | Relationship |
|------|-------------|
| [feature/024_session_touch.md](../feature/024_session_touch.md) | Touch trigger conditions and algorithm |
| [feature/009_token_usage.md](../feature/009_token_usage.md) | Quota display; 5h/7d/7d-Son column semantics |

### Algorithms

| File | Relationship |
|------|-------------|
| [algorithm/003](../algorithm/003_quota_status_groups.md) | Status groups depend on window utilization |

### Subprocess

| File | Relationship |
|------|-------------|
| [subprocess/004](../subprocess/004_session_touch_invocation.md) | Touch invocation |
