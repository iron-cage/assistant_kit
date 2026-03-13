//! Tool directory parameter builder method tests (TSK-073)
//!
//! ## Purpose
//!
//! Verify the four tool directory `with_*()` methods add the correct CLI flags.
//!
//! ## Evidence
//!
//! - `with_add_dir("/p")` adds `--add-dir /p` (Pattern F: repeated-flag per value)
//! - `with_add_dir` called twice produces two `--add-dir` pairs
//! - `with_allowed_tools(["bash","read"])` adds `--allowedTools bash read` (Pattern E: space-sep)
//! - `with_disallowed_tools(["write"])` adds `--disallowedTools write` (Pattern E)
//! - `with_tools(["bash"])` adds `--tools bash` (Pattern E)
//!
//! ## Test Coverage Matrix
//!
//! | Method | flag present | repeated | accumulate |
//! |--------|-------------|----------|------------|
//! | `with_add_dir` | ✅ | ✅ | ✅ |
//! | `with_allowed_tools` | ✅ | — | — |
//! | `with_disallowed_tools` | ✅ | — | — |
//! | `with_tools` | ✅ | — | — |
//!
//! Empty-iterator tests for Pattern E (`with_allowed_tools`, `with_disallowed_tools`, `with_tools`)
//! and Pattern F are covered in `pattern_e_empty_and_edge_cases_test.rs`.

use claude_runner_core::ClaudeCommand;

fn args_of( cmd: &ClaudeCommand ) -> Vec<String> {
  let c = cmd.build_command_for_test();
  c.get_args().map( |a| a.to_string_lossy().to_string() ).collect()
}

// with_add_dir (Pattern F: repeated-flag)

#[test]
fn with_add_dir_adds_flag_and_path() {
  let cmd = ClaudeCommand::new().with_add_dir( "/some/path" );
  let args = args_of( &cmd );
  assert!( args.contains( &"--add-dir".to_string() ) );
  assert!( args.contains( &"/some/path".to_string() ) );
}

#[test]
fn with_add_dir_called_twice_produces_two_pairs() {
  let cmd = ClaudeCommand::new()
    .with_add_dir( "/path/a" )
    .with_add_dir( "/path/b" );
  let args = args_of( &cmd );
  let flag_count = args.iter().filter( |a| *a == "--add-dir" ).count();
  assert_eq!( flag_count, 2, "--add-dir must appear twice for two paths: {args:?}" );
  assert!( args.contains( &"/path/a".to_string() ) );
  assert!( args.contains( &"/path/b".to_string() ) );
}

// with_allowed_tools (Pattern E: space-separated)

#[test]
fn with_allowed_tools_single_value_adds_flag() {
  let cmd = ClaudeCommand::new().with_allowed_tools( [ "bash" ] );
  let args = args_of( &cmd );
  assert!( args.contains( &"--allowedTools".to_string() ) );
  assert!( args.contains( &"bash".to_string() ) );
}

#[test]
fn with_allowed_tools_multiple_values_uses_single_flag() {
  let cmd = ClaudeCommand::new().with_allowed_tools( [ "bash", "read", "write" ] );
  let args = args_of( &cmd );
  let flag_count = args.iter().filter( |a| *a == "--allowedTools" ).count();
  assert_eq!( flag_count, 1, "--allowedTools must appear exactly once: {args:?}" );
  assert!( args.contains( &"bash".to_string() ) );
  assert!( args.contains( &"read".to_string() ) );
  assert!( args.contains( &"write".to_string() ) );
}

// with_disallowed_tools (Pattern E: space-separated)

#[test]
fn with_disallowed_tools_adds_flag() {
  let cmd = ClaudeCommand::new().with_disallowed_tools( [ "bash" ] );
  let args = args_of( &cmd );
  assert!( args.contains( &"--disallowedTools".to_string() ) );
  assert!( args.contains( &"bash".to_string() ) );
}

#[test]
fn with_disallowed_tools_multiple_values_uses_single_flag() {
  let cmd = ClaudeCommand::new().with_disallowed_tools( [ "bash", "write" ] );
  let args = args_of( &cmd );
  let flag_count = args.iter().filter( |a| *a == "--disallowedTools" ).count();
  assert_eq!( flag_count, 1, "--disallowedTools must appear exactly once: {args:?}" );
}

// with_tools (Pattern E: space-separated)

#[test]
fn with_tools_adds_flag() {
  let cmd = ClaudeCommand::new().with_tools( [ "bash" ] );
  let args = args_of( &cmd );
  assert!( args.contains( &"--tools".to_string() ) );
  assert!( args.contains( &"bash".to_string() ) );
}

#[test]
fn with_tools_multiple_values_uses_single_flag() {
  let cmd = ClaudeCommand::new().with_tools( [ "bash", "read" ] );
  let args = args_of( &cmd );
  let flag_count = args.iter().filter( |a| *a == "--tools" ).count();
  assert_eq!( flag_count, 1, "--tools must appear exactly once: {args:?}" );
}

