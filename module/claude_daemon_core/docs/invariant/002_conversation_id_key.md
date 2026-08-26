# Invariant: Conversation Id Key

### Scope

- **Purpose**: Guarantee that a client's handle on a session keeps working after Claude Code re-hosts that session under a different process.
- **Governs**: `SessionTable`'s key type, and every `session_id` field in `Request` and `SessionSummary`.
- **In Scope**: Anything a client uses to address a session.
- **Out of Scope**: `SessionSummary::pid`, which is reported for diagnostics and is explicitly not an address.

### Rule

`SessionTable` MUST be keyed by conversation id (`String`), never by PID. Every request that targets a session MUST carry `session_id`. No API may accept a PID as a way to name a session.

### Rationale

Claude Code's own daemon re-hosts a live session with `--fork-session` on auto-update or recovery. The replacement process has:

- a **different PID**
- **no inherited environment** — every variable the original was spawned with is gone
- a **new conversation id** for the forked continuation

A PID-keyed table detaches silently at that moment. The failure is specifically bad because it fires during *recovery*: the client is holding a handle that no longer resolves, at exactly the point where the mechanism was supposed to help. Nothing errors — the session is simply gone from the table, and the next request says so.

This is the same underlying fact `claude_session_core` documents from the other direction in [its liveness invariant](../../../claude_session_core/docs/invariant/001_liveness_four_clauses.md): **a bare PID number never identifies a process across time.** There, it produced phantom live rows from recycled numbers; here, it would produce phantom dead sessions from re-hosted ones. Both follow from treating a PID as an identity.

### What About the New Conversation Id

A `--fork-session` re-host produces a *new* conversation id for the continuation, so the handle is not literally immutable across a fork either. The difference is that the change is observable and recoverable: the daemon learns the new id from the registry and can re-point the entry, because the fork is a documented relationship between two conversation ids. A recycled PID carries no such relationship — the new occupant has nothing to do with the old one, and there is no way to tell from the number alone.

`SessionSummary::pid` is still reported, for correlating a session against `ps` output or against `claude_session_core`'s registry scan. It is diagnostic data, not a handle, and no request accepts it.

### Verification

```bash
cargo test -p claude_daemon_core --test table_test
```

```bash
# Every request variant that targets a session carries session_id and nothing
# PID-shaped. Prints nothing when the invariant holds:
cd module/claude_daemon_core && \
  grep -n 'pid' src/protocol.rs | grep -v 'SessionSummary\|/// \|pub pid'
```

`tests/table_test.rs` asserts that inserting a session, then inserting again under the same conversation id with a different PID, replaces the entry rather than creating a second one.

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| source | `src/table.rs` | The `HashMap< String, HostedSession >` key |
| source | `src/protocol.rs` | `session_id` on every targeted request |
| doc | [feature/003_session_table.md](../feature/003_session_table.md) | The table this constrains |
| doc | [feature/002_wire_protocol.md](../feature/002_wire_protocol.md) | Where `session_id` appears on the wire |
| test | `tests/table_test.rs` | Replacement-by-conversation-id |
