# Test: Invariant — Slot-Wait Message Differentiation

Test case planning for [invariant/013_slot_wait_message_differentiation.md](../../../docs/invariant/013_slot_wait_message_differentiation.md). Tests validate that `wait_for_session_slot()`'s poll-loop diagnostic names which of the three independent non-admission causes fired — `[at capacity]` when `has_capacity` was `false`; `[slot held by another session]` when `has_capacity` was `true` and `acquire_slot()` returned `Err(SlotDenialCause::HeldByLive)`; `[lost reservation race]` when `has_capacity` was `true` and `acquire_slot()` returned `Err(SlotDenialCause::LostReclaimRace)` — and that the pre-existing `"active; waiting"` substring is preserved unchanged.

**Source:** [invariant/013_slot_wait_message_differentiation.md](../../../docs/invariant/013_slot_wait_message_differentiation.md)
**Related:** [invariant/012_gate_slot_atomicity.md](../../../docs/invariant/012_gate_slot_atomicity.md) (admission correctness for the same two false-branches this invariant's message must distinguish)

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| IN-1 | 2 racers, `--max-sessions 1`, 0 pre-existing occupiers → losing racer's stderr names `"slot held by another session"` | Invariant Hold |
| IN-2 | Same race as IN-1 → neither racer's stderr names `"at capacity"` or `"lost reservation race"` | Invariant Hold |
| IN-3 | 2 racers, `--max-sessions 1`, pre-seeded confirmed-dead owner → losing racer's first poll attempt names `"lost reservation race"` | Invariant Hold |
| IN-4 | 1 long-running occupier already active, `--max-sessions 1`, second invocation polls → stderr names `"at capacity"`, not `"lost reservation race"` or `"slot held by another session"` | Invariant Boundary |
| IN-5 | Any non-admission message → still contains the literal substring `"active; waiting"` unchanged (regression guard for the 7 pre-existing substring assertions) | Regression Guard |

## Test Coverage Summary

- Invariant Hold: 3 tests (IN-1, IN-2, IN-3)
- Invariant Boundary: 1 test (IN-4)
- Regression Guard: 1 test (IN-5)

**Total:** 5 invariant test cases (minimum for `invariant` doc type is 2; this spec exceeds it to cover all three message-differentiation directions plus the preserved-substring regression guard)

## Architectural Constraint

All 5 cases are integration tests in `tests/concurrency_gate_test.rs` — the differentiation logic lives entirely inside `wait_for_session_slot()`'s poll loop and can only be observed by capturing a real racing `clr` subprocess's stderr (not `Stdio::null()`, the gap BUG-393's own `## Why Not Caught` identified in the pre-fix T08/T14 tests). IN-1 and IN-2 are the two assertions implemented by T15 (`t15_slot_wait_message_names_live_hold_when_owner_alive`) against a fresh-claim race fixture with no pre-existing dead owner — they are listed as separate IDs here because they assert two independent invariant directions (racer names the live-hold cause; racer does NOT name the exhaustion or reclaim-race causes) even though one test function covers both. IN-3 is implemented by T16 (`t16_slot_wait_message_names_genuine_reclaim_race_for_dead_owner`), added for BUG-396 to prove `"lost reservation race"` still fires for the one cause it is actually accurate for (a pre-seeded confirmed-dead owner, contended via an injected reclaim delay). IN-4 and IN-5 remain the coverage this invariant doc requires beyond T15/T16's existing scope: IN-4 needs a genuine-exhaustion fixture (not a race) to prove `"at capacity"` is reachable at all, and IN-5 is a substring-preservation regression guard.

## Implementation Notes

| ID | Test Function | File | Status |
|----|---------------|------|--------|
| IN-1 | `t15_slot_wait_message_names_live_hold_when_owner_alive` | `tests/concurrency_gate_test.rs` | ✅ |
| IN-2 | `t15_slot_wait_message_names_live_hold_when_owner_alive` | `tests/concurrency_gate_test.rs` | ✅ |
| IN-3 | `t16_slot_wait_message_names_genuine_reclaim_race_for_dead_owner` | `tests/concurrency_gate_test.rs` | ✅ |
| IN-4 | *(not yet implemented)* | `tests/concurrency_gate_test.rs` | ⏳ |
| IN-5 | *(not yet implemented)* | `tests/concurrency_gate_test.rs` | ⏳ |

---

### IN-1: 2 racers, `--max-sessions 1`, 0 pre-existing occupiers → losing racer's stderr names `"slot held by another session"`

- **Given:** `CLR_GATE_DIR` and `CLR_PROC_DIR` freshly created and shared between two racers; `--max-sessions 1`; `--journal off`; both racers launched simultaneously with `count_u32 < max` momentarily true for both before either's `acquire_slot()` commits; no pre-existing dead owner is seeded
- **When:** two `clr --print --max-sessions 1` invocations race with stderr captured (not `Stdio::null()`) for both
- **Then:** exactly one racer's captured stderr contains the literal substring `"slot held by another session"`
- **Note:** `bug_reproducer(BUG-396)` — reproduces the corrected classification: the loser observes the winner's slot record (alive, and per BUG-396's empirical finding, potentially still a `/proc`-visible zombie for the whole observation window even after this test's harness would otherwise reap it), which is `HeldByLive`, never a race, since no dead owner and no reclaim ticket are ever involved in this fixture
- **Source:** [invariant/013_slot_wait_message_differentiation.md](../../../docs/invariant/013_slot_wait_message_differentiation.md) Invariant Statement table row 2 (`has_capacity=true`, `HeldByLive`)

---

### IN-2: Same race as IN-1 → neither racer's stderr names `"at capacity"` or `"lost reservation race"`

- **Given:** identical fixture to IN-1 — 2 racers, `--max-sessions 1`, 0 pre-existing occupiers, both stderr streams captured
- **When:** the same two-racer invocation as IN-1
- **Then:** neither racer's captured stderr contains the literal substring `"at capacity"` (both observe `count_u32 < max` on their contended attempt, so exhaustion never fires) nor `"lost reservation race"` (no dead owner exists in this fixture, so no reclaim-ticket contention is possible)
- **Note:** `bug_reproducer(BUG-393)`/`bug_reproducer(BUG-396)` — proves the fix selects among all three suffixes correctly rather than defaulting any non-capacity denial to "race"
- **Source:** [invariant/013_slot_wait_message_differentiation.md](../../../docs/invariant/013_slot_wait_message_differentiation.md) Invariant Statement table row 2

---

### IN-3: 2 racers, `--max-sessions 1`, pre-seeded confirmed-dead owner → losing racer's first poll attempt names `"lost reservation race"`

- **Given:** the sole slot (`--max-sessions 1`) is pre-seeded with a confirmed-dead owner (a real `true` process spawned and reaped, so its PID is guaranteed not alive and not recyclable within the test window); two racers then contend to reclaim it, with `CLR_GATE_RECLAIM_TEST_DELAY_MS` injecting a delay to widen the reclaim-ticket contention window
- **When:** both racers observe `count_u32 < max` and a dead recorded owner, and both attempt the atomic reclaim-ticket sequence in `acquire_slot()`
- **Then:** the losing racer's stderr is non-empty (the winner returns immediately with no wait-loop output) and its FIRST line contains the literal substring `"lost reservation race"` — later poll attempts may legitimately shift to `"slot held by another session"` once the winner's own slot record becomes observable, so only the first line is asserted
- **Note:** `bug_reproducer(BUG-396)` — proves `"lost reservation race"` still fires for the one cause it is actually accurate for; without this case, BUG-396's fix could over-correct and make the label unreachable entirely
- **Source:** [invariant/013_slot_wait_message_differentiation.md](../../../docs/invariant/013_slot_wait_message_differentiation.md) Invariant Statement table row 3 (`has_capacity=true`, `LostReclaimRace`)

---

### IN-4: Genuine exhaustion (1 long-running occupier, `--max-sessions 1`) → stderr names `"at capacity"`, not `"lost reservation race"` or `"slot held by another session"`

- **Given:** one long-running occupier already holds the sole slot (`--max-sessions 1`) via a pre-seeded live slot file with a confirmed-alive PID; a second `clr --print --max-sessions 1` invocation polls at least once with stderr captured
- **When:** the second invocation's poll observes `count_u32 >= max` (`has_capacity=false`) — no reservation attempt is even made
- **Then:** the second invocation's captured stderr contains the literal substring `"at capacity"` and does NOT contain `"lost reservation race"` or `"slot held by another session"`
- **Note:** without this case, the test suite only ever exercises the `has_capacity=true` branches (IN-1/IN-2/IN-3) — the exhaustion branch's message text is never independently verified reachable
- **Source:** [invariant/013_slot_wait_message_differentiation.md](../../../docs/invariant/013_slot_wait_message_differentiation.md) Invariant Statement table row 1 (`has_capacity=false`, exhaustion)

---

### IN-5: Any non-admission message → preserves the literal substring `"active; waiting"` unchanged

- **Given:** any fixture above (IN-1/IN-2's live-hold case, IN-3's reclaim-race case, or IN-4's exhaustion case) with stderr captured
- **When:** the poll-loop diagnostic is emitted for any non-admission cause
- **Then:** the message contains the unmodified literal substring `"active; waiting"` — the differentiating `[at capacity]` / `[slot held by another session]` / `[lost reservation race]` suffix is appended after this substring, never spliced into or replacing it
- **Note:** regression guard for the 7 pre-existing assertions on this exact substring (`tests/config_file_test.rs` lines 96,149,198,250,299; `tests/concurrency_gate_test.rs` T01/T04 lines 220,377) that predate this invariant and must not be broken by any future change to this message
- **Source:** [invariant/013_slot_wait_message_differentiation.md](../../../docs/invariant/013_slot_wait_message_differentiation.md) § Invariant Statement, "Preserved substring"
