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

### `5h Reset` in Display when `resets_at = None`

When `resets_at = None` for the 5h window, the server has reported no active 5h session window. This is expected — not an error, not missing data. What `.usage` renders in the `5h Reset` column then depends on whether a *corroborated* local touch record exists for that account:

| Local touch record | `5h Reset` cell | Meaning |
|---|---|---|
| none (or older than `TOUCH_GRACE_SECS`) | `—` | genuinely idle; no window to count down |
| present and corroborated | `~in Xh Ym` | window projected from the touch instant, not reported by the server |

The projected form is the same derive-and-flag-with-`~` convention the `~Renews` column already uses: a leading `~` always marks a locally-derived estimate, never a server value. The projection is `floor_to_10_minutes( last_touch_at ) + 5h` — Anthropic's 5h windows begin on a 10-minute boundary, so flooring the touch instant reproduces the server's own `resets_at` exactly (±60s) once it propagates. See [feature/024 AC-20](../feature/024_session_touch.md) and BUG-551.

"Corroborated" excludes touch records the cache itself refutes: if the quota was fetched more than `TOUCH_PROPAGATION_SECS` (300s) after the touch and *still* carried no 5h window, that touch demonstrably opened nothing and the cell falls back to `—`. See BUG-552.

### Behavioral Invariants

- Only Sonnet-family API calls can start the `seven_day_sonnet` window; Haiku cannot open it.
- When `resets_at = None` for the 5h window, `.usage` displays `—` when no corroborated touch record exists, and a `~`-prefixed projected countdown when one does. Neither is an error condition.
- Every `5h Reset` value carrying a `~` prefix is locally derived; every value without one came from the server's `resets_at`.
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
