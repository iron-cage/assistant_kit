//! Wire-shape tests for requests, plus the Claude-specific `SessionSummary`.
//!
//! These assert the bytes on the wire, not just that a value survives a
//! round-trip. A protocol whose shape is only tested against itself will happily
//! rename a field and keep passing while every existing client breaks.
//!
//! The response envelope (`Response`/`OkTrue`/`OkFalse`) is [`daemon_kit`]'s —
//! payload-agnostic, so its wire-shape tests live there now, in
//! `daemon_kit/tests/response_test.rs`. `OutputSlice` is [`child_supervisor`]'s,
//! tested in `child_supervisor/tests/output_test.rs`. What stays here is what
//! stays Claude-specific: `Request` and `SessionSummary`.
//!
//! ## Specification References
//!
//! - `docs/feature/002_wire_protocol.md` — the request and response shapes
//! - `docs/invariant/002_conversation_id_key.md` — why requests name a session id
//!
//! ## Coverage
//!
//! | TC | Scenario | Expected |
//! |----|----------|----------|
//! | proto01 | `Ping` / `ListSessions` / `StopDaemon` | Method name and nothing else |
//! | proto02 | `Spawn` with and without a prompt | `cwd` plus a `prompt` that may be null |
//! | proto03 | `Spawn` with `prompt` absent | Deserializes to `None` |
//! | proto04 | `Send`, `Resize`, `Shutdown` | Each names `session_id`, never a pid |
//! | proto05 | Every request round-trips | Value in == value out |
//! | proto10 | An unknown method | Rejected |
//! | proto11 | `SessionSummary` | Field names as documented |
//! | proto12 | One request per line | No embedded newlines |
//! | proto13 | `Read` | Names a session id and a cursor |
//! | proto14 | `Read` with `cursor` absent | Deserializes to `0` |
//! | proto16 | `ContextSummary` | Names a session id and nothing else |
//!
//! IDs are identifiers, not indices — proto06/07/08/09/15 moved out rather than
//! being renumbered; see `daemon_kit`'s and `child_supervisor`'s own suites.

use std::path::PathBuf;

use claude_daemon_core::{ Request, SessionSummary };
use serde_json::json;

/// Serialize `request` to a `Value` for shape comparison.
fn wire( request : &Request ) -> serde_json::Value
{
  serde_json::to_value( request ).expect( "request failed to serialize" )
}

/// Every request variant, for the round-trip sweep.
fn every_request() -> Vec< Request >
{
  vec![
    Request::Ping,
    Request::ListSessions,
    Request::Spawn { cwd : PathBuf::from( "/work" ), prompt : None },
    Request::Spawn { cwd : PathBuf::from( "/work" ), prompt : Some( "hello".into() ) },
    Request::Send { session_id : "conv-1".into(), text : "hi".into() },
    Request::Read { session_id : "conv-1".into(), cursor : 0 },
    Request::Read { session_id : "conv-1".into(), cursor : 8_192 },
    Request::ContextSummary { session_id : "conv-1".into() },
    Request::Resize { session_id : "conv-1".into(), rows : 40, cols : 132 },
    Request::Shutdown { session_id : "conv-1".into() },
    Request::StopDaemon,
  ]
}

/// proto01: the requests with no arguments carry only their method name.
#[ test ]
fn proto01_unit_requests_carry_only_a_method()
{
  assert_eq!( wire( &Request::Ping ), json!( { "method" : "ping" } ) );
  assert_eq!( wire( &Request::ListSessions ), json!( { "method" : "list_sessions" } ) );
  assert_eq!( wire( &Request::StopDaemon ), json!( { "method" : "stop_daemon" } ) );
}

/// proto02: `Spawn` carries the working directory and an optional prompt.
#[ test ]
fn proto02_spawn_carries_cwd_and_optional_prompt()
{
  assert_eq!(
    wire( &Request::Spawn { cwd : PathBuf::from( "/work" ), prompt : None } ),
    json!( { "method" : "spawn", "cwd" : "/work", "prompt" : null } ),
  );
  assert_eq!(
    wire( &Request::Spawn { cwd : PathBuf::from( "/work" ), prompt : Some( "go".into() ) } ),
    json!( { "method" : "spawn", "cwd" : "/work", "prompt" : "go" } ),
  );
}

/// proto03: an omitted `prompt` is `None`, not a parse failure.
///
/// The common case is a spawn with no opening prompt, and requiring an explicit
/// `null` would make the simplest request the wordiest.
#[ test ]
fn proto03_absent_prompt_defaults_to_none()
{
  let request : Request = serde_json::from_str( r#"{ "method": "spawn", "cwd": "/work" }"# )
    .expect( "spawn without a prompt should parse" );

  assert_eq!( request, Request::Spawn { cwd : PathBuf::from( "/work" ), prompt : None } );
}

/// proto04: every session-directed request names a conversation id, never a pid.
///
/// A PID-keyed request detaches the moment Claude Code re-hosts the session with
/// `--fork-session`, which is exactly when recovery is supposed to be working.
#[ test ]
fn proto04_session_requests_are_keyed_by_conversation_id()
{
  assert_eq!(
    wire( &Request::Send { session_id : "conv-1".into(), text : "hi".into() } ),
    json!( { "method" : "send", "session_id" : "conv-1", "text" : "hi" } ),
  );
  assert_eq!(
    wire( &Request::Resize { session_id : "conv-1".into(), rows : 40, cols : 132 } ),
    json!( { "method" : "resize", "session_id" : "conv-1", "rows" : 40, "cols" : 132 } ),
  );
  assert_eq!(
    wire( &Request::Shutdown { session_id : "conv-1".into() } ),
    json!( { "method" : "shutdown", "session_id" : "conv-1" } ),
  );
  assert_eq!(
    wire( &Request::ContextSummary { session_id : "conv-1".into() } ),
    json!( { "method" : "context_summary", "session_id" : "conv-1" } ),
  );

  for request in every_request()
  {
    let text = serde_json::to_string( &request ).expect( "serialize failed" );
    assert!( !text.contains( "\"pid\"" ), "a request carries a pid: {text}" );
  }
}

/// proto05: every request survives a round-trip unchanged.
#[ test ]
fn proto05_every_request_round_trips()
{
  for request in every_request()
  {
    let text = serde_json::to_string( &request ).expect( "serialize failed" );
    let back : Request = serde_json::from_str( &text ).expect( "deserialize failed" );
    assert_eq!( back, request, "round-trip changed the request: {text}" );
  }
}

/// proto10: an unknown method is refused rather than silently ignored.
#[ test ]
fn proto10_unknown_method_is_rejected()
{
  assert!(
    serde_json::from_str::< Request >( r#"{ "method": "self_destruct" }"# ).is_err(),
    "an unknown method was accepted",
  );
}

/// proto11: a session summary names the conversation id first and the pid as
/// advisory detail.
#[ test ]
fn proto11_session_summary_shape()
{
  let summary : SessionSummary = serde_json::from_value( json!(
  {
    "session_id" : "conv-1",
    "pid" : 4242,
    "cwd" : "/work",
    "busy" : true,
  } ) )
  .expect( "summary failed to parse" );

  assert_eq!( summary.session_id, "conv-1" );
  assert_eq!( summary.pid, 4242 );
  assert_eq!( summary.cwd, PathBuf::from( "/work" ) );
  assert!( summary.busy );

  assert_eq!(
    serde_json::to_value( &summary ).expect( "serialize failed" ),
    json!( { "session_id" : "conv-1", "pid" : 4242, "cwd" : "/work", "busy" : true } ),
  );
}

/// proto12: a serialized request never contains a newline.
///
/// The framing is one JSON object per line, so an embedded newline would split a
/// single message into two unparseable halves. The same property for the
/// response envelope is `daemon_kit`'s `resp05`.
#[ test ]
fn proto12_serialized_requests_are_single_lines()
{
  for request in every_request()
  {
    let text = serde_json::to_string( &request ).expect( "serialize failed" );
    assert!( !text.contains( '\n' ), "request serialized across lines: {text}" );
  }
}

/// proto13: `Read` names a session and a position within it.
///
/// The cursor is what makes reading non-destructive: two clients watching one
/// session each hold their own, and neither consumes the other's output.
#[ test ]
fn proto13_read_carries_a_session_id_and_cursor()
{
  assert_eq!(
    wire( &Request::Read { session_id : "conv-1".into(), cursor : 8_192 } ),
    json!( { "method" : "read", "session_id" : "conv-1", "cursor" : 8192 } ),
  );
}

/// proto14: an omitted `cursor` starts from the beginning.
///
/// A client attaching to a session for the first time has no cursor to send, and
/// what it wants is everything still retained — which is what `0` means.
#[ test ]
fn proto14_absent_cursor_defaults_to_zero()
{
  let request : Request = serde_json::from_str( r#"{ "method": "read", "session_id": "conv-1" }"# )
    .expect( "read without a cursor should parse" );

  assert_eq!( request, Request::Read { session_id : "conv-1".into(), cursor : 0 } );
}

/// proto16: `ContextSummary` names a session id and carries nothing else.
///
/// It is a pure read of the session's own transcript. Carrying no cursor, no
/// text, and no options is what makes it safe to issue against a session with a
/// turn in flight — there is nothing in the request that could disturb one.
#[ test ]
fn proto16_context_summary_names_only_a_session()
{
  let request = Request::ContextSummary { session_id : "conv-1".into() };
  let wire_form = wire( &request );

  assert_eq!( wire_form, json!( { "method" : "context_summary", "session_id" : "conv-1" } ) );

  let object = wire_form.as_object().expect( "request should be a JSON object" );
  assert_eq!( object.len(), 2, "context_summary carries only method and session_id" );

  let back : Request = serde_json::from_value( wire_form ).expect( "failed to parse" );
  assert_eq!( back, request );
}
