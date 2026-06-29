use claude_core::process::find_claude_processes;
use std::path::PathBuf;
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
/// In production: always 30 seconds.
/// In tests: `_CLR_GATE_POLL_SECS` env var overrides so tests don't wait 30s per attempt.
/// The `_` prefix signals internal/test-only use — not exposed in `--help`.
fn gate_poll_secs() -> u64
{
  std::env::var( "_CLR_GATE_POLL_SECS" )
    .ok()
    .and_then( | s | s.parse().ok() )
    .unwrap_or( 30 )
}

/// Block until fewer than `max` `claude` sessions are running, or until the 100-attempt
/// limit is exhausted.  `max == 0` means unlimited — returns immediately without checking.
///
/// While waiting, writes a JSON state file to `$CLR_GATE_DIR/{pid}.json` so that
/// `clr ps` can display this process in its "Queued CLR Processes" table.  The file
/// is updated each polling iteration and removed automatically by the `GateFile` Drop
/// guard on both normal and panic exit paths.
///
/// When the 100-attempt limit is reached, applies Runner-class retry via
/// `apply_runner_retry()` — the entire 100-attempt polling sequence is retried
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
  let max_attempts = 100_u32;

  // Gate state file — best-effort; I/O failures must not abort the caller.
  let pid        = std::process::id();
  let dir        = gate_dir();
  let _          = std::fs::create_dir_all( &dir );
  let state_path = dir.join( format!( "{pid}.json" ) );
  let cwd        = std::env::current_dir()
    .map( |p| p.display().to_string() )
    .unwrap_or_default();
  let since = unix_now();
  let _     = std::fs::write(
    &state_path,
    format!( r#"{{"cwd":"{cwd}","since":{since},"attempt":0,"message":"waiting for session slot"}}"# ),
  );

  // Drop guard ensures the gate file is removed on return, panic, or exit(1).
  let _guard         = GateFile( state_path.clone() );
  let mut runner_attempt = 0u32;
  let wait_start     = std::time::Instant::now();
  let mut gate_emitted = false;

  // Outer loop: each iteration is one full 100-poll-attempt sequence.
  // apply_runner_retry() either returns (retries the sequence) or exits.
  loop
  {
    for attempt in 1..=max_attempts
    {
      let count = find_claude_processes().len();
      if u32::try_from( count ).unwrap_or( u32::MAX ) < max
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
        return; // _guard.drop() removes the file
      }
      if attempt == max_attempts
      {
        // Fix(BUG-298): add [Runner] prefix + correct message text to match 14_error_class.md.
        // Fix(BUG-299): wrap gate-timeout in runner retry instead of unconditional exit(1).
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
        format!( r#"{{"cwd":"{cwd}","since":{since},"attempt":{attempt},"message":"waiting for session slot"}}"# ),
      );
      std::thread::sleep( poll );
    }
  }
}
