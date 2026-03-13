//! I/O parameter builder method tests (TSK-072)
//!
//! ## Purpose
//!
//! Verify the six I/O parameter `with_*()` methods add the correct CLI flags.
//!
//! ## Evidence
//!
//! - `with_print(true)` adds `-p`
//! - `with_print(false)` adds nothing
//! - `with_output_format(OutputFormat::Json)` adds `--output-format json`
//! - `with_output_format(OutputFormat::StreamJson)` adds `--output-format stream-json` (hyphen!)
//! - `with_input_format(InputFormat::StreamJson)` adds `--input-format stream-json` (hyphen!)
//! - `with_include_partial_messages(true)` adds `--include-partial-messages`
//! - `with_include_partial_messages(false)` adds nothing
//! - `with_replay_user_messages(true)` adds `--replay-user-messages`
//! - `with_replay_user_messages(false)` adds nothing
//! - `with_json_schema(s)` adds `--json-schema <s>`
//!
//! ## Test Coverage Matrix
//!
//! | Method | flag present | flag absent (false) |
//! |--------|-------------|---------------------|
//! | with_print | ✅ | ✅ |
//! | with_output_format | ✅ | — |
//! | with_input_format | ✅ | — |
//! | with_include_partial_messages | ✅ | ✅ |
//! | with_replay_user_messages | ✅ | ✅ |
//! | with_json_schema | ✅ | — |

use claude_runner_core::{ ClaudeCommand, InputFormat, OutputFormat };

fn args_of( cmd: &ClaudeCommand ) -> Vec<String> {
  let c = cmd.build_command_for_test();
  c.get_args().map( |a| a.to_string_lossy().to_string() ).collect()
}

// with_print

#[test]
fn with_print_true_adds_p_flag() {
  let cmd = ClaudeCommand::new().with_print( true );
  assert!( args_of( &cmd ).contains( &"-p".to_string() ) );
}

#[test]
fn with_print_false_adds_nothing() {
  let cmd = ClaudeCommand::new().with_print( false );
  assert!( !args_of( &cmd ).contains( &"-p".to_string() ) );
}

// with_output_format

#[test]
fn with_output_format_text_adds_flag() {
  let cmd = ClaudeCommand::new().with_output_format( OutputFormat::Text );
  let args = args_of( &cmd );
  assert!( args.contains( &"--output-format".to_string() ) );
  assert!( args.contains( &"text".to_string() ) );
}

#[test]
fn with_output_format_json_adds_flag() {
  let cmd = ClaudeCommand::new().with_output_format( OutputFormat::Json );
  let args = args_of( &cmd );
  assert!( args.contains( &"--output-format".to_string() ) );
  assert!( args.contains( &"json".to_string() ) );
}

#[test]
fn with_output_format_stream_json_uses_hyphen() {
  // Fix(issue-output-format-stream-json-hyphen): stream-json uses hyphen not underscore
  let cmd = ClaudeCommand::new().with_output_format( OutputFormat::StreamJson );
  let args = args_of( &cmd );
  assert!( args.contains( &"--output-format".to_string() ) );
  assert!( args.contains( &"stream-json".to_string() ), "must use hyphen: {args:?}" );
  assert!( !args.contains( &"stream_json".to_string() ), "must NOT use underscore" );
}

// with_input_format

#[test]
fn with_input_format_text_adds_flag() {
  let cmd = ClaudeCommand::new().with_input_format( InputFormat::Text );
  let args = args_of( &cmd );
  assert!( args.contains( &"--input-format".to_string() ) );
  assert!( args.contains( &"text".to_string() ) );
}

#[test]
fn with_input_format_stream_json_uses_hyphen() {
  // Fix(issue-input-format-stream-json-hyphen): stream-json uses hyphen not underscore
  let cmd = ClaudeCommand::new().with_input_format( InputFormat::StreamJson );
  let args = args_of( &cmd );
  assert!( args.contains( &"--input-format".to_string() ) );
  assert!( args.contains( &"stream-json".to_string() ), "must use hyphen: {args:?}" );
  assert!( !args.contains( &"stream_json".to_string() ), "must NOT use underscore" );
}

// with_include_partial_messages

#[test]
fn with_include_partial_messages_true_adds_flag() {
  let cmd = ClaudeCommand::new().with_include_partial_messages( true );
  assert!( args_of( &cmd ).contains( &"--include-partial-messages".to_string() ) );
}

#[test]
fn with_include_partial_messages_false_adds_nothing() {
  let cmd = ClaudeCommand::new().with_include_partial_messages( false );
  assert!( !args_of( &cmd ).contains( &"--include-partial-messages".to_string() ) );
}

// with_replay_user_messages

#[test]
fn with_replay_user_messages_true_adds_flag() {
  let cmd = ClaudeCommand::new().with_replay_user_messages( true );
  assert!( args_of( &cmd ).contains( &"--replay-user-messages".to_string() ) );
}

#[test]
fn with_replay_user_messages_false_adds_nothing() {
  let cmd = ClaudeCommand::new().with_replay_user_messages( false );
  assert!( !args_of( &cmd ).contains( &"--replay-user-messages".to_string() ) );
}

// with_json_schema

#[test]
fn with_json_schema_adds_flag_and_value() {
  let schema = r#"{"type":"object"}"#;
  let cmd = ClaudeCommand::new().with_json_schema( schema );
  let args = args_of( &cmd );
  assert!( args.contains( &"--json-schema".to_string() ) );
  assert!( args.contains( &schema.to_string() ) );
}

#[test]
fn with_json_schema_passes_value_verbatim() {
  let schema = r#"{"type":"array","items":{"type":"string"}}"#;
  let cmd = ClaudeCommand::new().with_json_schema( schema );
  let args = args_of( &cmd );
  assert!( args.contains( &schema.to_string() ), "schema must be passed verbatim: {args:?}" );
}
