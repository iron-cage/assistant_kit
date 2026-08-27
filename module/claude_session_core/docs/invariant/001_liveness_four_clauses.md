# Invariant: Liveness Four Clauses

### Scope

- **Purpose**: Guarantee that "this recorded PID is alive" means the recorded *process* is running — not merely that something currently answers to that number.
- **Governs**: `pid_alive` in `src/liveness.rs`, and every consumer that decides whether a session record is stale.
- **In Scope**: All liveness decisions in this crate and in consumers that call `pid_alive` or `SessionRecord::is_alive`.
- **Out of Scope**: Non-Linux platforms, where `/proc` does not exist and `pid_alive` returns `false` unconditionally.

### Rule

`pid_alive( pid, recorded_starttime )` returns `true` only when **all four** clauses hold:

| Clause | Check | Rejects |
|--------|-------|---------|
| (a) | `/proc/{pid}/stat` is readable | A PID number nothing occupies |
| (b) | State field ∉ `{ Z }` | An exited-but-unreaped zombie |
| (c) | `Tgid == pid` in `/proc/{pid}/status` | A non-leader thread id of an unrelated process |
| (d) | `stat` field 22 equals `recorded_starttime`, when recorded | A recycled PID belonging to a different incarnation |

No clause may be dropped. Two of the four were paid for by production failures:

**Clause (b) — BUG-479.** A bare `/proc/{pid}` existence probe read unreaped zombies as live. An exited child keeps its `/proc` entry for as long as its parent fails to `wait()`, so under a non-reaping supervisor every dead slot owner and queued waiter became permanent — 7 of 8 gate slots starved, with `Queued · 84 waiting` against 4 genuinely live processes. **`/proc/{pid}` existence proves a PID exists, not that a process runs.**

**Clauses (c) and (d) — BUG-488.** Clauses (a) and (b) together test only that *something* with this number is running, never that it is the recorded *process*. Linux resolves direct `/proc/<tid>` lookups for readdir-invisible non-leader thread ids of unrelated processes, and a full PID-space wrap recycles a leader number to a new process. Either occupancy made a dead record read alive forever — observed live, a `dockerd` startup thread with TID 1744061 masked a dead gate waiter as a phantom `Queued` row for 76 hours. **A bare PID number never identifies a process across time; bind records to the `(pid, starttime)` incarnation and verify both on read.**

### Two Details That Are Not Optional

**The state field follows the *last* `)`, not the first.** A process's `comm` field is unquoted and may itself contain spaces and parentheses. Splitting `stat` on the first `)` misreads the state of any process whose name contains one.

**Absent `recorded_starttime` is not a mismatch.** Clause (d) is additive: a record without the field keeps clauses (a)–(c) only. This is what lets a mixed fleet upgrade without mass-reclaiming slots held by live pre-fix sessions. Treating `None` as a failed match would turn an upgrade into an outage.

### Why It Lives Here

This predicate was `pub( super )` inside a binary crate, unreachable by any other consumer. Promoting it verbatim is what prevents the next consumer from re-deriving it — and re-introducing both bugs, since the naive implementation is the one that fails.

The original copy in `claude_runner/src/cli/gate_liveness.rs` is deleted, not deprecated. Leaving it in place would have been the same defect one level up: two implementations of a four-clause rule whose whole history is clauses being missing from one of them. Its consumers — `gate_slot::acquire_slot` for reclaim eligibility, `ps::build_queued_table` for the queued-waiter display self-heal — now call this one, alongside `claude_daemon_core`'s registration wait.

Verify nothing re-derives it:

```bash
# Only src/liveness.rs should define these. Every other hit is a call or a comment.
grep -rn "fn pid_alive\|fn proc_tgid\|fn starttime_from_stat" module/
```

### Verification

```bash
cargo test -p claude_session_core --test liveness_test
```

Directly, against a real zombie:

```bash
# A zombie's stat state field is Z; its /proc entry still exists.
awk '{ print $3 }' /proc/self/stat        # R — running
```

`tests/liveness_test.rs` creates a real unreaped child and asserts `pid_alive` returns `false` for it while `/proc/{pid}` still exists.

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| source | `src/liveness.rs` | `pid_alive` and its helpers |
| source | `src/registry.rs` | `SessionRecord::is_alive` and `scan_live` |
| doc | [feature/001_registry_scan.md](../feature/001_registry_scan.md) | Where liveness is applied |
| test | `tests/liveness_test.rs` | Zombie, non-leader, and starttime-mismatch cases |
| doc | [`claude_runner/docs/invariant/012_gate_slot_atomicity.md`](../../../claude_runner/docs/invariant/012_gate_slot_atomicity.md) | The gate consumer, and the contract it depends on |
