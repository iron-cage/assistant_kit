# Invariant: Gate Slot Atomicity

### Scope

- **Purpose**: Guarantee that the `--max-sessions` concurrency gate never admits more concurrent print-mode sessions than the configured limit, even when multiple `clr` invocations race the admission check simultaneously — on both the fresh-claim path and the dead-owner reclaim path.
- **Responsibility**: State the atomic reservation mechanism that closes the check-then-act race, which function owns it, and why the reservation must outlive the wait loop itself.
- **In Scope**: `slot_path()`, `claim_slot_file()`, `acquire_slot()`, the admission condition in `wait_for_session_slot()`, the index-derivation rule that makes atomicity meaningful across racers, the fallback scan across every other index when the count-derived candidate is unavailable (see Provenance : BUG-404), the ticket-arbitrated reclaim of dead-owner slots — including walking a chain of orphaned reclaim tickets left behind by a claimant that died before completing its own handoff, so the reclaim path self-heals from an interrupted reclaimer just as it self-heals from an interrupted original owner (see Provenance : BUG-392, BUG-402), and cleaning up a ticket this same caller just won but failed to complete, so a caller can never lose a fair race to its own still-running self (see Provenance : BUG-405) — the opt-in staleness threshold (`gate_stale_secs()`, `CLR_GATE_STALE_SECS`) that makes a live-but-stalled owner reclaim-eligible after a configured elapsed time (see Provenance : BUG-400) — and the claim-vs-content atomicity of `claim_slot_file()` itself: a path must never become observable to a concurrent reader before its full content is already durably present (see Provenance : BUG-407).
- **Out of Scope**: Gate poll interval / attempt-limit configuration (→ `cli/003_env_param.md` Env Param 5), gate-state JSON escaping (→ BUG-384, `json_escape_str()`), `clr ps` queued-table display of gate files (→ `ps.rs` `build_queued_table()` directly), gate-timeout retry behavior (→ `006_exit_codes.md`).

### Invariant Statement

When `wait_for_session_slot()` evaluates admission for a candidate slot index, it MUST require both: (a) the live print-mode process count is below `--max-sessions`, AND (b) this process wins an atomic, kernel-arbitrated reservation at that same index. Neither condition alone is sufficient — (a) alone is the pre-BUG-387 check-then-act race; (b) alone without deriving the index from the just-read count would arbitrate between racers targeting unrelated paths, arbitrating nothing.

| Condition | Outcome |
|-----------|---------|
| Live count `< max` AND slot reservation at the count-derived index succeeds | Admitted — proceed to spawn the session |
| Live count `>= max` | Not admitted — no reservation attempted; falls to wait-and-retry |
| Live count `< max` BUT slot reservation at the count-derived index fails (another racer already holds it) | Falls back to scanning every other index in `0..max`; admitted on the first one that succeeds (see Provenance : BUG-404) — not admitted only if every index in `0..max` is unavailable, in which case it falls to wait-and-retry exactly as the `>= max` case does |
| A losing reservation attempt (at any index tried, count-derived or fallback) finds the existing slot's owning PID no longer alive | Slot is reclaimed via a ticket-arbitrated atomic handoff in the same call — race-free against other simultaneous reclaimers, and self-healing **even when a reclaiming caller is itself interrupted**, by walking the resulting chain of orphaned tickets (see Provenance : BUG-392, BUG-402) |
| A losing reservation attempt finds the existing slot's owning PID still alive, `CLR_GATE_STALE_SECS` is set, AND the owner's recorded `since` is older than that threshold | Owner is treated as reclaim-eligible — the SAME ticket-arbitrated atomic handoff used for a dead owner runs for this index (see Provenance : BUG-400) |
| A losing reservation attempt finds the existing slot's owning PID still alive, and either `CLR_GATE_STALE_SECS` is unset or the owner's recorded `since` has not yet exceeded it | Not admitted — reported as `HeldByLive`; unchanged from the pre-BUG-400 default |

<!-- BUG-400 — fixed: an opt-in CLR_GATE_STALE_SECS threshold (unset by default, so behavior is unchanged unless an operator sets it) now lets a live-but-stalled owner's slot become reclaim-eligible via the same ticket-arbitrated handoff a dead owner's slot already used, once the owner's recorded `since` exceeds the threshold — see Enforcement Mechanism : 3, "Staleness threshold", and Provenance : BUG-400 -->
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
  let tmp = path.with_file_name( format!(
    "{}.tmp.{pid}.{since}",
    path.file_name().map_or_else( Default::default, | n | n.to_string_lossy().into_owned() )
  ) );
  if std::fs::write( &tmp, &content ).is_err()
  {
    return false;
  }
  let claimed = std::fs::hard_link( &tmp, path ).is_ok();
  let _ = std::fs::remove_file( &tmp );
  claimed
}
```
The claim and its content are published as one atomic unit (Fix(BUG-407)): the full content is written to a uniquely-named temp file first, then `hard_link()`s it onto `path`. `hard_link()` maps to the same `O_EXCL`-equivalent semantics `create_new(true)` (`O_CREAT | O_EXCL`) already relied on — it fails with `AlreadyExists` if `path` already exists, preserving the identical exactly-one-winner arbitration every call site depends on. Unlike the prior `create_new` + separate `write!()` shape, `path` never becomes observable to a concurrent reader before its content is already complete, because `path` does not exist at all until the content behind it is already whole — see "Claim-vs-content atomicity" below.

**Claim-vs-content atomicity (Fix(BUG-407)):** `create_new`'s atomicity alone only guarantees exactly-one-winner over the *path* — it says nothing about whether the *content* behind that path is complete at the moment a concurrent reader can already see the path exists. The prior two-step shape (`create_new(true).open(path)` succeeding, then a separate `write!()`) left a window where a process terminated between the two steps (`SIGKILL`, OOM, host crash, container preemption) leaves `path` existing on disk, permanently, with no (or truncated) content: `read_slot_owner_record()` returns `None` for it forever, and `acquire_slot()`'s `None` arm (Fix(BUG-396) below) denies `HeldByLive` unconditionally, with no owner PID to check liveness of and no reclaim path ever engaging — this was possible at ANY of `claim_slot_file()`'s call sites, including the single most common one (the primary fresh-claim attempt, no reclaim precondition needed at all). The write-to-temp-then-`hard_link()` rewrite closes that window: `path` is only ever created by `hard_link()`, and only once its content already exists in full at `tmp`. There is no observable intermediate state.

**Explicitly accepted residual:** this fix is a pure internal change to `claim_slot_file()` — none of its call sites, `acquire_slot()`'s branch logic, or `SlotDenialCause` change. A path that ALREADY exists with unparseable content *before* `claim_slot_file()` is ever called against it (e.g. a leftover artifact from a crash under a pre-upgrade binary, or an out-of-band write) is **not** repaired by this fix: `hard_link()`, like `create_new()`, cannot claim a path that already exists, so that scenario still denies via `acquire_slot()`'s unconditional `None` → `HeldByLive` branch, identically pre-fix and post-fix. This is a documented, stable residual (regression-guarded by T24), not a silent gap — recovering from pre-existing corruption would require a DIFFERENT mechanism (e.g. file-mtime-based staleness for the unparseable-content case), which is out of scope for this fix.

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
  let is_stale = gate_stale_secs()
    .is_some_and( | threshold | unix_now().saturating_sub( owner_since ) > threshold );
  if pid_alive( owner ) && !is_stale
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

**Staleness threshold (Fix(BUG-400)):** `pid_alive(owner)` alone cannot distinguish a healthy, actively-progressing session from one that is alive but permanently stuck (hung, deadlocked, suspended) — both read identically as "still alive", so neither could ever be reclaimed. `gate_stale_secs()` reads `CLR_GATE_STALE_SECS`; unlike `gate_poll_secs()`, it has no hardcoded default and returns `None` when the var is unset or unparseable, so the feature is a strict opt-in — a caller that never sets the var observes byte-for-byte the same `HeldByLive` denial as before this fix, for any owner age. When the var IS set, `is_stale` compares `unix_now().saturating_sub( owner_since )` (the slot's recorded elapsed age) against the threshold; once it is exceeded, `pid_alive( owner ) && !is_stale` evaluates to `false` even though the owner is genuinely alive, and control falls through to the exact same ticket-arbitrated reclaim path (`reclaim_test_delay()` onward) a dead owner already uses — no separate reclaim mechanism was added. This means a stale-alive reclaim is subject to the identical single-winner guarantee, chain-walk self-healing (Fix(BUG-402)), and self-collision cleanup (Fix(BUG-405)) that dead-owner reclaims already have, at no additional implementation cost.

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

If `claim_slot_file()` reverts to a separate `create_new()`-then-`write!()` shape instead of write-to-temp-then-`hard_link()`:
- A process terminated between the two steps leaves `path` existing on disk permanently with no (or truncated) content — every future caller targeting that index reads an unparseable record and is denied `HeldByLive` forever, with no owner PID to check liveness of and no reclaim path ever engaging. This can happen at ANY call site, including the single most common one (a fresh, uncontested claim, no reclaim precondition needed) — see Provenance : BUG-407.

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
| `../../tests/concurrency_gate_test.rs` | T19: pre-seeds a dead-owner slot with no pre-existing ticket, forces the real `clr` binary's own ticket-win attempt to fail its tmp-claim exactly once (`CLR_GATE_FORCE_TMP_CLAIM_FAIL_ONCE`); asserts the same invocation still acquires the slot on a later retry instead of self-denying forever (BUG-405) |
| `../../tests/concurrency_gate_test.rs` | T20: extends T17 to a three-generation orphaned-ticket chain (two dead claimants stacked before an unclaimed generation); asserts `acquire_slot()` walks past both to acquire the slot fresh, confirming the chain-walk capability generalizes beyond a single hop (BUG-402) |
| `../../tests/concurrency_gate_test.rs` | T21: pre-seeds a slot owned by a genuinely-alive real child process with a decades-old recorded `since`; sub-case a (`CLR_GATE_STALE_SECS` unset) asserts the pre-existing unconditional-denial behavior is unchanged; sub-case b (`CLR_GATE_STALE_SECS` set below the elapsed age) asserts the same live owner's slot is reclaimed and the caller admitted promptly (BUG-400) |
| `../../tests/concurrency_gate_test.rs` | T22: a single, uncontested fresh claim followed by an immediate read of the on-disk slot file — asserts the content is already fully-valid JSON with this invocation's own pid, proving there is no create-then-populate window to observe by construction (BUG-407) |
| `../../tests/concurrency_gate_test.rs` | T23: 8 racers contending for the SAME never-before-seen slot path (`--max-sessions 1`, empty gate dir); asserts peak concurrently-admitted children never exceeds 1, confirming the write-to-temp-then-`hard_link()` rewrite preserves the exactly-one-claimant guarantee (BUG-407) |
| `../../tests/concurrency_gate_test.rs` | T24: pre-seeds a 0-byte slot file (content that was already corrupted before any call touches it); asserts a fresh caller is STILL denied, identically pre-fix and post-fix — documents the explicitly accepted residual boundary of the atomic-publish fix (BUG-407) |

### Provenance

| Source | Notes |
|--------|-------|
| BUG-387 | Root bug: `find_claude_processes().count() < max` was a pure check-then-act read with no write-side reservation — concurrent invocations could jointly exceed `max`. Fix: index-derived atomic slot reservation documented above. |
| BUG-392 | Residual bug: the reclaim branch added by BUG-387's own fix was itself non-atomic for the crash-recovery case (`read_slot_owner()` → `remove_file()` → `claim_slot_file()`, three independently-fallible steps). Fixed: ticket-arbitrated atomic handoff — see Enforcement Mechanism : 3 above. |
| BUG-400 | Fixed: `gate_stale_secs()` reads an opt-in `CLR_GATE_STALE_SECS` threshold (`None` — no reclaim — unless explicitly set); once set and the owner's recorded `since` exceeds it, a live-but-stalled owner (hung, deadlocked, suspended) becomes reclaim-eligible via the same ticket-arbitrated handoff a dead owner already uses — see Enforcement Mechanism : 3, "Staleness threshold" above. Regression test T21 confirms both the opt-in default (unchanged denial) and the reclaim-when-set behavior. |
| BUG-402 | Fixed: `acquire_slot()` now walks the reclaim-ticket chain, rechecking each next claimant's liveness before advancing (see Enforcement Mechanism : 3 above), so a reclaiming caller's own death no longer permanently blocks the slot index. Distinct from BUG-400: requires the *original* owner to be dead (BUG-400 requires it alive). The regression test (T17) passes reliably, confirmed across repeated full-suite runs. |
| BUG-404 | Fixed: the admission call site tried only the single count-derived index per attempt, with no fallback to any other index in `0..max` — a waiter starved whenever that one index collided with an unavailable slot, even while other indices sat completely free or dead-and-reclaimable (confirmed empirically in production: 4 of 6 real slots dead-and-untried during a live user report). Fix: try the count-derived index first, then scan every other index before giving up on the attempt — see Enforcement Mechanism : 4 above. Architecturally prior to and broader than BUG-400 (which addresses only the staleness of the one targeted index); fixing this also resolves BUG-400's reported scenario in the common case, though BUG-400 remains independently valid when the live count itself plateaus at `max`. |
| BUG-405 | Fixed: `acquire_slot()`'s ticket-win branch now removes the ticket it just won on both non-admission paths (tmp-claim failure, rename failure) before returning `LostReclaimRace`, so a caller cannot read back its own abandoned claim on a later retry and self-deny — see Enforcement Mechanism : 3, "Self-collision cleanup" above. Distinct from BUG-402: that finding requires a DIFFERENT process to have died mid-handoff; this finding is pure self-collision within one invocation's own retry loop, discovered via adversarial MAAV review of the BUG-402 fix rather than a live symptom report. Regression test T19 was empirically confirmed to fail on pre-fix code (exact predicted symptom) and pass post-fix; T20 confirms the surrounding chain-walk capability is unaffected. |
| BUG-407 | Fixed (with a documented residual): `claim_slot_file()` now publishes its claim and content as one atomic unit via write-to-temp-then-`hard_link()` instead of a separate `create_new()`-then-`write!()` — see Enforcement Mechanism : 2, "Claim-vs-content atomicity" above. Closes the window where a crash DURING a live claim attempt (at any of the function's call sites, including the single most common one — a fresh, uncontested claim) leaves a new permanently-corrupted slot/ticket/temp file behind. Does **not** repair content that was already unparseable *before* `claim_slot_file()` was ever called against that path (e.g. a leftover artifact from a crash under a pre-upgrade binary) — `hard_link()`, like `create_new()`, cannot claim a path that already exists, so that narrower scenario still denies `HeldByLive` unconditionally, identically pre-fix and post-fix; this is an explicitly accepted residual (T24), not a silent gap. T22 confirms content is valid immediately after a successful claim; T23 confirms concurrent-racer arbitration is unweakened. Two independent Tier 4 Paired Verification passes (primary + adversarial code trace) confirmed the residual boundary before T24 was written, after the original bug filing's Prevention sketch (which expected full recovery) was found to contradict its own, more precise Fix Location scope. |

### Features

| File | Relationship |
|------|--------------|
| `../feature/001_runner_tool.md` | Defines the `run`/`ask` execution paths whose concurrency this invariant gates |
