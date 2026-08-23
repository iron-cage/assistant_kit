# Test: Invariant — Slot-Wait Message Differentiation

Test case planning for [invariant/013_slot_wait_message_differentiation.md](../../../docs/invariant/013_slot_wait_message_differentiation.md). Tests validate that `wait_for_session_slot()`'s poll-loop diagnostic names which of the three independent non-admission causes fired — `[at capacity]` when `has_capacity` was `false`; `[slot held by another session]` when `has_capacity` was `true` and `acquire_slot()` returned `Err(SlotDenialCause::HeldByLive)`; `[lost reservation race]` when `has_capacity` was `true` and `acquire_slot()` returned `Err(SlotDenialCause::LostReclaimRace)` — and that the TSK-452 structured prefix `"gate-wait  active="` is present in every diagnostic emission. BUG-480 extended the invariant: slot-side denial diagnostics (the two `has_capacity=true` causes) must additionally carry the occupancy the denying sweep measured — ` slots=H/M` on the poll line, `, slots=H/M held` on both exhaustion messages — with at-capacity lines exempt (the sweep never ran there, so the tally is unmeasured).

**Source:** [invariant/013_slot_wait_message_differentiation.md](../../../docs/invariant/013_slot_wait_message_differentiation.md)
**Related:** [invariant/012_gate_slot_atomicity.md](../../../docs/invariant/012_gate_slot_atomicity.md) (admission correctness for the same two false-branches this invariant's message must distinguish)

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| IN-1 | 2 racers, `--max-sessions 1`, 0 pre-existing occupiers, second racer spawned only once the first is observed holding the slot (rendezvous, BUG-532) → second racer's stderr names `"slot held by another session"` and the first's does not | Invariant Hold |
| IN-2 | Same fixture as IN-1 → neither racer's stderr names `"at capacity"` or `"lost reservation race"` | Invariant Hold |
| IN-3 | 2 racers, `--max-sessions 1`, pre-seeded confirmed-dead owner → losing racer's first poll attempt names one of the two legitimate dead-owner-contention causes (narrowed by BUG-530; see IN-8) | Invariant Hold |
| IN-4 | 1 long-running occupier already active, `--max-sessions 1`, second invocation polls → stderr names `"at capacity"`, not `"lost reservation race"` or `"slot held by another session"` | Invariant Boundary |
| IN-5 | Any non-admission message → contains the literal substring `"gate-wait  active="` (TSK-452 structured format; replaced pre-TSK-452 `"active; waiting"` regression guard) | Regression Guard |
| IN-6 | Any non-admission message → contains the literal substring `"gate-wait  active="` (BUG-431 regression guard, updated to TSK-452 format; print-mode scope now architectural rather than label-encoded) | Regression Guard |
| IN-7 | Live-owned sole slot, empty census (census `0/1`, occupancy `1/1`) → poll line carries both `active=0/1` and `slots=1/1`; exhaustion message carries `slots=1/1 held` | Invariant Hold |
| IN-8 | Dead-owner sole slot whose reclaim ticket is pre-seeded with a **live** claimant → stderr names `"lost reservation race"` and neither `"slot held by another session"` nor `"at capacity"` — deterministically, with no second process | Invariant Hold |

## Test Coverage Summary

- Invariant Hold: 5 tests (IN-1, IN-2, IN-3, IN-7, IN-8)
- Invariant Boundary: 1 test (IN-4)
- Regression Guard: 2 tests (IN-5, IN-6)

**Total:** 8 invariant test cases (minimum for `invariant` doc type is 2; this spec exceeds it to cover all three message-differentiation directions, the measured-occupancy display, the preserved-substring regression guard, and the mode-qualifier regression guard)

## Architectural Constraint

All 8 cases are integration tests, split across `tests/concurrency_gate_ext_test.rs` (IN-1/IN-2/IN-3/IN-8), `tests/concurrency_gate_ext2_test.rs` (IN-4/IN-5/IN-6), and `tests/concurrency_gate_test.rs` (IN-7) — the differentiation logic lives entirely inside `wait_for_session_slot()`'s poll loop and can only be observed by capturing a real `clr` subprocess's stderr (not `Stdio::null()`, the gap BUG-393's own `## Why Not Caught` identified in the pre-fix T08/T14 tests). IN-1 and IN-2 are the two assertions implemented by T15 (`t15_slot_wait_message_names_live_hold_when_owner_alive`) against a fresh-claim fixture with no pre-existing dead owner — they are listed as separate IDs here because they assert two independent invariant directions (racer names the live-hold cause; racer does NOT name the exhaustion or reclaim-race causes) even though one test function covers both. Per BUG-532, T15 is a *rendezvous*, not a symmetric race: racer A is observed holding the slot before racer B is spawned at all, so the two processes' overlap is established by construction rather than assumed. T15 nonetheless remains a genuine two-process test — B observes a real, externally-owned live slot acquired through the normal claim path, which is what distinguishes it from IN-7/T38 and T44, where the "owner" is a pre-seeded record naming the test's own PID. IN-3 is implemented by T16 (`t16_slot_wait_message_names_genuine_reclaim_race_for_dead_owner`), added for BUG-396 as a genuine two-process race over a pre-seeded confirmed-dead owner; BUG-530 narrowed its assertion to "one of the two legitimate dead-owner-contention causes" because which one fires depends on inter-process spawn skew that no fixture can bound (see IN-3's own Note). IN-8 is implemented by T23 (`t23_slot_wait_message_names_lost_reclaim_race_without_a_race`), added for BUG-530 to carry the exact-cause assertion IN-3 gave up: it pre-seeds both the dead-owner slot record and its reclaim ticket (claimed by the test's own live PID), so the single `clr` process under test is forced down the `LostReclaimRace` branch with no second process and no timing window involved. The two are deliberately complementary — T16 proves the race is survivable and never mislabels, T23 proves the label itself is correct. IN-4 is implemented by T33 (`t33_slot_wait_message_names_at_capacity_for_exhaustion`), providing a genuine-exhaustion fixture (not a race) to prove `"at capacity"` is reachable. IN-5 is implemented by T34 (`t34_non_admission_message_preserves_active_waiting_substring`), guarding the `"gate-wait  active="` prefix (TSK-452 format; function name retained for historical traceability). IN-6 is implemented by `t_gate_progress_message_names_print_sessions`, the BUG-431 regression guard ensuring the mode qualifier is never silently dropped. IN-7 is implemented by T38 (`t38_slot_side_denial_names_measured_occupancy`), added for BUG-480: it seeds the sole slot's record with the test process's own live PID while leaving the census (proc dir) empty, forcing census and occupancy to diverge (`active=0/1` vs `slots=1/1`) — the one fixture shape that proves the `slots=` token reports the sweep's measurement rather than echoing the census counter.

## Implementation Notes

| ID | Test Function | File | Status |
|----|---------------|------|--------|
| IN-1 | `t15_slot_wait_message_names_live_hold_when_owner_alive` | `tests/concurrency_gate_ext_test.rs` | ✅ |
| IN-2 | `t15_slot_wait_message_names_live_hold_when_owner_alive` | `tests/concurrency_gate_ext_test.rs` | ✅ |
| IN-3 | `t16_slot_wait_message_names_genuine_reclaim_race_for_dead_owner` | `tests/concurrency_gate_ext_test.rs` | ✅ |
| IN-4 | `t33_slot_wait_message_names_at_capacity_for_exhaustion` | `tests/concurrency_gate_ext2_test.rs` | ✅ |
| IN-5 | `t34_non_admission_message_preserves_active_waiting_substring` | `tests/concurrency_gate_ext2_test.rs` | ✅ |
| IN-6 | `t_gate_progress_message_names_print_sessions` | `tests/concurrency_gate_ext2_test.rs` | ✅ |
| IN-7 | `t38_slot_side_denial_names_measured_occupancy` | `tests/concurrency_gate_test.rs` | ✅ |
| IN-8 | `t23_slot_wait_message_names_lost_reclaim_race_without_a_race` | `tests/concurrency_gate_ext_test.rs` | ✅ |

<!-- BUG-480 — fixed: IN-7 registered below (census/occupancy-divergence fixture asserting the slots=H/M token and the slots=H/M held exhaustion suffix); implemented by T38 -->
<!-- BUG-530 — fixed: IN-3 narrowed to "one of the two legitimate causes" (its exact-cause assertion depended on an unenforceable spawn-skew bound), and IN-8 registered below to carry that assertion deterministically; implemented by T23 -->

---

### IN-1: 2 racers, `--max-sessions 1`, 0 pre-existing occupiers → losing racer's stderr names `"slot held by another session"`

- **Given:** `CLR_GATE_DIR` and `CLR_PROC_DIR` freshly created and shared between two racers; `--max-sessions 1`; `--journal off`; no pre-existing dead owner is seeded. **Rendezvous (BUG-532):** racer A is launched first with a fake `claude` that announces itself and then sleeps, and racer B is launched only once that announcement has been observed — so A provably holds the slot, with a live PID, at the moment B reads it. `CLR_PROC_DIR` stays empty, so the census reads `count_u32 = 0 < max = 1` for both and B is denied on the slot-CAS rather than on capacity
- **When:** racer B invokes `clr --print --max-sessions 1` against the slot A is holding, with stderr captured (not `Stdio::null()`) for both
- **Then:** racer B's captured stderr contains the literal substring `"slot held by another session"`, and racer A's does not
- **Note:** `bug_reproducer(BUG-396)`/`bug_reproducer(BUG-532)` — reproduces the corrected classification: the loser observes the winner's slot record (alive), which is `HeldByLive`, never a race, since no dead owner and no reclaim ticket are ever involved in this fixture. **Scope strengthened by BUG-532:** this case originally spawned both racers back-to-back and asserted only that *exactly one* named the cause. That left the overlap unenforced — the winner's critical section was microseconds, so once spawn skew exceeded it the winner released before the loser read, the loser claimed a free slot and printed nothing, and the test timed out waiting for a message that was never coming. With the rendezvous the overlap is established by construction, which also makes the winner deterministic, so the assertion now names *which* racer must report the hold
- **Source:** [invariant/013_slot_wait_message_differentiation.md](../../../docs/invariant/013_slot_wait_message_differentiation.md) Invariant Statement table row 2 (`has_capacity=true`, `HeldByLive`)

---

### IN-2: Same race as IN-1 → neither racer's stderr names `"at capacity"` or `"lost reservation race"`

- **Given:** identical fixture to IN-1, including its BUG-532 rendezvous — 2 racers, `--max-sessions 1`, 0 pre-existing occupiers, both stderr streams captured
- **When:** the same two-racer invocation as IN-1
- **Then:** neither racer's captured stderr contains the literal substring `"at capacity"` (the census scans an empty `CLR_PROC_DIR`, so `count_u32 = 0 < max` even while A actually holds the slot — a live racer does not raise the census, since `find_claude_processes()` never looks at the real `/proc`) nor `"lost reservation race"` (no dead owner exists in this fixture, so no reclaim-ticket contention is possible)
- **Note:** `bug_reproducer(BUG-393)`/`bug_reproducer(BUG-396)` — proves the fix selects among all three suffixes correctly rather than defaulting any non-capacity denial to "race"
- **Source:** [invariant/013_slot_wait_message_differentiation.md](../../../docs/invariant/013_slot_wait_message_differentiation.md) Invariant Statement table row 2

---

### IN-3: 2 racers, `--max-sessions 1`, pre-seeded confirmed-dead owner → losing racer's first poll attempt names one of the two legitimate dead-owner-contention causes

- **Given:** the sole slot (`--max-sessions 1`) is pre-seeded with a confirmed-dead owner (a real `true` process spawned and reaped, so its PID is guaranteed not alive and not recyclable within the test window); two racers then contend to reclaim it, with `CLR_GATE_RECLAIM_TEST_DELAY_MS` injecting a delay to widen the reclaim-ticket contention window
- **When:** both racers observe `count_u32 < max` and a dead recorded owner, and at least one attempts the atomic reclaim-ticket sequence in `acquire_slot()`
- **Then:** the losing racer's stderr is non-empty (the winner returns immediately with no wait-loop output) and its first `gate-wait` line contains either `"lost reservation race"` or `"slot held by another session"` — and never any third cause
- **Note:** `bug_reproducer(BUG-396)`/`bug_reproducer(BUG-530)`. **Scope narrowed by BUG-530:** this case can no longer assert *which* of the two causes appears. `acquire_slot()` returns `HeldByLive` **before** `reclaim_test_delay()` (both in `gate_slot.rs`), so the injected delay widens only the window in which the dead owner stays visible — reaching the ticket branch requires both racers to execute their slot-read within it, a constraint on inter-process spawn skew that no fixture can enforce. Under parallel-suite load the winner's `rename()` routinely lands first and the loser then *correctly* reports a live hold. The deterministic assertion that `LostReclaimRace` produces `"lost reservation race"` moved to **IN-8**, which needs no race at all
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

---

### IN-7: Live-owned sole slot, empty census → poll line carries both `active=0/1` and `slots=1/1`; exhaustion message carries `slots=1/1 held`

- **Given:** the sole slot (`--max-sessions 1`) is pre-seeded with a slot file owned by the test process's own PID (guaranteed alive for the whole run, no child to manage) while `CLR_PROC_DIR` is left empty, so the census reads `0` sessions and the sweep's occupancy reads `1/1` — the two conjuncts of the admission condition are forced to diverge; `CLR_GATE_POLL_SECS=1`, `CLR_GATE_MAX_ATTEMPTS=2`, `--retry-override 0`, `CLR_GATE_STALE_SECS` explicitly removed (no staleness comparison ever runs against a live owner)
- **When:** one `clr -p --max-sessions 1` invocation polls to exhaustion with stderr captured
- **Then:** the invocation exits non-success; the denial line naming `reason: slot held by another session` contains BOTH the unchanged census half `"gate-wait  active=0/1"` AND the measured occupancy `"slots=1/1"`; the `session gate timed out` message contains `"slots=1/1 held"`
- **Note:** `bug_reproducer(BUG-480)` — the census/occupancy divergence (`0/1` vs `1/1`) is the load-bearing fixture property: a fixture where both counters agree could pass even if `slots=` merely echoed the census counter. Empirically confirmed to fail pre-fix (denial line carried no `slots=` token) and pass post-fix. At-capacity lines are deliberately NOT asserted to carry `slots=` — the exemption is pinned by the unchanged T29/T31 full-line guards in `tests/concurrency_gate_ext2_test.rs`
- **Source:** [invariant/013_slot_wait_message_differentiation.md](../../../docs/invariant/013_slot_wait_message_differentiation.md) § Invariant Statement, "Measured occupancy (BUG-480)"

---

### IN-8: Dead-owner sole slot with a pre-seeded live reclaim ticket → stderr names `"lost reservation race"`, deterministically and with no second process

- **Given:** the sole slot (`--max-sessions 1`) is pre-seeded with a confirmed-dead owner (a real `true` process spawned and reaped, so its PID is guaranteed not alive), AND that owner's reclaim ticket `reclaim_0_{dead_pid}_0.lock` is pre-seeded as already claimed by the test process's own PID (guaranteed alive for the whole run); `CLR_PROC_DIR` left empty so the census never denies first; `CLR_GATE_POLL_SECS=1`, `CLR_GATE_MAX_ATTEMPTS=2`, `--retry-override 0`, `--journal off`; exactly one `clr` process is launched
- **When:** that single invocation observes `count_u32 < max` and a dead recorded owner, proceeds past the `HeldByLive` early return into the reclaim-ticket branch, and finds the ticket already held by a live claimant
- **Then:** the invocation terminates within a bounded wait and its captured stderr contains the literal substring `"lost reservation race"`, and contains neither `"slot held by another session"` nor `"at capacity"`
- **Note:** `bug_reproducer(BUG-396)`/`bug_reproducer(BUG-530)` — this is the deterministic counterpart to IN-3, carrying the exact-cause assertion IN-3 surrendered. The fixture's load-bearing property is that **both** the dead slot record and its live-claimed ticket are pre-seeded: that combination is the only state from which `acquire_slot()` can reach `LostReclaimRace`, and pre-seeding it removes every timing dependency — no second process, no `CLR_GATE_RECLAIM_TEST_DELAY_MS`, no spawn-skew bound. Per BUG-530, `acquire_slot()` returns `HeldByLive` before `reclaim_test_delay()` (both in `gate_slot.rs`) ever runs, which is precisely why a two-racer fixture cannot guarantee this branch is reached at all
- **Source:** [invariant/013_slot_wait_message_differentiation.md](../../../docs/invariant/013_slot_wait_message_differentiation.md) Invariant Statement table row 3 (`has_capacity=true`, `LostReclaimRace`)
