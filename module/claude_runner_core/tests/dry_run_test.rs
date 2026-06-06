//! Dry-run mode and `describe_compact` tests
//!
//! ## Purpose
//!
//! Verify `with_dry_run()` short-circuits execution and `describe_compact()` returns
//! only the invocation line (without any leading `cd /dir` line).
//!
//! ## Root Cause (design invariant)
//!
//! `describe()` returns two lines when `working_directory` is set:
//! `"cd /dir\nenv -u CLAUDECODE claude ..."`. `describe_compact()` MUST extract only the last line
//! via `self.describe().lines().last()` to avoid the double-cd pitfall.
//!
//! ## Evidence
//!
//! - `describe_compact()` returns `"env -u CLAUDECODE claude ..."` (single line, no cd prefix; BUG-246 fix)
//! - `describe_compact()` with `working_dir` set still returns only the invocation line
//! - `execute()` with `dry_run=true` returns `describe_compact()` as stdout without spawning
//! - `execute()` with `dry_run=true` returns `exit_code` 0
//! - `execute()` with `dry_run=true` returns empty stderr
//! - `execute_interactive()` with `dry_run=true` returns `ExitStatus` with code 0
//! - `with_dry_run(false)` (default) does not affect command args
//!
//! ## Test Coverage Matrix
//!
//! | Scenario | describe_compact | execute dry | execute_interactive dry |
//! |----------|-----------------|-------------|------------------------|
//! | no working dir | ✅ | ✅ | ✅ |
//! | with working dir | ✅ | — | — |
//! | dry_run=false | — | — | — |
//! | CLAUDECODE env removal (BUG-246) | ✅ | — | — |

use claude_runner_core::ClaudeCommand;

// describe_compact tests

#[test]
fn describe_compact_returns_single_line() {
  let cmd = ClaudeCommand::new()
    .with_message( "hello" );
  let compact = cmd.describe_compact();
  assert_eq!( compact.lines().count(), 1, "describe_compact must return exactly one line" );
}

// Fix(BUG-246): describe_compact() now starts with "env -u CLAUDECODE" (the default).
// Root cause: describe() was WYSIWYG-broken — ClaudeCommand::new() unsets CLAUDECODE by default
//   via env_remove(), but describe() showed "claude ..." hiding the env manipulation.
// Pitfall: describe() and build_command() must stay in sync; any env_remove() in build_command()
//   must appear explicitly in describe() output so trace/dry-run is WYSIWYG.
#[test]
fn describe_compact_starts_with_env_unset_claudecode() {
  let cmd = ClaudeCommand::new();
  let compact = cmd.describe_compact();
  assert!(
    compact.starts_with( "env -u CLAUDECODE" ),
    "describe_compact must start with 'env -u CLAUDECODE' (default: unset_claudecode=true), got: {compact}"
  );
}

#[test]
fn describe_compact_excludes_cd_prefix_when_working_dir_set() {
  // Fix(issue-describe-compact-double-cd): describe_compact must NOT include cd line
  // Root cause: describe() returns "cd /dir\nclaude ..." when working_directory is set
  // Pitfall: Callers who rebuild from parts will get double-cd; always use describe().lines().last()
  let cmd = ClaudeCommand::new()
    .with_working_directory( "/tmp/work" )
    .with_message( "hello" );
  let compact = cmd.describe_compact();
  assert!( !compact.contains( "cd " ), "describe_compact must not contain 'cd', got: {compact}" );
  assert!(
    compact.starts_with( "env -u CLAUDECODE" ),
    "describe_compact must start with 'env -u CLAUDECODE', got: {compact}"
  );
}

#[test]
fn describe_compact_includes_flags_set_on_command() {
  let cmd = ClaudeCommand::new()
    .with_skip_permissions( true );
  let compact = cmd.describe_compact();
  assert!( compact.contains( "--dangerously-skip-permissions" ), "compact must contain skip-permissions flag" );
}

// dry_run execute() tests

#[test]
fn execute_dry_run_returns_describe_compact_as_stdout() {
  let cmd = ClaudeCommand::new()
    .with_message( "hello" )
    .with_dry_run( true );
  let expected_compact = ClaudeCommand::new()
    .with_message( "hello" )
    .describe_compact();
  let output = cmd.execute().expect( "dry_run execute must not fail" );
  assert_eq!( output.stdout, expected_compact, "dry_run stdout must equal describe_compact()" );
}

#[test]
fn execute_dry_run_returns_exit_code_zero() {
  let output = ClaudeCommand::new()
    .with_dry_run( true )
    .execute()
    .expect( "dry_run execute must not fail" );
  assert_eq!( output.exit_code, 0 );
}

#[test]
fn execute_dry_run_returns_empty_stderr() {
  let output = ClaudeCommand::new()
    .with_dry_run( true )
    .execute()
    .expect( "dry_run execute must not fail" );
  assert!( output.stderr.is_empty(), "dry_run stderr must be empty" );
}

#[test]
fn execute_dry_run_with_working_dir_compact_has_no_cd() {
  let output = ClaudeCommand::new()
    .with_working_directory( "/tmp/work" )
    .with_dry_run( true )
    .execute()
    .expect( "dry_run execute must not fail" );
  assert!( !output.stdout.contains( "cd " ), "dry_run stdout must not contain 'cd'" );
  assert!(
    output.stdout.starts_with( "env -u CLAUDECODE" ),
    "dry_run stdout must start with 'env -u CLAUDECODE', got: {}", output.stdout
  );
}

// dry_run execute_interactive() tests

#[test]
fn execute_interactive_dry_run_returns_success_exit_status() {
  let status = ClaudeCommand::new()
    .with_dry_run( true )
    .execute_interactive()
    .expect( "dry_run execute_interactive must not fail" );
  assert!( status.success(), "dry_run execute_interactive must return success status" );
}

// with_dry_run builder tests

#[test]
fn with_dry_run_false_does_not_add_flag() {
  let cmd = ClaudeCommand::new()
    .with_dry_run( false );
  let desc = cmd.describe();
  assert!( !desc.contains( "dry" ), "with_dry_run(false) must not add any dry flag" );
}

#[test]
fn with_dry_run_true_does_not_add_flag_to_args() {
  // dry_run is a Rust-only behavior control — never passed to the CLI
  let cmd = ClaudeCommand::new()
    .with_dry_run( true );
  let desc = cmd.describe();
  assert!( !desc.contains( "dry" ), "dry_run is not a CLI flag — must not appear in describe()" );
}
