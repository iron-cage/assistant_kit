# Test: Invariant — Gate Slot Atomicity

Test case planning for [invariant/012_gate_slot_atomicity.md](../../../docs/invariant/012_gate_slot_atomicity.md). Tests validate that `wait_for_session_slot()`'s admission condition never admits more concurrent print-mode sessions than `--max-sessions`, on both the fresh-claim path and the dead-owner ticket-arbitrated reclaim path (including walking a chain of orphaned reclaim tickets left by a claimant that died before completing its own handoff), and that a denial at the count-derived index falls back to scanning every other index in `0..max` before giving up.

**Source:** [invariant/012_gate_slot_atomicity.md](../../../docs/invariant/012_gate_slot_atomicity.md)
**Related:** [invariant/013_slot_wait_message_differentiation.md](../../../docs/invariant/013_slot_wait_message_differentiation.md) (message-text differentiation for the same non-admission causes this invariant's admission condition produces)

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| IN-1 | 8 concurrent real `clr` invocations, `--max-sessions 3` → peak concurrently-held slots never exceeds the limit | Invariant Hold |
| IN-2 | Pre-seeded confirmed-dead owner, 8 concurrent racers reclaim → peak concurrently-admitted children never exceeds 1 | Invariant Hold |
| IN-3 | Orphaned reclaim ticket (ticket's own claimant also confirmed dead) → fresh caller still acquires the slot promptly, not permanently blocked | Invariant Hold |
| IN-4 | Count-derived candidate index `HeldByLive`, another index completely free → fallback scan admits promptly instead of starving | Invariant Hold |

## Test Coverage Summary

- Invariant Hold: 4 tests (IN-1, IN-2, IN-3, IN-4)

**Total:** 4 invariant test cases (minimum for `invariant` doc type is 2; this spec exceeds it to cover the fresh-claim burst case, the single-generation reclaim race, the multi-generation orphaned-ticket chain walk, and the fallback-scan starvation guard)

## Architectural Constraint

All 4 cases are integration tests in `tests/concurrency_gate_test.rs` — the atomicity guarantee lives entirely inside `acquire_slot()` and its call site in `wait_for_session_slot()`, and can only be observed by launching real `clr` subprocesses (or, for IN-1/IN-2, real concurrent racers) against a shared `CLR_GATE_DIR` fixture. IN-3 exercises the chain-walking loop added for BUG-402: when a reclaim ticket's own recorded claimant is also dead (never reached `rename()`), `acquire_slot()` advances to the next-generation ticket keyed by that dead claimant's own `(pid, since)` instead of treating the existing ticket as permanent defeat, rechecking the slot's current owner at each generation in case a concurrent caller completed a rename while this call was still walking the chain.

## Implementation Notes

| ID | Test Function | File | Status |
|----|---------------|------|--------|
| IN-1 | `t08_concurrent_clr_invocations_never_exceed_max_sessions` | `tests/concurrency_gate_test.rs` | ✅ |
| IN-2 | `t14_reclaim_race_admits_at_most_one_caller_for_a_dead_owners_slot` | `tests/concurrency_gate_test.rs` | ✅ |
| IN-3 | `t17_orphaned_reclaim_ticket_does_not_permanently_block_slot` | `tests/concurrency_gate_test.rs` | ✅ |
| IN-4 | `t18_gate_tries_other_free_index_when_count_derived_index_is_live_held` | `tests/concurrency_gate_test.rs` | ✅ |

---

### IN-1: 8 concurrent real `clr` invocations, `--max-sessions 3` → peak concurrently-held slots never exceeds the limit

- **Given:** `CLR_GATE_DIR` and `CLR_PROC_DIR` freshly created and shared across 8 concurrently-launched real `clr --print` invocations; `--max-sessions 3`; a background thread mirrors each spawned child into the shared proc dir so the live count actually varies during the burst
- **When:** all 8 invocations race the admission check simultaneously
- **Then:** the peak number of concurrently-held slots observed at any instant during the burst never exceeds 3
- **Note:** `bug_reproducer(BUG-387)` — the original check-then-act race this invariant's atomic reservation scheme closes; timing-dependent and does not reproduce under sequential/manual testing
- **Source:** [invariant/012_gate_slot_atomicity.md](../../../docs/invariant/012_gate_slot_atomicity.md) Invariant Statement table row 1

---

### IN-2: Pre-seeded confirmed-dead owner, 8 concurrent racers reclaim → peak concurrently-admitted children never exceeds 1

- **Given:** the sole slot is pre-seeded with a confirmed-dead owner (a real `true` process spawned and reaped); 8 concurrent `clr` invocations then race to reclaim it
- **When:** all 8 racers observe the same dead-owner record and attempt the atomic reclaim-ticket sequence in `acquire_slot()`
- **Then:** the peak number of concurrently-admitted children never exceeds 1 — exactly one racer wins the reclaim ticket
- **Note:** `bug_reproducer(BUG-392)` — proves the ticket-arbitrated handoff is atomic against concurrent reclaimers, closing the `remove_file()` + `claim_slot_file()` two-step race the prior reclaim sequence had
- **Source:** [invariant/012_gate_slot_atomicity.md](../../../docs/invariant/012_gate_slot_atomicity.md) Invariant Statement table row 4

---

### IN-3: Orphaned reclaim ticket (ticket's own claimant also confirmed dead) → fresh caller still acquires the slot promptly

- **Given:** two distinct, independently confirmed-dead PIDs — one recorded as the slot's owner, one recorded as the pre-existing reclaim ticket's claimant (mirroring the exact on-disk state a reclaimer would leave behind had it won the ticket and then crashed before `rename()`); `--max-sessions 1`; no live contender anywhere in the chain
- **When:** a fresh `clr --print --max-sessions 1` invocation calls `acquire_slot()` against this fixture
- **Then:** the invocation is admitted (exit 0) within the gate-wait budget, and stderr never contains `"session gate timed out"` — the pre-existing orphaned ticket does not permanently block the slot
- **Note:** `bug_reproducer(BUG-402)` — proves `acquire_slot()` walks to the next-generation ticket (keyed by the dead claimant's own `(pid, since)`) instead of treating a single pre-existing ticket as permanent defeat
- **Source:** [invariant/012_gate_slot_atomicity.md](../../../docs/invariant/012_gate_slot_atomicity.md) Violation Consequences, "If a reclaim ticket's own claimant terminates before completing the `rename()` handoff"

---

### IN-4: Count-derived candidate index `HeldByLive`, another index completely free → fallback scan admits promptly instead of starving

- **Given:** `--max-sessions 2`; the count-derived index (1) is pre-seeded as genuinely `HeldByLive` by a real, alive occupier PID; the other index (0) is left completely unclaimed
- **When:** a fresh `clr --print --max-sessions 2` invocation calls `acquire_slot()`, fails at the count-derived index, and falls back to scanning `0..max`
- **Then:** the invocation is admitted (exit 0) promptly via index 0, instead of exhausting the gate-wait budget on the single count-derived index alone
- **Note:** `bug_reproducer(BUG-404)` — proves a denial at one index does not starve a waiter while another index sits completely free
- **Source:** [invariant/012_gate_slot_atomicity.md](../../../docs/invariant/012_gate_slot_atomicity.md) Invariant Statement table row 3, Violation Consequences "If the admission call site reverts to trying only the count-derived index"
