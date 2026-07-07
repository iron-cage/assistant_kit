# Invariant: Gate Slot Atomicity

### Scope

- **Purpose**: Guarantee that the `--max-sessions` concurrency gate never admits more concurrent print-mode sessions than the configured limit, even when multiple `clr` invocations race the admission check simultaneously — on both the fresh-claim path and the dead-owner reclaim path.
- **Responsibility**: State the atomic reservation mechanism that closes the check-then-act race, which function owns it, and why the reservation must outlive the wait loop itself.
- **In Scope**: `slot_path()`, `claim_slot_file()`, `acquire_slot()`, the admission condition in `wait_for_session_slot()`, the index-derivation rule that makes atomicity meaningful across racers, and the ticket-arbitrated reclaim of dead-owner slots (atomic against concurrent reclaimers, and self-healing only when the reclaiming caller itself completes without interruption — see Provenance : BUG-392, BUG-402).
- **Out of Scope**: Gate poll interval / attempt-limit configuration (→ `cli/003_env_param.md` Env Param 5), gate-state JSON escaping (→ BUG-384, `json_escape_str()`), `clr ps` queued-table display of gate files (→ `ps.rs` `build_queued_table()` directly), gate-timeout retry behavior (→ `006_exit_codes.md`).

### Invariant Statement

When `wait_for_session_slot()` evaluates admission for a candidate slot index, it MUST require both: (a) the live print-mode process count is below `--max-sessions`, AND (b) this process wins an atomic, kernel-arbitrated reservation at that same index. Neither condition alone is sufficient — (a) alone is the pre-BUG-387 check-then-act race; (b) alone without deriving the index from the just-read count would arbitrate between racers targeting unrelated paths, arbitrating nothing.

| Condition | Outcome |
|-----------|---------|
| Live count `< max` AND slot reservation at that index succeeds | Admitted — proceed to spawn the session |
| Live count `>= max` | Not admitted — no reservation attempted; falls to wait-and-retry |
| Live count `< max` BUT slot reservation at that index fails (another racer already holds it) | Not admitted — falls to wait-and-retry exactly as the `>= max` case does |
| A losing reservation attempt finds the existing slot's owning PID no longer alive | Slot is reclaimed via a ticket-arbitrated atomic handoff in the same call — race-free against other simultaneous reclaimers, and self-healing **only when the reclaiming caller completes the handoff without interruption** (see Provenance : BUG-392, BUG-402) |

<!-- BUG-400 — this table defines only two owner states (alive/dead); no third alive-but-stalled category exists, so a live-but-non-progressing owner blocks reclaim indefinitely with no staleness escape hatch -->
<!-- BUG-402 — "self-healing" above is not unconditional: if the reclaiming caller itself dies between winning the ticket and completing rename(), the ticket is never cleaned up (by design) and no liveness recheck exists on the ticket's own claimant, so that slot index is permanently and unrecoverably blocked, not merely self-healed on the next attempt -->

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

**2. `claim_slot_file()` — atomic kernel-arbitrated claim:**
```rust
fn claim_slot_file( path : &Path, pid : u32, since : u64 ) -> bool
{
  let Ok( mut f ) = std::fs::OpenOptions::new().write( true ).create_new( true ).open( path )
  else
  {
    return false;
  };
  let _ = write!( f, r#"{{"pid":{pid},"since":{since}}}"# );
  true
}
```
`create_new(true)` maps to `O_CREAT | O_EXCL` — the file's creation and existence-check are a single atomic kernel operation. No separate exists-check precedes it; that separation is exactly what the pre-fix race exploited.

**3. `acquire_slot()` — claim, or reclaim a dead owner's slot via ticket-arbitrated atomic handoff:**
```rust
fn acquire_slot( dir : &Path, index : u32, pid : u32, since : u64 ) -> bool
{
  let path = slot_path( dir, index );
  if claim_slot_file( &path, pid, since )
  {
    return true;
  }
  let Some( ( owner, owner_since ) ) = read_slot_owner_record( &path )
  else
  {
    return false;
  };
  if pid_alive( owner )
  {
    return false;
  }
  reclaim_test_delay();
  let ticket = dir.join( format!( "reclaim_{index}_{owner}_{owner_since}.lock" ) );
  if !claim_slot_file( &ticket, pid, since )
  {
    return false;
  }
  let tmp = dir.join( format!( "reclaim_tmp_{index}_{pid}" ) );
  if !claim_slot_file( &tmp, pid, since )
  {
    return false;
  }
  let claimed = std::fs::rename( &tmp, &path ).is_ok();
  if !claimed
  {
    let _ = std::fs::remove_file( &tmp );
  }
  claimed
}
```
The reclaim branch (Fix(BUG-392)) reuses `claim_slot_file()`'s `create_new` atomicity twice more: first to arbitrate a ticket keyed by `(index, owner pid, owner since)` — every racer observing the identical dead-owner record computes the identical ticket path, so exactly one wins it — then to write a per-caller-unique temp file. Only the ticket winner reaches the `rename()`, which POSIX guarantees is an atomic replace: unlike the prior `remove_file()` + `claim_slot_file()` sequence, the destination path is never observably absent in between. The ticket file is deliberately never cleaned up — removing it would let a later caller win a "new" ticket for an already-completed generation and clobber the legitimate holder via its own `rename()`. Ticket and temp filenames use a `reclaim_` prefix (never `slot_`) and no `.json` extension, keeping them invisible to both `ps.rs::build_queued_table()` (filters on `.json`) and this crate's own `count_live_held_slots()` test helper (filters on a `slot_`-prefixed stem, regardless of extension).

**4. Admission call site in `wait_for_session_slot()`:**
```rust
let count = find_claude_processes()
  .iter()
  .filter( | p | super::ps::classify_mode( &p.args ) == "print" )
  .count();
let count_u32 = u32::try_from( count ).unwrap_or( u32::MAX );
if count_u32 < max && acquire_slot( &dir, count_u32, pid, since )
{
  // … journal GateWait event emission elided — irrelevant to the atomicity guarantee …
  return; // _guard.drop() removes only the {pid}.json telemetry file —
          // the slot reservation from acquire_slot() is deliberately
          // left in place for the rest of this session; see slot_path().
}
```

`pid_alive()` checks `/proc/{pid}` existence (Linux-only host assumption, matching the identical convention `build_queued_table()` already uses in `ps.rs`). `read_slot_owner_record()` reads back the `(pid, since)` pair a slot file records — the reclaim branch needs both fields to key its ticket path (see Enforcement Mechanism : 3 above), where the pre-BUG-392 `read_slot_owner()` returned only `pid`.

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

### Sources

| File | Notes |
|------|-------|
| `../../src/cli/gate.rs` | `slot_path()`, `claim_slot_file()`, `read_slot_owner_record()`, `pid_alive()`, `acquire_slot()`, and the admission call site in `wait_for_session_slot()` |

### Tests

| File | Notes |
|------|-------|
| `../../tests/concurrency_gate_test.rs` | T08: launches 8 real `clr` print-mode invocations concurrently sharing one `CLR_GATE_DIR`, with `--max-sessions 3`; a background thread mirrors spawned children into the shared proc dir so the live count actually varies during the burst; asserts peak concurrently-held slots never exceeds the configured limit |
| `../../tests/concurrency_gate_test.rs` | T14: pre-seeds a slot owned by a PID confirmed dead, then races 8 concurrent `clr` invocations against that single dead-owner slot; asserts peak concurrently-admitted children never exceeds 1 (BUG-392) |

### Provenance

| Source | Notes |
|--------|-------|
| BUG-387 | Root bug: `find_claude_processes().count() < max` was a pure check-then-act read with no write-side reservation — concurrent invocations could jointly exceed `max`. Fix: index-derived atomic slot reservation documented above. |
| BUG-392 | Residual bug: the reclaim branch added by BUG-387's own fix was itself non-atomic for the crash-recovery case (`read_slot_owner()` → `remove_file()` → `claim_slot_file()`, three independently-fallible steps). Fixed: ticket-arbitrated atomic handoff — see Enforcement Mechanism : 3 above. |
| BUG-400 | Open (filed, not yet fixed): the reclaim-eligibility test (`pid_alive(owner)` alone) has no staleness/elapsed-time supplementary condition, so a live-but-stalled owner (hung, deadlocked, suspended) blocks reclaim indefinitely — the Invariant Statement's two-state owner model (alive/dead) has no third alive-but-stalled category. |
| BUG-402 | Open (filed, not yet fixed): the reclaim ticket built by BUG-392's own fix has no liveness recheck on its own claimant — if that claimant dies between winning the ticket and completing `rename()`, the ticket is never cleaned up (by design) and the slot index is permanently and unrecoverably blocked, not merely self-healed on the next attempt. Distinct from BUG-400: requires the *original* owner to be dead (BUG-400 requires it alive). |

### Features

| File | Relationship |
|------|--------------|
| `../feature/001_runner_tool.md` | Defines the `run`/`ask` execution paths whose concurrency this invariant gates |
