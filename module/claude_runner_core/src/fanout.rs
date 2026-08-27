//! Bounded fan-out: run many child commands at once, but never more than N.
//!
//! Forwarding one prompt to every topic means starting one child process per
//! topic. Starting them all at once is the obvious implementation and the wrong
//! one — twenty topics is twenty concurrent Claude Code sessions, twenty times
//! the token spend in the same instant, and a rate-limit wall that fails every
//! one of them instead of queueing. Running them strictly in sequence is the
//! other wrong one: the whole point of fan-out is that the topics are
//! independent, and a twenty-minute serial wait throws that away.
//!
//! So: a fixed pool of workers draining a shared queue. The concurrency bound is
//! the caller's, the ordering guarantee is this module's, and neither depends on
//! how long any individual child takes.
//!
//! ## What this module deliberately does not do
//!
//! - **No timeout.** A child that hangs hangs. Killing a Claude Code process
//!   from the outside leaves its session file mid-write and, on some paths, an
//!   orphaned subprocess of its own; the honest place for a deadline is inside
//!   each child (`clr --timeout`), where the runner already owns the cleanup.
//! - **No cancellation.** One child failing says nothing about whether its
//!   siblings should stop — they are separate conversations in separate
//!   sessions. Every command in the batch runs, and every one reports.
//! - **No streaming.** Output is captured whole, per child, and returned when
//!   the batch is done. Interleaving twenty live stdout streams onto one
//!   terminal produces something no one can read; the caller renders the
//!   collected outcomes instead.

/// Exit code recorded when the child could not be started at all.
///
/// Distinguishable from any real result by construction: a process that runs
/// reports either its own status (`0..=255`) or `128 + signal` via
/// [`crate::signal_exit_code`], and neither is ever negative. So a negative code
/// means the failure happened before the process existed, and
/// [`FanoutOutcome::stderr`] carries the spawn error rather than the child's.
pub const SPAWN_FAILED_EXIT_CODE : i32 = -1;

/// What one child in the batch did.
///
/// Produced for every command handed to [`run_bounded`], including ones that
/// never started — a batch of N commands always yields N outcomes.
#[ derive( Debug, Clone, PartialEq, Eq ) ]
pub struct FanoutOutcome
{
  /// Caller-supplied name for this command, echoed back unchanged.
  ///
  /// The commands themselves are opaque here; this is what lets the caller say
  /// *which topic* produced a given result without re-deriving it from argv.
  pub label : String,
  /// The child's exit code, or [`SPAWN_FAILED_EXIT_CODE`] if it never started.
  pub exit_code : i32,
  /// Everything the child wrote to stdout, lossily decoded as UTF-8.
  pub stdout : String,
  /// Everything the child wrote to stderr — or the spawn error, when the child
  /// never ran.
  pub stderr : String,
}

impl FanoutOutcome
{
  /// True when the child ran and exited zero.
  ///
  /// A spawn failure is not a success, which follows from
  /// [`SPAWN_FAILED_EXIT_CODE`] being non-zero — stated as a method so callers
  /// tallying results never have to remember that.
  #[ inline ]
  #[ must_use ]
  pub const fn is_success( &self ) -> bool
  {
    self.exit_code == 0
  }
}

/// Run every command with at most `concurrency` of them in flight, and return
/// one outcome per command **in the order the commands were given**.
///
/// The input order guarantee is the reason this returns a `Vec` rather than
/// handing outcomes back as they complete: a fan-out report that reorders itself
/// by whichever topic happened to answer first is not comparable across runs,
/// and comparing runs is most of what a fan-out report is for.
///
/// `concurrency` is clamped into `1..=commands.len()` — `0` would otherwise mean
/// "start nothing and wait forever", and a bound above the batch size just
/// allocates threads with no queue left to drain.
///
/// # Panics
///
/// Does not panic under normal operation. A worker thread panicking (only
/// reachable if the shared queue mutex is poisoned by an earlier panic) is
/// propagated by [`std::thread::scope`] at the end of the batch.
///
/// # Example
///
/// ```no_run
/// use claude_runner_core::fanout::run_bounded;
///
/// let mut jobs = Vec::new();
/// for topic in [ "review", "docs", "bench" ]
/// {
///   let mut cmd = std::process::Command::new( "clr" );
///   cmd.arg( "--topic" ).arg( topic ).arg( "summarize the last change" );
///   jobs.push( ( topic.to_string(), cmd ) );
/// }
///
/// // Three topics, but never more than two Claude sessions at once.
/// for outcome in run_bounded( jobs, 2 )
/// {
///   println!( "{}: exit {}", outcome.label, outcome.exit_code );
/// }
/// ```
#[ inline ]
#[ must_use ]
pub fn run_bounded
(
  commands : Vec< ( String, std::process::Command ) >,
  concurrency : usize,
) -> Vec< FanoutOutcome >
{
  if commands.is_empty()
  {
    return Vec::new();
  }
  let workers = concurrency.clamp( 1, commands.len() );

  // Indices travel with the work so a worker can write its result into the slot
  // the command came from. Draining an iterator under one mutex is what makes
  // the bound a real bound: a worker takes the next job only when it is free,
  // so at most `workers` children exist at any instant regardless of how
  // unevenly the runtimes fall.
  let queue = std::sync::Mutex::new( commands.into_iter().enumerate() );
  let results = std::sync::Mutex::new( Vec::new() );

  std::thread::scope( | scope |
  {
    for _ in 0..workers
    {
      scope.spawn( ||
      {
        loop
        {
          // Scoped so the queue lock is released before the child runs — holding
          // it across `output()` would serialize the whole batch behind whichever
          // child is currently executing, silently turning `workers` into 1.
          let next = match queue.lock()
          {
            Ok( mut guard ) => guard.next(),
            Err( _ ) => return,
          };
          let Some( ( index, ( label, mut command ) ) ) = next else { return };

          let outcome = execute_one( label, &mut command );
          if let Ok( mut guard ) = results.lock()
          {
            guard.push( ( index, outcome ) );
          }
        }
      } );
    }
  } );

  let mut collected = results.into_inner().unwrap_or_else( std::sync::PoisonError::into_inner );
  collected.sort_by_key( | &( index, _ ) | index );
  collected.into_iter().map( | ( _, outcome ) | outcome ).collect()
}

/// Run one command to completion and describe what happened.
///
/// Uses `output()` rather than a hand-rolled spawn-and-read: it drains stdout and
/// stderr concurrently, which a naive "wait, then read" does not — a child that
/// fills the stderr pipe buffer while the parent blocks on `wait()` deadlocks,
/// and a verbose Claude Code run fills it easily.
fn execute_one( label : String, command : &mut std::process::Command ) -> FanoutOutcome
{
  match command.output()
  {
    Ok( output ) => FanoutOutcome
    {
      label,
      exit_code : crate::signal_exit_code( &output.status ),
      stdout : String::from_utf8_lossy( &output.stdout ).into_owned(),
      stderr : String::from_utf8_lossy( &output.stderr ).into_owned(),
    },
    Err( e ) => FanoutOutcome
    {
      label,
      exit_code : SPAWN_FAILED_EXIT_CODE,
      stdout : String::new(),
      stderr : format!( "cannot start command: {e}" ),
    },
  }
}
