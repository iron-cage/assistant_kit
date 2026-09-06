//! Wire-shape tests for the response envelope.
//!
//! These assert the bytes on the wire, not just that a value survives a
//! round-trip. A protocol whose shape is only tested against itself will happily
//! rename a field and keep passing while every existing client breaks.
//!
//! ## Specification References
//!
//! - `claude_daemon_core/docs/feature/002_wire_protocol.md` — the response shape
//!   and why the `ok` discriminant is explicit
//!
//! ## Coverage
//!
//! | TC | Scenario | Expected |
//! |----|----------|----------|
//! | resp01 | `Response::ok` | `{"ok":true,"result":…}` |
//! | resp02 | `Response::err` | `{"ok":false,"error":…}` |
//! | resp03 | Deserializing both response forms | Correct variant each time |
//! | resp04 | `ok:true` with an `error` field | Rejected — neither variant matches |
//! | resp05 | A response never contains a newline | Even with one embedded in the message |

use daemon_kit::Response;
use serde_json::json;

/// resp01, resp02: the two response shapes.
///
/// The explicit `ok` discriminant is what lets a client written against the
/// earlier per-PID `query.rs` protocol read these responses unchanged.
#[ test ]
fn resp01_response_shapes()
{
  assert_eq!(
    serde_json::to_value( Response::ok( json!( { "version" : "1.2.0" } ) ) )
      .expect( "serialize failed" ),
    json!( { "ok" : true, "result" : { "version" : "1.2.0" } } ),
  );
  assert_eq!(
    serde_json::to_value( Response::err( "no such session: conv-9" ) ).expect( "serialize failed" ),
    json!( { "ok" : false, "error" : "no such session: conv-9" } ),
  );
}

/// resp03: both forms deserialize to the variant their `ok` field names.
#[ test ]
fn resp03_both_response_forms_deserialize()
{
  let success : Response = serde_json::from_str( r#"{ "ok": true, "result": [1,2] }"# )
    .expect( "success response failed to parse" );
  assert_eq!( success, Response::ok( json!( [ 1, 2 ] ) ) );

  let failure : Response = serde_json::from_str( r#"{ "ok": false, "error": "boom" }"# )
    .expect( "error response failed to parse" );
  assert_eq!( failure, Response::err( "boom" ) );
}

/// resp04: `ok` and the payload must agree.
///
/// `ok:true` alongside an `error` matches neither variant. Accepting it would
/// let a client read a failure as a success whose result happened to be missing.
#[ test ]
fn resp04_mismatched_ok_and_payload_is_rejected()
{
  for body in [
    r#"{ "ok": true, "error": "boom" }"#,
    r#"{ "ok": false, "result": 1 }"#,
    r#"{ "ok": true }"#,
  ]
  {
    assert!(
      serde_json::from_str::< Response >( body ).is_err(),
      "accepted a self-contradictory response: {body}",
    );
  }
}

/// resp05: a serialized response never contains a newline.
///
/// The framing is one JSON object per line, so an embedded newline would split a
/// single message into two unparseable halves.
#[ test ]
fn resp05_serialized_response_is_a_single_line()
{
  let response = serde_json::to_string( &Response::err( "line one\nline two" ) )
    .expect( "serialize failed" );
  assert!(
    !response.contains( '\n' ),
    "a newline inside an error message reached the wire unescaped: {response}",
  );
}
