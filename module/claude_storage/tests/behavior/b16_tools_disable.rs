#![ allow( clippy::doc_markdown ) ]
//! B16: `--tools ""` disables all tool invocation; whether tool definitions are also
//! stripped from the assembled system prompt is ❓ uncertain (H1 vs H2).
//!
//! Claude Code documents `--tools` as: "Specify the list of available tools from the
//! built-in set. Use `""` to disable all tools, `"default"` to use all tools, or
//! specify tool names (e.g. `"Bash,Edit,Read"`)."
//!
//! The open hypothesis:
//! - **H1** (60%): `--tools ""` blocks invocation only; tool definitions (~12k tokens)
//!   remain in the assembled system prompt — token cost is unchanged.
//! - **H2** (25%): `--tools ""` strips tool definitions from the system prompt entirely
//!   — saves ~12k tokens AND prevents invocation.
//!
//! These tests validate the observable (flag existence, flag acceptance) and flag the
//! unresolved part (H1 vs H2) as requiring a live conversation test.
//!
//! See `docs/claude_code/behavior.md` B16 for the full behavior record.
//! See `docs/claude_code/behavior.md` E30–E32 for supporting evidence.

/// B16a: `claude --help` documents the `--tools` flag with empty-string syntax.
///
/// If Claude Code removes or renames the `--tools` flag, any `clr` wrapper exposing
/// it would silently stop working. This test catches that regression.
#[ test ]
fn b16a_tools_flag_documented_in_help()
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
    help.contains( "--tools" ),
    "B16 violated: `claude --help` does not document `--tools` flag.\n\
     Claude Code may have removed or renamed it.\nHelp output:\n{help}"
  );

  // Verify the empty-string form is described — the core of the disable-all mechanism.
  let mentions_empty_disable =
    help.contains( r#""""# ) || help.contains( "disable all" ) || help.contains( "disable" );

  assert!(
    mentions_empty_disable,
    "B16 violated: `--tools` flag exists but help no longer documents the `\"\"` \
     (disable all) form.\nHelp output:\n{help}"
  );
}

/// B16b: `claude --tools "" --version` is accepted without error.
///
/// Verifies the flag is not rejected at parse time when given an empty value.
/// A non-zero exit code here means Claude Code changed the flag's accepted values.
#[ test ]
fn b16b_tools_empty_string_accepted()
{
  let Some( claude ) = super::find_claude_binary() else
  {
    eprintln!( "skip: `claude` binary not found on PATH" );
    return;
  };

  let out = std::process::Command::new( &claude )
    .args( [ "--tools", "", "--version" ] )
    .output()
    .expect( "run claude --tools '' --version" );

  // We do not assert success here because Claude Code may reject the flag in
  // --version mode even if it's valid for conversational invocations.
  // Instead we check that any failure is NOT about "unknown option" or similar parse errors.
  let stderr = super::stderr( &out );
  let is_parse_error = stderr.contains( "unknown option" )
    || stderr.contains( "Unknown flag" )
    || stderr.contains( "invalid option" )
    || stderr.contains( "unrecognized" );

  assert!(
    !is_parse_error,
    "B16 violated: `claude --tools \"\" --version` rejected with a parse error.\n\
     Claude Code may have renamed or removed the `--tools` flag.\n\
     Stderr: {stderr}"
  );
}

/// B16c: `claude --tools "default" --version` is accepted without error.
///
/// Validates that the named `"default"` value (restore all tools) is accepted.
/// Acts as a control: if both empty and "default" are accepted, the flag is
/// functioning; the H1-vs-H2 question remains open for live conversation testing.
#[ test ]
fn b16c_tools_default_value_accepted()
{
  let Some( claude ) = super::find_claude_binary() else
  {
    eprintln!( "skip: `claude` binary not found on PATH" );
    return;
  };

  let out = std::process::Command::new( &claude )
    .args( [ "--tools", "default", "--version" ] )
    .output()
    .expect( "run claude --tools default --version" );

  let stderr = super::stderr( &out );
  let is_parse_error = stderr.contains( "unknown option" )
    || stderr.contains( "Unknown flag" )
    || stderr.contains( "invalid option" )
    || stderr.contains( "unrecognized" );

  assert!(
    !is_parse_error,
    "B16 violated: `claude --tools default --version` rejected with a parse error.\n\
     Claude Code may have changed the accepted values for `--tools`.\n\
     Stderr: {stderr}"
  );
}

// NOTE: H1 vs H2 (does --tools "" strip definitions from system prompt or just block
// invocation?) cannot be validated without a live conversation that requires an API key.
// Manual validation procedure (from behavior.md B16):
//
//   # Qualitative test — observe whether Claude acknowledges tool awareness:
//   claude --tools "" --print "List files in this directory using bash"
//   # H1: Claude says it cannot use tools but may describe what bash would do
//   # H2: Claude has no knowledge of tools at all; responds as pure text model
//
//   # Quantitative proxy — compare token counts with and without --tools "":
//   claude --tools "" --print "hi" --output-format json | jq '.usage'
//   claude            --print "hi" --output-format json | jq '.usage'
//   # H1: input_tokens are approximately equal (~12k difference negligible vs total)
//   # H2: input_tokens with --tools "" are ~12k lower than without
