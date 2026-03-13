#![ allow( clippy::doc_markdown ) ]
//! B4: `-c` / `--continue` is an explicit alias for the default continuation behavior.
//!
//! Validates via `claude --help` that `-c` / `--continue` exists and is described
//! with continuation semantics.

/// B4: `claude --help` lists `-c` / `--continue` as continuation flag.
///
/// If Claude Code removed the `-c` flag or changed its semantics,
/// this test would fail.
#[ test ]
fn b4_continue_flag_documented_in_help()
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
    help.contains( "--continue" ) || help.contains( "-c" ),
    "B4 violated: `claude --help` does not mention -c / --continue flag.\n\
     Claude Code may have removed the continue flag.\nHelp output:\n{help}"
  );
}
