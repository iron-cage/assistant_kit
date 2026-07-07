use claude_core::process::find_claude_processes;
use core::fmt::Write as _;
use std::io::Write as _;
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

/// Return the gate poll interval in seconds.
///
/// Default: 30 seconds. `CLR_GATE_POLL_SECS` env var overrides — public,
/// documented override, no CLI flag, no `--args-file` key. Invalid values fall
/// back to 30 silently.
fn gate_poll_secs() -> u64
{
  std::env::var( "CLR_GATE_POLL_SECS" )
    .ok()
    .and_then( | s | s.parse().ok() )
    .unwrap_or( 30 )
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

/// Attempt limit override for the concurrency gate. Public, documented
/// override — no CLI flag, no `--args-file` key. Invalid values fall
/// back to 1000 silently.
fn gate_max_attempts() -> u32
{
  gate_max_attempts_from( std::env::var( "CLR_GATE_MAX_ATTEMPTS" ).ok().as_deref() )
}

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

// Attempt to atomically create the slot file at `path`, recording the owning
// PID and timestamp. Returns `true` on success; `false` if the slot is
// already held (`AlreadyExists`) or any other I/O failure occurs.
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

// Return the (pid, since) recorded in a slot file, if the file is readable and
// well-formed. Fix(BUG-392) needs `since` in addition to `pid` to key the
// reclaim ticket path deterministically — see acquire_slot() below.
fn read_slot_owner_record( path : &Path ) -> Option< ( u32, u64 ) >
{
  let content = std::fs::read_to_string( path ).ok()?;
  let pid     = u32::try_from( super::ps::parse_json_u64( &content, "pid" )? ).ok()?;
  let since   = super::ps::parse_json_u64( &content, "since" )?;
  Some( ( pid, since ) )
}

// Return whether `pid` is currently alive via `/proc/{pid}` existence —
// matches the identical liveness convention `build_queued_table()` already
// uses in `ps.rs` (this module targets Linux hosts only).
fn pid_alive( pid : u32 ) -> bool
{
  Path::new( &format!( "/proc/{pid}" ) ).exists()
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

/// Block until fewer than `max` `claude` sessions are running, or until the 1000-attempt
/// limit is exhausted.  `max == 0` means unlimited — returns immediately without checking.
///
/// While waiting, writes a JSON state file to `$CLR_GATE_DIR/{pid}.json` so that
/// `clr ps` can display this process in its "Queued CLR Processes" table.  The file
/// is updated each polling iteration and removed automatically by the `GateFile` Drop
/// guard on both normal and panic exit paths.
///
/// When the 1000-attempt limit is reached, applies Runner-class retry via
/// `apply_runner_retry()` — the entire 1000-attempt polling sequence is retried
/// `--retry-on-runner N` times before giving up.
pub( super ) fn wait_for_session_slot(
  max   : u32,
  quiet : bool,
  cli   : &super::parse::CliArgs,
  journal   : Option< &JournalWriter >,
)
{
  if max == 0 { return; }
  let poll_secs    = gate_poll_secs();
  let poll         = core::time::Duration::from_secs( poll_secs );
  let max_attempts = gate_max_attempts();

  // Gate state file — best-effort; I/O failures must not abort the caller.
  let pid        = std::process::id();
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
  let _     = std::fs::write(
    &state_path,
    format!( r#"{{"cwd":"{cwd_escaped}","since":{since},"attempt":0,"message":"waiting for session slot"}}"# ),
  );

  // Drop guard ensures the gate file is removed on return, panic, or exit(1).
  let _guard         = GateFile( state_path.clone() );
  let mut runner_attempt = 0u32;
  let wait_start     = std::time::Instant::now();
  let mut gate_emitted = false;

  // Outer loop: each iteration is one full 1000-poll-attempt sequence.
  // apply_runner_retry() either returns (retries the sequence) or exits.
  loop
  {
    for attempt in 1..=max_attempts
    {
      // Print-mode only: interactive sessions never contend for a print-mode slot.
      let count = find_claude_processes()
        .iter()
        .filter( | p | super::ps::classify_mode( &p.args ) == "print" )
        .count();
      let count_u32 = u32::try_from( count ).unwrap_or( u32::MAX );
      // Fix(BUG-387): admission now additionally requires winning the atomic
      // reservation at index `count_u32` — see slot_path() for why the index
      // is derived from this same count read instead of a separate counter.
      // A losing race falls through to the existing wait-and-retry tail below,
      // exactly as the old `count >= max` case already did.
      if count_u32 < max && acquire_slot( &dir, count_u32, pid, since )
      {
        // Emit GateWait event if we actually waited at least one poll cycle.
        if gate_emitted
        {
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
        return; // _guard.drop() removes only the {pid}.json telemetry file —
                // the slot reservation from acquire_slot() is deliberately
                // left in place for the rest of this session; see slot_path().
      }
      if attempt == max_attempts
      {
        // Fix(BUG-298): add [Runner] prefix + correct message text to match 14_error_class.md.
        // Root cause: gate-timeout message lacked [Runner] class prefix; display showed no class label.
        // Pitfall: every message-construction site must inject the [Runner] prefix, not only spawn paths.
        // Fix(BUG-299): wrap gate-timeout in apply_runner_retry() instead of unconditional exit(1).
        // Root cause: gate-timeout path called exit(1) directly; runner retry system not invoked here.
        // Pitfall: every early-exit path (including gate timeouts) must route through apply_runner_retry().
        let e = std::io::Error::other(
          format!( "session gate timed out — {count} active sessions, max-sessions={max}" )
        );
        super::execution::apply_runner_retry( cli, &e, &mut runner_attempt, journal );
        break; // non-exhaustion path: restart outer poll loop
      }
      if !quiet
      {
        eprintln!(
          "Info: {count}/{max} sessions active; waiting {poll_secs}s for a slot... (attempt {attempt}/{max_attempts})"
        );
      }
      gate_emitted = true;
      let _ = std::fs::write(
        &state_path,
        format!( r#"{{"cwd":"{cwd_escaped}","since":{since},"attempt":{attempt},"message":"waiting for session slot"}}"# ),
      );
      std::thread::sleep( poll );
    }
  }
}
