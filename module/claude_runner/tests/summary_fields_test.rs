#![ allow( missing_docs ) ]
#![ cfg( unix ) ]
//! `--summary-fields` Integration Tests (EC-01–EC-12)
//!
//! Covers EC-01 through EC-12 from `tests/docs/cli/param/071_summary_fields.md`.

mod cli_binary_test_helpers;
use cli_binary_test_helpers::{ fake_claude_dir, run_cli, run_cli_with_env };

const FULL_FIXTURE : &str = r#"{"type":"result","subtype":"success","session_id":"00000000-0000-0000-0000-000000000001","is_error":false,"duration_ms":100,"duration_api_ms":90,"num_turns":1,"result":"hello","stop_reason":"end_turn","total_cost_usd":0.001,"uuid":"00000000-0000-0000-0000-000000000002","fast_mode_state":"off","usage":{"input_tokens":3,"output_tokens":4,"cache_creation_input_tokens":0,"cache_read_input_tokens":0,"service_tier":"standard","speed":"standard","inference_geo":"","server_tool_use":{"web_search_requests":0,"web_fetch_requests":0},"cache_creation":{"ephemeral_1h_input_tokens":0,"ephemeral_5m_input_tokens":0},"iterations":[]},"modelUsage":{"claude-opus-4-6":{"inputTokens":3,"outputTokens":4,"cacheReadInputTokens":0,"cacheCreationInputTokens":0,"webSearchRequests":0,"costUSD":0.001,"contextWindow":200000,"maxOutputTokens":32000}},"permission_denials":[]}"#;

/// Run `clr` with a fake claude that emits the full CLR envelope fixture.
///
/// Removes `CLR_OUTPUT_STYLE` and `CLR_SUMMARY_FIELDS` from the subprocess env so
/// ambient host state does not affect tests.  Pass `extra_envs` to inject env vars
/// after the remove (can re-set either for env-var-specific test cases).
fn run_full_claude( args : &[ &str ], extra_envs : &[ ( &str, &str ) ] ) -> std::process::Output
{
  let body = format!( "echo '{FULL_FIXTURE}'" );
  let ( _dir, path ) = fake_claude_dir( &body );
  let bin = env!( "CARGO_BIN_EXE_clr" );
  std::process::Command::new( bin )
    .args( args )
    .env( "PATH", &path )
    .env_remove( "CLR_OUTPUT_STYLE" )
    .env_remove( "CLR_SUMMARY_FIELDS" )
    .envs( extra_envs.iter().copied() )
    .output()
    .expect( "Failed to invoke clr binary" )
}

// ── EC-01: Default full — all 32 header fields present ─────────────────────────

/// EC-01: Without `--summary-fields` or `CLR_SUMMARY_FIELDS`, all 32 header fields
/// are present in the summary output (default profile is `full`).
#[ test ]
fn ec01_default_full_all_fields_present()
{
  let out = run_full_claude( &[ "-p", "--max-sessions", "0", "x" ], &[] );
  assert!( out.status.success(), "exit must be 0: {out:?}" );
  let s = String::from_utf8_lossy( &out.stdout );
  assert!( s.contains( "---" ), "must have --- separator. Got:\n{s}" );
  for f in [ "duration_ms:", "uuid:", "model:", "permission_denials:" ]
  {
    assert!( s.contains( f ), "full profile must contain '{f}'. Got:\n{s}" );
  }
}

// ── EC-02: Explicit --summary-fields full — same as default ────────────────────

/// EC-02: Explicit `--summary-fields full` behaves identically to the default.
#[ test ]
fn ec02_explicit_full_same_as_default()
{
  let out = run_full_claude(
    &[ "-p", "--max-sessions", "0", "--summary-fields", "full", "x" ],
    &[],
  );
  assert!( out.status.success(), "exit must be 0: {out:?}" );
  let s = String::from_utf8_lossy( &out.stdout );
  for f in [ "duration_ms:", "model:" ]
  {
    assert!( s.contains( f ), "explicit full must contain '{f}'. Got:\n{s}" );
  }
}

// ── EC-03: --summary-fields minimal — 7 fields only ───────────────────────────

/// EC-03: `--summary-fields minimal` shows only the 7 minimal-profile fields.
#[ test ]
fn ec03_minimal_seven_fields_only()
{
  let out = run_full_claude(
    &[ "-p", "--max-sessions", "0", "--summary-fields", "minimal", "x" ],
    &[],
  );
  assert!( out.status.success(), "exit must be 0: {out:?}" );
  let s = String::from_utf8_lossy( &out.stdout );
  for f in
  [
    "type:", "subtype:", "session_id:", "is_error:",
    "input_tokens:", "output_tokens:", "total_cost_usd:",
  ]
  {
    assert!( s.contains( f ), "minimal must contain '{f}'. Got:\n{s}" );
  }
  for f in [ "duration_ms:", "uuid:", "model:", "stop_reason:" ]
  {
    assert!( !s.contains( f ), "minimal must NOT contain '{f}'. Got:\n{s}" );
  }
}

// ── EC-04: --summary-fields standard — 14 fields ──────────────────────────────

/// EC-04: `--summary-fields standard` shows the 14 standard-profile fields.
#[ test ]
fn ec04_standard_fourteen_fields()
{
  let out = run_full_claude(
    &[ "-p", "--max-sessions", "0", "--summary-fields", "standard", "x" ],
    &[],
  );
  assert!( out.status.success(), "exit must be 0: {out:?}" );
  let s = String::from_utf8_lossy( &out.stdout );
  for f in
  [
    "stop_reason:", "num_turns:", "duration_ms:",
    "cache_creation_input_tokens:", "service_tier:", "model:",
  ]
  {
    assert!( s.contains( f ), "standard must contain '{f}'. Got:\n{s}" );
  }
  for f in [ "uuid:", "fast_mode_state:", "duration_api_ms:", "model_context_window:" ]
  {
    assert!( !s.contains( f ), "standard must NOT contain '{f}'. Got:\n{s}" );
  }
}

// ── EC-05: Custom whitelist — only specified fields ────────────────────────────

/// EC-05: Custom field list `type,session_id,total_cost_usd` shows only those 3 fields.
#[ test ]
fn ec05_custom_whitelist_three_fields()
{
  let out = run_full_claude(
    &[ "-p", "--max-sessions", "0", "--summary-fields", "type,session_id,total_cost_usd", "x" ],
    &[],
  );
  assert!( out.status.success(), "exit must be 0: {out:?}" );
  let s = String::from_utf8_lossy( &out.stdout );
  for f in [ "type:", "session_id:", "total_cost_usd:" ]
  {
    assert!( s.contains( f ), "custom must contain '{f}'. Got:\n{s}" );
  }
  for f in [ "subtype:", "is_error:", "input_tokens:", "duration_ms:" ]
  {
    assert!( !s.contains( f ), "custom must NOT contain '{f}'. Got:\n{s}" );
  }
}

// ── EC-06: --output-style raw ignores --summary-fields ─────────────────────────

/// EC-06: `--output-style raw` bypasses `render_summary()`; `--summary-fields` silently ignored.
#[ test ]
fn ec06_raw_style_ignores_summary_fields()
{
  let out = run_full_claude(
    &[ "-p", "--max-sessions", "0", "--output-style", "raw", "--summary-fields", "minimal", "x" ],
    &[],
  );
  assert!( out.status.success(), "exit must be 0: {out:?}" );
  let s = String::from_utf8_lossy( &out.stdout );
  assert!( !s.contains( "---" ), "raw style must NOT produce --- separator. Got:\n{s}" );
}

// ── EC-07: Invalid profile name → exit 1 ──────────────────────────────────────

/// EC-07: `--summary-fields bogus` is rejected with exit 1 and validation message.
#[ test ]
fn ec07_invalid_profile_rejected()
{
  let out = run_cli( &[ "--summary-fields", "bogus" ] );
  assert_eq!( out.status.code(), Some( 1 ), "exit must be 1: {out:?}" );
  let e = String::from_utf8_lossy( &out.stderr );
  assert!(
    e.contains( "invalid summary-fields 'bogus'" ),
    "stderr must name the invalid profile. Got:\n{e}"
  );
}

// ── EC-08: Custom with unknown field → exit 1 ─────────────────────────────────

/// EC-08: `--summary-fields type,nonexistent_field` is rejected — stderr names the unknown field.
#[ test ]
fn ec08_unknown_custom_field_rejected()
{
  let out = run_cli( &[ "--summary-fields", "type,nonexistent_field" ] );
  assert_eq!( out.status.code(), Some( 1 ), "exit must be 1: {out:?}" );
  let e = String::from_utf8_lossy( &out.stderr );
  assert!(
    e.contains( "unknown field 'nonexistent_field'" ),
    "stderr must name the unknown field. Got:\n{e}"
  );
}

// ── EC-09: CLR_SUMMARY_FIELDS=minimal env var — 7 fields ──────────────────────

/// EC-09: `CLR_SUMMARY_FIELDS=minimal` env var applies when no `--summary-fields` flag present.
#[ test ]
fn ec09_env_var_minimal()
{
  let out = run_full_claude(
    &[ "-p", "--max-sessions", "0", "x" ],
    &[ ( "CLR_SUMMARY_FIELDS", "minimal" ) ],
  );
  assert!( out.status.success(), "exit must be 0: {out:?}" );
  let s = String::from_utf8_lossy( &out.stdout );
  assert!(
    !s.contains( "duration_ms:" ),
    "minimal via env must NOT contain 'duration_ms:'. Got:\n{s}"
  );
  for f in [ "type:", "total_cost_usd:" ]
  {
    assert!( s.contains( f ), "minimal via env must contain '{f}'. Got:\n{s}" );
  }
}

// ── EC-10: CLR_SUMMARY_FIELDS=minimal + --summary-fields full → flag wins ─────

/// EC-10: CLI flag `--summary-fields full` overrides `CLR_SUMMARY_FIELDS=minimal`.
#[ test ]
fn ec10_cli_flag_wins_over_env_var()
{
  let out = run_full_claude(
    &[ "-p", "--max-sessions", "0", "--summary-fields", "full", "x" ],
    &[ ( "CLR_SUMMARY_FIELDS", "minimal" ) ],
  );
  assert!( out.status.success(), "exit must be 0: {out:?}" );
  let s = String::from_utf8_lossy( &out.stdout );
  for f in [ "duration_ms:", "model:" ]
  {
    assert!(
      s.contains( f ),
      "CLI flag full must override env minimal — must contain '{f}'. Got:\n{s}"
    );
  }
}

// ── EC-11: CLR_SUMMARY_FIELDS=bogus → exit 1 ─────────────────────────────────

/// EC-11: Invalid `CLR_SUMMARY_FIELDS` env var is rejected with exit 1.
#[ test ]
fn ec11_env_var_bogus_rejected()
{
  let out = run_cli_with_env(
    &[ "x" ],
    &[ ( "CLR_SUMMARY_FIELDS", "bogus" ) ],
  );
  assert_eq!( out.status.code(), Some( 1 ), "exit must be 1: {out:?}" );
  let e = String::from_utf8_lossy( &out.stderr );
  assert!(
    e.contains( "CLR_SUMMARY_FIELDS: invalid value 'bogus'" ),
    "stderr must contain env var validation message. Got:\n{e}"
  );
}

// ── EC-12: --summary-fields minimal — result text body always rendered ─────────

/// EC-12: Even with `minimal` profile, the result text body after `---` is always rendered.
#[ test ]
fn ec12_minimal_result_body_always_rendered()
{
  let out = run_full_claude(
    &[ "-p", "--max-sessions", "0", "--summary-fields", "minimal", "x" ],
    &[],
  );
  assert!( out.status.success(), "exit must be 0: {out:?}" );
  let s = String::from_utf8_lossy( &out.stdout );
  assert!( s.contains( "---" ), "minimal must have --- separator. Got:\n{s}" );
  assert!( s.contains( "hello" ), "result body 'hello' must appear after ---. Got:\n{s}" );
}
