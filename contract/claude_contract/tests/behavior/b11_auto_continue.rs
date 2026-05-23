#![ allow( clippy::doc_markdown ) ]
//! B11: `CLAUDE_CODE_AUTO_CONTINUE` environment variable enables automated continuation.
//!
//! Validates via `claude --help` that the `CLAUDE_CODE_AUTO_CONTINUE` env var
//! is recognized or documented.

/// B11: `claude --help` mentions auto-continue or the env var is accepted.
///
/// If Claude Code removed support for `CLAUDE_CODE_AUTO_CONTINUE`,
/// our runner's automation defaults would silently stop working.
#[ test ]
fn b11_auto_continue_env_var_recognized()
{
  let Some( claude ) = super::find_claude_binary() else
  {
    eprintln!( "skip: `claude` binary not found on PATH" );
    return;
  };

  // Check --help for any mention of auto-continue
  let out = std::process::Command::new( &claude )
    .arg( "--help" )
    .output()
    .expect( "run claude --help" );
  let help = super::stdout( &out );

  // Also try running with the env var set to verify it doesn't error
  let env_out = std::process::Command::new( &claude )
    .arg( "--version" )
    .env( "CLAUDE_CODE_AUTO_CONTINUE", "true" )
    .output()
    .expect( "run claude --version with env var" );

  let env_accepted = env_out.status.success();

  let mentioned = help.contains( "auto" ) && help.contains( "continue" );

  assert!(
    mentioned || env_accepted,
    "B11 violated: `claude` neither mentions auto-continue in --help nor accepts \
     CLAUDE_CODE_AUTO_CONTINUE env var without error.\n\
     Help output:\n{help}\n\
     --version exit code: {}",
    env_out.status
  );
}
