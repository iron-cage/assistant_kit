//! Turn-boundary detection, and the reason it is not simply `status == "idle"`.
//!
//! # The background-task trap
//!
//! Claude Code's `Stop` hook payload carries a `background_tasks` array, whose
//! documented purpose is to let a hook *"distinguish 'session is done' from
//! 'session is paused waiting for background work to wake it'"*. A non-empty
//! array means the session is not finished — it is parked, and will resume
//! without any new user input.
//!
//! The registry's `status` field does not expose that array. Worse, whether
//! `status` even accounts for outstanding background work is controlled by an
//! environment variable that defaults to **off**: with
//! `CLAUDE_CODE_BG_TASKS_REPORT_RUNNING` unset, a session with background tasks
//! in flight reports `idle`.
//!
//! A caller that treats the first `busy` → `idle` transition as "the answer is
//! ready" will therefore return control to the user mid-turn, intermittently and
//! unreproducibly, depending on whether that particular prompt happened to spawn
//! background work.
//!
//! # The mitigation
//!
//! Spawn every observed session with [`BG_TASKS_REPORT_RUNNING_ENV`] set to
//! `"1"`. Then `busy` covers background work too, and the transition to `idle`
//! is a real turn boundary. [`TurnWatcher`] requires the caller to state whether
//! that was done, so a session observed without the guarantee is *labelled*
//! rather than silently trusted.

use crate::registry::SessionStatus;

/// Environment variable that makes `status` account for outstanding background
/// tasks.
///
/// Defaults to off inside Claude Code. Set it to `"1"` on every session this
/// crate is expected to observe.
pub const BG_TASKS_REPORT_RUNNING_ENV : &str = "CLAUDE_CODE_BG_TASKS_REPORT_RUNNING";

/// Whether a session was started with background-task reporting enabled.
///
/// Not a detail the caller may skip: it decides whether an observed `idle` is
/// trustworthy, and there is no way to recover the answer from the registry.
#[ derive( Debug, Clone, Copy, PartialEq, Eq ) ]
pub enum BackgroundReporting
{
  /// The session was spawned with [`BG_TASKS_REPORT_RUNNING_ENV`] set to `"1"`,
  /// so `idle` accounts for background work.
  Enabled,
  /// The session was spawned without the guarantee — it may report `idle` while
  /// background tasks are still outstanding.
  Unknown,
}

/// What an observed status transition means.
#[ derive( Debug, Clone, Copy, PartialEq, Eq ) ]
pub enum TurnEvent
{
  /// A turn started.
  Started,
  /// A turn ended, and the session was spawned with background reporting on —
  /// this is a real boundary.
  Settled,
  /// A turn appears to have ended, but the session was spawned without
  /// background-task reporting, so an outstanding background task would look
  /// identical. Treat as advisory, not as completion.
  SettledUnverified,
}

/// Tracks a single session's status transitions and reports turn boundaries.
///
/// Edge-triggered: a boundary is reported only on a transition into `idle` from
/// a known-`busy` state. A watcher that first observes a session already `idle`
/// reports nothing, because a session that was idle before anyone looked has no
/// turn to have finished.
#[ derive( Debug, Clone ) ]
pub struct TurnWatcher
{
  reporting : BackgroundReporting,
  last : Option< SessionStatus >,
}

impl TurnWatcher
{
  /// Start watching a session whose background-reporting guarantee is
  /// `reporting`.
  #[ inline ]
  #[ must_use ]
  pub const fn new( reporting : BackgroundReporting ) -> Self
  {
    Self { reporting, last : None }
  }

  /// Feed the latest observed status; return the transition it represents.
  ///
  /// Returns `None` when the status has not changed, when the session was
  /// already idle on first observation, or when the status is one this crate
  /// does not model.
  #[ inline ]
  pub fn observe( &mut self, status : &SessionStatus ) -> Option< TurnEvent >
  {
    // First sighting is never a boundary — there is no prior turn to end, so an
    // already-idle session observed for the first time must not read as "just
    // settled".
    let previous = self.last.replace( status.clone() )?;
    match ( &previous, status )
    {
      ( SessionStatus::Busy, SessionStatus::Idle ) => Some( match self.reporting
      {
        BackgroundReporting::Enabled => TurnEvent::Settled,
        BackgroundReporting::Unknown => TurnEvent::SettledUnverified,
      } ),
      ( SessionStatus::Idle | SessionStatus::Other( _ ), SessionStatus::Busy ) => Some( TurnEvent::Started ),
      _ => None,
    }
  }

  /// The most recently observed status, if any.
  #[ inline ]
  #[ must_use ]
  pub const fn last( &self ) -> Option< &SessionStatus >
  {
    self.last.as_ref()
  }
}
