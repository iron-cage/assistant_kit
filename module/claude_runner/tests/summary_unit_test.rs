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
/// Compound gate: `if subtype.is_none() && msg_type != "result" { return None; }` where
/// `subtype = extract_str(json,"subtype")` and `msg_type = extract_str(json,"type")` —
/// accepts old SDK (`"type":"result"`, no `"subtype"`) OR new SDK (`"subtype"` present,
/// no top-level `"type"`).  `"subtype"` is top-level-only, never in `usage.iterations[]`.
///
/// # Prevention
/// After any gate change, verify both SDK variants produce `Some`: one fixture with
/// `"subtype"` but no top-level `"type"`, one with `"type":"result"` but no `"subtype"`.
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
/// Compound gate: `let is_result = extract_str(json,"subtype").is_some() || extract_str(json,"type").as_deref() == Some("result"); if !is_result { return None; }` —
/// accepts old SDK (`"type":"result"`, no `"subtype"`) OR new SDK (`"subtype"` present).
/// A subtype-only gate breaks old SDK envelopes where `"subtype"` is absent.
///
/// # Prevention
/// Silent-failure gates must have positive tests for BOTH SDK variants.  Add a fixture
/// with `"type":"result"` only (old SDK) alongside the new SDK fixture.
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
/// Compound gate in `extract_structured_output()`: same as BUG-436/437 —
/// `let is_result = extract_str(json,"subtype").is_some() || extract_str(json,"type").as_deref() == Some("result"); if !is_result { return None; }`.
///
/// # Prevention
/// When fixing an outer gate (BUG-436), audit all inner gates on the same call path for
/// the same root cause.  Add a test that reaches each inner gate for the new SDK format.
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

/// # Root Cause
/// `extract_str(json, "type")` at summary.rs:342 uses depth-unaware `s.find()`; for new SDK
/// envelopes with no top-level `"type"` field, it finds `usage.iterations[].type = "message"`.
/// After BUG-436's compound gate correctly passes the new SDK envelope, `msg_type = "message"`
/// reached the display logic at line 408 unchanged, producing `type: message` in the header.
///
/// # Why Not Caught
/// The BUG-436 test only asserted `is_some()` — did not check the string content. Display
/// corruption was invisible at the gate-correctness level.
///
/// # Fix Applied
/// One line inserted between the gate and the `subtype` rebind in `render_summary()`:
/// `let msg_type = if subtype.is_some() && msg_type != "result" { String::new() } else { msg_type };`
/// Clears `msg_type` when the new SDK path is taken (subtype present, no top-level "type":"result").
///
/// # Prevention
/// After any gate fix that admits a new envelope format, scan ALL downstream uses of the same
/// extracted fields in the same function for display-correctness regressions.
///
/// # Pitfall
/// `extract_str` is depth-unaware: any field that appears nested (e.g., `iterations[].type`)
/// will produce wrong values when the top-level field is absent and a nested one appears first.
// test_kind: bug_reproducer(BUG-440)
#[ test ]
fn render_summary_does_not_display_type_message_for_new_sdk_envelope()
{
  let result = render_summary( NEW_SDK_ENVELOPE, None );
  assert!(
    result.is_some(),
    "render_summary must return Some for new SDK envelope; got None"
  );
  let s = result.unwrap();
  assert!(
    !s.contains( "message" ),
    "render_summary must NOT contain 'message' in output for new SDK envelope — \
     'type: message' comes from depth-unaware extract_str finding iterations[].type \
     before absent top-level 'type' field (BUG-440). ANSI codes separate 'type:' \
     and 'message' in the rendered string but 'message' itself must not appear. \
     Got:\n{s}"
  );
}

/// Envelope where `usage.iterations[]` is serialized BEFORE `usage`'s own scalar fields —
/// the ordering BUG-439 identifies as latent-but-real (JSON object field order is
/// unspecified per RFC 8259 §4; the SDK currently happens to order scalars first).
/// `iterations[0]` carries per-iteration values (111/222/333/444) distinct from the
/// correct totals (9001/9002/9003/9004) at `usage`'s own top level, so a depth-unaware
/// first-occurrence extraction is observably wrong rather than accidentally correct.
const NEW_SDK_ENVELOPE_REORDERED_USAGE : &str = r#"{"subtype":"success","session_id":"00000000-0000-0000-0000-000000000001","is_error":false,"duration_ms":100,"duration_api_ms":90,"num_turns":1,"result":"hello","stop_reason":"end_turn","total_cost_usd":0.001,"uuid":"00000000-0000-0000-0000-000000000002","fast_mode_state":"off","usage":{"iterations":[{"input_tokens":111,"output_tokens":222,"cache_read_input_tokens":333,"cache_creation_input_tokens":444,"type":"message"}],"input_tokens":9001,"output_tokens":9002,"cache_creation_input_tokens":9004,"cache_read_input_tokens":9003,"service_tier":"standard","speed":"standard","inference_geo":"","server_tool_use":{"web_search_requests":0,"web_fetch_requests":0},"cache_creation":{"ephemeral_1h_input_tokens":0,"ephemeral_5m_input_tokens":0}},"modelUsage":{"claude-opus-4-8":{"inputTokens":3,"outputTokens":4,"cacheReadInputTokens":0,"cacheCreationInputTokens":0,"webSearchRequests":0,"costUSD":0.001,"contextWindow":200000,"maxOutputTokens":32000}},"permission_denials":[]}"#;

/// BUG-439 regression: `render_summary()` must extract `usage.*` totals correctly even
/// when the SDK serializes `usage.iterations[]` BEFORE `usage`'s own scalar fields.
///
/// # Root Cause
/// `usage_str` (summary.rs) is built via `s.find("\"usage\":{")` and was consumed with
/// `extract_u64`/`extract_str`, both depth-unaware first-occurrence searches. Extraction
/// was correct only because the current SDK serializes `usage.*` scalars BEFORE
/// `usage.iterations[]`; nothing enforced that order.
///
/// # Why Not Caught
/// All prior fixtures (`FULL_ENVELOPE`, `NEW_SDK_ENVELOPE`, …) serialize `usage.iterations`
/// LAST within `usage`, matching the SDK's current (but unspecified) field order — so the
/// depth-unaware search always happened to land on the correct top-level field first.
///
/// # Fix Applied
/// Added `find_key_shallow()` — a bracket-depth-tracking key search that only matches a
/// key at depth 0 relative to the object being searched, and stops the scan the moment
/// depth would go negative (naturally bounding the search to the enclosing object without
/// requiring a pre-trimmed slice). `extract_u64_shallow`/`extract_str_shallow` use it in
/// place of `extract_u64`/`extract_str` for every field pulled from `usage_str`/`stu_str`/
/// `cc_str`; `nested_object_shallow` replaces the marker-based `stu_str`/`cc_str` location
/// with the same depth-0 search.
///
/// # Prevention
/// Any consumer of an unbounded suffix slice (`&json[pos..]`) that performs first-occurrence
/// key search must use a depth-aware/bounded search, not `.find()`, whenever the slice can
/// contain a nested object/array carrying a same-named field (e.g. `usage.iterations[]`).
///
/// # Pitfall
/// A depth-unaware search over an object that has a same-named field nested inside one of
/// its own children is correct only by accident of the current serializer's field order —
/// JSON object field order is unspecified (RFC 8259 §4) and may change without notice.
// test_kind: bug_reproducer(BUG-439)
#[ test ]
fn render_summary_uses_usage_totals_when_iterations_precedes_scalars()
{
  let result = render_summary( NEW_SDK_ENVELOPE_REORDERED_USAGE, None );
  assert!( result.is_some(), "render_summary must return Some; got None" );
  let s = result.unwrap();
  assert!(
    s.contains( "input_tokens:\u{1b}[0m \u{1b}[33m9001\u{1b}[0m" ),
    "input_tokens must show the usage-level total (9001), not iterations[0]'s value (111) \
     — depth-unaware extraction found iterations[0] first. Got:\n{s}"
  );
  assert!(
    s.contains( "output_tokens:\u{1b}[0m \u{1b}[33m9002\u{1b}[0m" ),
    "output_tokens must show the usage-level total (9002), not iterations[0]'s value (222). \
     Got:\n{s}"
  );
  assert!(
    s.contains( "cache_creation_input_tokens:\u{1b}[0m \u{1b}[33m9004\u{1b}[0m" ),
    "cache_creation_input_tokens must show the usage-level total (9004), not iterations[0]'s \
     value (444). Got:\n{s}"
  );
  assert!(
    s.contains( "cache_read_input_tokens:\u{1b}[0m \u{1b}[33m9003\u{1b}[0m" ),
    "cache_read_input_tokens must show the usage-level total (9003), not iterations[0]'s \
     value (333). Got:\n{s}"
  );
}

// ── BUG-442: permission_denials array boundary ──────────────────────────────

/// # Root Cause
/// `count_permission_denials()` (summary.rs) located the `permission_denials` array's
/// closing bracket via a single unbounded `rest.find(']')` call — the first `]`
/// character anywhere in the remainder of the JSON, with no bracket-depth tracking and
/// no string-literal awareness. A denial entry's own `reason`/`tool` string field
/// containing a literal `]` (e.g. quoting a blocked array-index term) was mistaken for
/// the array's own closing bracket, truncating the scan before the array's true end.
///
/// # Why Not Caught
/// The defect only manifests when a `permission_denials` entry's own text content
/// contains a literal `]` — most denial reasons/tool names don't, so ordinary use
/// rarely exercised this path. The miscounted result is a syntactically valid `u64`
/// with no error signal, indistinguishable from a genuine lower count.
///
/// # Fix Applied
/// `count_permission_denials()` now tracks bracket depth and string-literal state
/// (quote toggling with backslash-escape lookahead) while scanning forward from the
/// array's opening `[`, stopping only at the `]` that returns depth to zero outside
/// any string.
///
/// # Prevention
/// This test pins a 2-entry `permission_denials` array where the first entry's own
/// `reason` field contains an in-content `]` ahead of the array's real close — any
/// regression that drops depth/string-literal tracking truncates the scan at that
/// in-content `]`, undercounting 2 entries as 1.
///
/// # Pitfall
/// Counting delimiter-separated entries in a JSON array by scanning for the first
/// closing bracket, without tracking bracket depth or string-literal state, silently
/// truncates the count whenever any entry's own content contains that same
/// closing-bracket character.
// test_kind: bug_reproducer(BUG-442)
#[ test ]
fn bug442_permission_denials_in_content_bracket_does_not_truncate_count()
{
  // Entry 1's own "reason" value contains a literal, unescaped ']' — valid JSON, since
  // '[' / ']' need no escaping inside a string. Before the fix, count_permission_denials()
  // truncates its scan at that in-content ']', undercounting this genuine 2-entry array as 1.
  let json = r#"{"type":"result","subtype":"success","is_error":false,"duration_ms":1,"duration_api_ms":1,"num_turns":1,"result":"hello","permission_denials":[{"tool":"Bash","reason":"blocked: index [3] out of range"},{"tool":"Write","reason":"denied"}]}"#;

  let rendered = render_summary( json, None ).expect( "must parse" );

  assert!(
    rendered.contains( "permission_denials:\u{1b}[0m \u{1b}[33m2\u{1b}[0m" ),
    "BUG-442: permission_denials count must survive an in-content ']' in an entry's own \
     reason text — expected count 2 (both entries), got a different value. Got:\n{rendered}"
  );
}

// ── BUG-476 / BUG-477: modelUsage boundary + multi-entry aggregation ────────

/// # Root Cause
/// `render_summary()`'s `modelUsage` extraction located the `"modelUsage":{` marker and
/// scanned the entire remainder of the JSON string for the model name's two delimiting
/// quotes — never determining where the `modelUsage` object itself ends. With an empty
/// `"modelUsage":{}` (turn terminated before any model was dispatched), the unbounded
/// scan read past the object's closing brace and returned the next JSON key's name
/// (`permission_denials`) as the model name.
///
/// # Why Not Caught
/// Only manifests when `modelUsage` is empty — a turn that terminates before any model
/// is dispatched (session-limit/quota exhaustion before the first successful model
/// call). No prior fixture exercised an empty `"modelUsage":{}`, and the corrupted
/// output is a syntactically valid, populated-looking `model:` line with no error
/// signal.
///
/// # Fix Applied
/// `object_extent()` (summary.rs) bounds `mu_str` to `modelUsage`'s own closing brace
/// before any quote scan; an empty object yields no entries, so `model:` renders an
/// empty value and every `model_*` field stays at its 0 default.
///
/// # Prevention
/// This test pins an empty `"modelUsage":{}` envelope (the real Job #22 shape from the
/// bug report) and asserts the rendered `model:` value is empty — any regression back
/// to an unbounded scan resurfaces `permission_denials` here.
///
/// # Pitfall
/// Locating a JSON field by its opening marker alone and scanning forward for
/// delimiters with no closing-boundary check silently reads into sibling fields
/// whenever the field is empty — the scan doesn't fail, it returns wrong, unrelated
/// data with no error signal.
// test_kind: bug_reproducer(BUG-476)
#[ test ]
fn bug476_empty_modelusage_renders_empty_model_not_next_key()
{
  // Turn terminated before any model was dispatched (session-limit exhaustion before
  // the first successful model call): modelUsage is an EMPTY object and
  // permission_denials is the next key in the envelope schema — the exact Job #22
  // shape from the bug report.
  let json = r#"{"type":"result","subtype":"success","is_error":true,"num_turns":1,"duration_api_ms":0,"modelUsage":{},"permission_denials":[]}"#;

  let rendered = render_summary( json, None ).expect( "must parse" );

  assert!(
    rendered.contains( "model:\u{1b}[0m \u{1b}[32m\u{1b}[0m" ),
    "BUG-476: with an empty modelUsage object the model: line must render an empty \
     value, not data scavenged from past the object's closing brace. Got:\n{rendered}"
  );
  assert!(
    !rendered.contains( "\u{1b}[32mpermission_denials\u{1b}[0m" ),
    "BUG-476: the next JSON key's name (permission_denials) must never appear as the \
     model: value. Got:\n{rendered}"
  );
}

/// # Root Cause
/// `render_summary()`'s `modelUsage` extraction read only the first model entry — one
/// `.find('{')` with no loop — so every `model_*` field silently reflected only the
/// first entry when a turn dispatched two or more models, underreporting cost/tokens
/// by up to 4 orders of magnitude while the independently-extracted `total_cost_usd`
/// stayed correct.
///
/// # Why Not Caught
/// Only turns dispatching 2+ models in one envelope exercise the path; single-model
/// turns render correctly. The first-model-only scoping was disclosed in a source
/// comment but never in the rendered output labels, and no fixture carried a
/// multi-entry `modelUsage` object.
///
/// # Fix Applied
/// The extraction now walks every entry inside `modelUsage`'s own extent
/// (`object_extent()`-bounded) and sums the additive fields (`inputTokens`,
/// `outputTokens`, `cacheReadInputTokens`, `cacheCreationInputTokens`,
/// `webSearchRequests`, `costUSD`); `model:` and the per-model capability fields
/// (`contextWindow`, `maxOutputTokens`) stay with the first entry.
///
/// # Prevention
/// This test pins a 2-entry `modelUsage` mirroring run 3193 Job #1's real values and
/// asserts each rendered `model_*` total equals the sum across both entries — any
/// regression to first-entry-only extraction reports 506/$0.0006 instead.
///
/// # Pitfall
/// Treating "first entry found" as "the complete field" for a structurally-collection
/// JSON field silently drops every later entry — the output looks plausible because
/// it is internally consistent for the one entry that was read.
// test_kind: bug_reproducer(BUG-477)
#[ test ]
fn bug477_multi_entry_modelusage_aggregates_all_entries()
{
  // Two models dispatched in one turn — first entry (haiku) tiny, second (sonnet)
  // carries the bulk. Mirrors run 3193 Job #1: total_cost_usd 6.4133 = 0.0006 +
  // 6.4127; input tokens 36329 = 506 + 35823; output 24117 = 14 + 24103.
  let json = r#"{"type":"result","subtype":"success","total_cost_usd":6.4133,"modelUsage":{"claude-haiku-4-5-20251001":{"inputTokens":506,"outputTokens":14,"cacheReadInputTokens":0,"cacheCreationInputTokens":0,"webSearchRequests":0,"costUSD":0.0006,"contextWindow":200000,"maxOutputTokens":64000},"claude-sonnet-5":{"inputTokens":35823,"outputTokens":24103,"cacheReadInputTokens":1403813,"cacheCreationInputTokens":0,"webSearchRequests":0,"costUSD":6.4127,"contextWindow":200000,"maxOutputTokens":64000}},"permission_denials":[]}"#;

  let rendered = render_summary( json, None ).expect( "must parse" );

  assert!(
    rendered.contains( "model_input_tokens:\u{1b}[0m \u{1b}[33m36329\u{1b}[0m" ),
    "BUG-477: model_input_tokens must aggregate across both modelUsage entries \
     (506 + 35823 = 36329), not report the first entry alone. Got:\n{rendered}"
  );
  assert!(
    rendered.contains( "model_output_tokens:\u{1b}[0m \u{1b}[33m24117\u{1b}[0m" ),
    "BUG-477: model_output_tokens must aggregate across both entries \
     (14 + 24103 = 24117). Got:\n{rendered}"
  );
  assert!(
    rendered.contains( "model_cache_read_input_tokens:\u{1b}[0m \u{1b}[33m1403813\u{1b}[0m" ),
    "BUG-477: model_cache_read_input_tokens must include the second entry's 1403813. \
     Got:\n{rendered}"
  );
  assert!(
    rendered.contains( "model_cost_usd:\u{1b}[0m \u{1b}[33m6.4133\u{1b}[0m" ),
    "BUG-477: model_cost_usd must sum costUSD across both entries (0.0006 + 6.4127 = \
     6.4133), matching the envelope's own total_cost_usd. Got:\n{rendered}"
  );
  assert!(
    rendered.contains( "model:\u{1b}[0m \u{1b}[32mclaude-haiku-4-5-20251001\u{1b}[0m" ),
    "model: keeps the first entry's name (no meaningful single-value aggregate \
     exists). Got:\n{rendered}"
  );
  assert!(
    rendered.contains( "model_context_window:\u{1b}[0m \u{1b}[33m200000\u{1b}[0m" ),
    "model_context_window is a per-model capability — first entry's value, never a \
     sum. Got:\n{rendered}"
  );
}
