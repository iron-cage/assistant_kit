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

/// Block until fewer than `max` `claude` sessions are running, or until the 100-attempt
/// limit is exhausted.  `max == 0` means unlimited — returns immediately without checking.
///
/// While waiting, writes a JSON state file to `$CLR_GATE_DIR/{pid}.json` so that
/// `clr ps` can display this process in its "Queued CLR Processes" table.  The file
/// is updated each polling iteration and deleted on both exit paths.
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

  for attempt in 1..=max_attempts
  {
    let count = find_claude_processes().len();
    if u32::try_from( count ).unwrap_or( u32::MAX ) < max
    {
      let _ = std::fs::remove_file( &state_path );
      return;
    }
    if attempt == max_attempts
    {
      let _ = std::fs::remove_file( &state_path );
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
