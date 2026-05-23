#![ allow( clippy::doc_markdown ) ]
//! B22: `--no-session-persistence` disables writing the session to disk.
//! No `.jsonl` file is created and the session cannot be resumed later.
//! Only works with `--print` mode.
//!
//! Validates via `claude --help` that `--no-session-persistence` is documented.

/// B22: `claude --help` lists `--no-session-persistence` as a disk-write suppression flag.
///
/// If Claude Code removed this flag, ephemeral invocations (e.g., CI one-shot queries)
/// would silently accumulate session files in `~/.claude/projects/`.
#[ test ]
fn b22_no_session_persistence_flag_documented_in_help()
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
    help.contains( "--no-session-persistence" ),
    "B22 violated: `claude --help` does not mention --no-session-persistence flag.\n\
     Claude Code may have removed the ephemeral-session flag.\nHelp output:\n{help}"
  );
}
