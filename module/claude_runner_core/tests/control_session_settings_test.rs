//! Real-subprocess tests: live session-configuration control methods — `setPermissionMode`,
//! `setModel`, `setMaxThinkingTokens`, `applyFlagSettings`, `streamInput` (task 415 Test
//! Matrix rows IT-11, IT-12, IT-13, IT-14, IT-25).
//!
//! No mocking anywhere below — every test spawns a real `claude` subprocess via
//! [`claude_runner_core::ClaudeCommand::spawn_control_session`].

mod control_session_common;

use claude_runner_core::ThinkingDisplay;
use core::time::Duration;
use std::time::Instant;

/// IT-11: `setPermissionMode(mode)` succeeds live against a real session.
#[ test ]
fn it_11_set_permission_mode_succeeds_live()
{
  let ( session, _dir ) = control_session_common::spawn_session();
  session.set_permission_mode( claude_runner_core::PermissionMode::AcceptEdits )
    .expect( "set_permission_mode() must succeed against a real session" );
}

/// IT-12: `setModel(model?)` succeeds live, both with an explicit model and with `None`
/// (which omits the `model` field entirely rather than sending a null).
#[ test ]
fn it_12_set_model_succeeds_live()
{
  let ( session, _dir ) = control_session_common::spawn_session();
  session.set_model( Some( "claude-sonnet-5" ) )
    .expect( "set_model(Some(..)) must succeed against a real session" );
  session.set_model( None )
    .expect( "set_model(None) must succeed against a real session" );
}

/// IT-13: `setMaxThinkingTokens(n, thinkingDisplay?)` succeeds live against a real session.
#[ test ]
fn it_13_set_max_thinking_tokens_succeeds_live()
{
  let ( session, _dir ) = control_session_common::spawn_session();
  session.set_max_thinking_tokens( Some( 2048 ), Some( ThinkingDisplay::Summarized ) )
    .expect( "set_max_thinking_tokens() must succeed against a real session" );
}

/// IT-14: `applyFlagSettings(settings)` succeeds live against a real session, applying
/// settings without a session restart.
#[ test ]
fn it_14_apply_flag_settings_succeeds_live()
{
  let ( session, _dir ) = control_session_common::spawn_session();
  session.apply_flag_settings( serde_json::json!( {} ) )
    .expect( "apply_flag_settings() must succeed against a real session" );
}

/// IT-25: `streamInput(content)` is confirmed NOT a `control_request` — it resolves as soon
/// as the write succeeds, never awaiting a wire acknowledgment (there is none).
#[ test ]
fn it_25_stream_input_resolves_on_write_without_awaiting_ack()
{
  let ( session, _dir ) = control_session_common::spawn_session();
  let started = Instant::now();
  session.stream_input( "respond with exactly: OK" )
    .expect( "stream_input() must succeed (resolves on write, no ack awaited)" );
  assert!(
    started.elapsed() < Duration::from_secs( 2 ),
    "stream_input() must return almost immediately — it never awaits a control_response"
  );
}
