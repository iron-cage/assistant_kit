#![ allow( clippy::doc_markdown ) ]
//! B19: `--resume` / `-r` resumes a specific prior session by UUID.
//!
//! Validates via `claude --help` that `--resume` / `-r` is documented.
//! The flag appends to the specified session's `.jsonl` file rather than
//! the most recently modified one.

/// B19: `claude --help` lists `--resume` / `-r` as a resume-by-UUID flag.
///
/// If Claude Code removed the `--resume` flag, our runner's resume-by-UUID
/// feature would silently fall back to default continuation.
#[ test ]
fn b19_resume_flag_documented_in_help()
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
    help.contains( "--resume" ) || help.contains( "-r" ),
    "B19 violated: `claude --help` does not mention --resume / -r flag.\n\
     Claude Code may have removed the resume-by-UUID flag.\nHelp output:\n{help}"
  );
}
