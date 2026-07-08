# Invariant: Gate Slot Atomicity

### Scope

- **Purpose**: Guarantee that the `--max-sessions` concurrency gate never admits more concurrent print-mode sessions than the configured limit, even when multiple `clr` invocations race the admission check simultaneously — on both the fresh-claim path and the dead-owner reclaim path.
- **Responsibility**: State the atomic reservation mechanism that closes the check-then-act race, which function owns it, and why the reservation must outlive the wait loop itself.
- **In Scope**: `slot_path()`, `claim_slot_file()`, `acquire_slot()`, the admission condition in `wait_for_session_slot()`, the index-derivation rule that makes atomicity meaningful across racers, the fallback scan across every other index when the count-derived candidate is unavailable (see Provenance : BUG-404), the ticket-arbitrated reclaim of dead-owner slots — including walking a chain of orphaned reclaim tickets left behind by a claimant that died before completing its own handoff, so the reclaim path self-heals from an interrupted reclaimer just as it self-heals from an interrupted original owner (see Provenance : BUG-392, BUG-402), and cleaning up a ticket this same caller just won but failed to complete, so a caller can never lose a fair race to its own still-running self (see Provenance : BUG-405) — and `claim_slot_file()`'s own guarantee that a claim and its content are published together, so no path it claims is ever observable existing-but-empty (see Provenance : BUG-407).
- **Out of Scope**: Gate poll interval / attempt-limit configuration (→ `cli/003_env_param.md` Env Param 5), gate-state JSON escaping (→ BUG-384, `json_escape_str()`), `clr ps` queued-table display of gate files (→ `ps.rs` `build_queued_table()` directly), gate-timeout retry behavior (→ `006_exit_codes.md`).

### Invariant Statement

When `wait_for_session_slot()` evaluates admission for a candidate slot index, it MUST require both: (a) the live print-mode process count is below `--max-sessions`, AND (b) this process wins an atomic, kernel-arbitrated reservation at that same index. Neither condition alone is sufficient — (a) alone is the pre-BUG-387 check-then-act race; (b) alone without deriving the index from the just-read count would arbitrate between racers targeting unrelated paths, arbitrating nothing.

| Condition | Outcome |
|-----------|---------|
| Live count `< max` AND slot reservation at the count-derived index succeeds | Admitted — proceed to spawn the session |
| Live count `>= max` | Not admitted — no reservation attempted; falls to wait-and-retry |
| Live count `< max` BUT slot reservation at the count-derived index fails (another racer already holds it) | Falls back to scanning every other index in `0..max`; admitted on the first one that succeeds (see Provenance : BUG-404) — not admitted only if every index in `0..max` is unavailable, in which case it falls to wait-and-retry exactly as the `>= max` case does |
| A losing reservation attempt (at any index tried, count-derived or fallback) finds the existing slot's owning PID no longer alive | Slot is reclaimed via a ticket-arbitrated atomic handoff in the same call — race-free against other simultaneous reclaimers, and self-healing **even when a reclaiming caller is itself interrupted**, by walking the resulting chain of orphaned tickets (see Provenance : BUG-392, BUG-402) |

<!-- BUG-400 — this table defines only two owner states (alive/dead); no third alive-but-stalled category exists, so a live-but-non-progressing owner blocks reclaim indefinitely with no staleness escape hatch -->
<!-- BUG-402 — fixed: acquire_slot() walks the reclaim-ticket chain, rechecking each next claimant's liveness before advancing, so a reclaiming caller's own death no longer permanently blocks the slot index. The regression test for this (T17) passes reliably, including under full-suite concurrent execution; see Provenance : BUG-402 -->
<!-- BUG-404 — prior to this fix, a denial at the count-derived index alone fell straight to wait-and-retry with no fallback scan, even while other indices sat free or dead-and-reclaimable -->

**Reservation lifetime:** once acquired, a slot is held for the entire remaining lifetime of the session — not just the polling wait. There is deliberately no `Drop` guard releasing it when `wait_for_session_slot()` returns. Releasing early would free the slot before the admitted child process becomes `/proc`-visible, reopening the exact race this invariant closes.

### Rationale — Why a Fixed-Index Reservation, Not a PID-Keyed One

The slot index passed to `acquire_slot()` is the same live-process count the caller just read, not a private per-caller identifier (e.g. PID-keyed). This is load-bearing: concurrent invocations racing on the same stale count all target the **identical** slot path, so `create_new`'s atomicity genuinely arbitrates between them — exactly one wins, regardless of how many are racing, on both the fresh-claim path and (via the ticket file keyed off the same identical dead-owner record every racer reads) the reclaim path (see Provenance : BUG-392). A PID-keyed variant (each racer reserving its own uniquely-named slot, gated by a preceding non-atomic count check) was evaluated and confirmed still racy, for exactly the reason a PID-keyed path never collides with anything and therefore never arbitrates.

Deriving the index from `find_claude_processes()`'s live count — rather than a private `clr`-only counter — also preserves system-wide accounting: `--max-sessions` counts every `claude` print-mode process on the host, launched via `clr` or not (`cli/param/033_max_sessions.md`). Substituting a `clr`-only view would go blind to non-`clr`-launched sessions occupying the same capacity.

### Enforcement Mechanism

Four functions in `src/cli/gate.rs` collaborate to enforce this invariant:

**1. `slot_path()` — deterministic per-index path:**
```rust
fn slot_path( dir : &Path, index : u32 ) -> PathBuf
{
  dir.join( format!( "slot_{index}.json" ) )
}
```

**2. `claim_slot_file()` — atomic kernel-arbitrated claim, published as one unit with its content:**
```rust
fn claim_slot_file( path : &Path, pid : u32, since : u64 ) -> bool
{
  let content = format!( r#"{{"pid":{pid},"since":{since}}}"# );
  let dir     = path.parent().unwrap_or_else( || Path::new( "." ) );
  let tmp     = dir.join( format!( "claim_tmp_{pid}_{since}" ) );
  if std::fs::write( &tmp, &content ).is_err()
  {
    return false;
  }
  let claimed = std::fs::hard_link( &tmp, path ).is_ok();
  let _ = std::fs::remove_file( &tmp );
  claimed
}
```
The content is written to a uniquely-named temp file FIRST; only once it is fully durable does `hard_link()` publish it at `path`. `hard_link()` fails with `AlreadyExists` exactly like `create_new()` does (Fix(BUG-407)) — preserving the same single-atomic-kernel-operation arbitration property `create_new` alone provided — but now `path` never becomes visible to a concurrent reader before its content is complete. The pre-fix version called `create_new(true).open(path)` directly, then wrote the content as a separate, subsequent statement — `path` became visible to concurrent readers the instant the create succeeded, before its content existed; a process killed in between left `path` permanently existing but empty (see Provenance : BUG-407).

**3. `acquire_slot()` — claim, or reclaim a dead owner's slot by walking the reclaim-ticket chain via atomic handoff:**
```rust
fn acquire_slot( dir : &Path, index : u32, pid : u32, since : u64 ) -> Result< (), SlotDenialCause >
{
  let path = slot_path( dir, index );
  if claim_slot_file( &path, pid, since )
  {
    return Ok( () );
  }
  let Some( ( owner, owner_since ) ) = read_slot_owner_record( &path )
  else
  {
    return Err( SlotDenialCause::HeldByLive );
  };
  if pid_alive( owner )
  {
    return Err( SlotDenialCause::HeldByLive );
  }
  reclaim_test_delay();
  let mut ticket_owner = owner;
  let mut ticket_since = owner_since;
  loop
  {
    let ticket = dir.join( format!( "reclaim_{index}_{ticket_owner}_{ticket_since}.lock" ) );
    if claim_slot_file( &ticket, pid, since )
    {
      let tmp = dir.join( format!( "reclaim_tmp_{index}_{pid}" ) );
      if !claim_slot_file( &tmp, pid, since )
      {
        let _ = std::fs::remove_file( &ticket );
        return Err( SlotDenialCause::LostReclaimRace );
      }
      return if std::fs::rename( &tmp, &path ).is_ok()
      {
        Ok( () )
      }
      else
      {
        let _ = std::fs::remove_file( &tmp );
        let _ = std::fs::remove_file( &ticket );
        Err( SlotDenialCause::LostReclaimRace )
      };
    }
    let Some( ( next_claimant, next_claimant_since ) ) = read_slot_owner_record( &ticket )
    else
    {
      return Err( SlotDenialCause::LostReclaimRace );
    };
    if pid_alive( next_claimant )
    {
      return Err( SlotDenialCause::LostReclaimRace );
    }
    let Some( ( current_owner, _ ) ) = read_slot_owner_record( &path )
    else
    {
      return Err( SlotDenialCause::HeldByLive );
    };
    if current_owner != owner
    {
      return Err( SlotDenialCause::HeldByLive );
    }
    ticket_owner = next_claimant;
    ticket_since = next_claimant_since;
  }
}
```
The reclaim branch (Fix(BUG-392)) reuses `claim_slot_file()`'s `create_new` atomicity twice more: first to arbitrate a ticket keyed by `(index, owner pid, owner since)` — every racer observing the identical dead-owner record computes the identical ticket path, so exactly one wins it — then to write a per-caller-unique temp file. Only the ticket winner reaches the `rename()`, which POSIX guarantees is an atomic replace: unlike the prior `remove_file()` + `claim_slot_file()` sequence, the destination path is never observably absent in between. After a SUCCESSFUL rename, the ticket file is deliberately never cleaned up — removing it would let a later caller win a "new" ticket for an already-completed generation and clobber the legitimate holder via its own `rename()`. On the two non-admission paths (tmp-claim failure, rename failure), the ticket winner now removes its own ticket before returning (Fix(BUG-405)) — otherwise this same caller's own later retry reads back its own abandoned claim as a live contender and self-denies permanently; see Provenance : BUG-405. Ticket and temp filenames use a `reclaim_` prefix (never `slot_`) and no `.json` extension, keeping them invisible to both `ps.rs::build_queued_table()` (filters on `.json`) and this crate's own `count_live_held_slots()` test helper (filters on a `slot_`-prefixed stem, regardless of extension).

**Chain walk (Fix(BUG-402)):** if the ticket at the current generation is already claimed, that does not necessarily mean a live reclaimer is actively contending it — the claimant recorded on that ticket may itself be dead, having won the ticket and then died before reaching `rename()`. Rather than treat any pre-existing ticket as permanent defeat, the `loop` reads the ticket's own recorded claimant and, if that claimant is also dead, advances to the *next*-generation ticket keyed by the dead claimant's own `(pid, since)` and retries the identical atomic arbitration. Before advancing, it rechecks the slot's current owner against the original `owner` read at the top of the function — if a concurrent caller has since completed its own `rename()` onto the slot, the chain no longer leads anywhere and the call correctly reports `HeldByLive` instead of continuing to chase it. The chain is finite (bounded by however many dead-claimant ticket files already exist on disk for this index), so the loop always terminates — either by winning an unclaimed generation, by finding a live claimant genuinely contending it (`LostReclaimRace`), or by observing the slot has already been claimed by someone else (`HeldByLive`).

**Self-collision cleanup (Fix(BUG-405)):** winning the ticket is not the same as completing admission — the winner still has to win the temp-file claim and then `rename()` it onto the slot path, and either can fail (e.g. a transient fs fault). `pid`/`since` are fixed for the caller's entire `wait_for_session_slot()` invocation, reused unchanged across every polling attempt, so if the ticket were left in place after such a failure, the caller's own next retry would recompute the identical ticket path, find it already claimed, read back its own `(pid, since)` as `next_claimant`, and observe `pid_alive()` trivially `true` for its own still-running self — a permanent, self-inflicted `LostReclaimRace` on every subsequent retry, indistinguishable from genuine contention but with no other contender involved at all. Both non-admission returns in the ticket-win branch now remove the ticket they just won before returning, so the next retry re-contends this same generation fresh rather than reading back its own abandoned claim. This is orthogonal to the permanent-retention rule above: that rule governs the ticket left behind by a SUCCESSFUL rename (never removed, by design); this cleanup fires only when the winner is confirmed to have never reached that success.

**4. Admission call site in `wait_for_session_slot()`:**
```rust
let count = find_claude_processes()
  .iter()
  .filter( | p | super::ps::classify_mode( &p.args ) == "print" )
  .count();
let count_u32 = u32::try_from( count ).unwrap_or( u32::MAX );
let has_capacity = count_u32 < max;
let claim = if has_capacity
{
  // Try the count-derived index first (preserves the shared-stale-count
  // arbitration property above for the common contested-same-index case),
  // then fall back to every other index in 0..max — a denial at the single
  // count-derived index does not mean no index anywhere is available
  // (see Provenance : BUG-404).
  let mut result = acquire_slot( &dir, count_u32, pid, since );
  if result.is_err()
  {
    for candidate in 0..max
    {
      if candidate == count_u32 { continue; }
      result = acquire_slot( &dir, candidate, pid, since );
      if result.is_ok() { break; }
    }
  }
  Some( result )
}
else { None };
if let Some( Ok( () ) ) = claim
{
  // … journal GateWait event emission elided — irrelevant to the atomicity guarantee …
  return; // _guard.drop() removes only the {pid}.json telemetry file —
          // the slot reservation from acquire_slot() is deliberately
          // left in place for the rest of this session; see slot_path().
}
```

`pid_alive()` checks `/proc/{pid}` existence (Linux-only host assumption, matching the identical convention `build_queued_table()` already uses in `ps.rs`). `read_slot_owner_record()` reads back the `(pid, since)` pair a slot file records — the reclaim branch needs both fields to key its ticket path (see Enforcement Mechanism : 3 above), where the pre-BUG-392 `read_slot_owner()` returned only `pid`. Each `acquire_slot()` call is independently atomic (`create_new`); trying several within one attempt introduces no new race — it only widens which single index the attempt can land on (see Provenance : BUG-404).

### Violation Consequences

If the admission condition reverts to checking only `count_u32 < max` without `acquire_slot()`:
- Concurrent `clr` invocations can each observe the same stale sub-limit count before any of their spawned children become `/proc`-visible, jointly admitting more sessions than `--max-sessions` allows — the original BUG-387 symptom.
- The violation is timing-dependent and will not reproduce under sequential/manual testing — it requires genuine concurrent launches (see T08 in Tests below).

If the slot index is changed to a per-caller-unique value (e.g. PID-keyed) instead of the shared live-count value:
- `create_new`'s atomicity still succeeds for every racer (each targets a distinct path), so admission silently reverts to unbounded — the same race as above, just with an atomic-looking operation masking it.

If a `Drop` guard is added to release the slot when `wait_for_session_slot()` returns:
- The slot frees before the admitted child is spawned or becomes `/proc`-visible, reopening the race within the gap between gate-passage and child spawn.

If the reclaim branch reverts to `remove_file()` + `claim_slot_file()` without the ticket arbitration (the pre-BUG-392 shape):
- Two or more callers observing the same dead-owner record can each reclaim the slot — the second caller's `remove_file()` deletes the first caller's freshly-recreated file, and both then return `true` from `acquire_slot()` for the same index (see Provenance : BUG-392).
- This requires a slot owner to have died without releasing (`SIGKILL`, host crash/reboot) — narrower than the BUG-387 precondition, and will not reproduce via live-occupier races alone (see T14 in Tests below).

If the reclaim's ticket or temp files are cleaned up after a successful `rename()`:
- A later caller could win a "new" ticket keyed to the same `(index, pid, since)` and clobber the legitimate current holder via its own `rename()` — this is why both are left in place permanently rather than deleted.

If the reclaim-eligibility test remains `pid_alive(owner)` alone, with no supplementary staleness/elapsed-time condition:
- A slot owner that is alive but has stopped making forward progress (hung, deadlocked, suspended) is functionally equivalent to a dead one from the gate's perspective — it never releases capacity — but the current design has no path to reclaim from it. A waiter whose recomputed index collides with that owner is blocked for as long as the owner remains alive, independent of how low the aggregate live count drops (see Provenance : BUG-400).

If a reclaim ticket's own claimant terminates before completing the `rename()` handoff, with no liveness recheck on the ticket's own claimant:
- The ticket is never cleaned up (by design, to close the PID/timestamp-recycling hazard the Fix(BUG-392) comment describes), and every later caller targeting that index recomputes the identical ticket path and fails at the identical `claim_slot_file()` call — permanently, independent of how long ago the original dead owner exited and independent of how many later callers try. Unlike the BUG-400 scenario above, this block never self-resolves: it does not depend on any process remaining alive, and each additional occurrence permanently consumes a distinct slot index, monotonically shrinking effective capacity (see Provenance : BUG-402).

If the admission call site reverts to trying only the count-derived index, with no fallback scan of the other indices in `0..max`:
- A waiter whose single computed index collides with ANY unavailable slot — even one held by a perfectly healthy, actively-progressing session — starves for as long as the live process count stays at a value mapping to that same index, regardless of how many other indices sit completely free or dead-and-reclaimable the entire time. Because each live process holds at most one slot, whenever `count < max` at least `max - count` indices are guaranteed free-or-dead — a single-index-only design ignores that guarantee entirely (see Provenance : BUG-404).

If `claim_slot_file()` reverts to `create_new()` directly on `path` followed by a separate content write, instead of publishing via write-to-temp-then-`hard_link`:
- A process killed between the two steps leaves `path` existing on disk permanently, with no (or truncated) content. `read_slot_owner_record()` then returns `None` for every future reader, and every call site in `acquire_slot()` classifies that `None` as an unconditional, unrecheckable denial (`HeldByLive` at the primary slot, `LostReclaimRace` at a reclaim ticket) — with no owner PID ever parsed out, so `pid_alive()` is never even reached and no liveness recheck or reclaim path can ever engage. This differs from every other failure mode above in requiring no dead owner and no contending reclaimer as a precondition — it is reachable from the single most common path through `acquire_slot()`, a fresh uncontested claim (see Provenance : BUG-407).

### Sources

| File | Notes |
|------|-------|
| `../../src/cli/gate.rs` | `slot_path()`, `claim_slot_file()`, `read_slot_owner_record()`, `pid_alive()`, `acquire_slot()`, and the admission call site in `wait_for_session_slot()` |

### Tests

| File | Notes |
|------|-------|
| `../../tests/concurrency_gate_test.rs` | T08: launches 8 real `clr` print-mode invocations concurrently sharing one `CLR_GATE_DIR`, with `--max-sessions 3`; a background thread mirrors spawned children into the shared proc dir so the live count actually varies during the burst; asserts peak concurrently-held slots never exceeds the configured limit |
| `../../tests/concurrency_gate_test.rs` | T14: pre-seeds a slot owned by a PID confirmed dead, then races 8 concurrent `clr` invocations against that single dead-owner slot; asserts peak concurrently-admitted children never exceeds 1 (BUG-392) |
| `../../tests/concurrency_gate_test.rs` | T17: pre-seeds a slot owned by one confirmed-dead PID and an orphaned reclaim ticket claimed by a second, independently confirmed-dead PID; asserts a fresh caller still acquires the slot promptly instead of being permanently blocked (BUG-402) |
| `../../tests/concurrency_gate_test.rs` | T18: `--max-sessions 2` with the count-derived index (1) pre-seeded as genuinely `HeldByLive` and the other index (0) left completely free; asserts prompt admission via the fallback scan rather than gate-wait exhaustion (BUG-404) |
| `../../tests/concurrency_gate_test.rs` | T19: widens `claim_slot_file()`'s internal claim-to-publish window via `CLR_GATE_CLAIM_TEST_DELAY_MS` and polls the target slot path throughout; asserts it is never observed existing-but-unparseable, only fully absent or fully valid (BUG-407) |
| `../../tests/concurrency_gate_test.rs` | T21: pre-seeds a dead-owner slot with no pre-existing ticket, forces the real `clr` binary's own ticket-win attempt to fail its tmp-claim exactly once (`CLR_GATE_FORCE_TMP_CLAIM_FAIL_ONCE`); asserts the same invocation still acquires the slot on a later retry instead of self-denying forever (BUG-405) |
| `../../tests/concurrency_gate_test.rs` | T22: extends T17 to a three-generation orphaned-ticket chain (two dead claimants stacked before an unclaimed generation); asserts `acquire_slot()` walks past both to acquire the slot fresh, confirming the chain-walk capability generalizes beyond a single hop (BUG-402) |

### Provenance

| Source | Notes |
|--------|-------|
| BUG-387 | Root bug: `find_claude_processes().count() < max` was a pure check-then-act read with no write-side reservation — concurrent invocations could jointly exceed `max`. Fix: index-derived atomic slot reservation documented above. |
| BUG-392 | Residual bug: the reclaim branch added by BUG-387's own fix was itself non-atomic for the crash-recovery case (`read_slot_owner()` → `remove_file()` → `claim_slot_file()`, three independently-fallible steps). Fixed: ticket-arbitrated atomic handoff — see Enforcement Mechanism : 3 above. |
| BUG-400 | Open (filed, not yet fixed): the reclaim-eligibility test (`pid_alive(owner)` alone) has no staleness/elapsed-time supplementary condition, so a live-but-stalled owner (hung, deadlocked, suspended) blocks reclaim indefinitely — the Invariant Statement's two-state owner model (alive/dead) has no third alive-but-stalled category. |
| BUG-402 | Fixed: `acquire_slot()` now walks the reclaim-ticket chain, rechecking each next claimant's liveness before advancing (see Enforcement Mechanism : 3 above), so a reclaiming caller's own death no longer permanently blocks the slot index. Distinct from BUG-400: requires the *original* owner to be dead (BUG-400 requires it alive). The regression test (T17) passes reliably, confirmed across repeated full-suite runs. |
| BUG-404 | Fixed: the admission call site tried only the single count-derived index per attempt, with no fallback to any other index in `0..max` — a waiter starved whenever that one index collided with an unavailable slot, even while other indices sat completely free or dead-and-reclaimable (confirmed empirically in production: 4 of 6 real slots dead-and-untried during a live user report). Fix: try the count-derived index first, then scan every other index before giving up on the attempt — see Enforcement Mechanism : 4 above. Architecturally prior to and broader than BUG-400 (which addresses only the staleness of the one targeted index); fixing this also resolves BUG-400's reported scenario in the common case, though BUG-400 remains independently valid when the live count itself plateaus at `max`. |
| BUG-405 | Fixed: `acquire_slot()`'s ticket-win branch now removes the ticket it just won on both non-admission paths (tmp-claim failure, rename failure) before returning `LostReclaimRace`, so a caller cannot read back its own abandoned claim on a later retry and self-deny — see Enforcement Mechanism : 3, "Self-collision cleanup" above. Distinct from BUG-402: that finding requires a DIFFERENT process to have died mid-handoff; this finding is pure self-collision within one invocation's own retry loop, discovered via adversarial MAAV review of the BUG-402 fix rather than a live symptom report. Regression test T21 was empirically confirmed to fail on pre-fix code (exact predicted symptom) and pass post-fix; T22 confirms the surrounding chain-walk capability is unaffected. |
| BUG-407 | Fixed: `claim_slot_file()`'s `create_new()` and its content `write!()` were two independent, non-atomic steps — a process killed between them left its claimed path permanently existing but empty, which `acquire_slot()` classified as an unconditional, unrecheckable denial with no dead-owner or contending-reclaimer precondition required. Fix: write the full content to a uniquely-named temp file first, then publish atomically via `hard_link()` — see Enforcement Mechanism : 2 above. One level beneath, and orthogonal to, BUG-392/BUG-402 (which hardened `acquire_slot()`'s own reclaim-branch sequencing on top of `claim_slot_file()`, not `claim_slot_file()`'s own internal atomicity). |

### Features

| File | Relationship |
|------|--------------|
| `../feature/001_runner_tool.md` | Defines the `run`/`ask` execution paths whose concurrency this invariant gates |
