//! Unix-only integration tests.
#![ cfg( unix ) ]
//! `--output-format` Integration Tests
//!
//! Covers EC-1 through EC-14 from `tests/docs/cli/param/061_output_format.md`.

mod cli_binary_test_helpers;
use cli_binary_test_helpers::{ run_cli, run_cli_with_env, fake_claude_dir, run_with_path, stderr_str, stdout_str };

// ── EC-1: --output-format json → forwarded to assembled command ───────────────

/// EC-1: `--output-format json` appears in the assembled command.
#[ test ]
fn ec1_output_format_json_forwarded()
{
  let out = run_cli( &[ "--dry-run", "--output-format", "json", "Fix bug" ] );
  assert!( out.status.success(), "exit must be 0: {out:?}" );
  let stdout = String::from_utf8_lossy( &out.stdout );
  assert!(
    stdout.contains( "--output-format" ),
    "assembled command must contain --output-format. Got:\n{stdout}"
  );
  assert!(
    stdout.contains( "json" ),
    "assembled command must contain the value json. Got:\n{stdout}"
  );
}

// ── EC-2: --output-format without value → exit 1 ─────────────────────────────

/// EC-2: `--output-format` without a value → exit 1 with a missing-value error.
#[ test ]
fn ec2_output_format_missing_value()
{
  let out = run_cli( &[ "--output-format" ] );
  assert_eq!(
    out.status.code(),
    Some( 1 ),
    "exit must be 1 when --output-format has no value: {out:?}"
  );
}

// ── EC-3: --output-format at end of argv → exit 1 ────────────────────────────

/// EC-3: `--output-format` at end of argv → exit 1 (missing value).
#[ test ]
fn ec3_output_format_at_end_of_argv()
{
  let out = run_cli( &[ "Fix bug", "--output-format" ] );
  assert_eq!(
    out.status.code(),
    Some( 1 ),
    "exit must be 1 when --output-format appears at end of argv: {out:?}"
  );
}

// ── EC-4: --output-format text → forwarded ────────────────────────────────────

/// EC-4: `--output-format text` appears in the assembled command.
#[ test ]
fn ec4_output_format_text_forwarded()
{
  let out = run_cli( &[ "--dry-run", "--output-format", "text", "Fix bug" ] );
  assert!( out.status.success(), "exit must be 0: {out:?}" );
  let stdout = String::from_utf8_lossy( &out.stdout );
  assert!(
    stdout.contains( "--output-format" ),
    "assembled command must contain --output-format. Got:\n{stdout}"
  );
  assert!(
    stdout.contains( "text" ),
    "assembled command must contain the value text. Got:\n{stdout}"
  );
}

// ── EC-5: --output-format stream-json → forwarded ────────────────────────────

/// EC-5: `--output-format stream-json` appears in the assembled command.
#[ test ]
fn ec5_output_format_stream_json_forwarded()
{
  let out = run_cli( &[ "--dry-run", "--output-format", "stream-json", "Fix bug" ] );
  assert!( out.status.success(), "exit must be 0: {out:?}" );
  let stdout = String::from_utf8_lossy( &out.stdout );
  assert!(
    stdout.contains( "--output-format" ),
    "assembled command must contain --output-format. Got:\n{stdout}"
  );
  assert!(
    stdout.contains( "stream-json" ),
    "assembled command must contain the value stream-json. Got:\n{stdout}"
  );
}

// ── EC-6: `--help` lists `--output-format` ────────────────────────────────────

/// EC-6: `clr --help` output contains `--output-format`.
#[ test ]
fn ec6_output_format_help_listed()
{
  let out = run_cli( &[ "--help" ] );
  assert!( out.status.success(), "clr --help must exit 0" );
  let stdout = String::from_utf8_lossy( &out.stdout );
  assert!(
    stdout.contains( "--output-format" ),
    "`clr --help` must list --output-format. Got:\n{stdout}"
  );
}

// ── EC-7: Without --output-format → runner auto-injects json for summary mode ─

/// EC-7: Without `--output-format`, print-mode dry-run shows `--output-format json`
/// because the runner auto-injects it via Path B for default summary rendering.
#[ test ]
fn ec7_output_format_absent_auto_injected_in_summary_mode()
{
  let out = run_cli( &[ "--dry-run", "Fix bug" ] );
  assert!( out.status.success(), "exit must be 0: {out:?}" );
  let stdout = String::from_utf8_lossy( &out.stdout );
  // Path B in builder.rs auto-injects --output-format json when output_style is summary
  // (the default) and no explicit --output-format flag is present.
  assert!(
    stdout.contains( "--output-format" ),
    "assembled command must contain auto-injected --output-format. Got:\n{stdout}"
  );
  assert!(
    stdout.contains( "json" ),
    "assembled command must contain auto-injected json value. Got:\n{stdout}"
  );
}

// ── EC-8: CLR_OUTPUT_FORMAT=json env var → forwarded ─────────────────────────

/// EC-8: `CLR_OUTPUT_FORMAT=json` env var causes `--output-format json` to appear.
#[ test ]
fn ec8_output_format_env_var_forwarded()
{
  let out = run_cli_with_env(
    &[ "--dry-run", "Fix bug" ],
    &[ ( "CLR_OUTPUT_FORMAT", "json" ) ],
  );
  assert!( out.status.success(), "exit must be 0: {out:?}" );
  let stdout = String::from_utf8_lossy( &out.stdout );
  assert!(
    stdout.contains( "--output-format" ),
    "assembled command must contain --output-format from env var. Got:\n{stdout}"
  );
  assert!(
    stdout.contains( "json" ),
    "assembled command must contain the value json. Got:\n{stdout}"
  );
}

// ── EC-9: --output-format summary dry-run → --output-format json in command ───

/// EC-9: `--output-format summary` is intercepted by the runner; assembled command
/// contains `--output-format json` (NOT `summary`).
#[ test ]
fn ec9_output_format_summary_forwarded_as_json()
{
  let out = run_cli( &[ "--dry-run", "--output-format", "summary", "Fix bug" ] );
  assert!( out.status.success(), "exit must be 0: {out:?}" );
  let stdout = String::from_utf8_lossy( &out.stdout );
  assert!(
    stdout.contains( "--output-format" ),
    "assembled command must contain --output-format. Got:\n{stdout}"
  );
  assert!(
    stdout.contains( "json" ),
    "assembled command must contain json (not summary) when summary is requested. Got:\n{stdout}"
  );
  // "summary" must NOT appear as the forwarded value (it's runner-intercepted)
  // Note: check for " summary" (with space) to avoid false match on e.g. "stream-json summary" prefix
  assert!(
    !stdout.contains( "--output-format summary" ),
    "assembled command must NOT forward --output-format summary verbatim. Got:\n{stdout}"
  );
}

// ── EC-12: CLR_OUTPUT_FORMAT=summary env var → forwarded as json ─────────────

/// EC-12: `CLR_OUTPUT_FORMAT=summary` env var → assembled command contains
/// `--output-format json` (NOT `summary`); interception applies to env var path too.
#[ test ]
fn ec12_output_format_summary_env_var_forwarded_as_json()
{
  let out = run_cli_with_env(
    &[ "--dry-run", "Fix bug" ],
    &[ ( "CLR_OUTPUT_FORMAT", "summary" ) ],
  );
  assert!( out.status.success(), "exit must be 0: {out:?}" );
  let stdout = String::from_utf8_lossy( &out.stdout );
  assert!(
    stdout.contains( "--output-format" ),
    "assembled command must contain --output-format from env var. Got:\n{stdout}"
  );
  assert!(
    stdout.contains( "json" ),
    "assembled command must contain json (not summary) when CLR_OUTPUT_FORMAT=summary. Got:\n{stdout}"
  );
  assert!(
    !stdout.contains( "--output-format summary" ),
    "assembled command must NOT forward --output-format summary verbatim. Got:\n{stdout}"
  );
}

// ── EC-10: summary mode → CLR envelope header in stdout ──────────────────────

/// EC-10: `--output-format summary` with fake claude CLR envelope → stdout contains
/// summary header with `session_id:`, `input_tokens:`, `is_error:` and `---` separator.
#[ cfg( unix ) ]
#[ test ]
fn ec10_output_format_summary_yaml_header()
{
  let json = r#"{"type":"result","subtype":"success","session_id":"00000000-0000-0000-0000-000000000001","is_error":false,"result":"hello","usage":{"input_tokens":1,"output_tokens":1},"total_cost_usd":0.0}"#;
  let ( _dir, path ) = fake_claude_dir( &format!( "echo '{json}'" ) );
  let out = run_with_path(
    &[ "--output-format", "summary", "--max-sessions", "0", "msg" ],
    &path,
  );
  assert!( out.status.success(), "exit must be 0: {out:?}" );
  let stdout = stdout_str( &out );
  assert!( stdout.contains( "session_id" ),    "stdout must contain 'session_id:'. Got:\n{stdout}" );
  assert!( stdout.contains( "input_tokens" ),  "stdout must contain 'input_tokens:'. Got:\n{stdout}" );
  assert!( stdout.contains( "is_error" ),      "stdout must contain 'is_error:'. Got:\n{stdout}" );
  assert!(
    stdout.contains( "---" ),
    "stdout must contain '---' separator. Got:\n{stdout}"
  );
}

// ── EC-11: summary mode → text body after separator ──────────────────────────

/// EC-11: `--output-format summary` with fake claude CLR envelope → stdout contains
/// the `result` field value (`hello`) after the `---` separator.
#[ cfg( unix ) ]
#[ test ]
fn ec11_output_format_summary_text_body()
{
  let json = r#"{"type":"result","subtype":"success","session_id":"00000000-0000-0000-0000-000000000001","is_error":false,"result":"hello","usage":{"input_tokens":1,"output_tokens":1},"total_cost_usd":0.0}"#;
  let ( _dir, path ) = fake_claude_dir( &format!( "echo '{json}'" ) );
  let out = run_with_path(
    &[ "--output-format", "summary", "--max-sessions", "0", "msg" ],
    &path,
  );
  assert!( out.status.success(), "exit must be 0: {out:?}" );
  let stdout = stdout_str( &out );
  assert!(
    stdout.contains( "hello" ),
    "stdout must contain extracted text 'hello'. Got:\n{stdout}"
  );
  if let Some( sep ) = stdout.find( "---" )
  {
    assert!(
      stdout[ sep.. ].contains( "hello" ),
      "text 'hello' must appear after the --- separator. Got:\n{stdout}"
    );
  }
}

// ── EC-13: summary mode → error envelope ─────────────────────────────────────

/// EC-13: CLR envelope with `is_error: true` → summary header shows `is_error:` and
/// `subtype:`; error message appears in text body after `---`.
#[ cfg( unix ) ]
#[ test ]
fn ec13_output_format_summary_error_envelope()
{
  let json = r#"{"type":"result","subtype":"error","session_id":"00000000-0000-0000-0000-000000000002","is_error":true,"result":"Something went wrong","usage":{"input_tokens":2,"output_tokens":0},"total_cost_usd":0.0}"#;
  let ( _dir, path ) = fake_claude_dir( &format!( "echo '{json}'" ) );
  let out = run_with_path(
    &[ "--output-format", "summary", "--max-sessions", "0", "msg" ],
    &path,
  );
  assert!( out.status.success(), "exit must be 0: {out:?}" );
  let stdout = stdout_str( &out );
  assert!( stdout.contains( "is_error" ), "stdout must contain 'is_error'. Got:\n{stdout}" );
  assert!( stdout.contains( "subtype" ),  "stdout must contain 'subtype'. Got:\n{stdout}" );
  assert!(
    stdout.contains( "Something went wrong" ),
    "stdout must contain error message in text body. Got:\n{stdout}"
  );
}

// ── EC-14: claude non-zero exit with summary → raw stderr, no header ─────────

/// EC-14: When claude exits non-zero with `summary` mode, stderr is preserved,
/// no summary header appears in stdout, and the exit code propagates.
#[ cfg( unix ) ]
#[ test ]
fn ec14_output_format_summary_nonzero_exit_passthrough()
{
  let body = "echo 'Error: rate limit' >&2\nexit 2";
  let ( _dir, path ) = fake_claude_dir( body );
  let out = run_with_path(
    &[ "--output-format", "summary", "--max-sessions", "0", "--retry-on-transient", "0", "msg" ],
    &path,
  );
  assert_eq!(
    out.status.code(),
    Some( 2 ),
    "exit must be 2 (propagated from fake claude). Got: {out:?}"
  );
  let stderr = stderr_str( &out );
  assert!(
    stderr.contains( "rate limit" ),
    "stderr must contain 'rate limit'. Got:\n{stderr}"
  );
  let stdout = stdout_str( &out );
  assert!(
    !stdout.contains( "session_id" ) && !stdout.contains( "---" ),
    "stdout must NOT contain summary header on non-zero exit. Got:\n{stdout}"
  );
}
