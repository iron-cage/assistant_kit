# Test: Invariant — Slot-Wait Message Differentiation

Test case planning for [invariant/013_slot_wait_message_differentiation.md](../../../docs/invariant/013_slot_wait_message_differentiation.md). Tests validate that `wait_for_session_slot()`'s poll-loop diagnostic names which of the two independent non-admission causes fired — `[lost reservation race]` when `has_capacity` was `true` but `acquire_slot()` lost, `[at capacity]` when `has_capacity` was `false` — and that the pre-existing `"active; waiting"` substring is preserved unchanged.

**Source:** [invariant/013_slot_wait_message_differentiation.md](../../../docs/invariant/013_slot_wait_message_differentiation.md)
**Related:** [invariant/012_gate_slot_atomicity.md](../../../docs/invariant/012_gate_slot_atomicity.md) (admission correctness for the same two false-branches this invariant's message must distinguish)

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| IN-1 | 2 racers, `--max-sessions 1`, 0 pre-existing occupiers → losing racer's stderr names `"lost reservation race"` | Invariant Hold |
| IN-2 | Same race as IN-1 → neither racer's stderr names `"at capacity"` | Invariant Hold |
| IN-3 | 1 long-running occupier already active, `--max-sessions 1`, second invocation polls → stderr names `"at capacity"`, not `"lost reservation race"` | Invariant Boundary |
| IN-4 | Any non-admission message → still contains the literal substring `"active; waiting"` unchanged (regression guard for the 7 pre-existing substring assertions) | Regression Guard |

## Test Coverage Summary

- Invariant Hold: 2 tests (IN-1, IN-2)
- Invariant Boundary: 1 test (IN-3)
- Regression Guard: 1 test (IN-4)

**Total:** 4 invariant test cases (minimum for `invariant` doc type is 2; this spec exceeds it to cover both message-differentiation directions plus the preserved-substring regression guard)

## Architectural Constraint

All 4 cases are integration tests in `tests/concurrency_gate_test.rs` — the differentiation logic lives entirely inside `wait_for_session_slot()`'s poll loop and can only be observed by capturing a real racing `clr` subprocess's stderr (not `Stdio::null()`, the gap BUG-393's own `## Why Not Caught` identified in the pre-fix T08/T14 tests). IN-1 and IN-2 are the two assertions already implemented by T15 (`t15_slot_wait_message_distinguishes_race_loss_from_exhaustion`) against the same single race fixture — they are listed as separate IDs here because they assert two independent invariant directions (racer names the race-loss cause; racer does NOT name the exhaustion cause) even though one test function covers both. IN-3 and IN-4 are the remaining coverage this invariant doc requires beyond T15's existing scope: IN-3 needs a genuine-exhaustion fixture (not a race) to prove `"at capacity"` is reachable at all, and IN-4 is a substring-preservation regression guard.

## Implementation Notes

| ID | Test Function | File | Status |
|----|---------------|------|--------|
| IN-1 | `t15_slot_wait_message_distinguishes_race_loss_from_exhaustion` | `tests/concurrency_gate_test.rs` | ✅ |
| IN-2 | `t15_slot_wait_message_distinguishes_race_loss_from_exhaustion` | `tests/concurrency_gate_test.rs` | ✅ |
| IN-3 | *(not yet implemented)* | `tests/concurrency_gate_test.rs` | ⏳ |
| IN-4 | *(not yet implemented)* | `tests/concurrency_gate_test.rs` | ⏳ |

---

### IN-1: 2 racers, `--max-sessions 1`, 0 pre-existing occupiers → losing racer's stderr names `"lost reservation race"`

- **Given:** `CLR_GATE_DIR` and `CLR_PROC_DIR` freshly created and shared between two racers; `--max-sessions 1`; `--journal off`; both racers launched simultaneously with `count_u32 < max` momentarily true for both before either's `acquire_slot()` commits
- **When:** two `clr --print --max-sessions 1` invocations race with stderr captured (not `Stdio::null()`) for both
- **Then:** exactly one racer's captured stderr contains the literal substring `"lost reservation race"`
- **Note:** `bug_reproducer(BUG-393)` — reproduces BUG-393's root defect: pre-fix, this fixture's losing racer's stderr is byte-identical to a genuine-exhaustion message with no distinguishing field
- **Source:** [invariant/013_slot_wait_message_differentiation.md](../../../docs/invariant/013_slot_wait_message_differentiation.md) Invariant Statement table row 1 (`has_capacity=true`, race-loss)

---

### IN-2: Same race as IN-1 → neither racer's stderr names `"at capacity"`

- **Given:** identical fixture to IN-1 — 2 racers, `--max-sessions 1`, 0 pre-existing occupiers, both stderr streams captured
- **When:** the same two-racer invocation as IN-1
- **Then:** neither racer's captured stderr contains the literal substring `"at capacity"` — because both racers observe `count_u32 < max` (`has_capacity=true`) on their contended attempt, the exhaustion branch never fires for either
- **Note:** `bug_reproducer(BUG-393)` — proves the fix does not merely add a second possible suffix but correctly selects between them based on `has_capacity`, not unconditionally
- **Source:** [invariant/013_slot_wait_message_differentiation.md](../../../docs/invariant/013_slot_wait_message_differentiation.md) Invariant Statement table row 1

---

### IN-3: Genuine exhaustion (1 long-running occupier, `--max-sessions 1`) → stderr names `"at capacity"`, not `"lost reservation race"`

- **Given:** one long-running occupier already holds the sole slot (`--max-sessions 1`) via a pre-seeded live slot file with a confirmed-alive PID; a second `clr --print --max-sessions 1` invocation polls at least once with stderr captured
- **When:** the second invocation's poll observes `count_u32 >= max` (`has_capacity=false`) — no reservation attempt is even made
- **Then:** the second invocation's captured stderr contains the literal substring `"at capacity"` and does NOT contain `"lost reservation race"`
- **Note:** without this case, the test suite only ever exercises the race-loss branch (IN-1/IN-2) — the exhaustion branch's message text is never independently verified reachable
- **Source:** [invariant/013_slot_wait_message_differentiation.md](../../../docs/invariant/013_slot_wait_message_differentiation.md) Invariant Statement table row 2 (`has_capacity=false`, exhaustion)

---

### IN-4: Any non-admission message → preserves the literal substring `"active; waiting"` unchanged

- **Given:** either fixture above (IN-1/IN-2's race-loss case or IN-3's exhaustion case) with stderr captured
- **When:** the poll-loop diagnostic is emitted for either non-admission cause
- **Then:** the message contains the unmodified literal substring `"active; waiting"` — the differentiating `[lost reservation race]` / `[at capacity]` suffix is appended after this substring, never spliced into or replacing it
- **Note:** regression guard for the 7 pre-existing assertions on this exact substring (`tests/config_file_test.rs` lines 96,149,198,250,299; `tests/concurrency_gate_test.rs` T01/T04 lines 220,377) that predate this invariant and must not be broken by any future change to this message
- **Source:** [invariant/013_slot_wait_message_differentiation.md](../../../docs/invariant/013_slot_wait_message_differentiation.md) § Invariant Statement, "Preserved substring"
