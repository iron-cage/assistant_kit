use claude_core::process::find_claude_processes;
use core::fmt::Write as _;
use std::path::{ Path, PathBuf };
use claude_journal::{ EventRecord, EventType, JournalWriter };

// Return the gate state directory — $CLR_GATE_DIR or <sys-temp>/clr-gate.
//
// $CLR_GATE_DIR is the single test-injection point; tests override it to a temp
// dir so IT-10/IT-11 never touch the real default path on the host.
pub( super ) fn gate_dir() -> PathBuf
{
  std::env::var( "CLR_GATE_DIR" )
    .ok()
    .filter( |s| !s.is_empty() )
    .map_or_else( || std::env::temp_dir().join( "clr-gate" ), PathBuf::from )
}

// Return current Unix timestamp in seconds.
pub( super ) fn unix_now() -> u64
{
  std::time::SystemTime::now()
    .duration_since( std::time::UNIX_EPOCH )
    .map_or( 0, |d| d.as_secs() )
}

// Fix(BUG-293): RAII guard for gate file cleanup.
// Root cause: wait_for_session_slot() had no Drop impl — abnormal exit
// (panic, Ctrl+C) left orphaned gate files on disk permanently.
// Pitfall: Drop does NOT run on SIGKILL (bypasses destructors) — the
// /proc/{pid} liveness filter in build_queued_table() handles those
// orphans via self-healing deletion.
struct GateFile( PathBuf );

impl Drop for GateFile
{
  fn drop( &mut self )
  {
    let _ = std::fs::remove_file( &self.0 );
  }
}

/// Resolve the attempt-limit override from a raw env var string. Pure — no I/O —
/// so the parse-or-default fallback can be unit-tested directly. This crate's
/// tests never call `std::env::set_var` (see `tests/env_var_test.rs`); taking the raw
/// value as a parameter instead of reading the environment internally is what makes
/// that possible here, and means `remove_var` is never needed either.
#[ inline ]
#[ must_use ]
pub fn gate_max_attempts_from( raw : Option< &str > ) -> u32
{
  raw.and_then( | s | s.parse().ok() ).unwrap_or( 1000 )
}

/// Resolve the gate poll interval (seconds) from a raw env var string. Pure — no I/O.
/// Default: 30 seconds. Sibling of [`gate_max_attempts_from`] — see its doc for why
/// this takes the raw value as a parameter instead of reading the environment directly.
///
/// This is the one-shot resolver: `isolated` (which has no CLI flag or config-file tier
/// for this knob) calls it directly against the raw env var. `run`/`ask` instead resolve
/// `CliArgs.gate_poll_secs` through the full CLI > `--args-file` JSON > env var > config.toml
/// tier chain in `apply_env_vars()`/`apply_config_defaults()` (src/cli/env.rs, src/cli/config.rs),
/// falling back to this same 30s default only once every tier has been checked.
#[ inline ]
#[ must_use ]
pub fn gate_poll_secs_from( raw : Option< &str > ) -> u64
{
  raw.and_then( | s | s.parse().ok() ).unwrap_or( 30 )
}

/// Resolve the staleness threshold (seconds) from a raw env var string. Pure — no I/O.
/// `None` (unset or invalid) disables staleness-based reclaim entirely — there is no
/// safe numeric default (see the Fix(BUG-400) note below for why). Sibling of
/// [`gate_max_attempts_from`]; same one-shot-vs-tiered split as [`gate_poll_secs_from`]
/// applies — `isolated` calls this directly, `run`/`ask` resolve through the tier chain.
#[ inline ]
#[ must_use ]
pub fn gate_stale_secs_from( raw : Option< &str > ) -> Option< u64 >
{
  raw.and_then( | s | s.parse().ok() )
}

/// Resolve the remaining external timeout budget (seconds) from a raw env var string.
/// Pure — no I/O. `None` (unset or non-numeric) means no external budget is imposed —
/// polling is limited only by `CLR_GATE_MAX_ATTEMPTS`. Unlike the other gate knobs,
/// this value is env-var-only (no CLI flag, no config-file tier): it is set by a
/// wrapping job runner (e.g. `wplan_executor`) before spawning `clr`, not by the
/// operator configuring `clr` itself. See [`wait_for_session_slot`]'s `Fix(BUG-423)`
/// note for the full semantics and clamping behaviour.
#[ inline ]
#[ must_use ]
pub fn gate_remaining_timeout_secs_from( raw : Option< &str > ) -> Option< u64 >
{
  raw.and_then( | s | s.parse().ok() )
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
  // the claim to this process incarnation — see pid_alive() below for the
  // full fix comment. None (start time unreadable) writes the legacy shape.
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
  let pid       = u32::try_from( super::ps::parse_json_u64( &content, "pid" )? ).ok()?;
  let since     = super::ps::parse_json_u64( &content, "since" )?;
  let starttime = super::ps::parse_json_u64( &content, "starttime" );
  Some( ( pid, since, starttime ) )
}

// BUG-479 task/claude_runner/bug/479_zombie_blind_pid_liveness.md — fixed: bare /proc/{pid}
// existence read unreaped zombies as live, blocking acquire_slot()'s
// owner-reclaim and reclaim-ticket claimant checks indefinitely; details below.
// Fix(BUG-479): liveness now reads the /proc/{pid}/stat state field instead of
// probing bare /proc/{pid} existence, and is the single shared predicate for
// both consumers (acquire_slot() here, build_queued_table() in ps.rs).
// Root cause: an exited-but-unreaped child (state `Z`) keeps its /proc entry
// for as long as its parent fails to wait(), so an existence probe read
// zombies as alive — under a non-reaping supervisor every dead slot owner and
// queued waiter became permanent (7/8 slots starved; `Queued · 84 waiting`
// with 4 live).
// Pitfall: /proc/{pid} existence proves a PID exists, not that a process runs
// — any reclaim/display protocol keyed on it deadlocks/inflates the moment
// children stop being reaped. Liveness = stat readable AND state ∉ {Z}.
//
// Fix(BUG-488) task/claude_runner/bug/488_pid_liveness_thread_id_blind.md:
// liveness additionally requires thread-group leadership (`Tgid == pid` from
// /proc/{pid}/status) and — when the caller's record carries the writer's
// start time — a matching /proc/{pid}/stat field 22 on the current occupant.
// Root cause: the two BUG-479 clauses test only that SOMETHING with this
// number is running, never that it is the recorded PROCESS: Linux resolves
// direct /proc/<tid> lookups for readdir-invisible non-leader thread IDs of
// unrelated processes, and a full PID-space wrap recycles a leader number to
// a new process — either occupancy made a dead recorded owner read alive
// forever (observed live: dockerd startup thread TID 1744061 masking a dead
// gate waiter as a phantom `Queued` row for 76+ hours).
// Pitfall: a bare PID number never identifies a process across time — bind
// records to the (pid, starttime) incarnation and verify both on read. The
// starttime clause is additive: legacy records without the field keep the
// (a)-(c) semantics, so a mid-upgrade mixed fleet never mass-reclaims slots
// held by live pre-fix sessions (absence of the field is NOT a mismatch).
//
// Return whether `pid` is a live, running process — and, when
// `recorded_starttime` is Some, the same process incarnation that wrote the
// record (Linux-only host assumption, unchanged). Clauses: (a) stat readable,
// (b) state ∉ {Z}, (c) thread-group leader, (d) start time matches when
// recorded. The state field follows the LAST ')' — comm may contain
// spaces/parens.
pub( super ) fn pid_alive( pid : u32, recorded_starttime : Option< u64 > ) -> bool
{
  let Ok( stat ) = std::fs::read_to_string( format!( "/proc/{pid}/stat" ) )
  else
  {
    return false; // (a) no such PID number at all
  };
  let running = stat.rsplit_once( ')' )
    .and_then( | ( _, rest ) | rest.trim_start().chars().next() )
    .is_some_and( | state | state != 'Z' );
  if !running
  {
    return false; // (b) exited-but-unreaped zombie
  }
  if proc_tgid( pid ) != Some( pid )
  {
    return false; // (c) a non-leader thread merely occupies the number
  }
  match recorded_starttime
  {
    None             => true, // legacy record — incarnation clause inert
    Some( recorded ) => starttime_from_stat( &stat ) == Some( recorded ), // (d)
  }
}

// Return the thread-group ID reported by /proc/{pid}/status. For a process
// (thread-group leader) `Tgid == pid`; for a bare thread TID resolved via
// direct lookup, Tgid names its owning process instead.
fn proc_tgid( pid : u32 ) -> Option< u32 >
{
  std::fs::read_to_string( format!( "/proc/{pid}/status" ) )
    .ok()?
    .lines()
    .find_map( | l | l.strip_prefix( "Tgid:" ).and_then( | v | v.trim().parse().ok() ) )
}

// Return the start-time token from raw /proc/{pid}/stat content: field 22
// (clock ticks since boot), stable for a process's entire life — token index
// 19 after the ')' split, since fields 1-2 (pid, comm) precede the ')'.
// Compared for exact equality only; never unit-converted.
fn starttime_from_stat( stat : &str ) -> Option< u64 >
{
  stat.rsplit_once( ')' )?
    .1
    .split_whitespace()
    .nth( 19 )?
    .parse()
    .ok()
}

// Return this-or-any process's own current start time (field 22) for
// recording into slot/ticket/waiter artifacts at write time.
fn proc_starttime( pid : u32 ) -> Option< u64 >
{
  starttime_from_stat( &std::fs::read_to_string( format!( "/proc/{pid}/stat" ) ).ok()? )
}

// Test-only injection point, same idiom as `gate_dir()`'s `$CLR_GATE_DIR`
// override above: widen the reclaim race window on demand so a regression
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
// apart either. See SlotDenialCause and its call site below.
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
enum SlotDenialCause
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
fn acquire_slot( dir : &Path, index : u32, pid : u32, since : u64, starttime : Option< u64 >, stale_secs : Option< u64 > ) -> Result< (), SlotDenialCause >
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

// Fix(BUG-384): escape a string for embedding as a JSON string value, per RFC 8259 §7.
//
// Root cause: the gate-state writer originally spliced `cwd` (an OS-controlled string)
// into a hand-rolled JSON literal with zero escaping. A first fix pass added
// `.replace('\\', ..).replace('"', ..)`, which closed the two most common cases but left
// raw control characters (bytes < 0x20 — Unix paths may legally contain a literal
// newline, tab, or other control byte) completely unescaped, still producing invalid
// JSON for such a `cwd`. This single-pass escaper closes that gap by handling every
// JSON-reserved character in one place instead of chaining ad hoc `.replace()` calls.
//
// Pitfall: never hand-roll JSON escaping via a growing chain of `.replace()` calls for
// individual characters — it's easy to cover the common cases (`"`, `\`) and forget the
// full control-character class the JSON grammar also requires escaping.
fn json_escape_str( s : &str ) -> String
{
  let mut out = String::with_capacity( s.len() );
  for c in s.chars()
  {
    match c
    {
      '"' => out.push_str( "\\\"" ),
      '\\' => out.push_str( "\\\\" ),
      '\u{08}' => out.push_str( "\\b" ),
      '\u{0C}' => out.push_str( "\\f" ),
      '\n' => out.push_str( "\\n" ),
      '\r' => out.push_str( "\\r" ),
      '\t' => out.push_str( "\\t" ),
      c if ( c as u32 ) < 0x20 => { let _ = write!( out, "\\u{:04x}", c as u32 ); },
      c => out.push( c ),
    }
  }
  out
}

// Fix(BUG-481) task/claude_runner/bug/481_silent_off_env_protection_boundary.md: resolve the
// deadline clamp WITH a resolution-state string, so the caller can announce
// which state the protection landed in (off-unset / off-unparseable /
// nonlimiting / engaged) instead of resolving silently.
// Root cause: env read → parse → strict-< selection carried zero diagnostic on
// every non-engaged path, so misconfiguration ("notanumber"), non-configuration
// (unset), and correct-but-nonlimiting configuration (quotient >= max_attempts)
// were output-indistinguishable — the sole pre-admission deadline mechanism
// could be dead while every surface looked healthy (Job #424: 66 polls
// advertising /1000 inside a 7200s external kill window). The same file
// recovers invalid input to safe defaults for its always-on knobs
// (gate_max_attempts_from/gate_poll_secs_from); only the optional protections
// switched off silently.
// Pitfall: an optional safety protection must announce its resolution state
// (raw input, parse outcome, engaged-or-off) exactly once, on a surface the
// operator reads — a silent off-state reads as health until the incident the
// feature existed to prevent. And: the state text must avoid the "budget"
// substring, which EC-3/EC-4 pin as engaged-path-only.
fn effective_gate_attempts( max_attempts : u32, poll_secs : u64, caller_timeout_secs : u64 ) -> ( u32, bool, String )
{
  // Fix(BUG-423): clamp effective_max_attempts to the remaining external timeout budget
  // so the gate does not outlive a wrapping job-runner deadline (e.g. wplan_executor).
  // Root cause: wait_for_session_slot() had no knowledge of any external timeout budget —
  // it polled to CLR_GATE_MAX_ATTEMPTS regardless of how much wall-clock time the
  // surrounding job was permitted to consume, causing gate-wait alone to exhaust
  // multi-hour job timeouts (observed: 258 attempts × 30s = 7740s > 7200s budget).
  // Pitfall: apply .max(1) so at least one admission attempt is always made before
  // declaring budget exhausted — a remaining budget below one poll interval must not
  // silently bypass the gate check entirely.
  // Fix(BUG-481): .max( 1 ) on the divisor — gate_poll_secs_from accepts "0"
  // (any parseable u64), and this division only runs when the env var parses
  // numeric, so poll_secs=0 reached an integer divide-by-zero here.
  // Root cause: the divisor was range-guarded at neither parse nor use site;
  // the panic needed two independently-valid env values meeting in one
  // expression, invisible to per-knob edge-case tests (T41 pins it).
  // Pitfall: parse acceptance is not arithmetic safety — guard env-derived
  // divisors at the division site. The floor affects the quotient only; the
  // gate's actual sleep cadence is untouched.
  // Fix(BUG-445): when CLR_REMAINING_TIMEOUT_SECS does not parse, an EXPRESSED
  // caller timeout (--timeout flag or CLR_TIMEOUT env; callers pass 0 for
  // unexpressed or an explicit `--timeout 0` opt-out) defaults the budget, so
  // `--timeout N` alone bounds total exposure instead of only the post-gate
  // execution phase. A parseable env var still wins — it is the deliberate
  // per-dispatch coupling signal (BUG-423); the flag is its fallback.
  // Root cause: the gate's sole timing input was the opt-in env var; a caller
  // expressing `--timeout N` as a total budget got zero gate-wait protection
  // (watchdog.sh health probes stalled 9697s/272s/903s against 60s).
  // Pitfall: only EXPRESSED timeouts may default the budget — the built-in
  // defaults (isolated 30s, print-mode 3600s) must never reach this fallback,
  // or every default invocation flips from queue-patiently (~8.3h ceiling) to
  // fail-fast. The state string names the source so env-vs-flag budgets stay
  // distinguishable; an unparseable env value masked by the fallback is still
  // reported (BUG-481's misconfiguration-visibility invariant).
  let raw      = std::env::var( "CLR_REMAINING_TIMEOUT_SECS" ).ok();
  let parsed   = gate_remaining_timeout_secs_from( raw.as_deref() );
  let from_env = parsed.is_some();
  let fallback = if caller_timeout_secs > 0 { Some( caller_timeout_secs ) } else { None };
  let budget_secs = parsed.or( fallback );
  let budget = budget_secs.map( | remaining | {
    u32::try_from( remaining / poll_secs.max( 1 ) ).unwrap_or( u32::MAX ).max( 1 )
  } );
  let effective = match budget {
    Some( b ) if b < max_attempts => b,
    _                             => max_attempts,
  };
  let limiting = effective < max_attempts;
  let state = match ( raw.as_deref(), budget_secs )
  {
    ( None, None )        => "off (CLR_REMAINING_TIMEOUT_SECS unset)".to_string(),
    ( Some( r ), None )   => format!( "off (CLR_REMAINING_TIMEOUT_SECS={r:?} unparseable)" ),
    ( raw_opt, Some( s ) ) =>
    {
      let source = if from_env { "" } else { " from --timeout" };
      let masked = match ( from_env, raw_opt )
      {
        ( false, Some( r ) ) => format!( "; CLR_REMAINING_TIMEOUT_SECS={r:?} unparseable" ),
        _                    => String::new(),
      };
      if limiting
      { format!( "engaged ({s}s{source} clamps to {effective} of {max_attempts} attempts{masked})" ) }
      else
      { format!( "nonlimiting ({s}s{source} covers all {max_attempts} attempts{masked})" ) }
    },
  };
  ( effective, limiting, state )
}

// Fix(BUG-445): --trace-gated stderr note when a caller expressed NO timeout
// (no --timeout flag, no CLR_TIMEOUT) and CLR_REMAINING_TIMEOUT_SECS is
// unusable, so gate-wait (if entered) has no deadline bound at all.
// Root cause: --timeout only bounded the post-gate execution phase; originally
// this warned callers who SET --timeout that gate-wait ignored it. Fix
// Location #2 made an expressed --timeout default the gate budget, so those
// callers are now protected and the old warning would be false for them —
// unbounded exposure remains only for callers who expressed nothing.
// Pitfall: only fires when the gate is actually active (max != 0) and the
// caller expressed no timeout — an explicit `--timeout 0` is a deliberate
// unlimited opt-out (timeout_expressed = true), and warning a caller who
// already declined any bound would be noise, not signal.
pub( super ) fn trace_gate_wait_exposure( max : u32, trace : bool, timeout_expressed : bool )
{
  if max == 0 || !trace || timeout_expressed { return; }
  let raw = std::env::var( "CLR_REMAINING_TIMEOUT_SECS" ).ok();
  if gate_remaining_timeout_secs_from( raw.as_deref() ).is_some() { return; }
  let why = match raw
  {
    None      => "unset".to_string(),
    Some( r ) => format!( "set but unparseable ({r:?})" ),
  };
  eprintln!(
    "Trace: gate-wait is unbounded — no --timeout given and CLR_REMAINING_TIMEOUT_SECS is {why}; \
     if this invocation queues for a --max-sessions slot, total exposure could reach the full \
     gate-wait ceiling (CLR_GATE_POLL_SECS x CLR_GATE_MAX_ATTEMPTS, ~8.3h by default). Pass \
     --timeout N (it also bounds gate-wait) or set CLR_REMAINING_TIMEOUT_SECS. See \
     docs/cli/param/033_max_sessions.md."
  );
}

fn emit_gate_wait_event(
  gate_emitted : bool,
  wait_start   : &std::time::Instant,
  journal      : Option< &JournalWriter >,
  max          : u32,
  attempt      : u32,
)
{
  if !gate_emitted { return; }
  let wait_ms = u64::try_from( wait_start.elapsed().as_millis() ).unwrap_or( u64::MAX );
  if let Some( w ) = journal
  {
    let mut ev              = EventRecord::new( EventType::GateWait );
    ev.fields.max_sessions  = Some( max );
    ev.fields.wait_ms       = Some( wait_ms );
    ev.fields.gate_attempts = Some( attempt.saturating_sub( 1 ) );
    ev.fields.gate_outcome  = Some( "acquired".to_string() );
    let _ = w.append( &ev );
  }
}

/// Block until fewer than `max` `claude` sessions are running, or until `max_attempts`
/// is exhausted.  `max == 0` means unlimited — returns immediately without checking.
///
/// Exits the process immediately (loud failure) if the process scanner
/// ([`claude_core::process::proc_scan_available`]) cannot read the process list —
/// e.g. `/proc` missing on a non-Linux host, or a misconfigured `CLR_PROC_DIR` in
/// tests. Silently proceeding would make [`claude_core::process::find_claude_processes`]'s
/// deliberately-silent empty-`Vec` fallback look identical to "zero sessions running",
/// letting the gate wave through unlimited concurrent sessions while believing it
/// still enforces `max` — see `pid_alive()` below for why this module targets Linux
/// hosts only.
///
/// While waiting, writes a JSON state file to `$CLR_GATE_DIR/{pid}.json` so that
/// `clr ps` can display this process in its "Queued CLR Processes" table.  The file
/// is updated each polling iteration and removed automatically by the `GateFile` Drop
/// guard on both normal and panic exit paths.
///
/// When `max_attempts` is reached, calls `on_exhausted` with the timeout error instead
/// of retrying internally — callers with Runner-class retry (`run`/`ask`, via
/// `apply_runner_retry()`) retry the whole polling sequence from their closure; callers
/// without it (`isolated`) report and exit from theirs. Either way, returning from the
/// closure resumes the outer poll loop; a closure that exits the process never returns.
///
/// `caller_timeout_secs` carries the caller's EXPRESSED timeout (`--timeout` flag or
/// `CLR_TIMEOUT` env) for BUG-445's gate-budget defaulting: pass the expressed value in
/// seconds, or `0` when the caller expressed nothing (built-in defaults never qualify)
/// or explicitly opted out via `--timeout 0` — `0` disables the fallback, leaving
/// `CLR_REMAINING_TIMEOUT_SECS` as the only budget input, exactly the pre-fix behavior.
#[ allow( clippy::too_many_lines, clippy::too_many_arguments ) ] // gate admission orchestration — census read, slot sweep, one-time resolution announcement, exhaustion routing, and telemetry in one coherent poll loop (mirrors execution.rs retry orchestration); the 8th param is BUG-445's expressed-timeout budget input, resolved per-caller
pub( super ) fn wait_for_session_slot(
  max                 : u32,
  quiet               : bool,
  poll_secs           : u64,
  max_attempts        : u32,
  stale_secs          : Option< u64 >,
  caller_timeout_secs : u64,
  journal             : Option< &JournalWriter >,
  on_exhausted        : &mut dyn FnMut( &std::io::Error ),
)
{
  if max == 0 { return; }

  // Fix: fail loudly instead of silently no-op'ing when the process scanner is
  // unavailable — see doc comment above.
  if !claude_core::process::proc_scan_available()
  {
    eprintln!(
      "Error: [Runner] session gate unavailable — process scanner cannot read the process list (--max-sessions requires working /proc; pass --max-sessions 0 to disable the gate) (exit 1)"
    );
    std::process::exit( 1 );
  }

  let ( effective_max_attempts, budget_is_limiting, deadline_state ) = effective_gate_attempts( max_attempts, poll_secs, caller_timeout_secs );
  // Fix(BUG-481): resolved state of the staleness-reclaim protection, from the
  // already-tier-resolved parameter (run/ask: 5-tier chain; isolated: env-only)
  // rather than re-reading CLR_GATE_STALE_SECS — the announcement must report
  // what this gate entry actually runs with, whichever tier supplied it.
  // Root cause: gate_stale_secs_from's None (unset or invalid) silently
  // disabled staleness reclaim — the permanence enabler for BUG-479's
  // zombie-held slots — with no surface distinguishing off from on.
  // Pitfall: same silent-off pattern as the deadline clamp; both optional
  // protections announce in one line at the first denied attempt.
  let stale_state = match stale_secs
  {
    Some( s ) => format!( "on ({s}s)" ),
    None      => "off".to_string(),
  };

  let poll = core::time::Duration::from_secs( poll_secs );

  // Gate state file — best-effort; I/O failures must not abort the caller.
  let pid        = std::process::id();
  // Fix(BUG-488): capture this process's own start time once — recorded in
  // every slot/ticket/waiter artifact this call writes, binding each record
  // to this exact process incarnation. See pid_alive() for the full comment.
  let my_starttime = proc_starttime( pid );
  let dir        = gate_dir();
  let _          = std::fs::create_dir_all( &dir );
  let state_path = dir.join( format!( "{pid}.json" ) );
  let cwd        = std::env::current_dir()
    .map( |p| p.display().to_string() )
    .unwrap_or_default();
  // Fix(BUG-384): escape reserved JSON characters before interpolating cwd into the
  // hand-rolled JSON literal below — Unix paths may contain `"`, `\`, or raw control
  // characters, any of which would otherwise corrupt the gate-state file's JSON.
  // Root cause: format!() performs no JSON escaping; cwd was spliced in raw.
  // Pitfall: never hand-roll JSON from an OS-controlled string without escaping —
  // Unix paths permit any byte except `/` and NUL. See json_escape_str() above for
  // why a single-pass escaper replaced this fix's first, incomplete `.replace()` chain.
  let cwd_escaped = json_escape_str( &cwd );
  let since = unix_now();
  // Fix(BUG-488): waiter telemetry carries the same incarnation binding as
  // slot records, so ps.rs's display-liveness filter can verify it too.
  let starttime_field = my_starttime.map_or_else( String::new, | st | format!( r#","starttime":{st}"# ) );
  let _     = std::fs::write(
    &state_path,
    format!( r#"{{"cwd":"{cwd_escaped}","since":{since}{starttime_field},"attempt":0,"message":"waiting for session slot"}}"# ),
  );

  // Drop guard removes the gate file on return or unwinding panic ONLY —
  // std::process::exit() (e.g. the exhaustion path's exit(1)) and signals skip
  // destructors, leaving the file behind. Those orphans are cleaned up by the
  // zombie-aware liveness self-heal in ps.rs::build_queued_table() (BUG-479).
  let _guard         = GateFile( state_path.clone() );
  let wait_start     = std::time::Instant::now();
  let mut gate_emitted = false;
  let mut deadline_emitted = false;

  // Outer loop: each iteration is one full max_attempts-poll sequence.
  // on_exhausted() either returns (retries the sequence) or exits.
  loop
  {
    for attempt in 1..=effective_max_attempts
    {
      // Print-mode only: interactive sessions never contend for a print-mode slot.
      let count = find_claude_processes()
        .iter()
        .filter( | p | super::ps::classify_mode( &p.args ) == "print" )
        .count();
      let count_u32 = u32::try_from( count ).unwrap_or( u32::MAX );
      // BUG-422: this count can transiently over-read by 1 due to a fork→exec
      // race — a `claude --print` child briefly inherits the parent's cmdline
      // before exec() replaces it; a concurrent /proc scan counts both parent
      // and child, yielding count+1.  Impact is bounded: at most one wasted
      // 30s poll cycle.  Real admission is protected by the slot-file CAS
      // below (acquire_slot()), so no over-admission is possible.  The display
      // count is clamped to max at the eprintln! site further below.
      // Fix(BUG-387): admission now additionally requires winning the atomic
      // reservation at index `count_u32` — see slot_path() for why the index
      // is derived from this same count read instead of a separate counter.
      // A losing race falls through to the existing wait-and-retry tail below,
      // exactly as the old `count >= max` case already did.
      let has_capacity = count_u32 < max;
      // Fix(BUG-404): a denial at the single count-derived index does NOT mean
      // no index anywhere is available — count_u32 is just the live process
      // count, unrelated to which of the 0..max slot indices are actually
      // free or dead-and-reclaimable at this instant. Try count_u32 first
      // (preserves BUG-387's shared-stale-count arbitration for the common
      // contested-same-index case — concurrent racers observing the same
      // stale count still contend for the same primary index first), then
      // fall back to every other index in 0..max before giving up on this
      // attempt.
      // Root cause: wait_for_session_slot() computed and tried exactly one
      // candidate index per attempt; a live (even perfectly healthy) owner at
      // that one index starved the waiter even while other indices sat
      // completely free or dead — confirmed empirically in production (4 of 6
      // real /tmp/clr-gate slots dead-and-untried during a live user report).
      // Pitfall: each acquire_slot() call is independently atomic
      // (create_new); trying several within one attempt introduces no new
      // race — it only widens which single index this attempt can land on.
      // Fix(BUG-480) task/claude_runner/bug/480_gate_diagnostic_hides_slot_occupancy.md: tally denied
      // indices beside the sweep's surviving Result, so the blocking quantity
      // (slot occupancy) survives to the poll line and the exhaustion messages.
      // Root cause: admission is a conjunction (census AND slot-CAS), but this
      // sweep collapsed per-index outcomes into one fieldless Result — every
      // diagnostic then interpolated only the census conjunct's locals, so 66
      // consecutive `active=1/8` polls showed "7 free" while 8/8 slot files
      // were held, the actual blocker appearing on no surface.
      // Pitfall: when an admission predicate is a conjunction, thread at least
      // one measured value from the conjunct that actually failed into every
      // denial diagnostic — a message interpolating only the other conjunct's
      // variables misattributes the denial.
      let mut denied_slots : u32 = 0;
      let claim = if has_capacity
      {
        let mut result = acquire_slot( &dir, count_u32, pid, since, my_starttime, stale_secs );
        if result.is_err()
        {
          denied_slots += 1;
          for candidate in 0..max
          {
            if candidate == count_u32 { continue; }
            result = acquire_slot( &dir, candidate, pid, since, my_starttime, stale_secs );
            if result.is_ok() { break; }
            denied_slots += 1;
          }
        }
        Some( result )
      }
      else { None };
      if let Some( Ok( () ) ) = claim
      {
        // Emit GateWait event if we actually waited at least one poll cycle.
        emit_gate_wait_event( gate_emitted, &wait_start, journal, max, attempt );
        return; // _guard.drop() removes only the {pid}.json telemetry file —
                // the slot reservation from acquire_slot() is deliberately
                // left in place for the rest of this session; see slot_path().
      }
      // Fix(BUG-481): announce the resolved state of both optional gate
      // protections (deadline clamp, staleness reclaim) exactly once per gate
      // entry, on the first DENIED attempt — admission without waiting stays
      // silent (user_story/025 AC-001 promises no gate messages on immediate
      // admission), and retry-restarted sequences do not re-announce.
      // Root cause: every non-engaged resolution of either protection was
      // silent, so a dead deadline clamp and a disabled staleness reclaim
      // were indistinguishable from healthy engaged ones on every surface.
      // Pitfall: emit before the exhaustion branch, not inside the poll-line
      // branch — a max_attempts=1 run exhausts without ever emitting a poll
      // line, and the resolution state matters most on exactly that run.
      if !quiet && !deadline_emitted
      {
        deadline_emitted = true;
        eprintln!(
          "{}gate-deadline  {deadline_state} · stale-reclaim {stale_state}",
          claude_core::trace_ts()
        );
      }
      if attempt == effective_max_attempts
      {
        // Fix(BUG-298): add [Runner] prefix + correct message text to match 14_error_class.md.
        // Root cause: gate-timeout message lacked [Runner] class prefix; display showed no class label.
        // Pitfall: every message-construction site must inject the [Runner] prefix, not only spawn paths.
        // Fix(BUG-299): wrap gate-timeout in retry handling instead of unconditional exit(1).
        // Root cause: gate-timeout path called exit(1) directly; runner retry system not invoked here.
        // Pitfall: every early-exit path (including gate timeouts) must route through `on_exhausted` —
        // the caller's closure decides retry-vs-exit (e.g. run/ask wraps apply_runner_retry(); isolated
        // exits directly), rather than gate.rs hardcoding one policy for every caller.
        // Fix(BUG-433): gate timeout error omitted the "print" qualifier.
        // Root cause: `count` counts only print-mode processes, but the error said
        // "active sessions" — suggesting it counted all sessions (total occupancy).
        // Pitfall: "print sessions" must not be changed back to "active sessions";
        // t09/t29/t31/t16 timeout assertions guard the exact substring.
        // Fix(BUG-480): mirror the measured slot occupancy into both exhaustion
        // messages — only when this final attempt's denial was slot-side (the
        // sweep actually ran and measured). The at-capacity arm (claim == None)
        // never ran the sweep, so denied_slots is unmeasured there and the
        // field is omitted — which also keeps the T29/T31 full-line guards
        // (at-capacity fixtures) byte-identical.
        // Root cause: both messages interpolated only the census conjunct
        // (count/max), hiding the slot-side blocker at exhaustion exactly as
        // the poll line did during the wait.
        // Pitfall: emit `slots=` only for measured sweeps — interpolating an
        // unmeasured 0 on the at-capacity arm would misreport "0 held" while
        // slots may well be occupied.
        let slots_field = match claim
        {
          Some( Err( _ ) ) => format!( ", slots={denied_slots}/{max} held" ),
          _                => String::new(),
        };
        let e = if budget_is_limiting {
          // CLR_REMAINING_TIMEOUT_SECS budget exhausted: emit a distinct diagnostic
          // so operators can identify gate-wait budget exhaustion in job stderr output
          // without counting attempt lines manually (see Fix(BUG-423) above).
          std::io::Error::other(
            format!( "gate-wait budget exhausted — {count} print sessions, max-sessions={max}, budget={effective_max_attempts} attempt(s){slots_field}" )
          )
        } else {
          std::io::Error::other(
            format!( "session gate timed out — {count} print sessions, max-sessions={max}{slots_field}" )
          )
        };
        on_exhausted( &e );
        break; // non-exhaustion path: restart outer poll loop
      }
      if !quiet
      {
        // Fix(BUG-393): distinguish global exhaustion (no slot numerically free)
        // from every other non-admission cause — both previously produced
        // byte-identical text since the message only interpolated the
        // count/max counters shared across every false-branch of the
        // compound admission condition above.
        // Fix(BUG-396): the has_capacity-true branch itself further splits
        // into "another live session already holds this index" (confirmed
        // via production evidence to be the overwhelmingly common case: job
        // #40 reported "lost reservation race" at 4/6 sessions while
        // slot_4.json's recorded owner was actually alive — no reclaim was
        // ever attempted, so no race occurred) versus "the recorded owner
        // was dead but I lost the reclaim-ticket race to another concurrent
        // reclaimer" (the only scenario that is genuinely a race; see T14's
        // dead-owner fixture). BUG-393's original fix distinguished capacity
        // from non-capacity only, and mislabeled every non-capacity denial a
        // "race" regardless of which of acquire_slot()'s two distinct
        // `Err` branches actually produced it.
        // Root cause: acquire_slot() returned a bare `bool`, discarding which
        // of its 2 internal denial branches fired; the message site then had
        // no way to tell "owner alive, no race occurred" apart from "owner
        // dead, ticket race lost" and defaulted to naming both a "race".
        // Pitfall: a diagnostic that names a specific mechanism ("race") must
        // be verified against every code path that reaches it — the
        // overwhelmingly common non-admission cause (an unrelated live
        // session already owns this index) never contends with anything at
        // all, and calling it a "race" actively misleads an operator into
        // expecting imminent, capacity-unrelated resolution that may not
        // arrive until that specific session ends. The cause is placed in
        // the `(reason: ...)` trailer of the TSK-452 structured line
        // `"gate-wait  active=X/Y attempt=A/MA wait=Ss (reason: {cause})"`.
        // T01/T04/T27/T32 assert "gate-wait  active="; T02/T03/T06/T28
        // assert absence of "gate-wait"; 5 positive sites in config_file_test.rs
        // assert "gate-wait  active=" anchored to the count ratio.
        let cause = match claim
        {
          Some( Err( SlotDenialCause::HeldByLive ) )        => "slot held by another session",
          Some( Err( SlotDenialCause::LostReclaimRace ) )   => "lost reservation race",
          // None: has_capacity was false. Some(Ok(())): unreachable — the
          // admitted branch already returned above — but required for
          // match exhaustiveness, so it shares the "at capacity" arm rather
          // than duplicating it (clippy::match_same_arms).
          None | Some( Ok( () ) )                           => "at capacity",
        };
        // Fix(BUG-422): clamp display count to max — fork→exec race can produce
        // count > max transiently (see comment at count computation site above).
        // Root cause: /proc scan counts a forked-but-not-yet-exec'd child that
        // briefly inherits the parent's claude --print cmdline; yields count+1.
        // Pitfall: clamp applies to display only — has_capacity and acquire_slot()
        // are intentionally left unchanged; CAS provides the real admission gate.
        let display_count = count_u32.min( max );
        // Fix(BUG-431): gate progress message omitted the "print" qualifier.
        // Root cause: `count` is filtered to print-mode processes only, but the message
        // said "sessions active" — indistinguishable from a total count.
        // TSK-452: format changed to structured timestamp-prefixed line; the mode scope
        // is preserved — `display_count` counts print-mode processes only (same count
        // as before); `t_gate_progress_message_names_print_sessions` now guards
        // the `"gate-wait  active="` label rather than the old `"print sessions active"` phrase.
        // Fix(BUG-480): additive ` slots={denied_slots}/{max}` field after the
        // pinned `active=` ratio — only when this attempt's denial was slot-side
        // (the sweep ran and measured). At-capacity lines never carry the field:
        // denied_slots is unmeasured there, and every pinned `active=N/N`
        // format guard is an at-capacity fixture, so those lines stay
        // byte-identical. See the sweep-site Fix(BUG-480) comment above for
        // root cause and pitfall.
        let slots_field = match claim
        {
          Some( Err( _ ) ) => format!( " slots={denied_slots}/{max}" ),
          _                => String::new(),
        };
        eprintln!(
          "{}gate-wait  active={display_count}/{max}{slots_field} attempt={attempt}/{effective_max_attempts} wait={poll_secs}s (reason: {cause})",
          claude_core::trace_ts()
        );
      }
      gate_emitted = true;
      let _ = std::fs::write(
        &state_path,
        format!( r#"{{"cwd":"{cwd_escaped}","since":{since}{starttime_field},"attempt":{attempt},"message":"waiting for session slot"}}"# ),
      );
      std::thread::sleep( poll );
    }
  }
}
