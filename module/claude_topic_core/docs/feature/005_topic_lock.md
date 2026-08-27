# Feature: Topic Lock

### Scope

- **Purpose**: Keep two writers off one conversation, and let a crashed writer's claim be reclaimed rather than wedging the topic forever.
- **In Scope**: `TopicLock`, `LockDenied`, `try_lock`, `lock_file`, `enabled_for_run_path`, `LOCK_ENV`, `LOCK_DIR_ENV`.
- **Out of Scope**: Bounding total concurrency, which is the runner's gate and a different question (→ [`claude_runner`](../../../claude_runner/docs/cli/command/readme.md)); inferring whether a topic *looks* busy without holding it (→ [003](003_topic_selection.md)).

### The Hazard

A fork topic is addressed by a deterministic session id, so two concurrent
`claude --resume <id>` invocations target the same transcript file.

The concurrency gate does not prevent this. Its slots are indexed by the live
process *count*, deliberately, so that racing callers collide on one path and
`create_new` can arbitrate. That bounds how many sessions run at once and says
nothing whatsoever about *which* sessions they are — two of the permitted N can be
the same topic.

**Whether Claude Code itself guards against this has not been established here.**
This feature is therefore a mitigation for a hazard, not a fix for a confirmed
defect, and it is scoped accordingly.

### Scoping

| Caller | Locks? |
|--------|--------|
| Fan-out commands (delegate, broadcast) | Yes, by default — fanning a prompt over every topic is what makes a collision likely enough to be worth preventing |
| The ordinary run path (`clr topic`) | No, unless `CLR_TOPIC_LOCK=1` |

The second row is the deliberate part. Turning locking on there would make a second
concurrent `clr topic --topic x` start *failing* where today it runs — a behaviour
change this hazard does not yet justify. The switch exists so it can be flipped once
it does.

### What "Advisory" Means

The lock is a file, created with `create_new`, holding the owner's pid and — when
readable — its `/proc` start time. Dropping the guard removes it.

Drop does not run on `SIGKILL`, so a lock can outlive its owner. Reclaiming one is a
**compare-and-delete**: the file is removed only if it still holds the exact content
that was judged stale. That shrinks the window between "decided the owner is dead"
and "deleted the file" from a `/proc` read down to two adjacent filesystem calls — it
does not close it. Two processes reclaiming the same stale lock in the same instant
can still both proceed.

That is the bar a mitigation has to clear: the residual race degrades to *today's*
behaviour for that one invocation, not to something worse.

Liveness is `claude_session_core::pid_alive`, never a bare `/proc/<pid>` existence
check. That function's clauses encode two production bugs' worth of knowledge — a
non-leader thread id can occupy a number, and a full pid-space wrap can recycle one —
and reimplementing it here would reintroduce both.

**The start time is omitted rather than defaulted when unreadable.** A recorded `0`
would not match a live owner's real start time, so the next caller would read it as
dead and take the lock out from under a running process. An absent field reads as
"incarnation unknown", which is the honest answer and keeps the owner's claim.

### What It Does Not Do

`try_lock` never waits. A caller that wants to wait owns that policy, because how
long to wait for a topic depends entirely on why it is being asked for — a broadcast
might skip a busy topic and report it; an interactive command might block.

A `LockDenied::Unavailable` means the lock could not be worked with at all. Because
the lock is advisory, a caller may reasonably proceed anyway — but it should say so.

### Verification

```bash
cd module/claude_topic_core && ./verb/test
```

Or the single test binary, in-container:

```bash
cargo test -p claude_topic_core --test lock_test
```

tlk03 and tlk07 are the pair that matters: a dead owner's lock must be reclaimable,
and a live owner's must not be. Getting one right by breaking the other is the
obvious way to fail here, so both directions are asserted.

By hand, the lock is a plain file and can be watched:

```bash
ls "${CLR_TOPIC_LOCK_DIR:-${TMPDIR:-/tmp}/clr-topic-lock}"   # who holds what
cat "${CLR_TOPIC_LOCK_DIR:-${TMPDIR:-/tmp}/clr-topic-lock}"/*.lock   # "<pid> <starttime>"
```

A file naming a pid that `ps -p <pid>` does not find is a stale lock, and the next
`try_lock` on that topic will reclaim it.

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| source | `src/lock.rs` | Claim, reclaim, release, and the env switches |
| doc | [003_topic_selection.md](003_topic_selection.md) | The inference this feature turns into exclusion |
| doc | [api/001_topic_surface.md](../api/001_topic_surface.md) | Full signature contract |
| doc | [`claude_session_core`](../../../claude_session_core/docs/readme.md) | `pid_alive` and why it is not a `/proc` existence check |
| test | `tests/lock_test.rs` | Exclusion, drop release, dead-owner reclaim, live-owner respect |
