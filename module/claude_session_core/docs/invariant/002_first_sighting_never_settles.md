# Invariant: First Sighting Never Settles

### Scope

- **Purpose**: Guarantee that a consumer attaching to an already-running session is never told a turn just finished, when in fact nothing happened at all.
- **Governs**: `TurnWatcher::observe` in `src/turn.rs`.
- **In Scope**: Every path that produces a `TurnEvent`.
- **Out of Scope**: Whether a genuine `busy` → `idle` transition is trustworthy — that is [002_turn_detection.md](../feature/002_turn_detection.md)'s `BackgroundReporting` question, and is independent of this rule.

### Rule

`TurnWatcher::observe` MUST return `None` for the first status it is given, whatever that status is. A `TurnEvent` is produced only from a *transition* between two observed statuses, never from a level.

**Rationale:** A watcher is often created against a session that already exists — the daemon restarted, a client attached late, a `--fork-session` re-host produced a new process to watch. If the first observation of an already-idle session produced `Settled`, every such attach would report a turn boundary that never occurred, and the consumer would deliver a completion signal for a turn it never saw start.

The failure is specifically bad because it is *systematic on the reconnect path*: it fires exactly when a supervisor is recovering, which is when a spurious "done" is most likely to be acted on.

Level-triggering would also make the event stream depend on polling frequency rather than on what the session did — two consumers polling at different rates would disagree about how many turns occurred.

### Transition Table

| Previous | Current | Event |
|----------|---------|-------|
| *(none)* | anything | `None` |
| `Busy` | `Idle` | `Settled` or `SettledUnverified` |
| `Idle` \| `Other` | `Busy` | `Started` |
| `Busy` | `Busy` | `None` |
| `Idle` | `Idle` | `None` |
| anything | `Other` | `None` |

The implementation enforces this structurally: `observe` takes the previous value out of the watcher with `Option::replace` and returns early via `?` when there was none, so no match arm can reach a boundary verdict without a prior observation on record.

### Verification

```bash
cargo test -p claude_session_core --test turn_test
```

`tests/turn_test.rs` asserts that a fresh `TurnWatcher` fed `Idle` returns `None`, and that the same watcher fed `Busy` then `Idle` returns `Started` then a settled event — the same input sequence, differing only in whether a prior observation exists.

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| source | `src/turn.rs` | `TurnWatcher::observe` |
| doc | [feature/002_turn_detection.md](../feature/002_turn_detection.md) | The full detection contract |
| doc | [api/001_session_surface.md](../api/001_session_surface.md) | Signature contract |
| test | `tests/turn_test.rs` | First-sighting and transition cases |
