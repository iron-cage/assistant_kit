#![ allow( clippy::doc_markdown ) ]
//! B11: `CLAUDE_CODE_AUTO_CONTINUE` environment variable enables automated continuation.
//!
//! The env var is not listed in `--help` (Claude Code does not document env vars there),
//! but is injected by `claude_runner_core` before invoking the binary.
//!
//! Validates that the binary does not explicitly reject the env var on startup.
//! Invalidation: binary prints a rejection referencing `CLAUDE_CODE_AUTO_CONTINUE` by name.

/// B11: binary does not explicitly reject `CLAUDE_CODE_AUTO_CONTINUE` env var.
///
/// The prior version of this test used `mentioned || env_accepted` where `env_accepted`
/// was always true (`claude --version` always exits 0 regardless of env vars), making the
/// assert trivially pass and unable to go RED. This version asserts on stderr content only.
///
/// If Claude Code added explicit rejection of removed/unknown env vars and emitted
/// `CLAUDE_CODE_AUTO_CONTINUE` in an error message, this test would go RED.
#[ test ]
fn b11_auto_continue_env_var_recognized()
{
  let Some( claude ) = super::find_claude_binary() else
  {
    eprintln!( "skip: `claude` binary not found on PATH" );
    return;
  };

  // Run with env var set; check that binary does not explicitly reject it by name.
  let env_out = std::process::Command::new( &claude )
    .arg( "--version" )
    .env( "CLAUDE_CODE_AUTO_CONTINUE", "true" )
    .output()
    .expect( "run claude --version with CLAUDE_CODE_AUTO_CONTINUE set" );

  let err = super::stderr( &env_out );

  assert!(
    !err.contains( "CLAUDE_CODE_AUTO_CONTINUE" ),
    "B11 violated: binary explicitly rejected CLAUDE_CODE_AUTO_CONTINUE env var.\n\
     Stderr:\n{err}"
  );
}
