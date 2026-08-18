//! Atomic slot reservation for the session gate: claim, read, and dead-owner reclaim.
//!
//! Split out of `gate.rs` (which was over the line-count guideline) — this is the on-disk
//! CAS protocol (`slot_{index}.json` plus `reclaim_*.lock` tickets) that decides admission,
//! kept separate from the poll loop in `gate.rs` that drives it. Consumes
//! `gate_liveness::pid_alive` for reclaim eligibility.

use claude_runner_core::ps_table::parse_json_u64;
use std::path::{ Path, PathBuf };
use super::gate_liveness::pid_alive;

// Return current Unix timestamp in seconds — the `since` stamp every slot,
// ticket, and waiter record below is keyed on, which is why it lives here
// rather than beside the poll loop in gate.rs that also writes one.
pub( super ) fn unix_now() -> u64
{
  std::time::SystemTime::now()
    .duration_since( std::time::UNIX_EPOCH )
    .map_or( 0, |d| d.as_secs() )
}

// Fix(BUG-400): staleness threshold for reclaiming a live-but-stalled slot owner.
//
// Root cause: acquire_slot()'s reclaim-eligibility test was the single binary
// condition `pid_alive(owner)`, with no elapsed-time comparison against the
// recorded `owner_since` anywhere — a live-but-stalled (hung/deadlocked/
// SIGSTOPped) slot holder blocked a waiter indefinitely even when aggregate
// capacity existed elsewhere, because the waiter's candidate index is
// deterministically re-derived from the same live count every poll, making
// the collision sticky rather than a one-off.
//
// Pitfall: unlike `poll_secs`/`max_attempts` (resolved upstream in
// apply_env_vars()/apply_config_defaults(), see src/cli/env.rs), the
// `stale_secs` parameter threaded through here must resolve to `None`
// (feature off) whenever unset or invalid, never a numeric fallback — there
// is no safe default staleness threshold that would not risk reclaiming a
// legitimately long-running session.

// Fix(BUG-387): fixed-index reservation slot backing the concurrency gate.
//
// Root cause: the prior admission check (`find_claude_processes().count() < max`)
// was a pure check-then-act read with no write-side reservation — concurrent
// `clr` invocations could each observe the same stale sub-limit count before any
// of their spawned children became /proc-visible, jointly exceeding `max`.
//
// Fix: the slot index passed to this function is the SAME count just read by
// the caller, so concurrent invocations racing on the same stale count all
// target the identical path — `create_new`'s atomicity then genuinely
// arbitrates between them (exactly one wins, for any number of racers),
// rather than being applied to a per-caller-unique path (e.g. PID-keyed)
// where it would arbitrate nothing. A PID-keyed variant, gated by a preceding
// non-atomic count check, was independently confirmed still racy for exactly
// that reason before this index-derived design was adopted.
//
// Deriving the index from `find_claude_processes()`'s count — rather than a
// private `clr`-only counter — is what preserves system-wide accounting:
// `--max-sessions` counts every `claude` print-mode process on the host,
// launched via `clr` or not (`docs/cli/param/033_max_sessions.md`), so the
// gate must keep reading that shared signal rather than substitute a
// `clr`-only view that would go blind to non-`clr`-launched sessions.
//
// Pitfall: the slot file's lifetime must span this process's ENTIRE session,
// not just the wait — releasing it as soon as `wait_for_session_slot()`
// returns (e.g. via a Drop guard, mirroring `GateFile`) would free the slot
// before the child even spawns, reopening the exact race this closes. There
// is deliberately no Drop guard for it; the file is reclaimed only when a
// future contender for that same index finds the owning PID no longer alive
// (mirroring the liveness self-heal `build_queued_table()` already applies
// to `GateFile` orphans in ps.rs).
fn slot_path( dir : &Path, index : u32 ) -> PathBuf
{
  dir.join( format!( "slot_{index}.json" ) )
}

// Fix(BUG-407): publish the claim and its content as one atomic unit.
//
// Root cause: the prior implementation's create_new(true).open(path) and its
// subsequent write!() were two independently-fallible, non-atomic steps — the
// create succeeded and became observable to any concurrent reader before the
// content write had even been attempted. A process terminated between them
// (SIGKILL, OOM, host crash, container preemption) left `path` existing on
// disk, permanently, with no (or truncated) content — read_slot_owner_record()
// returns None for it forever, and acquire_slot()'s None arm denies HeldByLive
// unconditionally, with no owner PID to check liveness of and no reclaim path
// ever engaging.
//
// Fix: write the full content to a uniquely-named temporary file first (same
// directory, so the eventual link is same-filesystem), then publish it
// atomically via hard_link() instead of create_new() directly on the target
// path. hard_link() fails with AlreadyExists exactly like create_new() did —
// preserving the existing exactly-one-winner arbitration semantics all call
// sites depend on — but by the instant the link succeeds, path's content (the
// linked inode) is already complete, because it was fully written to tmp
// before the link was attempted. There is no window where path exists with
// incomplete content, because path does not exist at all until the content
// behind it is already whole. The pid/since suffix keeps concurrent racers'
// own temp files from colliding with each other; each racer cleans up only
// its own uniquely-named temp file regardless of whether it won.
//
// Pitfall: the temp filename is derived purely from this call's own
// `(pid, since)`, never from `path`'s own filename — deriving it from
// `path` (e.g. appending a suffix to `slot_0.json`) would produce a
// `slot_`-prefixed temp filename, miscounted by `count_live_held_slots()`
// (see the `Fix(BUG-392)` Pitfall above) for the brief window it exists.
//
// Pitfall: this is a pure internal change — the external bool contract (true =
// this call created it now, false = it already existed) is unchanged, so none
// of this function's call sites, acquire_slot()'s branch logic, or
// SlotDenialCause need to change. A path that ALREADY exists before this call
// (e.g. a leftover artifact from a crash under a pre-upgrade binary) is NOT
// repaired by this fix — hard_link(), like create_new(), cannot claim a path
// that already exists, so that scenario still denies via acquire_slot()'s
// unconditional None -> HeldByLive branch. This is an explicitly accepted
// residual (see T24 in concurrency_gate_test.rs), not a regression: this fix
// closes the window where claim_slot_file() itself could CREATE a new
// incomplete file; it adds no repair path for content that was already
// unparseable before claim_slot_file() was ever called.
fn claim_slot_file( path : &Path, pid : u32, since : u64, starttime : Option< u64 > ) -> bool
{
  // Fix(BUG-488): record the writer's own start time when available, binding
  // the claim to this process incarnation — see pid_alive() in gate_liveness.rs
  // for the full fix comment. None (start time unreadable) writes the legacy shape.
  let starttime_field = starttime.map_or_else( String::new, | st | format!( r#","starttime":{st}"# ) );
  let content = format!( r#"{{"pid":{pid},"since":{since}{starttime_field}}}"# );
  let dir     = path.parent().unwrap_or_else( || Path::new( "." ) );
  let tmp     = dir.join( format!( "claim_tmp_{pid}_{since}" ) );
  if std::fs::write( &tmp, &content ).is_err()
  {
    return false;
  }
  claim_test_delay();
  let claimed = std::fs::hard_link( &tmp, path ).is_ok();
  let _ = std::fs::remove_file( &tmp );
  claimed
}

// Test-only injection point, same idiom as `reclaim_test_delay()` below:
// widen the window between `claim_slot_file()`'s temp file being fully
// written and its publish via `hard_link()`, so a regression test can
// deterministically observe that `path` never becomes visible before its
// content is complete, rather than relying on incidental OS scheduling
// jitter. Unset (production default): zero delay.
fn claim_test_delay()
{
  if let Some( ms ) = std::env::var( "CLR_GATE_CLAIM_TEST_DELAY_MS" ).ok().and_then( |s| s.parse::< u64 >().ok() )
  {
    std::thread::sleep( core::time::Duration::from_millis( ms ) );
  }
}

// Return the (pid, since, starttime) recorded in a slot file, if the file is
// readable and well-formed. Fix(BUG-392) needs `since` in addition to `pid`
// to key the reclaim ticket path deterministically — see acquire_slot()
// below. Fix(BUG-488): `starttime` is optional — legacy records written by a
// pre-fix binary lack the field and must stay parseable (None), never be
// treated as corrupt or mismatched.
fn read_slot_owner_record( path : &Path ) -> Option< ( u32, u64, Option< u64 > ) >
{
  let content   = std::fs::read_to_string( path ).ok()?;
  let pid       = u32::try_from( parse_json_u64( &content, "pid" )? ).ok()?;
  let since     = parse_json_u64( &content, "since" )?;
  let starttime = parse_json_u64( &content, "starttime" );
  Some( ( pid, since, starttime ) )
}

// Test-only injection point, same idiom as `gate_dir()`'s `$CLR_GATE_DIR`
// override in gate.rs: widen the reclaim race window on demand so a regression
// test can force many concurrent racers to all observe the same dead-owner
// record before any of them acts on it, rather than relying on incidental
// OS scheduling jitter. Unset (production default): zero delay.
fn reclaim_test_delay()
{
  if let Some( ms ) = std::env::var( "CLR_GATE_RECLAIM_TEST_DELAY_MS" ).ok().and_then( |s| s.parse::< u64 >().ok() )
  {
    std::thread::sleep( core::time::Duration::from_millis( ms ) );
  }
}

// Test-only injection point, same idiom as reclaim_test_delay() above: force
// the ticket-win branch's tmp-claim to fail exactly once, deterministically
// simulating a transient fs fault (disk full, permission race, etc.) on a
// caller that has just won a reclaim ticket — used by the BUG-405 regression
// test to prove the self-collision fix (see acquire_slot() below) survives a
// real forced failure. One-shot: the SAME process's later retries (same
// pid/since, per wait_for_session_slot()'s fixed binding across polling
// attempts) see the fault has cleared, matching a real transient fault's
// lifecycle. Unset (production default): never forces failure.
static FORCE_TMP_CLAIM_FAIL_ARMED : core::sync::atomic::AtomicBool = core::sync::atomic::AtomicBool::new( true );
fn force_tmp_claim_fail_once() -> bool
{
  if std::env::var( "CLR_GATE_FORCE_TMP_CLAIM_FAIL_ONCE" ).is_err()
  {
    return false;
  }
  FORCE_TMP_CLAIM_FAIL_ARMED.swap( false, core::sync::atomic::Ordering::SeqCst )
}

// Fix(BUG-392): atomic ticket-arbitrated handoff for the dead-owner reclaim branch.
//
// Root cause: the prior reclaim sequence — read_slot_owner() -> remove_file() ->
// claim_slot_file() — was three sequential, independently-fallible operations
// with no synchronization across them. remove_file() unconditionally unlinks
// whatever currently occupies the path; it cannot tell "is this still the same
// dead-owner file I read a moment ago". Two callers observing the identical
// dead owner could both run remove-then-recreate, with the second caller's
// remove_file() deleting the first caller's freshly-reclaimed file — both
// acquire_slot() calls then returned `true` for the same index.
//
// Fix: every racer that observes the same dead-owner record — keyed by
// (index, owner pid, owner since), identical for all racers reading the same
// file — computes the identical ticket path and calls claim_slot_file() on
// it. That reuses the SAME create_new/O_CREAT|O_EXCL primitive that already
// arbitrates the fresh-claim path above, so exactly one racer wins the
// ticket. Only the ticket winner writes a per-caller-unique temp file and
// atomically rename()s it onto the shared slot path — POSIX rename(2) is an
// atomic replace, so the destination is never observably absent (unlike
// remove_file() + claim_slot_file(), which has a window where the path
// doesn't exist at all). Losers return `false` and fall through to the
// existing wait-and-retry tail in wait_for_session_slot().
//
// Pitfall: the ticket file is deliberately never cleaned up. If it were
// removed after a successful rename, a later caller — observing a dead-owner
// record for some other, later generation that happened to hash to the same
// key — could win a "new" ticket and clobber the legitimate current holder
// via its own rename(). The (index, pid, since) key is only reused if the OS
// recycles the exact same PID at the exact same `since` timestamp for the
// exact same slot index — effectively never — so leaving the ticket in place
// permanently costs one small file and closes that reopening path entirely.
// Ticket and temp filenames deliberately start with `reclaim_`, never
// `slot_`, and avoid the `.json` extension: ps.rs's build_queued_table()
// only scans `.json` files, and this crate's own T08 regression test
// (`count_live_held_slots()` in concurrency_gate_test.rs) separately treats
// ANY file whose stem starts with `slot_` as a held session slot regardless
// of extension — so a `slot_`-prefixed ticket or temp file would be
// miscounted as an extra concurrently-admitted session for the brief window
// it exists, even though it represents no admission at all.
// Fix(BUG-396): distinguish "this index is currently held by a live session"
// from "this index's owner is dead, but I lost the atomic reclaim-ticket race
// to another concurrent reclaimer" — acquire_slot() previously collapsed both
// outcomes into a bare `false`, so wait_for_session_slot() could not tell them
// apart either. See SlotDenialCause below, and its call site in gate.rs's
// wait_for_session_slot().
// Root cause: the two non-admission returns below (owner alive; ticket/rename
// lost) are mechanistically different — the first never contends with
// anything (the slot is legitimately in active use by another session, for
// however long that session runs), the second genuinely races another
// caller over a dead slot's reclaim ticket — but both discarded that
// distinction the moment they returned a bare bool.
// Pitfall: "I lost a race" and "someone else already legitimately holds this"
// are not the same fact, even though both currently collapse to a
// `false`-shaped non-admission — collapsing them erases information the
// caller needs to build an accurate diagnostic (see wait_for_session_slot()).
pub( super ) enum SlotDenialCause
{
  /// The recorded owner of this index is alive — no reservation was
  /// contested; the slot is simply in active use for however long that
  /// session runs.
  HeldByLive,
  /// The recorded owner was dead, but another concurrent caller won the
  /// atomic reclaim-ticket race for this same index first.
  LostReclaimRace,
}

// Fix(BUG-402): walk the reclaim-ticket chain instead of treating a single
// existing ticket as permanent defeat.
//
// Root cause: the original single-shot check — claim_slot_file( &ticket )
// fails, therefore LostReclaimRace, unconditionally — conflated "another
// caller is actively reclaiming this slot right now" with "a previous
// reclaimer won this exact ticket and then died before completing its
// rename". Per the BUG-392 Pitfall above, the ticket file is permanent by
// design, so once any reclaimer crashes between winning its ticket and
// renaming, every later caller observes an occupied ticket forever —
// acquire_slot() returned LostReclaimRace on every call, and nothing on
// disk would ever change to make a later call succeed.
// wait_for_session_slot()'s poll-and-retry tail cannot recover from a
// deterministically-repeating denial; it only helps when the blocking
// condition can plausibly clear between polls.
//
// Fix: when a ticket's own recorded claimant is also dead, that claimant
// never reached rename() — the slot record is unchanged and the slot is
// genuinely still up for grabs. Advance to the NEXT ticket, keyed by the
// dead claimant's own (pid, since), and retry the identical atomic
// create_new() arbitration. Every concurrent caller that observes the same
// dead claimant computes the identical next-generation ticket path, so
// exactly one of them wins it — the single-winner invariant BUG-392
// established for the first generation holds at every generation the loop
// walks. The loop can only advance as many times as there are pre-existing
// dead-claimant ticket files already on disk for this index — a finite,
// already-written chain — so it always terminates.
//
// Pitfall: re-check the slot's CURRENT owner (not just the ticket
// claimant's liveness) before advancing generations. A concurrent caller
// can win an earlier generation and complete its rename while this call is
// still walking the chain; once that happens the slot legitimately belongs
// to someone new and this call must report HeldByLive, not keep chasing a
// chain that no longer leads anywhere.
//
// Fix(BUG-405): a caller that wins its own reclaim ticket but then fails to
// complete admission must not permanently self-deny on retry.
//
// Root cause: the ticket-win branch below returned LostReclaimRace on tmp-
// claim or rename failure without removing the ticket it had just won.
// Since pid/since are fixed for this caller's entire wait_for_session_slot()
// call (never reset across polling attempts), the next retry recomputes the
// identical ticket path, finds it already claimed by ITSELF, reads back its
// own (pid, since) as next_claimant, and pid_alive() reports true — a
// caller can never lose a fair race to its own still-running self, so every
// subsequent retry repeats the identical false denial forever, for that
// specific slot index.
//
// Fix: remove the ticket on both non-admission paths (tmp-claim failure,
// rename failure) before returning LostReclaimRace, so the next retry
// re-contends this same generation fresh instead of reading back its own
// abandoned claim.
//
// Pitfall: distinct from the Fix(BUG-392) Pitfall above — that pitfall
// concerns cleanup after a SUCCESSFUL rename (removing it there could let a
// later, unrelated generation collide with a recycled PID/timestamp key);
// this cleanup only fires when the winner is confirmed to have never
// completed admission, so no legitimate holder's ticket is ever disturbed
// and the permanent-retention guarantee for SUCCESSFUL claims is unchanged.
pub( super ) fn acquire_slot( dir : &Path, index : u32, pid : u32, since : u64, starttime : Option< u64 >, stale_secs : Option< u64 > ) -> Result< (), SlotDenialCause >
{
  let path = slot_path( dir, index );
  if claim_slot_file( &path, pid, since, starttime )
  {
    return Ok( () );
  }
  let Some( ( owner, owner_since, owner_starttime ) ) = read_slot_owner_record( &path )
  else
  {
    // Fix(BUG-396): an unreadable record here is classified HeldByLive, not
    // LostReclaimRace — empirically confirmed via T15: classifying this as
    // LostReclaimRace produced intermittent "lost reservation race" output
    // for a scenario with no dead owner and no reclaim attempt at all.
    // Fix(BUG-407): claim_slot_file() now publishes via write-to-temp-then-
    // hard_link(), so a path can no longer exist with content mid-write — it
    // becomes visible only once its content is already complete. An
    // unreadable record here therefore means the path already existed with
    // unparseable content BEFORE this call (e.g. a leftover artifact from a
    // crash under a pre-upgrade binary, or an out-of-band write) — genuine
    // on-disk corruption unrelated to the create-then-populate race this fix
    // closes. Recovering from that pre-existing corruption is out of scope
    // for this fix (see T24 in concurrency_gate_test.rs); it is a documented
    // residual, not a live in-progress writer.
    return Err( SlotDenialCause::HeldByLive );
  };
  // Fix(BUG-400): a live owner is reclaim-eligible once also stale — see
  // stale_secs doc above. Unset (default): is_stale is always false,
  // preserving pre-fix behavior exactly (pid_alive(owner) alone gates).
  let is_stale = stale_secs
    .is_some_and( | threshold | unix_now().saturating_sub( owner_since ) > threshold );
  if pid_alive( owner, owner_starttime ) && !is_stale
  {
    return Err( SlotDenialCause::HeldByLive );
  }
  reclaim_test_delay();
  let mut ticket_owner = owner;
  let mut ticket_since = owner_since;
  loop
  {
    let ticket = dir.join( format!( "reclaim_{index}_{ticket_owner}_{ticket_since}.lock" ) );
    if claim_slot_file( &ticket, pid, since, starttime )
    {
      let tmp = dir.join( format!( "reclaim_tmp_{index}_{pid}" ) );
      if force_tmp_claim_fail_once() || !claim_slot_file( &tmp, pid, since, starttime )
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
    let Some( ( next_claimant, next_claimant_since, next_claimant_starttime ) ) = read_slot_owner_record( &ticket )
    else
    {
      return Err( SlotDenialCause::LostReclaimRace );
    };
    if pid_alive( next_claimant, next_claimant_starttime )
    {
      return Err( SlotDenialCause::LostReclaimRace );
    }
    let Some( ( current_owner, _, _ ) ) = read_slot_owner_record( &path )
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
