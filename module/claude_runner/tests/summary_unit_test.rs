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

use claude_runner::{ render_summary, resolve_fields, extract_session_id };

const FULL_ENVELOPE : &str = r#"{"type":"result","subtype":"success","session_id":"00000000-0000-0000-0000-000000000001","is_error":false,"duration_ms":100,"duration_api_ms":90,"num_turns":1,"result":"hello","stop_reason":"end_turn","total_cost_usd":0.001,"uuid":"00000000-0000-0000-0000-000000000002","fast_mode_state":"off","usage":{"input_tokens":3,"output_tokens":4,"cache_creation_input_tokens":0,"cache_read_input_tokens":0,"service_tier":"standard","speed":"standard","inference_geo":"","server_tool_use":{"web_search_requests":0,"web_fetch_requests":0},"cache_creation":{"ephemeral_1h_input_tokens":0,"ephemeral_5m_input_tokens":0},"iterations":[]},"modelUsage":{"claude-opus-4-8":{"inputTokens":3,"outputTokens":4,"cacheReadInputTokens":0,"cacheCreationInputTokens":0,"webSearchRequests":0,"costUSD":0.001,"contextWindow":200000,"maxOutputTokens":32000}},"permission_denials":[]}"#;

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

// ── extract_session_id tests (IT-8–IT-10) ────────────────────────────────────

/// IT-8: Valid `"type":"result"` envelope with `"session_id"` returns the UUID.
#[ test ]
fn extract_session_id_returns_uuid_for_valid_envelope()
{
  let json   = r#"{"type":"result","subtype":"success","session_id":"abc-123","is_error":false,"result":"ok"}"#;
  let result = extract_session_id( json );
  assert_eq!( result, Some( "abc-123".to_string() ), "must return the session_id from a result envelope" );
}

/// IT-9: Envelope with `"type":"message"` (not `"result"`) must return `None`.
#[ test ]
fn extract_session_id_returns_none_for_non_result_type()
{
  let json   = r#"{"type":"message","session_id":"abc-123","content":"stream output"}"#;
  let result = extract_session_id( json );
  assert!( result.is_none(), "must return None when type is not 'result'; got Some" );
}

/// IT-10: Valid `"type":"result"` envelope without `"session_id"` field returns `None`.
#[ test ]
fn extract_session_id_returns_none_when_session_id_absent()
{
  let json   = r#"{"type":"result","subtype":"success","is_error":false,"result":"hello"}"#;
  let result = extract_session_id( json );
  assert!( result.is_none(), "must return None when session_id field is absent; got Some" );
}

/// IT-11: `"session_id"` value truncated before its closing quote returns `None`, not
/// `Some(<partial>)` (BUG-395).
///
/// # Root Cause
/// `extract_str()`'s char-by-char scan (which `extract_session_id` thinly wraps for the
/// `"session_id"` key) has no post-loop fallback for input exhausted before an unescaped
/// closing quote — the loop's only early return is on finding one, so exhaustion silently
/// fell through to the success value `Some(out)` instead of `None`.
///
/// # Why Not Caught
/// Every existing `extract_session_id` test (IT-8–IT-10) uses a complete, well-formed
/// `session_id` value; none construct an envelope truncated mid-value the way a `claude`
/// subprocess killed or cut off mid-stream would produce.
///
/// # Fix Applied
/// `extract_str()` now returns `None` (not `Some(out)`) when its scan loop exhausts the
/// input without finding an unescaped closing quote.
///
/// # Prevention
/// This is the direct-unit-test regression guard for the fix; the consequence this bug's
/// own Impact section documents (a false-positive BUG-320 session-mismatch warning) is a
/// mechanical result of `extract_session_id` now correctly returning `None` here — Rust's
/// `if let Some(actual) = extract_session_id(...)` simply does not execute for `None`, so
/// no separate execution.rs-level test is needed to prove that consequence.
///
/// # Pitfall
/// Don't mistake a truncated/malformed field value for a legitimately short one — a scan
/// loop's post-loop fallthrough must be a failure value whenever the loop's only early
/// return is "found the terminator".
// test_kind: bug_reproducer(BUG-395)
#[ test ]
fn extract_session_id_returns_none_for_unterminated_session_id()
{
  let json   = r#"{"type":"result","subtype":"success","is_error":false,"session_id":"abc-123"#;
  let result = extract_session_id( json );
  assert!(
    result.is_none(),
    "must return None when the session_id value is truncated before its closing quote \
     (unterminated string), not Some(<partial>); got {result:?}"
  );
}

// ── BUG-394 requirement-1 (escape-aware bounding) tests (IN-3, IN-4) ────────────

/// IN-3: `render_summary()`'s inline `model_name` extraction is not truncated at an
/// escaped `"` inside the `modelUsage` object's key.
///
/// # Root Cause
/// The two-call `model_name` extraction (`s.find('"')` then `inner.find('"')`) had no
/// escape-state tracking on either call, stopping at the first escaped `\"` inside the
/// model-identifier key instead of the true closing quote.
///
/// # Why Not Caught
/// Every existing fixture's `modelUsage` key (e.g. `claude-opus-4-8`) contains no quote
/// character at all, so the naive `.find('"')` always happened to land on the true
/// terminator by coincidence.
///
/// # Fix Applied
/// Both quote searches now route through `find_unescaped_quote()` (escape-aware scan)
/// instead of a bare `.find('"')`.
///
/// # Prevention
/// See `docs/invariant/014_json_string_extraction_escape_handling.md` IN-3.
///
/// # Pitfall
/// A two-call first/next `.find('"')` pattern is exactly as escape-unaware as a single
/// bare `.find('"')` — neither tracks whether the preceding character was an unescaped
/// backslash.
// test_kind: bug_reproducer(BUG-394)
#[ test ]
fn render_summary_model_name_escaped_quote_not_truncated()
{
  let json     = r#"{"type":"result","subtype":"success","is_error":false,"result":"hello","modelUsage":{"He said \"hi\"-model":{"inputTokens":10,"outputTokens":20,"cacheReadInputTokens":0,"cacheCreationInputTokens":0,"webSearchRequests":0,"costUSD":0.01,"contextWindow":200000,"maxOutputTokens":32000}}}"#;
  let rendered = render_summary( json, None ).expect( "must parse" );
  assert!(
    rendered.contains( "He said \\\"hi\\\"-model" ),
    "model name must be bounded at the true closing quote, not truncated at the escaped \
     quote (model_name is not unescaped, so the literal backslashes are expected in the \
     rendered form). Got:\n{rendered}"
  );
}

/// IN-4 (regression guard): `extract_str()`'s pre-existing escape-aware bounding for the
/// `"result"` field must not regress when BUG-395's fail-closed-on-exhaustion fix is
/// applied to the same function's loop body — `extract_str()` was already correct for
/// requirement 1 (escape-aware bounding); only requirement 2 (fail-closed on exhaustion)
/// was broken (see the unterminated-string tests above and below).
#[ test ]
fn extract_str_result_field_escaped_quote_not_truncated()
{
  let json     = r#"{"type":"result","subtype":"success","is_error":false,"result":"He said \"hi\" to me","usage":{"input_tokens":0,"output_tokens":0},"total_cost_usd":0.0}"#;
  let rendered = render_summary( json, None ).expect( "must parse" );
  assert!(
    rendered.contains( "He said \"hi\" to me" ),
    "result field must be correctly unescaped and bounded at the true closing quote, not \
     truncated at the escaped quote. Got:\n{rendered}"
  );
}

// ── BUG-395 downstream consumer test (IN-7) ─────────────────────────────────────

/// IN-7: `render_summary()`'s `"result"` field falls back to an empty string — not a
/// truncated partial value — when its value is unterminated.
///
/// # Root Cause
/// See the unterminated-session_id test above (`extract_str()`'s post-loop fallthrough).
/// `render_summary()`'s `"result"` extraction is `.unwrap_or_default()`-bounded, so a
/// requirement-2 violation here degrades display quality only; it does not gate the
/// overall `Some`/`None` return the way `extract_session_id()`'s `?`-propagation does.
///
/// # Why Not Caught
/// No existing `render_summary()` test constructs an envelope with a truncated/
/// unterminated `"result"` value; all prior fixtures close every string field.
///
/// # Fix Applied
/// Same fix as the unterminated-session_id case — `extract_str()` now returns `None` on
/// scan-loop exhaustion, so `.unwrap_or_default()` now correctly yields `""` instead of
/// the pre-fix partial text.
///
/// # Prevention
/// See `docs/invariant/014_json_string_extraction_escape_handling.md` IN-7.
///
/// # Pitfall
/// An `.unwrap_or_default()`-guarded call site silently masks a requirement-2 violation
/// as a display quirk (partial text) rather than a control-flow bug — easy to miss
/// without an explicit test for the unterminated case.
// test_kind: bug_reproducer(BUG-395)
#[ test ]
fn render_summary_result_field_unterminated_falls_back_to_empty()
{
  let json     = r#"{"type":"result","subtype":"success","is_error":false,"duration_ms":1,"duration_api_ms":1,"num_turns":1,"result":"partial text that never closes"#;
  let rendered = render_summary( json, None )
    .expect( "type gate must still pass; result field is unwrap_or_default-bounded" );
  assert!(
    !rendered.contains( "partial text that never closes" ),
    "result field must NOT show truncated partial text when its value is unterminated. \
     Got:\n{rendered}"
  );
}

// ── BUG-436/437/438: new SDK envelope regression tests ───────────────────────
//
// Newer Claude SDK envelopes omit the top-level `"type":"result"` field and include
// `usage.iterations[]` objects that each carry `"type":"message"`.  The depth-unaware
// `extract_str(json, "type")` call used in the old gate finds the nested field first,
// causing all three gated functions to return `None` for these envelopes.
// Fixed by gating on `"subtype"` presence — a field exclusive to the top-level envelope.

// Envelope without top-level `"type":"result"` but with `usage.iterations[].type="message"`.
const NEW_SDK_ENVELOPE : &str = r#"{"subtype":"success","session_id":"00000000-0000-0000-0000-000000000001","is_error":false,"duration_ms":100,"duration_api_ms":90,"num_turns":1,"result":"hello","stop_reason":"end_turn","total_cost_usd":0.001,"uuid":"00000000-0000-0000-0000-000000000002","fast_mode_state":"off","usage":{"input_tokens":3,"output_tokens":4,"cache_creation_input_tokens":0,"cache_read_input_tokens":0,"service_tier":"standard","speed":"standard","inference_geo":"","server_tool_use":{"web_search_requests":0,"web_fetch_requests":0},"cache_creation":{"ephemeral_1h_input_tokens":0,"ephemeral_5m_input_tokens":0},"iterations":[{"input_tokens":1,"output_tokens":2,"cache_read_input_tokens":0,"cache_creation_input_tokens":0,"type":"message"}]},"modelUsage":{"claude-opus-4-8":{"inputTokens":3,"outputTokens":4,"cacheReadInputTokens":0,"cacheCreationInputTokens":0,"webSearchRequests":0,"costUSD":0.001,"contextWindow":200000,"maxOutputTokens":32000}},"permission_denials":[]}"#;

// Variant with empty `result` and `structured_output` — exercises the BUG-438 code path
// inside `render_summary()` where `extract_structured_output()` is called for the body.
const NEW_SDK_ENVELOPE_STRUCTURED : &str = r#"{"subtype":"success","session_id":"00000000-0000-0000-0000-000000000001","is_error":false,"duration_ms":100,"duration_api_ms":90,"num_turns":1,"result":"","structured_output":{"answer":"schema_data"},"stop_reason":"end_turn","total_cost_usd":0.001,"uuid":"00000000-0000-0000-0000-000000000002","fast_mode_state":"off","usage":{"input_tokens":3,"output_tokens":4,"cache_creation_input_tokens":0,"cache_read_input_tokens":0,"service_tier":"standard","speed":"standard","inference_geo":"","server_tool_use":{"web_search_requests":0,"web_fetch_requests":0},"cache_creation":{"ephemeral_1h_input_tokens":0,"ephemeral_5m_input_tokens":0},"iterations":[{"input_tokens":1,"output_tokens":2,"cache_read_input_tokens":0,"cache_creation_input_tokens":0,"type":"message"}]},"modelUsage":{"claude-opus-4-8":{"inputTokens":3,"outputTokens":4,"cacheReadInputTokens":0,"cacheCreationInputTokens":0,"webSearchRequests":0,"costUSD":0.001,"contextWindow":200000,"maxOutputTokens":32000}},"permission_denials":[]}"#;

/// BUG-436 regression: `render_summary()` must return `Some` for new SDK envelope
/// where `usage.iterations[].type = "message"` is the first `"type":` in the JSON.
///
/// # Root Cause
/// `render_summary()` gated on `extract_str(json,"type") == "result"`.  The depth-unaware
/// `s.find()` inside `extract_str` finds the nested `"type":"message"` inside
/// `usage.iterations[0]` first (no top-level `"type":"result"` present), so the gate
/// fires and `render_summary()` returns `None` for all new SDK envelopes.
///
/// # Why Not Caught
/// All prior fixtures had `"type":"result"` at the top level.  No test used an envelope
/// where `"type"` is absent from the top level and present only inside iterations.
///
/// # Fix Applied
/// Gate changed to `extract_str(json,"subtype")?` — `"subtype"` is top-level-only,
/// never present inside `usage.iterations[]` objects.
///
/// # Pitfall
/// `extract_str` uses `s.find()` — any gate using it must target a key that is
/// EXCLUSIVE to the top-level CLR envelope, not reused in nested objects.
// test_kind: bug_reproducer(BUG-436)
#[ test ]
fn render_summary_with_nested_type_in_iterations_produces_summary()
{
  let result = render_summary( NEW_SDK_ENVELOPE, None );
  assert!(
    result.is_some(),
    "render_summary must return Some for new SDK envelope where usage.iterations[].type \
     = \"message\" is the first \"type:\" in the JSON; got None (old gate blocked by nested field)"
  );
  let s = result.unwrap();
  assert!( s.contains( "hello" ), "result body must appear in rendered output. Got:\n{s}" );
  assert!( s.contains( "---" ),   "separator must appear in rendered output. Got:\n{s}" );
}

/// BUG-437 regression: `extract_session_id()` must return the UUID for new SDK envelope
/// where `usage.iterations[].type = "message"` is the first `"type":` in the JSON.
///
/// # Root Cause
/// `extract_session_id()` applied the identical `"type":"result"` gate as `render_summary()`
/// (BUG-436).  For new SDK envelopes the gate fired and `extract_session_id()` returned
/// `None`, silently disabling BUG-320 session mismatch detection for all new-SDK invocations.
///
/// # Why Not Caught
/// No test used a new-format envelope against `extract_session_id()`.  The bug is invisible:
/// `None` from `extract_session_id()` produces no error — the BUG-320 warning simply does
/// not appear, indistinguishable from "no mismatch occurred."
///
/// # Fix Applied
/// Gate changed to `if extract_str(json,"subtype").is_none() { return None; }`.
///
/// # Pitfall
/// Silent-failure gates (where `None` is also the "no event" path) must have explicit
/// positive tests confirming they fire for the correct inputs.
// test_kind: bug_reproducer(BUG-437)
#[ test ]
fn extract_session_id_returns_uuid_for_new_sdk_envelope()
{
  let result = extract_session_id( NEW_SDK_ENVELOPE );
  assert_eq!(
    result,
    Some( "00000000-0000-0000-0000-000000000001".to_string() ),
    "extract_session_id must return the UUID for new SDK envelope where \
     usage.iterations[].type = \"message\" is the first \"type:\" in the JSON; \
     got None (BUG-320 detection was disabled for all new SDK envelopes)"
  );
}

/// BUG-438 regression: `render_summary()` body must contain `structured_output` content
/// for new SDK envelopes when `result` is empty.
///
/// # Root Cause
/// `extract_structured_output()` applied the same `"type":"result"` gate as BUG-436/437.
/// For new SDK envelopes the gate returned `None`, so `render_summary()` fell back to `""`
/// for the body — blank output for `--json-schema` sessions.  The bug was masked by
/// BUG-436: both the outer gate and `extract_structured_output()`'s own gate failed for
/// new SDK envelopes, so `render_summary()` returned `None` before reaching the body
/// computation.  BUG-438 only becomes observable after BUG-436 is fixed.
///
/// # Why Not Caught
/// No test exercised `extract_structured_output()` against a new SDK envelope.
///
/// # Fix Applied
/// Gate in `extract_structured_output()` changed to
/// `if extract_str(json,"subtype").is_none() { return None; }`.
///
/// # Pitfall
/// A second gate hidden inside a call path masks its own failure when the outer gate also
/// fails — both must be fixed together and verified with a test that reaches the inner call.
// test_kind: bug_reproducer(BUG-438)
#[ test ]
fn render_summary_uses_structured_output_for_new_sdk_envelope()
{
  let result = render_summary( NEW_SDK_ENVELOPE_STRUCTURED, None );
  assert!(
    result.is_some(),
    "render_summary must return Some for new SDK envelope with empty result; got None"
  );
  let s = result.unwrap();
  assert!(
    s.contains( "schema_data" ),
    "render_summary body must contain the structured_output value when result is empty; \
     got blank body (extract_structured_output returned None — BUG-438 gate not fixed). \
     Got:\n{s}"
  );
}
