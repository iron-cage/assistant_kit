#![ allow( clippy::doc_markdown ) ]
//! B23: `CLAUDE_CODE_SESSION_DIR` env var overrides the directory where session `.jsonl`
//! files are stored for the current invocation.
//!
//! The env var is not listed in `--help` (Claude Code does not document env vars there).
//! When set, Claude reads and writes session files from this path instead of the default
//! `~/.claude/projects/{encoded-path}/` directory.
//!
//! Validates that the binary does not explicitly reject the env var on startup.
//! Invalidation: binary prints a rejection referencing `CLAUDE_CODE_SESSION_DIR` by name.

/// B23: binary does not explicitly reject `CLAUDE_CODE_SESSION_DIR` env var.
///
/// The prior version of b11 taught us that `--version` exit code is trivially 0 regardless
/// of env vars. This test asserts on stderr content only: if the binary starts emitting
/// an explicit rejection for `CLAUDE_CODE_SESSION_DIR`, this test goes RED.
#[ test ]
fn b23_session_dir_env_var_not_rejected()
{
  let Some( claude ) = super::find_claude_binary() else
  {
    eprintln!( "skip: `claude` binary not found on PATH" );
    return;
  };

  // Run with env var set to a valid-looking path (does not need to exist for --version).
  let env_out = std::process::Command::new( &claude )
    .arg( "--version" )
    .env( "CLAUDE_CODE_SESSION_DIR", "/tmp/test-session-dir" )
    .output()
    .expect( "run claude --version with CLAUDE_CODE_SESSION_DIR set" );

  let err = super::stderr( &env_out );

  assert!(
    !err.contains( "CLAUDE_CODE_SESSION_DIR" ),
    "B23 violated: binary explicitly rejected CLAUDE_CODE_SESSION_DIR env var.\n\
     Stderr:\n{err}"
  );
}
