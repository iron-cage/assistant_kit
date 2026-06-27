//! Unit tests for `render_summary` and `resolve_fields`.
//!
//! Tests the CLR result-envelope parser and field-profile resolver in isolation,
//! without invoking the `clr` binary.
//!
//! # Root Cause (BUG-309)
//! Old parser hard-gated on `"id"` field (Messages API); CLR envelopes have
//! `"session_id"` instead — causing 100% production failure masked by wrong-schema fixtures.
//!
//! # Root Cause (BUG-310)
//! `render_summary()` gated on optional `"session_id"` field using `?`, returning `None`
//! for 7-field minimal envelopes that omit `session_id`.  Gate must be on `"type":"result"`.
//!
//! # Why Not Caught (BUG-309 / BUG-310)
//! All prior test fixtures included every optional field, so the `?` propagation on
//! absent optional fields was never exercised.
//!
//! # Fix Applied
//! `render_summary()` gates only on the invariant `"type"` field; `session_id` is
//! extracted with `.unwrap_or_default()`.  IT-1 / IT-4–IT-6 (below) are the
//! regression guards.
//!
//! # Prevention
//! IT-7 in `output_style_test.rs` performs a structural source scan to block re-introduction
//! of the `extract_str(json,"session_id")?` anti-pattern.
//!
//! # Pitfall
//! Any `?` on an optional CLR field in `render_summary()` silently breaks all envelopes
//! that omit that field — including envelopes from older `claude` binary versions.
//! Gate only on `"type":"result"` (invariant).
#![ cfg( feature = "enabled" ) ]

use claude_runner::{ render_summary, resolve_fields };

const FULL_ENVELOPE : &str = r#"{"type":"result","subtype":"success","session_id":"00000000-0000-0000-0000-000000000001","is_error":false,"duration_ms":100,"duration_api_ms":90,"num_turns":1,"result":"hello","stop_reason":"end_turn","total_cost_usd":0.001,"uuid":"00000000-0000-0000-0000-000000000002","fast_mode_state":"off","usage":{"input_tokens":3,"output_tokens":4,"cache_creation_input_tokens":0,"cache_read_input_tokens":0,"service_tier":"standard","speed":"standard","inference_geo":"","server_tool_use":{"web_search_requests":0,"web_fetch_requests":0},"cache_creation":{"ephemeral_1h_input_tokens":0,"ephemeral_5m_input_tokens":0},"iterations":[]},"modelUsage":{"claude-opus-4-6":{"inputTokens":3,"outputTokens":4,"cacheReadInputTokens":0,"cacheCreationInputTokens":0,"webSearchRequests":0,"costUSD":0.001,"contextWindow":200000,"maxOutputTokens":32000}},"permission_denials":[]}"#;

const MINIMAL_ENVELOPE : &str =
  r#"{"type":"result","subtype":"success","is_error":false,"duration_ms":1000,"duration_api_ms":900,"num_turns":1,"result":"hello"}"#;

/// EC-14: `render_summary()` returns `Some` for a valid full CLR result envelope.
#[ test ]
fn ec14_render_summary_clr_envelope_accepted()
{
  let rendered = render_summary( FULL_ENVELOPE, None );
  assert!( rendered.is_some(), "render_summary must return Some for valid CLR envelope; got None" );
  let s = rendered.unwrap();
  assert!( s.contains( "---" ),                 "rendered output must contain separator '---'. Got:\n{s}" );
  assert!( s.contains( "hello" ),               "rendered output must contain the result text. Got:\n{s}" );
  assert!( s.contains( "session_id:" ),         "output must contain 'session_id:'. Got:\n{s}" );
  assert!( s.contains( "model:" ),              "output must contain 'model:'. Got:\n{s}" );
  assert!( s.contains( "permission_denials:" ), "output must contain 'permission_denials:'. Got:\n{s}" );
  assert!( s.contains( "duration_ms:" ),        "output must contain 'duration_ms:'. Got:\n{s}" );
}

/// Unescape test: JSON `\n` in `result` field becomes actual newline in output.
#[ test ]
fn extract_str_unescapes_json_newlines()
{
  let json     = r#"{"type":"result","subtype":"success","session_id":"x","is_error":false,"result":"line1\nline2","usage":{"input_tokens":0,"output_tokens":0},"total_cost_usd":0.0}"#;
  let rendered = render_summary( json, None ).expect( "must parse" );
  assert!( rendered.contains( "line1\nline2" ), "\\n must be unescaped to actual newline. Got:\n{rendered}" );
}

#[ test ]
fn resolve_fields_full_returns_32()
{
  let fields = resolve_fields( "full" ).unwrap();
  assert_eq!( fields.len(), 32, "full profile must have 32 fields" );
}

#[ test ]
fn resolve_fields_minimal_returns_7()
{
  let fields = resolve_fields( "minimal" ).unwrap();
  assert_eq!( fields.len(), 7, "minimal profile must have 7 fields" );
  assert!( fields.contains( &"type" ) );
  assert!( fields.contains( &"total_cost_usd" ) );
}

#[ test ]
fn resolve_fields_standard_returns_14()
{
  let fields = resolve_fields( "standard" ).unwrap();
  assert_eq!( fields.len(), 14, "standard profile must have 14 fields" );
  assert!( fields.contains( &"model" ) );
  assert!( fields.contains( &"duration_ms" ) );
}

#[ test ]
fn resolve_fields_custom_whitelist()
{
  let fields = resolve_fields( "type,session_id,total_cost_usd" ).unwrap();
  assert_eq!( fields.len(), 3 );
  assert!( fields.contains( &"type" ) );
  assert!( fields.contains( &"session_id" ) );
  assert!( fields.contains( &"total_cost_usd" ) );
}

#[ test ]
fn resolve_fields_invalid_single_token()
{
  let err = resolve_fields( "bogus" ).unwrap_err();
  assert_eq!( err, "bogus" );
}

#[ test ]
fn resolve_fields_invalid_in_custom_list()
{
  let err = resolve_fields( "type,nonexistent_field" ).unwrap_err();
  assert_eq!( err, "nonexistent_field" );
}

/// `render_summary` with `minimal` profile renders only 7 header fields.
#[ test ]
fn render_summary_minimal_filters_fields()
{
  let rendered = render_summary( FULL_ENVELOPE, Some( "minimal" ) ).unwrap();
  assert!( rendered.contains( "type:" ),           "minimal must include type:" );
  assert!( rendered.contains( "total_cost_usd:" ), "minimal must include total_cost_usd:" );
  assert!( !rendered.contains( "duration_ms:" ),   "minimal must NOT include duration_ms:" );
  assert!( !rendered.contains( "model:" ),         "minimal must NOT include model:" );
  assert!( rendered.contains( "---" ),             "separator must always appear" );
  assert!( rendered.contains( "hello" ),           "result body must always appear" );
}

// ── BUG-310 gate invariant tests (IT-1, IT-4–IT-6) ───────────────────────────

/// IT-1 (BUG-310 regression): minimal 7-field CLR envelope without `session_id` must
/// return `Some` — gate is on `type=="result"`, not the optional `session_id` field.
#[ test ]
fn render_summary_accepts_envelope_without_session_id()
{
  let result = render_summary( MINIMAL_ENVELOPE, None );
  assert!( result.is_some(), "render_summary must return Some for 7-field envelope lacking session_id; got None" );
  let s = result.unwrap();
  assert!( s.contains( "---" ),   "separator must appear. Got:\n{s}" );
  assert!( s.contains( "hello" ), "result text must appear. Got:\n{s}" );
}

/// IT-4: JSON with `"type":"message"` must be rejected (not a CLR result envelope).
#[ test ]
fn render_summary_rejects_non_result_type()
{
  let json   = r#"{"type":"message","content":"some stream output"}"#;
  let result = render_summary( json, None );
  assert!( result.is_none(), "must return None for type!=result; got Some" );
}

/// IT-5: JSON without a `type` field at all must be rejected.
#[ test ]
fn render_summary_rejects_json_without_type()
{
  let json   = r#"{"session_id":"abc","result":"hello","is_error":false}"#;
  let result = render_summary( json, None );
  assert!( result.is_none(), "must return None for JSON lacking type field; got Some" );
}

/// IT-6: Non-JSON input must be rejected.
#[ test ]
fn render_summary_rejects_non_json()
{
  let result = render_summary( "this is not json at all", None );
  assert!( result.is_none(), "must return None for non-JSON input; got Some" );
}
