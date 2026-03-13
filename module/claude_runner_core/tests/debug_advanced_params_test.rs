//! Debug and advanced parameter builder method tests (TSK-078)
//!
//! ## Purpose
//!
//! Verify the six debug/advanced `with_*()` methods add the correct CLI flags.
//!
//! ## Evidence
//!
//! - `with_debug(None)` adds `-d` with no filter value
//! - `with_debug(Some("mcp"))` adds `-d mcp`
//! - `with_debug_file("/tmp/debug.log")` adds `--debug-file /tmp/debug.log`
//! - `with_betas(["b1","b2"])` adds two `--betas` pairs (Pattern F)
//! - `with_brief(true)` adds `--brief`; `with_brief(false)` adds nothing
//! - `with_disable_slash_commands(true)` adds `--disable-slash-commands`
//! - `with_file(["s1","s2"])` adds two `--file` pairs (Pattern F)
//!
//! ## Test Coverage Matrix
//!
//! | Method | flag present | flag absent | repeated | empty string |
//! |--------|-------------|-------------|----------|--------------|
//! | with_debug(None) | ✅ | — | — | — |
//! | with_debug(Some) | ✅ | — | — | ✅ |
//! | with_debug_file | ✅ | — | — | — |
//! | with_betas | ✅ | — | ✅ | — |
//! | with_brief | ✅ | ✅ | — | — |
//! | with_disable_slash_commands | ✅ | ✅ | — | — |
//! | with_file | ✅ | — | ✅ | — |

use claude_runner_core::ClaudeCommand;

fn args_of( cmd: &ClaudeCommand ) -> Vec<String> {
  let c = cmd.build_command_for_test();
  c.get_args().map( |a| a.to_string_lossy().to_string() ).collect()
}

// with_debug

#[test]
fn with_debug_none_adds_d_flag_only() {
  let cmd = ClaudeCommand::new().with_debug( None );
  let args = args_of( &cmd );
  assert!( args.contains( &"-d".to_string() ), "must contain -d: {args:?}" );
}

#[test]
fn with_debug_none_adds_exactly_one_extra_arg() {
  let baseline = args_of( &ClaudeCommand::new() ).len();
  let with_debug = args_of( &ClaudeCommand::new().with_debug( None ) ).len();
  assert_eq!( with_debug, baseline + 1, "with_debug(None) must add exactly 1 arg (-d)" );
}

#[test]
fn with_debug_some_adds_d_and_filter() {
  let cmd = ClaudeCommand::new().with_debug( Some( "mcp" ) );
  let args = args_of( &cmd );
  assert!( args.contains( &"-d".to_string() ) );
  assert!( args.contains( &"mcp".to_string() ) );
}

#[test]
fn with_debug_some_d_is_followed_by_filter() {
  let cmd = ClaudeCommand::new().with_debug( Some( "network" ) );
  let args = args_of( &cmd );
  let d_pos = args.iter().position( |a| a == "-d" ).expect( "-d not found" );
  assert_eq!( args.get( d_pos + 1 ).map( String::as_str ), Some( "network" ) );
}

// with_debug_file

#[test]
fn with_debug_file_adds_flag_and_path() {
  let cmd = ClaudeCommand::new().with_debug_file( "/tmp/debug.log" );
  let args = args_of( &cmd );
  assert!( args.contains( &"--debug-file".to_string() ) );
  assert!( args.contains( &"/tmp/debug.log".to_string() ) );
}

// with_betas (Pattern F: repeated-flag)

#[test]
fn with_betas_single_adds_flag_and_value() {
  let cmd = ClaudeCommand::new().with_betas( [ "computer-use-2024-10-22" ] );
  let args = args_of( &cmd );
  assert!( args.contains( &"--betas".to_string() ) );
  assert!( args.contains( &"computer-use-2024-10-22".to_string() ) );
}

#[test]
fn with_betas_two_values_produces_two_pairs() {
  let cmd = ClaudeCommand::new().with_betas( [ "beta1", "beta2" ] );
  let args = args_of( &cmd );
  let count = args.iter().filter( |a| *a == "--betas" ).count();
  assert_eq!( count, 2, "--betas must appear twice: {args:?}" );
}

// with_brief

#[test]
fn with_brief_true_adds_flag() {
  let cmd = ClaudeCommand::new().with_brief( true );
  assert!( args_of( &cmd ).contains( &"--brief".to_string() ) );
}

#[test]
fn with_brief_false_adds_nothing() {
  let cmd = ClaudeCommand::new().with_brief( false );
  assert!( !args_of( &cmd ).contains( &"--brief".to_string() ) );
}

// with_disable_slash_commands

#[test]
fn with_disable_slash_commands_true_adds_flag() {
  let cmd = ClaudeCommand::new().with_disable_slash_commands( true );
  assert!( args_of( &cmd ).contains( &"--disable-slash-commands".to_string() ) );
}

#[test]
fn with_disable_slash_commands_false_adds_nothing() {
  let cmd = ClaudeCommand::new().with_disable_slash_commands( false );
  assert!( !args_of( &cmd ).contains( &"--disable-slash-commands".to_string() ) );
}

// with_file (Pattern F: repeated-flag)

#[test]
fn with_file_single_adds_flag_and_spec() {
  let cmd = ClaudeCommand::new().with_file( [ "https://example.com/data.json" ] );
  let args = args_of( &cmd );
  assert!( args.contains( &"--file".to_string() ) );
  assert!( args.contains( &"https://example.com/data.json".to_string() ) );
}

#[test]
fn with_file_two_values_produces_two_pairs() {
  let cmd = ClaudeCommand::new().with_file( [ "spec1", "spec2" ] );
  let args = args_of( &cmd );
  let count = args.iter().filter( |a| *a == "--file" ).count();
  assert_eq!( count, 2, "--file must appear twice: {args:?}" );
}

// ── with_debug(Some("")) — empty filter string ────────────────────────────────

#[test]
fn with_debug_some_empty_string_adds_d_and_empty_value() {
  // with_debug(Some("")) is technically valid — passes empty string as filter.
  // Builder does not validate filter strings; empty string is passed through.
  // The -d flag will be followed by "" which the CLI may treat as "no filter" or error.
  let cmd = ClaudeCommand::new().with_debug( Some( "" ) );
  let args = args_of( &cmd );
  assert!( args.contains( &"-d".to_string() ), "-d must be present: {args:?}" );
  let d_pos = args.iter().position( |a| a == "-d" ).expect( "-d not found" );
  assert_eq!(
    args.get( d_pos + 1 ).map( String::as_str ), Some( "" ),
    "empty filter string must follow -d: {args:?}"
  );
}
