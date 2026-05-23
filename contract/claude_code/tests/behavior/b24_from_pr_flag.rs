#![ allow( clippy::doc_markdown ) ]
//! B24: `--from-pr [value]` resumes a session previously linked to a GitHub pull request.
//!
//! With no argument, opens an interactive picker. With a PR number or URL, resumes the
//! session associated with that PR directly.
//!
//! Validates via `claude --help` that `--from-pr` is documented.

/// B24: `claude --help` lists `--from-pr` as a PR-session resume flag.
///
/// If Claude Code removed the `--from-pr` flag, PR-linked session continuity
/// (code review workflows spanning multiple sessions) would silently break.
#[ test ]
fn b24_from_pr_flag_documented_in_help()
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
    help.contains( "--from-pr" ),
    "B24 violated: `claude --help` does not mention --from-pr flag.\n\
     Claude Code may have removed the PR-linked session resume flag.\nHelp output:\n{help}"
  );
}
