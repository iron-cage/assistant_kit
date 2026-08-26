//! PID liveness and process-incarnation checks against `/proc` (Linux-only).
//!
//! Promoted verbatim in behaviour from `claude_runner/src/cli/gate_liveness.rs`,
//! where it was `pub( super )` inside a binary crate and therefore unreachable by
//! any other consumer. That file is gone — this is the only copy, which is the
//! point: the two fixes documented below were paid for in production, and a
//! second copy is how the predicate acquires a third hole.
//!
//! Every consumer reads from here. `claude_runner`'s `gate_slot::acquire_slot`
//! for reclaim eligibility, its `ps::build_queued_table` for the queued-waiter
//! display self-heal (BUG-479/BUG-488 are the same defect seen from those two
//! sides), and `claude_daemon_core`'s registration wait for whether a spawned
//! child is still worth waiting on. The reclaim decision and the display cannot
//! drift apart from each other because there is nothing left to drift.

// BUG-479 — bare /proc/{pid} existence read unreaped zombies as live, blocking
// acquire_slot()'s owner-reclaim and reclaim-ticket claimant checks indefinitely.
// Fix(BUG-479): liveness reads the /proc/{pid}/stat state field instead of
// probing bare /proc/{pid} existence, and is the single shared predicate for
// every consumer.
// Root cause: an exited-but-unreaped child (state `Z`) keeps its /proc entry
// for as long as its parent fails to wait(), so an existence probe read
// zombies as alive — under a non-reaping supervisor every dead slot owner and
// queued waiter became permanent (7/8 slots starved; `Queued · 84 waiting`
// with 4 live).
// Pitfall: /proc/{pid} existence proves a PID exists, not that a process runs
// — any reclaim/display protocol keyed on it deadlocks/inflates the moment
// children stop being reaped. Liveness = stat readable AND state ∉ {Z}.
//
// BUG-488 — pid liveness was thread-id blind.
// Fix(BUG-488): liveness additionally requires thread-group leadership
// (`Tgid == pid` from /proc/{pid}/status) and — when the caller's record carries
// the writer's start time — a matching /proc/{pid}/stat field 22 on the current
// occupant.
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

/// Return whether `pid` is a live, running process — and, when
/// `recorded_starttime` is `Some`, the same process incarnation that wrote the
/// record.
///
/// Clauses: (a) `/proc/{pid}/stat` readable, (b) state ∉ {`Z`}, (c) thread-group
/// leader, (d) start time matches when recorded. The state field follows the
/// LAST `)` — a process's `comm` may itself contain spaces and parentheses.
///
/// Linux-only: returns `false` on any platform without `/proc`.
#[ inline ]
#[ must_use ]
pub fn pid_alive( pid : u32, recorded_starttime : Option< u64 > ) -> bool
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

/// Return the thread-group ID reported by `/proc/{pid}/status`.
///
/// For a process (thread-group leader) `Tgid == pid`; for a bare thread TID
/// resolved via direct lookup, `Tgid` names its owning process instead.
fn proc_tgid( pid : u32 ) -> Option< u32 >
{
  std::fs::read_to_string( format!( "/proc/{pid}/status" ) )
    .ok()?
    .lines()
    .find_map( | l | l.strip_prefix( "Tgid:" ).and_then( | v | v.trim().parse().ok() ) )
}

/// Return the start-time token from raw `/proc/{pid}/stat` content.
///
/// Field 22 (clock ticks since boot), stable for a process's entire life —
/// token index 19 after the `)` split, since fields 1–2 (`pid`, `comm`) precede
/// the `)`. Compared for exact equality only; never unit-converted.
fn starttime_from_stat( stat : &str ) -> Option< u64 >
{
  stat.rsplit_once( ')' )?
    .1
    .split_whitespace()
    .nth( 19 )?
    .parse()
    .ok()
}

/// Return `pid`'s current start time (field 22) for recording into an artifact
/// at write time.
///
/// Returns `None` when the process does not exist or `/proc` is unavailable.
#[ inline ]
#[ must_use ]
pub fn proc_starttime( pid : u32 ) -> Option< u64 >
{
  starttime_from_stat( &std::fs::read_to_string( format!( "/proc/{pid}/stat" ) ).ok()? )
}
