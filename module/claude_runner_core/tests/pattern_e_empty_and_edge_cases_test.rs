//! Pattern E / Pattern F corner case tests — empty iterator and accumulation
//!
//! ## Purpose
//!
//! Verify the behaviour of Pattern E (one-flag + N-space-separated-values) and
//! Pattern F (N × flag+value pairs) methods when called with an empty iterator,
//! and document how multiple calls to the same method accumulate.
//!
//! ## Root Cause (Pattern E empty-iterator bug)
//!
//! `with_allowed_tools`, `with_disallowed_tools`, and `with_tools` all push their
//! flag unconditionally before iterating over values.  When the caller passes an
//! empty iterator the flag is pushed with no values following it, producing a
//! bare dangling CLI argument such as `["--allowedTools"]`.  Pattern F methods
//! (`with_betas`, `with_mcp_config`, `with_plugin_dir`, `with_file`) are
//! implemented with an item-level loop that naturally produces nothing for an
//! empty iterator — so the inconsistency is a bug in Pattern E.
//!
//! ## Why Not Caught Initially
//!
//! All existing Pattern E tests used non-empty iterators.  The zero-item case
//! was never exercised, leaving the dangling-flag path undetected.
//!
//! ## Fix Applied
//!
//! Used a `peekable()` iterator in `with_allowed_tools`, `with_disallowed_tools`,
//! and `with_tools`: the flag is only pushed when the iterator has at least one
//! element.  This makes Pattern E consistent with Pattern F for empty inputs.
//!
//! ## Prevention
//!
//! When implementing "one flag + N values" patterns, always guard the flag push
//! behind a non-empty check.  Never push a flag before confirming there is at
//! least one value to follow it.
//!
//! ## Pitfall to Avoid
//!
//! Do not blindly push the flag before the loop.  A bare flag with no values is
//! either rejected by the CLI parser or silently misinterpreted as having no
//! restriction (tool-level semantics are ambiguous for an empty allowlist).
//!
//! ## Test Coverage Matrix
//!
//! | Method | empty produces no flag | non-empty flag present | two calls accumulate |
//! |--------|------------------------|------------------------|----------------------|
//! | with_allowed_tools | ✅ | ✅ | ✅ |
//! | with_disallowed_tools | ✅ | ✅ | — |
//! | with_tools | ✅ | ✅ | — |
//! | with_betas (Pattern F) | ✅ confirmed | — | — |
//! | with_mcp_config (Pattern F) | ✅ confirmed | — | — |
//! | with_plugin_dir (Pattern F) | ✅ confirmed | — | — |
//! | with_file (Pattern F) | ✅ confirmed | — | — |
//! | with_max_budget_usd float | ✅ exact format | — | — |

use claude_runner_core::ClaudeCommand;

fn args_of( cmd : &ClaudeCommand ) -> Vec< String > {
  cmd.build_command_for_test()
    .get_args()
    .map( |a| a.to_string_lossy().to_string() )
    .collect()
}

// ── Pattern E: empty iterator must produce no flag ───────────────────────────

/// Reproduces Pattern E empty-iterator bug: `with_allowed_tools([])` used to push
/// a bare `--allowedTools` flag with no values.
///
/// ## Root Cause
///
/// The flag was pushed unconditionally before iterating, so an empty iterator left
/// a dangling `["--allowedTools"]` in the args Vec.
///
/// ## Why Not Caught Initially
///
/// All pre-existing `with_allowed_tools` tests used non-empty slices.  The zero-item
/// path was never exercised, leaving the dangling-flag behaviour undetected.
///
/// ## Fix Applied
///
/// Changed the implementation to use a `peekable()` iterator; the flag is pushed
/// only when `iter.peek().is_some()`.
///
/// ## Prevention
///
/// When implementing one-flag + N-values patterns, always guard the flag push with a
/// non-empty check before the loop.
///
/// ## Pitfall to Avoid
///
/// Do not push the flag before the loop.  A bare flag is rejected or misinterpreted
/// by the claude CLI parser.
// test_kind: bug_reproducer(issue-pattern-e-empty-iterator)
#[test]
fn with_allowed_tools_empty_produces_no_flag()
{
  let cmd = ClaudeCommand::new().with_allowed_tools( [] as [&str; 0] );
  let args = args_of( &cmd );
  assert!(
    !args.contains( &"--allowedTools".to_string() ),
    "with_allowed_tools([]) must not push a bare --allowedTools flag: {args:?}"
  );
}

/// Regression: `with_disallowed_tools([])` must not push a bare `--disallowedTools`
/// flag (same Pattern E empty-iterator bug).
// test_kind: bug_reproducer(issue-pattern-e-empty-iterator)
#[test]
fn with_disallowed_tools_empty_produces_no_flag()
{
  let cmd = ClaudeCommand::new().with_disallowed_tools( [] as [&str; 0] );
  let args = args_of( &cmd );
  assert!(
    !args.contains( &"--disallowedTools".to_string() ),
    "with_disallowed_tools([]) must not push a bare --disallowedTools flag: {args:?}"
  );
}

/// Regression: `with_tools([])` must not push a bare `--tools` flag (same Pattern E
/// empty-iterator bug).
// test_kind: bug_reproducer(issue-pattern-e-empty-iterator)
#[test]
fn with_tools_empty_produces_no_flag()
{
  let cmd = ClaudeCommand::new().with_tools( [] as [&str; 0] );
  let args = args_of( &cmd );
  assert!(
    !args.contains( &"--tools".to_string() ),
    "with_tools([]) must not push a bare --tools flag: {args:?}"
  );
}

// ── Verify non-empty still works after fix ───────────────────────────────────

#[test]
fn with_allowed_tools_nonempty_still_adds_flag()
{
  let cmd = ClaudeCommand::new().with_allowed_tools( ["bash"] );
  let args = args_of( &cmd );
  assert!( args.contains( &"--allowedTools".to_string() ), "non-empty allowed_tools must still add flag: {args:?}" );
  assert!( args.contains( &"bash".to_string() ) );
}

#[test]
fn with_disallowed_tools_nonempty_still_adds_flag()
{
  let cmd = ClaudeCommand::new().with_disallowed_tools( ["write"] );
  let args = args_of( &cmd );
  assert!( args.contains( &"--disallowedTools".to_string() ) );
  assert!( args.contains( &"write".to_string() ) );
}

#[test]
fn with_tools_nonempty_still_adds_flag()
{
  let cmd = ClaudeCommand::new().with_tools( ["bash"] );
  let args = args_of( &cmd );
  assert!( args.contains( &"--tools".to_string() ) );
  assert!( args.contains( &"bash".to_string() ) );
}

// ── Pattern E: two calls accumulate correctly ────────────────────────────────

#[test]
fn with_allowed_tools_called_twice_accumulates_two_entries()
{
  // Both calls push flag+values; result has two --allowedTools groups.
  let cmd = ClaudeCommand::new()
    .with_allowed_tools( ["read"] )
    .with_allowed_tools( ["write"] );
  let args = args_of( &cmd );
  let flag_count = args.iter().filter( |a| *a == "--allowedTools" ).count();
  assert_eq!(
    flag_count, 2,
    "two with_allowed_tools calls accumulate two --allowedTools flags: {args:?}"
  );
  assert!( args.contains( &"read".to_string() ) );
  assert!( args.contains( &"write".to_string() ) );
}

// ── Pattern F: empty iterator confirms no flags (already correct, regression guard) ─

#[test]
fn with_betas_empty_produces_no_flag()
{
  let baseline = args_of( &ClaudeCommand::new() ).len();
  let after = args_of( &ClaudeCommand::new().with_betas( [] as [&str; 0] ) ).len();
  assert_eq!( baseline, after, "with_betas([]) must produce no extra args (Pattern F): baseline={baseline} after={after}" );
}

#[test]
fn with_mcp_config_empty_produces_no_flag()
{
  let baseline = args_of( &ClaudeCommand::new() ).len();
  let after = args_of( &ClaudeCommand::new().with_mcp_config( [] as [&str; 0] ) ).len();
  assert_eq!( baseline, after, "with_mcp_config([]) must produce no extra args (Pattern F): baseline={baseline} after={after}" );
}

#[test]
fn with_plugin_dir_empty_produces_no_flag()
{
  let baseline = args_of( &ClaudeCommand::new() ).len();
  let after = args_of( &ClaudeCommand::new().with_plugin_dir( [] as [&str; 0] ) ).len();
  assert_eq!( baseline, after, "with_plugin_dir([]) must produce no extra args (Pattern F): baseline={baseline} after={after}" );
}

#[test]
fn with_file_empty_produces_no_flag()
{
  let baseline = args_of( &ClaudeCommand::new() ).len();
  let after = args_of( &ClaudeCommand::new().with_file( [] as [&str; 0] ) ).len();
  assert_eq!( baseline, after, "with_file([]) must produce no extra args (Pattern F): baseline={baseline} after={after}" );
}

// ── with_max_budget_usd float formatting edge cases ──────────────────────────

#[test]
fn with_max_budget_usd_integer_value_formats_without_decimal_point()
{
  // Rust f64 Display: `1.0` → "1", `100.0` → "100" — no trailing ".0"
  // This is correct for the Claude CLI which accepts plain numbers.
  let cmd = ClaudeCommand::new().with_max_budget_usd( 1.0 );
  let args = args_of( &cmd );
  let pos = args.iter().position( |a| a == "--max-budget-usd" ).expect( "--max-budget-usd not found" );
  let value = &args[ pos + 1 ];
  assert_eq!( value, "1", "1.0 must format as '1' (Rust f64 Display), got: {value}" );
}

#[test]
fn with_max_budget_usd_large_integer_formats_cleanly()
{
  let cmd = ClaudeCommand::new().with_max_budget_usd( 100.0 );
  let args = args_of( &cmd );
  let pos = args.iter().position( |a| a == "--max-budget-usd" ).expect( "--max-budget-usd not found" );
  let value = &args[ pos + 1 ];
  assert_eq!( value, "100", "100.0 must format as '100', got: {value}" );
}

#[test]
fn with_max_budget_usd_fractional_formats_cleanly()
{
  let cmd = ClaudeCommand::new().with_max_budget_usd( 0.25 );
  let args = args_of( &cmd );
  let pos = args.iter().position( |a| a == "--max-budget-usd" ).expect( "--max-budget-usd not found" );
  let value = &args[ pos + 1 ];
  assert_eq!( value, "0.25", "0.25 must format as '0.25', got: {value}" );
}

// ── describe_compact returns single line even with all flags ─────────────────

#[test]
fn describe_compact_with_many_flags_still_single_line()
{
  let cmd = ClaudeCommand::new()
    .with_skip_permissions( true )
    .with_verbose( true )
    .with_print( true )
    .with_message( "test" )
    .with_dry_run( true );
  let compact = cmd.describe_compact();
  assert_eq!( compact.lines().count(), 1, "describe_compact must always return a single line: {compact:?}" );
  assert!( compact.starts_with( "claude" ), "describe_compact must start with 'claude'" );
}

// ── Pattern E empty then non-empty in sequence ───────────────────────────────

#[test]
fn with_allowed_tools_empty_then_nonempty_has_exactly_one_flag()
{
  // Empty call: no flag pushed. Non-empty call: one flag+value group.
  let cmd = ClaudeCommand::new()
    .with_allowed_tools( [] as [&str; 0] )
    .with_allowed_tools( ["bash"] );
  let args = args_of( &cmd );
  let flag_count = args.iter().filter( |a| *a == "--allowedTools" ).count();
  assert_eq!(
    flag_count, 1,
    "empty call produces no flag; non-empty call produces exactly one --allowedTools: {args:?}"
  );
  assert!( args.contains( &"bash".to_string() ) );
}
