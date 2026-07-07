# Invariant: Gate Slot Atomicity

### Scope

- **Purpose**: Guarantee that the `--max-sessions` concurrency gate never admits more concurrent print-mode sessions than the configured limit, even when multiple `clr` invocations race the admission check simultaneously.
- **Responsibility**: State the atomic reservation mechanism that closes the check-then-act race, which function owns it, and why the reservation must outlive the wait loop itself.
- **In Scope**: `slot_path()`, `claim_slot_file()`, `acquire_slot()`, the admission condition in `wait_for_session_slot()`, the index-derivation rule that makes atomicity meaningful across racers, and the self-healing reclaim of dead-owner slots.
- **Out of Scope**: Gate poll interval / attempt-limit configuration (→ `cli/003_env_param.md` Env Param 5), gate-state JSON escaping (→ BUG-384, `json_escape_str()`), `clr ps` queued-table display of gate files (→ `010_container_only_test_execution.md` is unrelated; see `ps.rs` `build_queued_table()` directly), gate-timeout retry behavior (→ `006_exit_codes.md`).

### Invariant Statement

When `wait_for_session_slot()` evaluates admission for a candidate slot index, it MUST require both: (a) the live print-mode process count is below `--max-sessions`, AND (b) this process wins an atomic, kernel-arbitrated reservation at that same index. Neither condition alone is sufficient — (a) alone is the pre-BUG-387 check-then-act race; (b) alone without deriving the index from the just-read count would arbitrate between racers targeting unrelated paths, arbitrating nothing.

| Condition | Outcome |
|-----------|---------|
| Live count `< max` AND slot reservation at that index succeeds | Admitted — proceed to spawn the session |
| Live count `>= max` | Not admitted — no reservation attempted; falls to wait-and-retry |
| Live count `< max` BUT slot reservation at that index fails (another racer already holds it) | Not admitted — falls to wait-and-retry exactly as the `>= max` case does |
| A losing reservation attempt finds the existing slot's owning PID no longer alive | Slot is reclaimed (deleted and re-claimed) in the same call — self-healing, no separate cleanup pass |

**Reservation lifetime:** once acquired, a slot is held for the entire remaining lifetime of the session — not just the polling wait. There is deliberately no `Drop` guard releasing it when `wait_for_session_slot()` returns. Releasing early would free the slot before the admitted child process becomes `/proc`-visible, reopening the exact race this invariant closes.

### Rationale — Why a Fixed-Index Reservation, Not a PID-Keyed One

The slot index passed to `acquire_slot()` is the same live-process count the caller just read, not a private per-caller identifier (e.g. PID-keyed). This is load-bearing: concurrent invocations racing on the same stale count all target the **identical** slot path, so `create_new`'s atomicity genuinely arbitrates between them — exactly one wins, regardless of how many are racing. A PID-keyed variant (each racer reserving its own uniquely-named slot, gated by a preceding non-atomic count check) was evaluated and confirmed still racy, for exactly the reason a PID-keyed path never collides with anything and therefore never arbitrates.

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

**3. `acquire_slot()` — claim, or reclaim a dead owner's slot:**
```rust
fn acquire_slot( dir : &Path, index : u32, pid : u32, since : u64 ) -> bool
{
  let path = slot_path( dir, index );
  if claim_slot_file( &path, pid, since )
  {
    return true;
  }
  if read_slot_owner( &path ).is_some_and( |owner| !pid_alive( owner ) )
  {
    let _ = std::fs::remove_file( &path );
    return claim_slot_file( &path, pid, since );
  }
  false
}
```

**4. Admission call site in `wait_for_session_slot()`:**
```rust
let count = find_claude_processes()
  .iter()
  .filter( | p | super::ps::classify_mode( &p.args ) == "print" )
  .count();
let count_u32 = u32::try_from( count ).unwrap_or( u32::MAX );
if count_u32 < max && acquire_slot( &dir, count_u32, pid, since )
{
  return; // reservation deliberately outlives this function — see Invariant Statement
}
```

`pid_alive()` checks `/proc/{pid}` existence (Linux-only host assumption, matching the identical convention `build_queued_table()` already uses in `ps.rs`).

### Violation Consequences

If the admission condition reverts to checking only `count_u32 < max` without `acquire_slot()`:
- Concurrent `clr` invocations can each observe the same stale sub-limit count before any of their spawned children become `/proc`-visible, jointly admitting more sessions than `--max-sessions` allows — the original BUG-387 symptom.
- The violation is timing-dependent and will not reproduce under sequential/manual testing — it requires genuine concurrent launches (see T08 in Tests below).

If the slot index is changed to a per-caller-unique value (e.g. PID-keyed) instead of the shared live-count value:
- `create_new`'s atomicity still succeeds for every racer (each targets a distinct path), so admission silently reverts to unbounded — the same race as above, just with an atomic-looking operation masking it.

If a `Drop` guard is added to release the slot when `wait_for_session_slot()` returns:
- The slot frees before the admitted child is spawned or becomes `/proc`-visible, reopening the race within the gap between gate-passage and child spawn.

### Sources

| File | Notes |
|------|-------|
| `../../src/cli/gate.rs` | `slot_path()`, `claim_slot_file()`, `read_slot_owner()`, `pid_alive()`, `acquire_slot()`, and the admission call site in `wait_for_session_slot()` |

### Tests

| File | Notes |
|------|-------|
| `../../tests/concurrency_gate_test.rs` | T08: launches 8 real `clr` print-mode invocations concurrently sharing one `CLR_GATE_DIR`, with `--max-sessions 3`; a background thread mirrors spawned children into the shared proc dir so the live count actually varies during the burst; asserts peak concurrently-held slots never exceeds the configured limit |

### Provenance

| Source | Notes |
|--------|-------|
| BUG-387 | Root bug: `find_claude_processes().count() < max` was a pure check-then-act read with no write-side reservation — concurrent invocations could jointly exceed `max`. Fix: index-derived atomic slot reservation documented above. |
