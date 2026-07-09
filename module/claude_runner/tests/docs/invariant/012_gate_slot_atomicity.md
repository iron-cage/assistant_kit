# Test: Invariant — Gate Slot Atomicity

Test case planning for [invariant/012_gate_slot_atomicity.md](../../../docs/invariant/012_gate_slot_atomicity.md). Tests validate that `wait_for_session_slot()`'s admission condition never admits more concurrent print-mode sessions than `--max-sessions`, on both the fresh-claim path and the dead-owner ticket-arbitrated reclaim path (including walking a chain of orphaned reclaim tickets left by a claimant that died before completing its own handoff, and cleaning up a ticket this same caller just won but failed to complete so it can never lose a fair race to its own still-running self), and that a denial at the count-derived index falls back to scanning every other index in `0..max` before giving up.

**Source:** [invariant/012_gate_slot_atomicity.md](../../../docs/invariant/012_gate_slot_atomicity.md)
**Related:** [invariant/013_slot_wait_message_differentiation.md](../../../docs/invariant/013_slot_wait_message_differentiation.md) (message-text differentiation for the same non-admission causes this invariant's admission condition produces)

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| IN-1 | 8 concurrent real `clr` invocations, `--max-sessions 3` → peak concurrently-held slots never exceeds the limit | Invariant Hold |
| IN-2 | Pre-seeded confirmed-dead owner, 8 concurrent racers reclaim → peak concurrently-admitted children never exceeds 1 | Invariant Hold |
| IN-3 | Orphaned reclaim ticket (ticket's own claimant also confirmed dead) → fresh caller still acquires the slot promptly, not permanently blocked | Invariant Hold |
| IN-4 | Count-derived candidate index `HeldByLive`, another index completely free → fallback scan admits promptly instead of starving | Invariant Hold |
| IN-5 | Ticket winner forced to fail its own tmp-claim → does not self-deny forever on retry | Invariant Hold |
| IN-6 | Three-generation orphaned-ticket chain (two dead claimants stacked) → chain-walk still acquires the slot fresh | Invariant Hold |
| IN-7 | Live owner with decades-old `since` → unreclaimable by default; reclaimable once `CLR_GATE_STALE_SECS` is set below the elapsed age | Invariant Hold |
| IN-8 | Single uncontested fresh claim → slot file content is fully-valid JSON immediately, no create-then-populate window observable | Invariant Hold |
| IN-9 | 8 racers contending for one never-before-seen slot path → peak concurrently-admitted children never exceeds 1 | Invariant Hold |
| IN-10 | Pre-seeded 0-byte slot file (content already corrupted before any call) → fresh caller still denied, identically pre-fix and post-fix | Documented Residual |

## Test Coverage Summary

- Invariant Hold: 9 tests (IN-1, IN-2, IN-3, IN-4, IN-5, IN-6, IN-7, IN-8, IN-9)
- Documented Residual: 1 test (IN-10)

**Total:** 10 invariant test cases (minimum for `invariant` doc type is 2; this spec exceeds it to cover the fresh-claim burst case, the single-generation reclaim race, the multi-generation orphaned-ticket chain walk, the fallback-scan starvation guard, the ticket-winner self-collision guard, the multi-generation chain-walk regression guard, the opt-in live-owner staleness threshold, the claim-vs-content atomicity of `claim_slot_file()` itself, the fresh-claim arbitration regression guard, and the explicitly accepted pre-existing-corruption residual boundary)

## Architectural Constraint

All 10 cases are integration tests in `tests/concurrency_gate_test.rs` — the atomicity guarantee lives entirely inside `acquire_slot()` and its call site in `wait_for_session_slot()`, and can only be observed by launching real `clr` subprocesses (or, for IN-1/IN-2/IN-9, real concurrent racers) against a shared `CLR_GATE_DIR` fixture. IN-3 and IN-6 exercise the chain-walking loop added for BUG-402: when a reclaim ticket's own recorded claimant is also dead (never reached `rename()`), `acquire_slot()` advances to the next-generation ticket keyed by that dead claimant's own `(pid, since)` instead of treating the existing ticket as permanent defeat, rechecking the slot's current owner at each generation in case a concurrent caller completed a rename while this call was still walking the chain. IN-5 exercises the self-collision cleanup added for BUG-405 via `CLR_GATE_FORCE_TMP_CLAIM_FAIL_ONCE`, a test-only injection point mirroring the existing `reclaim_test_delay()` idiom used to make an otherwise-narrow race deterministically reproducible. IN-7 exercises the opt-in staleness threshold added for BUG-400 via `CLR_GATE_STALE_SECS`: unlike IN-2/IN-3/IN-6, the pre-seeded owner is a genuinely alive real child process (not a spawned-and-reaped dead one) — the sub-case split (threshold unset vs. set) is what proves the fix is additive rather than a behavior change for existing callers. IN-8/IN-9/IN-10 exercise `claim_slot_file()`'s own claim-vs-content atomicity added for BUG-407: IN-8 and IN-9 are direct-correctness/regression guards (both pass identically pre-fix and post-fix in the absence of an injected crash — no red/green transition is claimed or required for either); IN-10 is a Documented Residual category (distinct from Invariant Hold) — it does not assert the invariant holds for pre-existing corruption, it asserts the KNOWN absence of that guarantee is stable across the fix, so a future change cannot silently narrow or widen the boundary unnoticed.

## Implementation Notes

| ID | Test Function | File | Status |
|----|---------------|------|--------|
| IN-1 | `t08_concurrent_clr_invocations_never_exceed_max_sessions` | `tests/concurrency_gate_test.rs` | ✅ |
| IN-2 | `t14_reclaim_race_admits_at_most_one_caller_for_a_dead_owners_slot` | `tests/concurrency_gate_test.rs` | ✅ |
| IN-3 | `t17_orphaned_reclaim_ticket_does_not_permanently_block_slot` | `tests/concurrency_gate_test.rs` | ✅ |
| IN-4 | `t18_gate_tries_other_free_index_when_count_derived_index_is_live_held` | `tests/concurrency_gate_test.rs` | ✅ |
| IN-5 | `t19_ticket_winner_that_fails_own_admission_does_not_self_deny_forever` | `tests/concurrency_gate_test.rs` | ✅ |
| IN-6 | `t20_acquire_slot_walks_multi_generation_reclaim_ticket_chain` | `tests/concurrency_gate_test.rs` | ✅ |
| IN-7 | `t21_stale_alive_owner_becomes_reclaimable_when_threshold_set` | `tests/concurrency_gate_test.rs` | ✅ |
| IN-8 | `t22_claim_slot_file_content_valid_immediately_after_call` | `tests/concurrency_gate_test.rs` | ✅ |
| IN-9 | `t23_concurrent_racers_still_yield_exactly_one_winner` | `tests/concurrency_gate_test.rs` | ✅ |
| IN-10 | `t24_preexisting_empty_slot_file_remains_a_documented_residual` | `tests/concurrency_gate_test.rs` | ✅ |

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

---

### IN-5: Ticket winner forced to fail its own tmp-claim → does not self-deny forever on retry

- **Given:** a pre-seeded dead-owner slot with no pre-existing reclaim ticket; `CLR_GATE_FORCE_TMP_CLAIM_FAIL_ONCE` arms a one-shot injection point forcing the real `clr` binary's own ticket-win branch to fail its tmp-claim exactly once immediately after winning the ticket
- **When:** the same invocation (fixed `pid`/`since` for its entire `wait_for_session_slot()` lifetime) retries after the forced failure
- **Then:** the invocation acquires the slot on a later retry instead of reading back its own abandoned ticket as a live contender and self-denying on every subsequent attempt
- **Note:** `bug_reproducer(BUG-405)` — proves both non-admission paths in the ticket-win branch (tmp-claim failure, rename failure) remove the ticket they just won before returning, so a caller cannot lose a fair race to its own still-running self
- **Source:** [invariant/012_gate_slot_atomicity.md](../../../docs/invariant/012_gate_slot_atomicity.md) Enforcement Mechanism : 3, "Self-collision cleanup (Fix(BUG-405))"

---

### IN-6: Three-generation orphaned-ticket chain (two dead claimants stacked) → chain-walk still acquires the slot fresh

- **Given:** a slot owned by a confirmed-dead PID, with a chain of two stacked reclaim tickets — each keyed to the previous generation's dead claimant — before an unclaimed final generation; no live contender anywhere in the chain
- **When:** a fresh caller's `acquire_slot()` walks the chain, rechecking each next claimant's liveness before advancing
- **Then:** the caller advances past both dead generations and acquires the slot fresh, instead of treating the first pre-existing ticket as permanent defeat
- **Note:** confirms the chain-walk capability added for BUG-402 generalizes beyond a single hop — not merely a two-party race
- **Source:** [invariant/012_gate_slot_atomicity.md](../../../docs/invariant/012_gate_slot_atomicity.md) Enforcement Mechanism : 3, "Chain walk (Fix(BUG-402))"

---

### IN-7: Live owner with decades-old `since` → unreclaimable by default; reclaimable once `CLR_GATE_STALE_SECS` is set below the elapsed age

- **Given:** a slot owned by a genuinely alive real child process (a real `/bin/sleep` child, not a `claude`/`clr` process — `pid_alive()` only checks `/proc/{pid}` existence) with `since: 0` recorded (an elapsed age of decades); `--max-sessions 1`; each sub-case uses its own `gate_dir` and its own owner process
- **When:** sub-case a runs a fresh `clr --print --max-sessions 1` invocation with `CLR_GATE_STALE_SECS` unset; sub-case b runs the same invocation shape with `CLR_GATE_STALE_SECS=10`
- **Then:** sub-case a exhausts its gate-wait budget with `"session gate timed out"` on stderr (unchanged pre-existing behavior); sub-case b is admitted (exit 0) promptly with no gate-timeout — the same live owner's slot became reclaim-eligible once its recorded age exceeded the threshold
- **Note:** `bug_reproducer(BUG-400)` — proves the reclaim-eligibility test gained a supplementary staleness condition without changing behavior for any caller that has not opted in via `CLR_GATE_STALE_SECS`
- **Source:** [invariant/012_gate_slot_atomicity.md](../../../docs/invariant/012_gate_slot_atomicity.md) Enforcement Mechanism : 3, "Staleness threshold (Fix(BUG-400))"

---

### IN-8: Single uncontested fresh claim → slot file content is fully-valid JSON immediately, no create-then-populate window observable

- **Given:** a completely empty gate dir (no pre-existing slot file at any index); a single `clr --print --max-sessions 6` invocation with no contenders
- **When:** the invocation completes its fresh claim and exits, and the test immediately reads `slot_0.json` off disk
- **Then:** the invocation is admitted (exit 0); the on-disk content is fully-formed JSON containing both a `pid` and a `since` field, never a partially-written or empty artifact; the recorded `pid` matches this `clr` invocation's own pid
- **Note:** direct-correctness check of the rewritten `claim_slot_file()` — proves there is no create-then-populate window to observe by construction, since the on-disk path only ever becomes visible via `hard_link()` from an already-fully-written temp file. Passes identically pre-fix and post-fix (no injected crash is present to distinguish the two) — this is a direct-correctness guard, not a red/green regression demonstration
- **Source:** [invariant/012_gate_slot_atomicity.md](../../../docs/invariant/012_gate_slot_atomicity.md) Enforcement Mechanism : 2, "Claim-vs-content atomicity (Fix(BUG-407))"

---

### IN-9: 8 racers contending for one never-before-seen slot path → peak concurrently-admitted children never exceeds 1

- **Given:** a completely empty gate dir and proc dir (no pre-seeded slot, no pre-existing occupiers); 8 concurrently-launched real `clr --print --max-sessions 1` invocations, all forced onto the same index (0) by the shared `--max-sessions 1` ceiling
- **When:** all 8 racers call `acquire_slot()` against the same never-before-seen slot path simultaneously
- **Then:** the peak number of concurrently-alive dispatched children observed at any instant never exceeds 1, and at least 1 racer is admitted
- **Note:** arbitration-preserved regression guard — confirms the write-to-temp-then-`hard_link()` rewrite did not weaken the exactly-one-claimant guarantee every call site depends on, since `hard_link()` fails with `AlreadyExists` exactly like `create_new()` did. Passes identically pre-fix and post-fix — a regression guard, not a red/green demonstration
- **Source:** [invariant/012_gate_slot_atomicity.md](../../../docs/invariant/012_gate_slot_atomicity.md) Enforcement Mechanism : 2, "Claim-vs-content atomicity (Fix(BUG-407))"

---

### IN-10: Pre-seeded 0-byte slot file (content already corrupted before any call) → fresh caller still denied, identically pre-fix and post-fix

- **Given:** `slot_0.json` pre-seeded as a 0-byte file BEFORE any call to the rewritten `claim_slot_file()` ever touches it (simulating a stray `touch` or a leftover artifact from a crash under a pre-upgrade binary); `--max-sessions 1`
- **When:** a fresh `clr --print --max-sessions 1` invocation calls `acquire_slot()` against this fixture
- **Then:** the invocation exits within the gate-wait budget with `"session gate timed out"` on stderr — denied, not admitted
- **Note:** Documented Residual, not Invariant Hold — this test does NOT assert the invariant holds for pre-existing corruption; it asserts the KNOWN absence of that guarantee is STABLE across the fix. `hard_link()`, like `create_new()`, cannot claim a path that already exists, so a fresh-claim attempt against pre-existing corrupted content still returns `false`, and `acquire_slot()`'s unconditional `None` → `HeldByLive` branch (Fix(BUG-396), unchanged by this fix) still denies forever — matching this fix's own Fix Location scope ("a `None` result ... can only mean genuine on-disk corruption unrelated to this race (out of scope)"). This scope boundary was confirmed via Tier 4 Paired Verification (independent primary and adversarial code trace) after BUG-407's own filing was found to contain a self-contradictory Prevention sketch (recovery expected) versus its more precise Fix Location section. If this test ever starts PASSING with no timeout, the residual boundary has shifted and both this doc and BUG-407's closure notes must be updated to reflect the new behavior — do not silently let that pass uninvestigated
- **Source:** [invariant/012_gate_slot_atomicity.md](../../../docs/invariant/012_gate_slot_atomicity.md) Enforcement Mechanism : 2, "Explicitly accepted residual"
