#![ allow( clippy::doc_markdown ) ]
//! B25: `CLAUDE_CODE_AUTO_COMPACT_WINDOW` environment variable sets the effective
//! context window in tokens for auto-compaction calculations.
//!
//! Introduced in Claude Code v2.1.75 (2026-03-13) alongside 1M context window support
//! for Opus 4.6. The value is capped at the model's actual context window.
//!
//! Validates that the binary accepts the env var without error: exits successfully AND
//! does not emit a rejection message referencing `CLAUDE_CODE_AUTO_COMPACT_WINDOW` by name.
//!
//! Invalidation: binary exits non-zero with the env var set, OR prints a rejection
//! referencing `CLAUDE_CODE_AUTO_COMPACT_WINDOW` on stderr.

use crate::skip_if_version_before;

/// B25: binary exits successfully and does not reject `CLAUDE_CODE_AUTO_COMPACT_WINDOW`.
///
/// Two assertions provide invalidation power:
/// 1. Exit code is 0 — binary does not crash or abort when env var is set.
/// 2. Stderr does not reference the env var by name — binary does not print
///    an "unknown env var" or "deprecated" warning.
///
/// If Claude Code removed support for the env var and added explicit rejection,
/// either assertion would fire.
#[ test ]
fn b25_auto_compact_window_env_var_recognized()
{
  // B25 requires claude >= v2.1.75; skip gracefully on older binaries.
  skip_if_version_before!( "2.1.75" );

  let Some( claude ) = super::find_claude_binary() else
  {
    eprintln!( "skip: `claude` binary not found on PATH" );
    return;
  };

  // Run with the standard-model default (200 000 tokens) as a representative value.
  let env_out = std::process::Command::new( &claude )
    .arg( "--version" )
    .env( "CLAUDE_CODE_AUTO_COMPACT_WINDOW", "200000" )
    .output()
    .expect( "run claude --version with CLAUDE_CODE_AUTO_COMPACT_WINDOW set" );

  assert!(
    env_out.status.success(),
    "B25 violated: `claude --version` exited non-zero with CLAUDE_CODE_AUTO_COMPACT_WINDOW set.\n\
     Exit code: {:?}\nStderr:\n{}",
    env_out.status.code(),
    super::stderr( &env_out ),
  );

  let err = super::stderr( &env_out );
  assert!(
    !err.contains( "CLAUDE_CODE_AUTO_COMPACT_WINDOW" ),
    "B25 violated: binary explicitly rejected CLAUDE_CODE_AUTO_COMPACT_WINDOW env var.\n\
     Stderr:\n{err}"
  );
}
