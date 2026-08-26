# Feature: Turn Detection

### Scope

- **Purpose**: Tell a consumer when a session has actually finished answering — the signal that makes an interactive session usable through a print-mode-shaped interface (prompt → output → prompt returns).
- **In Scope**: `TurnWatcher`, `TurnEvent`, `BackgroundReporting`, `BG_TASKS_REPORT_RUNNING_ENV`.
- **Out of Scope**: Reading the status in the first place (→ [001_registry_scan.md](001_registry_scan.md)), what to do with a boundary (→ `claude_daemon_core`).

### Why Not Just `status == "idle"`

Claude Code's `Stop` hook payload carries a `background_tasks` array, whose documented purpose is to let a hook distinguish "session is done" from "session is paused waiting for background work to wake it". A non-empty array means the session is *not* finished — it is parked and will resume with no new user input.

The registry's `status` field does not expose that array. Worse, whether `status` accounts for outstanding background work at all is controlled by an environment variable that **defaults to off**: with `CLAUDE_CODE_BG_TASKS_REPORT_RUNNING` unset, a session with background tasks in flight reports `idle`.

A consumer that treats the first `busy` → `idle` transition as "the answer is ready" therefore returns control to the user mid-turn — intermittently, and unreproducibly, depending on whether that particular prompt happened to spawn background work. This is the worst shape a bug of this kind can take: it works in testing and fails under load.

### The Mitigation

Spawn every observed session with `BG_TASKS_REPORT_RUNNING_ENV` set to `"1"`. Then `busy` covers background work too, and the transition to `idle` is a real boundary.

`TurnWatcher::new` requires the caller to state whether that was done, because the answer cannot be recovered from the registry afterward:

| `BackgroundReporting` | Meaning | `busy` → `idle` yields |
|-----------------------|---------|------------------------|
| `Enabled` | Spawned with the variable set to `"1"` | `TurnEvent::Settled` — a real boundary |
| `Unknown` | Spawned without the guarantee | `TurnEvent::SettledUnverified` — advisory only |

The unverified case is *labelled* rather than suppressed or silently trusted. A consumer observing a session it did not spawn still gets a usable signal; it simply gets told what that signal is worth.

### Edge-Triggered

`observe( &status )` reports a transition, not a level:

| Previous | Current | Event |
|----------|---------|-------|
| *(none — first sighting)* | anything | `None` |
| `Busy` | `Idle` | `Settled` / `SettledUnverified` |
| `Idle` or `Other` | `Busy` | `Started` |
| same as previous | | `None` |

The first sighting never produces an event — see [invariant/002_first_sighting_never_settles.md](../invariant/002_first_sighting_never_settles.md) for why that is a rule rather than an accident of implementation.

`last()` exposes the most recently observed status, so a consumer can render current state without keeping its own copy.

### Verification

```bash
cargo test -p claude_session_core --test turn_test
```

To confirm the environment variable's effect directly, spawn a session with it set and compare the registry `status` field against one spawned without, while a background task is outstanding:

```bash
CLAUDE_CODE_BG_TASKS_REPORT_RUNNING=1 claude   # status stays `busy`
claude                                          # status flips to `idle`
```

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| source | `src/turn.rs` | `TurnWatcher`, `TurnEvent`, `BackgroundReporting` |
| source | `src/registry.rs` | `SessionStatus`, the input to `observe` |
| doc | [invariant/002_first_sighting_never_settles.md](../invariant/002_first_sighting_never_settles.md) | The edge-trigger rule |
| doc | [api/001_session_surface.md](../api/001_session_surface.md) | Full signature contract |
| test | `tests/turn_test.rs` | Transition table and first-sighting behavior |
