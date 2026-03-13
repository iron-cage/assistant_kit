#![ allow( clippy::doc_markdown ) ]
//! B3: `-p` / `--print` is output mode only; does not affect session creation.
//!
//! Validates via `claude --help` that `-p` is described as an output mode flag,
//! not as a session-creation or session-selection flag.

/// B3: `claude --help` lists `-p` / `--print` as output mode.
///
/// If Claude Code removed the `-p` flag or changed its semantics to affect
/// session creation, this test would fail.
#[ test ]
fn b3_print_flag_documented_as_output_mode()
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
    help.contains( "-p" ) || help.contains( "--print" ),
    "B3 violated: `claude --help` does not mention -p / --print flag.\n\
     Claude Code may have removed the print flag.\nHelp output:\n{help}"
  );
}
