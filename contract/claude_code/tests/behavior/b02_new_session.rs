#![ allow( clippy::doc_markdown ) ]
//! B2: each `claude` invocation creates a separate `.jsonl` file; does not append to existing.
//!
//! The binary defaults to a new session on every invocation; `--continue` is the explicit
//! opt-in to resume. The presence of `--continue` in `claude --help` causally proves that
//! sessions are separate by default — if Claude switched to a single-file model, `--continue`
//! would be meaningless and removed.
//!
//! Invalidation: `--continue` removed from help (sessions are no longer separate) OR
//! no project in `~/.claude/projects/` has 2+ non-agent `.jsonl` files.

/// B2: `--continue` flag exists in binary help, proving default is new-session.
///
/// `--continue` is the opt-in for resuming — its existence proves sessions are
/// separate by default. If Claude Code merged all sessions into one file,
/// there'd be nothing to "continue" and the flag would be removed.
#[ test ]
fn b2_continue_flag_proves_separate_sessions()
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
    help.contains( "--continue" ),
    "B2 violated: --continue not found in `claude --help`.\n\
     If the flag is removed, sessions may no longer be separate by default."
  );
}

/// B2: at least one project has multiple session files (evidence of separate creation).
///
/// Secondary evidence: if Claude Code switched to a single-file-per-project model,
/// no project would have more than one `.jsonl` and this test would fail.
#[ test ]
fn b2_multiple_session_files_exist_in_real_project()
{
  let projects = super::find_projects();
  if projects.is_empty()
  {
    eprintln!( "skip: no ~/.claude/projects/ found" );
    return;
  }

  let has_multiple = projects.iter().any( | p | super::find_sessions( p ).len() >= 2 );
  assert!(
    has_multiple,
    "B2 violated: no project in ~/.claude/projects/ has 2+ non-empty non-agent .jsonl files.\n\
     Claude Code may no longer create separate session files per invocation."
  );
}
