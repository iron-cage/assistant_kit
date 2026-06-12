use crate::VerbosityLevel;
use claude_core::process::find_claude_processes;

/// Block until fewer than `max` `claude` sessions are running, or until the 100-attempt
/// limit is exhausted.  `max == 0` means unlimited — returns immediately without checking.
pub( super ) fn wait_for_session_slot( max : u32, verbosity : VerbosityLevel )
{
  if max == 0 { return; }
  let poll         = core::time::Duration::from_secs( 30 );
  let max_attempts = 100_u32;
  for attempt in 1..=max_attempts
  {
    let count = find_claude_processes().len();
    if u32::try_from( count ).unwrap_or( u32::MAX ) < max { return; }
    if attempt == max_attempts
    {
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
    std::thread::sleep( poll );
  }
}
