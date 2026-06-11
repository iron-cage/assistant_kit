use crate::VerbosityLevel;

/// Count running `claude` binary processes by scanning `/proc/*/cmdline`.
///
/// Reads the NUL-delimited argv from each numeric `/proc/<pid>/cmdline` entry and
/// checks whether the first argument's file-name component equals `"claude"`.
/// Returns 0 on any I/O failure so the gate degrades gracefully.
fn count_claude_sessions() -> usize
{
  let Ok( entries ) = std::fs::read_dir( "/proc" ) else { return 0; };
  entries
    .flatten()
    .filter( | e | e.file_name().to_string_lossy().chars().all( | c | c.is_ascii_digit() ) )
    .filter( | e |
    {
      let cmdline_path = e.path().join( "cmdline" );
      std::fs::read( cmdline_path )
        .is_ok_and( | bytes |
        {
          // cmdline is NUL-separated argv; first arg is the binary path.
          let first     = bytes.split( | &c | c == 0 ).next().unwrap_or_default();
          let first_str = String::from_utf8_lossy( first );
          std::path::Path::new( first_str.as_ref() )
            .file_name()
            .is_some_and( | n | n == "claude" )
        } )
    } )
    .count()
}

/// Block until fewer than `max` `claude` sessions are running, or until the 50-attempt
/// limit is exhausted.  `max == 0` means unlimited — returns immediately without checking.
pub( super ) fn wait_for_session_slot( max : u32, verbosity : VerbosityLevel )
{
  if max == 0 { return; }
  let poll         = core::time::Duration::from_secs( 30 );
  let max_attempts = 50_u32;
  for attempt in 1..=max_attempts
  {
    let count = count_claude_sessions();
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
