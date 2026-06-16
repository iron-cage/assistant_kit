use crate::VerbosityLevel;
use claude_core::process::find_claude_processes;
use std::path::PathBuf;

// Return the gate state directory — $CLR_GATE_DIR or /tmp/clr-gate.
//
// $CLR_GATE_DIR is the single test-injection point; tests override it to a temp
// dir so IT-10/IT-11 never touch the real /tmp/clr-gate on the host.
pub( super ) fn gate_dir() -> PathBuf
{
  std::env::var( "CLR_GATE_DIR" )
    .ok()
    .filter( |s| !s.is_empty() )
    .map_or_else( || PathBuf::from( "/tmp/clr-gate" ), PathBuf::from )
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

/// Block until fewer than `max` `claude` sessions are running, or until the 100-attempt
/// limit is exhausted.  `max == 0` means unlimited — returns immediately without checking.
///
/// While waiting, writes a JSON state file to `$CLR_GATE_DIR/{pid}.json` so that
/// `clr ps` can display this process in its "Queued CLR Processes" table.  The file
/// is updated each polling iteration and removed automatically by the `GateFile` Drop
/// guard on both normal and panic exit paths.
pub( super ) fn wait_for_session_slot( max : u32, verbosity : VerbosityLevel )
{
  if max == 0 { return; }
  let poll         = core::time::Duration::from_secs( 30 );
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
  let _guard = GateFile( state_path.clone() );

  for attempt in 1..=max_attempts
  {
    let count = find_claude_processes().len();
    if u32::try_from( count ).unwrap_or( u32::MAX ) < max
    {
      return; // _guard.drop() removes the file
    }
    if attempt == max_attempts
    {
      // BUG-299: gate timeout exits without runner retry wrapper — see task/claude_runner/bug/299_runner_retry_params_dead_configuration.md
      eprintln!(
        "Error: --max-sessions {count}/{max} active; gave up after {max_attempts} attempts."
      );
      std::process::exit( 1 );
    }
    if verbosity.shows_warnings()
    {
      eprintln!(
        "Info: {count}/{max} sessions active; waiting 30s for a slot... (attempt {attempt}/{max_attempts})"
      );
    }
    let _ = std::fs::write(
      &state_path,
      format!( r#"{{"cwd":"{cwd}","since":{since},"attempt":{attempt},"message":"waiting for session slot"}}"# ),
    );
    std::thread::sleep( poll );
  }
}
