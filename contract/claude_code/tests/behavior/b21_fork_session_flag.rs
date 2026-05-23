#![ allow( clippy::doc_markdown ) ]
//! B21: `--fork-session` creates a new session UUID when resuming; branches from a prior
//! checkpoint without modifying the original session file.
//!
//! Validates via `claude --help` that `--fork-session` is documented.

/// B21: `claude --help` lists `--fork-session` as a session-branching flag.
///
/// If Claude Code removed `--fork-session`, the only way to branch from a prior
/// session would be to copy the `.jsonl` file manually, which is not supported by the runner.
#[ test ]
fn b21_fork_session_flag_documented_in_help()
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
    help.contains( "--fork-session" ),
    "B21 violated: `claude --help` does not mention --fork-session flag.\n\
     Claude Code may have removed the session-branching flag.\nHelp output:\n{help}"
  );
}
