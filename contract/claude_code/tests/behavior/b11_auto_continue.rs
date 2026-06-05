#![ allow( clippy::doc_markdown ) ]
//! B11: `CLAUDE_CODE_AUTO_CONTINUE` environment variable enables automated continuation.
//!
//! The env var is not listed in `--help` (Claude Code does not document env vars there),
//! but is injected by `claude_runner_core` before invoking the binary.
//!
//! Validates that the binary accepts the env var without error: exits successfully AND
//! does not emit a rejection message referencing `CLAUDE_CODE_AUTO_CONTINUE` by name.
//!
//! Invalidation: binary exits non-zero with the env var set, OR prints a rejection
//! referencing `CLAUDE_CODE_AUTO_CONTINUE` on stderr.

/// B11: binary exits successfully and does not reject `CLAUDE_CODE_AUTO_CONTINUE`.
///
/// Two assertions provide invalidation power:
/// 1. Exit code is 0 — binary does not crash or abort when env var is set.
/// 2. Stderr does not reference the env var by name — binary does not print
///    an "unknown env var" or "deprecated" warning.
///
/// If Claude Code removed support for the env var and added explicit rejection,
/// either assertion would fire.
#[ test ]
fn b11_auto_continue_env_var_recognized()
{
  let Some( claude ) = super::find_claude_binary() else
  {
    eprintln!( "skip: `claude` binary not found on PATH" );
    return;
  };

  // Run with env var set; binary must exit 0 and not reject on stderr.
  let env_out = std::process::Command::new( &claude )
    .arg( "--version" )
    .env( "CLAUDE_CODE_AUTO_CONTINUE", "true" )
    .output()
    .expect( "run claude --version with CLAUDE_CODE_AUTO_CONTINUE set" );

  assert!(
    env_out.status.success(),
    "B11 violated: `claude --version` exited non-zero with CLAUDE_CODE_AUTO_CONTINUE set.\n\
     Exit code: {:?}\nStderr:\n{}",
    env_out.status.code(),
    super::stderr( &env_out ),
  );

  let err = super::stderr( &env_out );
  assert!(
    !err.contains( "CLAUDE_CODE_AUTO_CONTINUE" ),
    "B11 violated: binary explicitly rejected CLAUDE_CODE_AUTO_CONTINUE env var.\n\
     Stderr:\n{err}"
  );
}
