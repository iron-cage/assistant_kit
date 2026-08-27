# Feature :: Bounded Fan-Out

### Scope

- **Purpose**: FT- test cases verifying `run_bounded()` — the concurrency ceiling, the input-order guarantee, per-child stream capture, and failure isolation.
- **Responsibility**: Acceptance criteria confirming that every command owes exactly one outcome, that results never reorder themselves by completion time, that the bound is a real maximum rather than an average, and that a child which fails — or never starts at all — neither disappears from the results nor stops its siblings.
- **In Scope**: `run_bounded()`, `FanoutOutcome` field capture, `FanoutOutcome::is_success()`, `SPAWN_FAILED_EXIT_CODE`, clamping at both ends (`0` and above batch size), pipe-buffer drain.
- **Out of Scope**: Single-command execution (→ `docs/feature/001_execution_control.md`); which topics get fanned out over and how the batch is reported (→ `claude_runner`'s `delegate`/`broadcast`, `tests/delegate_command_test.rs` and `tests/broadcast_command_test.rs`).

Behavioral requirement cases for `claude_runner_core::fanout`. See
[feature/007_bounded_fanout.md](../../../docs/feature/007_bounded_fanout.md) for the specification.

Every case builds its children as `/bin/sh -c` one-liners rather than fixture binaries — the
whole surface under test is "what did this child do", so a shell script that sleeps, writes,
or exits with a chosen code is a more direct statement of the case than a compiled helper.

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| FT-1 | An empty batch yields an empty result and starts nothing | Boundary |
| FT-2 | Every command gets exactly one outcome, labelled as given | Result Completeness |
| FT-3 | Results come back in input order, not completion order | Ordering |
| FT-4 | `stdout` and `stderr` are captured per child, not merged | Stream Capture |
| FT-5 | A non-zero child exit is reported, not swallowed | Failure Reporting |
| FT-6 | A command that cannot start yields `SPAWN_FAILED_EXIT_CODE` | Failure Reporting |
| FT-7 | One failing child does not stop its siblings | Failure Isolation |
| FT-8 | `concurrency` is a real ceiling on simultaneous children | Concurrency Bound |
| FT-9 | `concurrency = 0` completes the batch rather than hanging | Clamping |
| FT-10 | `concurrency` above the batch size behaves like batch size | Clamping |
| FT-11 | A child writing more than one pipe buffer does not deadlock | Stream Capture |
| FT-12 | `is_success()` is true only for a child that ran and exited zero | Success Predicate |

## Test Coverage Summary

- Boundary: 1 test (FT-1)
- Result Completeness: 1 test (FT-2)
- Ordering: 1 test (FT-3)
- Stream Capture: 2 tests (FT-4, FT-11)
- Failure Reporting: 2 tests (FT-5, FT-6)
- Failure Isolation: 1 test (FT-7)
- Concurrency Bound: 1 test (FT-8)
- Clamping: 2 tests (FT-9, FT-10)
- Success Predicate: 1 test (FT-12)

**Total:** 12 feature cases

Implemented by `tests/fanout_test.rs` as `tfo01`–`tfo12`, in the same order.

---

### FT-1: An empty batch is a no-op, not a hang

- **Given:** an empty `Vec` of commands and a `concurrency` of 4
- **When:** `run_bounded()` is called
- **Then:** the returned vector is empty, and the call returns rather than blocking on workers with nothing to drain
- **Source:** [feature/007_bounded_fanout.md](../../../docs/feature/007_bounded_fanout.md)

---

### FT-2: N commands in, N outcomes out, labels echoed unchanged

- **Given:** three trivially-succeeding jobs labelled `alpha`, `beta`, `gamma`, `concurrency` 2
- **When:** `run_bounded()` returns
- **Then:** exactly three outcomes come back and their labels are `alpha`, `beta`, `gamma` — the label is echoed, not derived
- **Source:** [feature/007_bounded_fanout.md](../../../docs/feature/007_bounded_fanout.md)

---

### FT-3: Results follow input order even when the first listed finishes last

- **Given:** three jobs listed slowest-first — `slow` (`sleep 0.30`), `medium` (`sleep 0.15`), `fast` (`true`) — with `concurrency` 3 so all run at once
- **When:** `run_bounded()` returns
- **Then:** the labels come back as `slow`, `medium`, `fast`. Completion order would be the exact reverse, so an implementation that pushes results as they land fails this and only this case
- **Source:** [feature/007_bounded_fanout.md](../../../docs/feature/007_bounded_fanout.md)

---

### FT-4: The two streams stay separate, and belong to the child that wrote them

- **Given:** two jobs, each writing a distinct line to stdout and a distinct line to stderr
- **When:** `run_bounded()` returns
- **Then:** outcome 0 carries `out-one`/`err-one` and outcome 1 carries `out-two`/`err-two` — neither stream is merged into the other, and neither child's output leaks into the other's outcome
- **Source:** [feature/007_bounded_fanout.md](../../../docs/feature/007_bounded_fanout.md)

---

### FT-5: A child's own non-zero status reaches the caller intact

- **Given:** a single job whose script is `exit 7`
- **When:** `run_bounded()` returns
- **Then:** `exit_code` is `7` — the specific value, not a generic failure flag — and `is_success()` is false
- **Source:** [feature/007_bounded_fanout.md](../../../docs/feature/007_bounded_fanout.md)

---

### FT-6: A spawn failure is an outcome, distinguishable from any real exit

- **Given:** a command naming a binary that does not exist
- **When:** `run_bounded()` returns
- **Then:** exactly one outcome comes back; `exit_code` is `SPAWN_FAILED_EXIT_CODE`; `stderr` contains `cannot start command`; and `stdout` is empty, because a process that never ran wrote nothing
- **Source:** [feature/007_bounded_fanout.md](../../../docs/feature/007_bounded_fanout.md)

---

### FT-7: The batch is not a transaction — failures do not truncate the tail

- **Given:** four jobs in order — `before` (succeeds), `missing` (unspawnable), `exits-nonzero` (`exit 3`), `after` (succeeds) — run with `concurrency` 1
- **When:** `run_bounded()` returns
- **Then:** all four outcomes are present, in order, with the expected per-child results; in particular `after` still ran and produced its output despite following two failures
- **Note:** the bound is deliberately 1 here. Under a serial bound, a cancel-on-failure bug visibly truncates the tail; run in parallel, the later jobs may already have started and the bug would hide
- **Source:** [feature/007_bounded_fanout.md](../../../docs/feature/007_bounded_fanout.md)

---

### FT-8: The bound actually bounds — measured, not assumed

- **Given:** eight identical jobs and a `concurrency` of 3. Each child, under `flock`, appends one byte to a counter file on entry, records a new high-water mark if the file just grew past the previous one, sleeps 0.20s, then removes its byte on exit — so the file length is the number of children inside the critical section at that instant
- **When:** `run_bounded()` returns and the recorded peak is read
- **Then:** the observed peak is `<= 3` (the ceiling holds) **and** `> 1` (the workers genuinely overlapped, so a serial implementation cannot pass by accident)
- **Note:** the high-water mark is recorded rather than sampled because the bound is a claim about the maximum; an implementation that overshoots briefly is still broken. The `flock` is what makes this measure the fan-out's bound instead of the test's own read-modify-write race
- **Source:** [feature/007_bounded_fanout.md](../../../docs/feature/007_bounded_fanout.md)

---

### FT-9: `concurrency = 0` is clamped up rather than meaning "no workers"

- **Given:** two succeeding jobs `a` and `b`, with `concurrency` 0
- **When:** `run_bounded()` returns
- **Then:** both outcomes come back, in input order, and both are successes. Without the clamp, zero workers would mean the queue is never drained and the call never returns
- **Source:** [feature/007_bounded_fanout.md](../../../docs/feature/007_bounded_fanout.md)

---

### FT-10: A bound larger than the batch is harmless

- **Given:** one job, with `concurrency` 64
- **When:** `run_bounded()` returns
- **Then:** exactly one outcome comes back carrying the child's output — the surplus workers have no queue left to drain and cost nothing
- **Source:** [feature/007_bounded_fanout.md](../../../docs/feature/007_bounded_fanout.md)

---

### FT-11: The classic pipe-buffer deadlock is not reachable

- **Given:** one job writing 200 000 bytes to stdout and 200 000 bytes to stderr — far past the ~64 KiB pipe buffer on both streams at once
- **When:** `run_bounded()` returns
- **Then:** `stdout` and `stderr` are each exactly 200 000 bytes long. A naive wait-then-read blocks forever here, because the child blocks writing to a full pipe the parent is not yet draining
- **Source:** [feature/007_bounded_fanout.md](../../../docs/feature/007_bounded_fanout.md)

---

### FT-12: Success means ran-and-exited-zero — nothing else

- **Given:** three jobs — `ok` (`true`), `bad` (`exit 1`), and `gone` (unspawnable) — with `concurrency` 3
- **When:** `run_bounded()` returns
- **Then:** `is_success()` is true for `ok` only. `bad` fails on its non-zero exit; `gone` fails on the spawn sentinel, so callers get the right answer without having to remember `SPAWN_FAILED_EXIT_CODE` themselves
- **Source:** [feature/007_bounded_fanout.md](../../../docs/feature/007_bounded_fanout.md)
