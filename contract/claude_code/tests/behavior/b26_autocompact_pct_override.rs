#![ allow( clippy::doc_markdown ) ]
//! B26: `CLAUDE_AUTOCOMPACT_PCT_OVERRIDE` environment variable overrides the
//! auto-compaction percentage threshold.
//!
//! Applied as a percentage of the effective window set by `CLAUDE_CODE_AUTO_COMPACT_WINDOW`.
//! Naming asymmetry: uses `CLAUDE_` prefix (without `_CODE_`) unlike most Claude Code env vars.
//!
//! Validates that the binary accepts the env var without error: exits successfully AND
//! does not emit a rejection message referencing `CLAUDE_AUTOCOMPACT_PCT_OVERRIDE` by name.
//!
//! Invalidation: binary exits non-zero with the env var set, OR prints a rejection
//! referencing `CLAUDE_AUTOCOMPACT_PCT_OVERRIDE` on stderr.

use crate::skip_if_version_before;

/// B26: binary exits successfully and does not reject `CLAUDE_AUTOCOMPACT_PCT_OVERRIDE`.
///
/// Two assertions provide invalidation power:
/// 1. Exit code is 0 — binary does not crash or abort when env var is set.
/// 2. Stderr does not reference the env var by name — binary does not print
///    an "unknown env var" or "deprecated" warning.
///
/// If Claude Code removed support for the env var and added explicit rejection,
/// either assertion would fire.
#[ test ]
fn b26_autocompact_pct_override_env_var_recognized()
{
  // B26 requires claude >= v2.1.75; skip gracefully on older binaries.
  skip_if_version_before!( "2.1.75" );

  let Some( claude ) = super::find_claude_binary() else
  {
    eprintln!( "skip: `claude` binary not found on PATH" );
    return;
  };

  // Run with 80 as a representative percentage (compact at 80% of the window).
  let env_out = std::process::Command::new( &claude )
    .arg( "--version" )
    .env( "CLAUDE_AUTOCOMPACT_PCT_OVERRIDE", "80" )
    .output()
    .expect( "run claude --version with CLAUDE_AUTOCOMPACT_PCT_OVERRIDE set" );

  assert!(
    env_out.status.success(),
    "B26 violated: `claude --version` exited non-zero with CLAUDE_AUTOCOMPACT_PCT_OVERRIDE set.\n\
     Exit code: {:?}\nStderr:\n{}",
    env_out.status.code(),
    super::stderr( &env_out ),
  );

  let err = super::stderr( &env_out );
  assert!(
    !err.contains( "CLAUDE_AUTOCOMPACT_PCT_OVERRIDE" ),
    "B26 violated: binary explicitly rejected CLAUDE_AUTOCOMPACT_PCT_OVERRIDE env var.\n\
     Stderr:\n{err}"
  );
}
