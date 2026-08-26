//! Turn-boundary detection tests.
//!
//! ## Specification References
//!
//! - `docs/feature/002_turn_detection.md` — the detection contract
//! - `docs/invariant/002_first_sighting_never_settles.md` — the edge-trigger rule
//!
//! ## Coverage
//!
//! | TC | Scenario | Expected |
//! |----|----------|----------|
//! | turn01 | First observation is `Idle` | `None` — no turn to have ended |
//! | turn02 | First observation is `Busy` | `None` — no prior state to transition from |
//! | turn03 | `Busy` → `Idle`, reporting enabled | `Settled` |
//! | turn04 | `Busy` → `Idle`, reporting unknown | `SettledUnverified` |
//! | turn05 | `Idle` → `Busy` | `Started` |
//! | turn06 | `Other` → `Busy` | `Started` |
//! | turn07 | Repeated identical status | `None` |
//! | turn08 | Transition into `Other` | `None` — not a modelled boundary |
//! | turn09 | A full turn cycle | `Started` then `Settled` |
//! | turn10 | `last()` before any observation | `None` |
//! | turn11 | `last()` tracks the most recent status | Matches what was fed |
//! | turn12 | `SessionStatus::from_raw` | `busy`/`idle` known, anything else `Other` |
//! | turn13 | The env var name | Matches what Claude Code reads |

use claude_session_core::{ BackgroundReporting, SessionStatus, TurnEvent, TurnWatcher };
use claude_session_core::turn::BG_TASKS_REPORT_RUNNING_ENV;

/// A watcher whose sessions were spawned with the background guarantee.
fn verified() -> TurnWatcher
{
  TurnWatcher::new( BackgroundReporting::Enabled )
}

/// A watcher observing a session it did not spawn.
fn unverified() -> TurnWatcher
{
  TurnWatcher::new( BackgroundReporting::Unknown )
}

/// turn01: attaching to an already-idle session reports nothing.
///
/// This is the reconnect path — daemon restart, late client attach, post-fork
/// re-host. If it produced `Settled`, every reattach would deliver a completion
/// signal for a turn that was never observed to start.
#[ test ]
fn turn01_first_sighting_idle_is_not_a_boundary()
{
  let mut watcher = verified();

  assert_eq!( watcher.observe( &SessionStatus::Idle ), None );
}

/// turn02: the same holds when the first sighting is `Busy`.
#[ test ]
fn turn02_first_sighting_busy_is_not_a_boundary()
{
  let mut watcher = verified();

  assert_eq!( watcher.observe( &SessionStatus::Busy ), None );
}

/// turn03: a real boundary, from a session spawned with the guarantee.
#[ test ]
fn turn03_busy_to_idle_with_reporting_enabled_settles()
{
  let mut watcher = verified();

  assert_eq!( watcher.observe( &SessionStatus::Busy ), None );
  assert_eq!( watcher.observe( &SessionStatus::Idle ), Some( TurnEvent::Settled ) );
}

/// turn04: the same transition is labelled, not suppressed, without the guarantee.
///
/// With `CLAUDE_CODE_BG_TASKS_REPORT_RUNNING` unset, a session with background
/// work in flight reports `idle`. The consumer still gets a signal; it gets told
/// what the signal is worth.
#[ test ]
fn turn04_busy_to_idle_without_guarantee_is_unverified()
{
  let mut watcher = unverified();

  assert_eq!( watcher.observe( &SessionStatus::Busy ), None );
  assert_eq!( watcher.observe( &SessionStatus::Idle ), Some( TurnEvent::SettledUnverified ) );
}

/// turn05, turn06: entering `busy` from any non-busy state starts a turn.
#[ test ]
fn turn05_entering_busy_starts_a_turn()
{
  let mut from_idle = verified();
  assert_eq!( from_idle.observe( &SessionStatus::Idle ), None );
  assert_eq!( from_idle.observe( &SessionStatus::Busy ), Some( TurnEvent::Started ) );

  let mut from_other = verified();
  assert_eq!( from_other.observe( &SessionStatus::Other( "compacting".into() ) ), None );
  assert_eq!( from_other.observe( &SessionStatus::Busy ), Some( TurnEvent::Started ) );
}

/// turn07: a repeated status is not a transition.
///
/// Polling frequency must not change how many turns a consumer sees.
#[ test ]
fn turn07_repeated_status_is_not_a_transition()
{
  let mut watcher = verified();

  assert_eq!( watcher.observe( &SessionStatus::Busy ), None );
  assert_eq!( watcher.observe( &SessionStatus::Busy ), None );
  assert_eq!( watcher.observe( &SessionStatus::Busy ), None );
  assert_eq!( watcher.observe( &SessionStatus::Idle ), Some( TurnEvent::Settled ) );
  assert_eq!( watcher.observe( &SessionStatus::Idle ), None );
}

/// turn08: an unmodelled status is not a boundary in either direction.
#[ test ]
fn turn08_transition_into_other_is_not_a_boundary()
{
  let mut watcher = verified();

  assert_eq!( watcher.observe( &SessionStatus::Busy ), None );
  assert_eq!( watcher.observe( &SessionStatus::Other( "unknown".into() ) ), None );
}

/// turn09: a full cycle produces exactly one start and one settle.
#[ test ]
fn turn09_full_cycle_produces_one_start_and_one_settle()
{
  let mut watcher = verified();
  let sequence = [
    SessionStatus::Idle,
    SessionStatus::Busy,
    SessionStatus::Busy,
    SessionStatus::Idle,
  ];

  let events : Vec< TurnEvent > = sequence
    .iter()
    .filter_map( | status | watcher.observe( status ) )
    .collect();

  assert_eq!( events, vec![ TurnEvent::Started, TurnEvent::Settled ], "unexpected event stream" );
}

/// turn10, turn11: `last()` reflects what has been observed.
#[ test ]
fn turn10_last_tracks_observations()
{
  let mut watcher = verified();
  assert_eq!( watcher.last(), None, "a fresh watcher has observed nothing" );

  watcher.observe( &SessionStatus::Busy );
  assert_eq!( watcher.last(), Some( &SessionStatus::Busy ) );

  watcher.observe( &SessionStatus::Idle );
  assert_eq!( watcher.last(), Some( &SessionStatus::Idle ) );
}

/// turn12: status parsing keeps unknown values rather than collapsing them.
///
/// Collapsing an unrecognized status to `Idle` would make a future Claude Code
/// state read as "finished" — the failure mode this whole module exists to
/// avoid.
#[ test ]
fn turn12_status_from_raw_preserves_unknown_values()
{
  assert_eq!( SessionStatus::from_raw( "busy" ), SessionStatus::Busy );
  assert_eq!( SessionStatus::from_raw( "idle" ), SessionStatus::Idle );
  assert_eq!(
    SessionStatus::from_raw( "compacting" ),
    SessionStatus::Other( "compacting".to_string() ),
  );
}

/// turn13: the environment variable name Claude Code actually reads.
///
/// A typo here would silently disable the mitigation: the variable would be set,
/// nothing would read it, and every `Settled` would be a `SettledUnverified` in
/// disguise.
#[ test ]
fn turn13_background_reporting_env_var_name()
{
  assert_eq!(
    BG_TASKS_REPORT_RUNNING_ENV, "CLAUDE_CODE_BG_TASKS_REPORT_RUNNING",
    "the env var name changed — verify against Claude Code before updating",
  );
}
