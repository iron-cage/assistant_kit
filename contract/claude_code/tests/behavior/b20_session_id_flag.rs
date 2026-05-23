#![ allow( clippy::doc_markdown ) ]
//! B20: `--session-id <uuid>` assigns a deterministic UUID to the current session
//! instead of auto-generating one.
//!
//! Validates via `claude --help` that `--session-id` is documented.

/// B20: `claude --help` lists `--session-id` as a session-UUID override flag.
///
/// If Claude Code removed the `--session-id` flag, deterministic session tracking
/// (e.g., linking invocations to external systems by UUID) would break silently.
#[ test ]
fn b20_session_id_flag_documented_in_help()
{
  let Some( claude ) = super::find_claude_binary() else
  {
    eprintln!( "skip: `claude` binary not found on PATH" );
    return;
  };

  let out = std::process::Command::new( &claude )
    .arg( "--help" )
    .output()
    .expect( "run claude --help" );

  let help = super::stdout( &out );
  assert!(
    help.contains( "--session-id" ),
    "B20 violated: `claude --help` does not mention --session-id flag.\n\
     Claude Code may have removed the deterministic session UUID flag.\nHelp output:\n{help}"
  );
}
