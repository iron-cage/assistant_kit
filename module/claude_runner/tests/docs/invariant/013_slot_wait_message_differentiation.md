# Test: Invariant — Slot-Wait Message Differentiation

Test case planning for [invariant/013_slot_wait_message_differentiation.md](../../../docs/invariant/013_slot_wait_message_differentiation.md). Tests validate that `wait_for_session_slot()`'s poll-loop diagnostic names which of the three independent non-admission causes fired — `[at capacity]` when `has_capacity` was `false`; `[slot held by another session]` when `has_capacity` was `true` and `acquire_slot()` returned `Err(SlotDenialCause::HeldByLive)`; `[lost reservation race]` when `has_capacity` was `true` and `acquire_slot()` returned `Err(SlotDenialCause::LostReclaimRace)` — and that the TSK-452 structured prefix `"gate-wait  active="` is present in every diagnostic emission.

**Source:** [invariant/013_slot_wait_message_differentiation.md](../../../docs/invariant/013_slot_wait_message_differentiation.md)
**Related:** [invariant/012_gate_slot_atomicity.md](../../../docs/invariant/012_gate_slot_atomicity.md) (admission correctness for the same two false-branches this invariant's message must distinguish)

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| IN-1 | 2 racers, `--max-sessions 1`, 0 pre-existing occupiers → losing racer's stderr names `"slot held by another session"` | Invariant Hold |
| IN-2 | Same race as IN-1 → neither racer's stderr names `"at capacity"` or `"lost reservation race"` | Invariant Hold |
| IN-3 | 2 racers, `--max-sessions 1`, pre-seeded confirmed-dead owner → losing racer's first poll attempt names `"lost reservation race"` | Invariant Hold |
| IN-4 | 1 long-running occupier already active, `--max-sessions 1`, second invocation polls → stderr names `"at capacity"`, not `"lost reservation race"` or `"slot held by another session"` | Invariant Boundary |
| IN-5 | Any non-admission message → contains the literal substring `"gate-wait  active="` (TSK-452 structured format; replaced pre-TSK-452 `"active; waiting"` regression guard) | Regression Guard |
| IN-6 | Any non-admission message → contains the literal substring `"gate-wait  active="` (BUG-431 regression guard, updated to TSK-452 format; print-mode scope now architectural rather than label-encoded) | Regression Guard |

## Test Coverage Summary

- Invariant Hold: 3 tests (IN-1, IN-2, IN-3)
- Invariant Boundary: 1 test (IN-4)
- Regression Guard: 2 tests (IN-5, IN-6)

**Total:** 6 invariant test cases (minimum for `invariant` doc type is 2; this spec exceeds it to cover all three message-differentiation directions, the preserved-substring regression guard, and the mode-qualifier regression guard)

## Architectural Constraint

All 6 cases are integration tests in `tests/concurrency_gate_test.rs` — the differentiation logic lives entirely inside `wait_for_session_slot()`'s poll loop and can only be observed by capturing a real racing `clr` subprocess's stderr (not `Stdio::null()`, the gap BUG-393's own `## Why Not Caught` identified in the pre-fix T08/T14 tests). IN-1 and IN-2 are the two assertions implemented by T15 (`t15_slot_wait_message_names_live_hold_when_owner_alive`) against a fresh-claim race fixture with no pre-existing dead owner — they are listed as separate IDs here because they assert two independent invariant directions (racer names the live-hold cause; racer does NOT name the exhaustion or reclaim-race causes) even though one test function covers both. IN-3 is implemented by T16 (`t16_slot_wait_message_names_genuine_reclaim_race_for_dead_owner`), added for BUG-396 to prove `"lost reservation race"` still fires for the one cause it is actually accurate for (a pre-seeded confirmed-dead owner, contended via an injected reclaim delay). IN-4 is implemented by T33 (`t33_slot_wait_message_names_at_capacity_for_exhaustion`), providing a genuine-exhaustion fixture (not a race) to prove `"at capacity"` is reachable. IN-5 is implemented by T34 (`t34_non_admission_message_preserves_active_waiting_substring`), guarding the `"gate-wait  active="` prefix (TSK-452 format; function name retained for historical traceability). IN-6 is implemented by `t_gate_progress_message_names_print_sessions`, the BUG-431 regression guard ensuring the mode qualifier is never silently dropped.

## Implementation Notes

| ID | Test Function | File | Status |
|----|---------------|------|--------|
| IN-1 | `t15_slot_wait_message_names_live_hold_when_owner_alive` | `tests/concurrency_gate_test.rs` | ✅ |
| IN-2 | `t15_slot_wait_message_names_live_hold_when_owner_alive` | `tests/concurrency_gate_test.rs` | ✅ |
| IN-3 | `t16_slot_wait_message_names_genuine_reclaim_race_for_dead_owner` | `tests/concurrency_gate_test.rs` | ✅ |
| IN-4 | `t33_slot_wait_message_names_at_capacity_for_exhaustion` | `tests/concurrency_gate_test.rs` | ✅ |
| IN-5 | `t34_non_admission_message_preserves_active_waiting_substring` | `tests/concurrency_gate_test.rs` | ✅ |
| IN-6 | `t_gate_progress_message_names_print_sessions` | `tests/concurrency_gate_test.rs` | ✅ |

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

### IN-5: Any non-admission message → preserves the literal substring `"gate-wait  active="` (TSK-452 format)

- **Given:** any fixture above (IN-1/IN-2's live-hold case, IN-3's reclaim-race case, or IN-4's exhaustion case) with stderr captured
- **When:** the poll-loop diagnostic is emitted for any non-admission cause
- **Then:** the message contains the literal substring `"gate-wait  active="` — the TSK-452 structured prefix that replaced the pre-TSK-452 `"active; waiting"` body; the differentiating `[at capacity]` / `[slot held by another session]` / `[lost reservation race]` suffix appears in the `(reason: ...)` trailer at the end of the same line
- **Note:** regression guard ensuring the `"gate-wait  active="` prefix is never silently dropped from the cause-labeled branch; pre-TSK-452, assertions pattern-matched `"active; waiting"` (5 in `config_file_test.rs`, T01/T04 here) — those have been updated to `"gate-wait  active="` by TSK-452
- **Source:** [invariant/013_slot_wait_message_differentiation.md](../../../docs/invariant/013_slot_wait_message_differentiation.md) § Invariant Statement, "Preserved substring"

---

### IN-6: Any non-admission message → contains the literal substring `"gate-wait  active="` (BUG-431 regression guard, TSK-452 format)

- **Given:** any fixture above (IN-1/IN-2's live-hold case, IN-3's reclaim-race case, or IN-4's exhaustion case) with stderr captured
- **When:** the poll-loop diagnostic is emitted for any non-admission cause
- **Then:** the message contains the literal substring `"gate-wait  active="` — TSK-452 replaced the BUG-431 `"print sessions active"` string with the structured `"gate-wait  active=X/Y"` prefix; the print-mode scope is now architectural (the count displayed is print-mode-only by construction) rather than encoded in the label text
- **Note:** regression guard for the BUG-431 fix; TSK-452 updated the format from `"print sessions active; waiting"` to `"gate-wait  active=X/Y ..."`; this guard now provides a second fixture (different process lifecycle from IN-5/T34) asserting the same TSK-452 prefix is present — additional coverage breadth rather than an independent string check
- **Source:** [invariant/013_slot_wait_message_differentiation.md](../../../docs/invariant/013_slot_wait_message_differentiation.md) § Enforcement Mechanism (`eprintln!` format string)
