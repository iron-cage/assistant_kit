# Feature: Bounded Fan-Out

### Scope

- **Purpose**: Document `run_bounded()` — running many child commands with a fixed ceiling on how many are alive at once, and a fixed order for the results.
- **Responsibility**: Describe the concurrency bound, the input-order guarantee, per-child output capture, and the failure-isolation rule.
- **In Scope**: `run_bounded()`, `FanoutOutcome`, `SPAWN_FAILED_EXIT_CODE`, clamping behavior, why there is no timeout and no cancellation.
- **Out of Scope**: What the children *are* — choosing topics, building `clr` argument lists, rendering the report (→ `claude_runner`'s `delegate`/`broadcast` commands); single-command execution (→ [feature/001](001_execution_control.md)).

### Design

`run_bounded( commands, concurrency )` takes a `Vec< ( String, Command ) >` — a
label paired with a fully-built command — and returns one `FanoutOutcome` per
input, in input order.

```rust,ignore
let outcomes = claude_runner_core::fanout::run_bounded( jobs, 4 );
for outcome in &outcomes
{
  println!( "{}: exit {}", outcome.label, outcome.exit_code );
}
```

**The bound is the point.** Forwarding one prompt to twenty topics means twenty
child processes. Started all at once, that is twenty concurrent Claude Code
sessions, twenty times the token spend in the same instant, and a rate-limit wall
that fails all twenty rather than queueing any. Run strictly one at a time, the
independence that made fan-out worth doing is thrown away. A fixed worker pool
draining a shared queue is the middle: exactly `concurrency` children exist at
any instant, and a worker takes its next job only when it is free — so an uneven
mix of fast and slow children never overshoots.

**Clamping.** `concurrency` is clamped into `1..=commands.len()`. Zero would
otherwise mean "start nothing and wait", and a bound above the batch size only
allocates workers with no queue left to drain. An empty batch returns an empty
vector without starting a thread.

### Guarantees

| Guarantee | What it means | Why |
|-----------|---------------|-----|
| **Input order** | Result *i* describes command *i*, whatever finished first | A report that reorders itself by whichever topic answered fastest is not comparable between runs, and comparing runs is most of what a fan-out report is for |
| **One outcome per command** | N in, N out — including children that never started | A missing row is indistinguishable from a silent success; every command owes an answer |
| **Separate streams** | Each outcome's `stdout`/`stderr` is that child's alone | Children write to captured pipes, never to the parent's terminal, so nothing interleaves |
| **Failure isolation** | One child failing never stops or cancels another | Topics are separate conversations in separate sessions; a batch is not a transaction |
| **No deadlock on volume** | A child may write far past the pipe buffer on both streams | `Command::output()` drains both concurrently; the naive wait-then-read hangs here |

### Spawn Failure

A command that cannot be started at all still produces an outcome:
`exit_code` is `SPAWN_FAILED_EXIT_CODE` (`-1`) and `stderr` carries the spawn
error rather than the child's.

That value is distinguishable from every real result by construction — a process
that runs reports either its own status (`0..=255`) or `128 + signal` via
`signal_exit_code()`, and neither is ever negative. So a negative code means the
failure happened before the process existed.

`FanoutOutcome::is_success()` is `exit_code == 0`, which makes a spawn failure a
non-success without the caller having to remember the sentinel.

### Deliberate Omissions

**No timeout.** A child that hangs hangs. Killing a Claude Code process from the
outside leaves its session file mid-write and, on some paths, an orphaned
subprocess of its own. The honest place for a deadline is inside each child
(`clr --timeout`), where the runner already owns the cleanup.

**No cancellation.** One child failing says nothing about whether its siblings
should stop. Every command in the batch runs, and every one reports.

**No streaming.** Output is captured whole, per child, and returned when the
batch is done. Interleaving twenty live stdout streams onto one terminal produces
something no one can read; the caller renders the collected outcomes instead.

### Verification

The mechanical check is `tests/fanout_test.rs` — `tfo08` measures the actual
high-water mark of simultaneous children against the bound, `tfo03` asserts input
order against a batch deliberately listed slowest-first, and `tfo11` writes
200 KB on both streams to prove the pipe-buffer deadlock is not reachable:

```bash
cd module/claude_runner_core && ./verb/test
```

By hand, the bound is observable from outside — run a fan-out and count the
children while it is in flight:

```bash
clr broadcast --concurrency 2 "status?" &
sleep 1 && pgrep -c -f 'claude ' ; wait
```

The count should never exceed the `--concurrency` value.

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| source | `../../src/fanout.rs` | `run_bounded`, `FanoutOutcome`, `SPAWN_FAILED_EXIT_CODE` |
| source | `../../src/exit_code.rs` | `signal_exit_code` — why a real exit is never negative |
| doc | [feature/001_execution_control.md](001_execution_control.md) | Single-command execution, which this fans out over |
| test | `../../tests/fanout_test.rs` | tfo01–tfo12 |
