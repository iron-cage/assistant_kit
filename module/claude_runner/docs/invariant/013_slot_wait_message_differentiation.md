# Invariant: Slot-Wait Message Differentiation

### Scope

- **Purpose**: Ensure the operator-facing slot-wait diagnostic names which of the two independent non-admission causes fired, so an operator can distinguish a genuine capacity problem from transient reservation contention.
- **Responsibility**: State the two distinguishable non-admission causes at the admission condition in `wait_for_session_slot()`, the message-layer contract for encoding which cause fired, and the boundary with `012_gate_slot_atomicity.md` (which governs admission correctness, not message content).
- **In Scope**: `wait_for_session_slot()`'s poll-loop diagnostic message construction (`gate.rs`), the `has_capacity` boolean and its role in message differentiation, the `[lost reservation race]` / `[at capacity]` message suffixes and their mapping to the two false-branches of the admission condition.
- **Out of Scope**: Admission condition correctness and atomicity (→ `012_gate_slot_atomicity.md`), gate-timeout exhaustion message at `max_attempts` (→ `006_exit_codes.md`), gate poll interval / attempt-limit configuration (→ `cli/003_env_param.md` Env Param 5).

### Invariant Statement

When `wait_for_session_slot()` does not admit a candidate on a given poll attempt and `!quiet`, the `eprintln!` diagnostic MUST encode which of the two independent false-branches of the compound admission condition (`has_capacity && acquire_slot(...)`) produced the non-admission. A message built only from counters shared across both branches (`count`, `max`, `poll_secs`, `attempt`, `max_attempts`) MUST NOT be emitted without an additional field naming the specific cause.

| Condition | `has_capacity` | Message content |
|-----------|----------------|------------------|
| `count_u32 >= max` (no slot numerically free anywhere) | `false` | `... [at capacity]` |
| `count_u32 < max` but `acquire_slot()` returned `false` (another racer won the same index first) | `true` | `... [lost reservation race]` |

**Distinguishing field:** `has_capacity` is computed once, before the short-circuited `&&`, and is the sole boolean that distinguishes the two false-branches at the point message construction happens. It must be captured at admission-check time — the information does not survive past that point unless deliberately threaded through to the message site.

**Preserved substring:** The pre-existing `"active; waiting"` substring in the message text is preserved unchanged; the differentiating suffix is appended after it, not spliced into it. This substring is asserted by 7 pre-existing tests (`tests/config_file_test.rs` lines 96,149,198,250,299; `tests/concurrency_gate_test.rs` T01/T04 lines 220,377) that predate this invariant and are not this invariant's concern to modify, only to not break.

### Boundary With Invariant 012 (Gate Slot Atomicity)

This invariant is deliberately narrow: it governs **what the message says**, not **whether admission is correct**. `012_gate_slot_atomicity.md`'s own Condition table already documents both false-branches as legitimate, correctly-arbitrated outcomes ("falls to wait-and-retry exactly as the `>= max` case does") — that document is authoritative for *why* both branches are correct admission behavior. This invariant does not restate or challenge that; it adds the orthogonal requirement that the two already-correct outcomes must be distinguishable in the operator-facing text describing them.

### Enforcement Mechanism

In `src/cli/gate.rs`, `wait_for_session_slot()` must apply the differentiation as follows:

```rust
// Fix(BUG-393): distinguish global exhaustion (no slot numerically free)
// from local race-loss (a slot was free but another racer's acquire_slot()
// won the same index first) — both previously produced byte-identical
// text since the message only interpolated the count/max counters shared
// across every false-branch of the compound admission condition above.
// Root cause: the eprintln! captured no field recording which disjunct
// of the admission condition produced this non-admission — the
// distinguishing information existed transiently at `has_capacity` but
// was discarded before the message was constructed.
// Pitfall: a diagnostic built only from counters shared across every
// false-branch of a compound condition silently erases which branch
// fired — capture the branch itself at the point of construction.
let has_capacity = count_u32 < max;
if has_capacity && acquire_slot( &dir, count_u32, pid, since )
{
  // ... admitted, return ...
}
// ...
if !quiet
{
  let cause = if has_capacity { "lost reservation race" } else { "at capacity" };
  eprintln!(
    "Info: {count}/{max} sessions active; waiting {poll_secs}s for a slot... (attempt {attempt}/{max_attempts}) [{cause}]"
  );
}
```

`has_capacity` is bound before the admission check (short-circuit semantics preserved — `acquire_slot()` is still only called when `has_capacity` is `true`) and reused at the message site rather than recomputed, so the message always reflects the same evaluation that drove the admission decision on this attempt.

### Violation Consequences

If the message reverts to interpolating only `count`/`max`/`poll_secs`/`attempt`/`max_attempts` with no `has_capacity`-derived field:

- An operator reading the wait diagnostic cannot tell "the gate is genuinely full, more sessions must finish" (global exhaustion — may warrant a `--max-sessions` increase) from "a numerically free slot existed but I lost a race for it, will likely succeed on the next attempt with no capacity change needed" (local race-loss — transient, self-resolving).
- T15 (`t15_slot_wait_message_distinguishes_race_loss_from_exhaustion` in `tests/concurrency_gate_test.rs`) fails — it asserts exactly one racer's captured stderr names `"lost reservation race"` and neither racer's stderr names `"at capacity"` in a scenario constructed so `count_u32 < max` is momentarily true for both racers.
- The regression is silent at the admission-correctness layer: `012_gate_slot_atomicity.md`'s own tests (T08, T14) continue to pass unchanged, because they assert on admitted-count bounds, not message content — this invariant's own test (T15) is the only coverage that would catch a reversion.

### Sources

| File | Notes |
|------|-------|
| `../../src/cli/gate.rs` | `wait_for_session_slot()` — `has_capacity` binding and the differentiated `eprintln!` at the poll-loop's non-admission fall-through |

### Tests

| File | Notes |
|------|-------|
| `../../tests/concurrency_gate_test.rs` | T15: races exactly 2 concurrent `clr` invocations against `--max-sessions 1` with 0 pre-existing occupiers, guaranteeing one racer's non-admission is a genuine race-loss (`count_u32 < max` momentarily true for both); asserts the losing racer's captured stderr names `"lost reservation race"`, not `"at capacity"` |

### Provenance

| Source | Notes |
|--------|-------|
| BUG-393 | Root bug: `eprintln!` at `gate.rs:368-373` (pre-fix line numbers) interpolated only shared counters, producing byte-identical text for both false-branches of the compound admission condition at `gate.rs:334`. Fix: `has_capacity` bound before the check and threaded into the message as a `[lost reservation race]` / `[at capacity]` suffix appended after the pre-existing `"active; waiting"` text. |

### Invariants

| File | Relationship |
|------|--------------|
| [invariant/012_gate_slot_atomicity.md](012_gate_slot_atomicity.md) | Governs admission correctness for the same two false-branches this invariant's message must distinguish; that document is authoritative for why both outcomes are correct, this one for how they must be reported |

### Features

| File | Relationship |
|------|--------------|
| [feature/001_runner_tool.md](../feature/001_runner_tool.md) | Defines the print-mode execution path whose concurrency gate emits this diagnostic |
