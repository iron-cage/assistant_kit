#![ cfg( unix ) ]
#![ cfg_attr( not( unix ), allow( missing_docs ) ) ]
//! `--output-style` Integration Tests (EC-01–EC-14, IT-7)
//!
//! Covers EC-01 through EC-14 from `tests/docs/cli/param/070_output_style.md`;
//! IT-7 (structural anti-pattern guard) from `tests/docs/invariant/08_render_summary_gate.md`.

mod cli_binary_test_helpers;
use cli_binary_test_helpers::{ fake_claude_dir, run_cli, run_cli_with_env };

const JSON_FIXTURE : &str = r#"{"type":"result","subtype":"success","session_id":"00000000-0000-0000-0000-000000000001","is_error":false,"result":"hello","usage":{"input_tokens":1,"output_tokens":1},"total_cost_usd":0.0}"#;

/// Minimal 7-field CLR envelope — no `session_id`, no `usage`, no `total_cost_usd`.
/// Used by EC-14 to verify BUG-310 fix: `render_summary()` must gate on `type=="result"`,
/// not on the optional `session_id` field.
const MINIMAL_ENVELOPE : &str = r#"{"type":"result","subtype":"success","is_error":false,"duration_ms":1000,"duration_api_ms":900,"num_turns":1,"result":"hello"}"#;

/// Run `clr` with a fake claude that emits the JSON fixture.
///
/// Removes `CLR_OUTPUT_STYLE` from the subprocess env so ambient host state does not
/// affect tests.  Pass `extra_envs` to inject env vars after the remove (can re-set
/// `CLR_OUTPUT_STYLE` for EC-04 and EC-11).
fn run_json_claude( args : &[ &str ], extra_envs : &[ ( &str, &str ) ] ) -> std::process::Output
{
  let body = format!( "echo '{JSON_FIXTURE}'" );
  let ( _dir, path ) = fake_claude_dir( &body );
  let bin = env!( "CARGO_BIN_EXE_clr" );
  std::process::Command::new( bin )
    .args( args )
    .env( "PATH", &path )
    .env_remove( "CLR_OUTPUT_STYLE" )
    .envs( extra_envs.iter().copied() )
    .output()
    .expect( "Failed to invoke clr binary" )
}

/// Run `clr` with a fake claude that emits the minimal 7-field CLR envelope (no `session_id`).
///
/// Used for EC-14 to verify BUG-310 fix: `render_summary()` gate on `type=="result"` handles
/// absent `session_id` via `.unwrap_or_default()` instead of propagating `None`.
fn run_minimal_claude( args : &[ &str ] ) -> std::process::Output
{
  let body = format!( "echo '{MINIMAL_ENVELOPE}'" );
  let ( _dir, path ) = fake_claude_dir( &body );
  let bin = env!( "CARGO_BIN_EXE_clr" );
  std::process::Command::new( bin )
    .args( args )
    .env( "PATH", &path )
    .env_remove( "CLR_OUTPUT_STYLE" )
    .output()
    .expect( "Failed to invoke clr binary" )
}

/// Run `clr` with a fake claude that emits plain text (`echo hello`).
///
/// Used for EC-05 and EC-13 where `--output-format text`/`stream-json` is forwarded to
/// claude, causing `render_summary()` to receive non-JSON input and return `None`.
fn run_text_claude( args : &[ &str ] ) -> std::process::Output
{
  let ( _dir, path ) = fake_claude_dir( "echo hello" );
  let bin = env!( "CARGO_BIN_EXE_clr" );
  std::process::Command::new( bin )
    .args( args )
    .env( "PATH", &path )
    .env_remove( "CLR_OUTPUT_STYLE" )
    .output()
    .expect( "Failed to invoke clr binary" )
}

// ── EC-01: Default output-style is summary — stdout contains --- ─────────────

/// EC-01: Without any `--output-style` flag or `CLR_OUTPUT_STYLE`, print mode invokes
/// `render_summary()` and stdout contains the `---` separator.
#[ test ]
fn ec01_default_output_style_is_summary()
{
  let out = run_json_claude( &[ "-p", "--max-sessions", "0", "x" ], &[] );
  assert!( out.status.success(), "exit must be 0: {out:?}" );
  let stdout = String::from_utf8_lossy( &out.stdout );
  assert!(
    stdout.contains( "---" ),
    "stdout must contain '---' (render_summary fired by default). Got:\n{stdout}"
  );
}

// ── EC-02: Explicit --output-style summary → stdout contains --- ─────────────

/// EC-02: Explicit `--output-style summary` behaves identically to the default path.
#[ test ]
fn ec02_explicit_summary_renders()
{
  let out = run_json_claude(
    &[ "-p", "--max-sessions", "0", "--output-style", "summary", "x" ],
    &[],
  );
  assert!( out.status.success(), "exit must be 0: {out:?}" );
  let stdout = String::from_utf8_lossy( &out.stdout );
  assert!(
    stdout.contains( "---" ),
    "stdout must contain '---' with explicit --output-style summary. Got:\n{stdout}"
  );
}

// ── EC-03: --output-style raw → stdout does NOT contain --- ──────────────────

/// EC-03: `--output-style raw` bypasses `render_summary()`; raw claude output reaches stdout.
#[ test ]
fn ec03_raw_style_bypasses_render()
{
  let out = run_json_claude(
    &[ "-p", "--max-sessions", "0", "--output-style", "raw", "x" ],
    &[],
  );
  assert!( out.status.success(), "exit must be 0: {out:?}" );
  let stdout = String::from_utf8_lossy( &out.stdout );
  assert!(
    !stdout.contains( "---" ),
    "stdout must NOT contain '---' with --output-style raw. Got:\n{stdout}"
  );
}

// ── EC-04: CLR_OUTPUT_STYLE=raw env var → stdout does NOT contain --- ────────

/// EC-04: `CLR_OUTPUT_STYLE=raw` applies when no CLI flag is present; no summary rendered.
#[ test ]
fn ec04_env_raw_bypasses_render()
{
  let out = run_json_claude(
    &[ "-p", "--max-sessions", "0", "x" ],
    &[ ( "CLR_OUTPUT_STYLE", "raw" ) ],
  );
  assert!( out.status.success(), "exit must be 0: {out:?}" );
  let stdout = String::from_utf8_lossy( &out.stdout );
  assert!(
    !stdout.contains( "---" ),
    "stdout must NOT contain '---' with CLR_OUTPUT_STYLE=raw. Got:\n{stdout}"
  );
}

// ── EC-05: --output-format text --output-style summary → no --- ──────────────

/// EC-05: `--output-format text` goes through Path A in builder.rs (forwarded verbatim).
/// `render_summary()` IS called (style defaults to summary) but receives plain text
/// from the fake claude, returns `None`, and `unwrap_or(out)` passes the raw text through.
#[ test ]
fn ec05_text_format_with_summary_style_no_summary()
{
  let out = run_text_claude(
    &[ "-p", "--max-sessions", "0", "--output-format", "text", "--output-style", "summary", "x" ],
  );
  assert!( out.status.success(), "exit must be 0: {out:?}" );
  let stdout = String::from_utf8_lossy( &out.stdout );
  assert!(
    !stdout.contains( "---" ),
    "stdout must NOT contain '---' when render_summary receives non-JSON text. Got:\n{stdout}"
  );
}

// ── EC-06: --output-format json --output-style raw → no --- ──────────────────

/// EC-06: `--output-style raw` suppresses `render_summary()` even when the subprocess
/// emits valid JSON via `--output-format json`.
#[ test ]
fn ec06_json_format_raw_style_no_summary()
{
  let out = run_json_claude(
    &[ "-p", "--max-sessions", "0", "--output-format", "json", "--output-style", "raw", "x" ],
    &[],
  );
  assert!( out.status.success(), "exit must be 0: {out:?}" );
  let stdout = String::from_utf8_lossy( &out.stdout );
  assert!(
    !stdout.contains( "---" ),
    "stdout must NOT contain '---' with --output-style raw even when JSON available. Got:\n{stdout}"
  );
}

// ── EC-07: --output-style invalid → exit 1, validation message ───────────────

/// EC-07: Invalid `--output-style` value is rejected at parse time with exit 1.
#[ test ]
fn ec07_invalid_style_value_rejected()
{
  let out = run_cli( &[ "--output-style", "invalid" ] );
  assert_eq!(
    out.status.code(),
    Some( 1 ),
    "exit must be 1 when --output-style value is invalid: {out:?}"
  );
  let stderr = String::from_utf8_lossy( &out.stderr );
  assert!(
    stderr.contains( "invalid output-style 'invalid'" ),
    "stderr must contain validation message. Got:\n{stderr}"
  );
  assert!(
    stderr.contains( "expected: summary, raw" ),
    "stderr must contain expected values. Got:\n{stderr}"
  );
}

// ── EC-08: --output-format summary legacy alias → stdout contains --- ────────

/// EC-08: `--output-format summary` (legacy alias) fires both Path A (translates to
/// `--output-format json` in the subprocess command) and the `render_summary()` predicate
/// in `execution.rs` (because `output_style` is `None` → defaults to `"summary"`).
#[ test ]
fn ec08_output_format_summary_legacy_alias()
{
  let out = run_json_claude(
    &[ "-p", "--max-sessions", "0", "--output-format", "summary", "x" ],
    &[],
  );
  assert!( out.status.success(), "exit must be 0: {out:?}" );
  let stdout = String::from_utf8_lossy( &out.stdout );
  assert!(
    stdout.contains( "---" ),
    "stdout must contain '---' via --output-format summary legacy alias. Got:\n{stdout}"
  );
}

// ── EC-09: clr ask default → stdout contains --- ─────────────────────────────

/// EC-09: `clr ask` with a message uses the same print-mode path as `clr run -p`; default
/// output-style is `summary`, so `render_summary()` fires and stdout contains `---`.
#[ test ]
fn ec09_ask_default_style_is_summary()
{
  let out = run_json_claude( &[ "ask", "--max-sessions", "0", "x" ], &[] );
  assert!( out.status.success(), "exit must be 0: {out:?}" );
  let stdout = String::from_utf8_lossy( &out.stdout );
  assert!(
    stdout.contains( "---" ),
    "stdout must contain '---' with clr ask (default summary). Got:\n{stdout}"
  );
}

// ── EC-10: --dry-run --output-style summary → assembled command has --output-format json

/// EC-10: In dry-run mode, Path B in `builder.rs` injects `--output-format json` when
/// `use_print` is true, `output_style` is `"summary"`, and no explicit `--output-format`
/// is set.  The assembled command (stdout) contains `--output-format json`.
#[ test ]
fn ec10_dry_run_summary_injects_json_format()
{
  let out = run_cli( &[ "-p", "--dry-run", "--max-sessions", "0", "--output-style", "summary", "x" ] );
  assert!( out.status.success(), "exit must be 0: {out:?}" );
  let stdout = String::from_utf8_lossy( &out.stdout );
  assert!(
    stdout.contains( "--output-format" ),
    "dry-run output must contain '--output-format' (auto-injected). Got:\n{stdout}"
  );
  assert!(
    stdout.contains( "json" ),
    "dry-run output must contain 'json' (auto-injected value). Got:\n{stdout}"
  );
}

// ── EC-11: CLR_OUTPUT_STYLE=raw + --output-style summary flag → flag wins ─────

/// EC-11: CLI flag `--output-style summary` wins over `CLR_OUTPUT_STYLE=raw` env var
/// because `apply_env_vars()` only sets `output_style` when it is still `None`.
#[ test ]
fn ec11_cli_flag_wins_over_env_var()
{
  let out = run_json_claude(
    &[ "-p", "--max-sessions", "0", "--output-style", "summary", "x" ],
    &[ ( "CLR_OUTPUT_STYLE", "raw" ) ],
  );
  assert!( out.status.success(), "exit must be 0: {out:?}" );
  let stdout = String::from_utf8_lossy( &out.stdout );
  assert!(
    stdout.contains( "---" ),
    "stdout must contain '---' — CLI flag wins over CLR_OUTPUT_STYLE=raw. Got:\n{stdout}"
  );
}

// ── EC-12: CLR_OUTPUT_STYLE=bogus → exit 1, env var validation message ────────

/// EC-12: Invalid `CLR_OUTPUT_STYLE` value is rejected by `apply_env_vars()` with exit 1.
#[ test ]
fn ec12_env_bogus_value_rejected()
{
  let out = run_cli_with_env(
    &[ "--dry-run", "x" ],
    &[ ( "CLR_OUTPUT_STYLE", "bogus" ) ],
  );
  assert_eq!(
    out.status.code(),
    Some( 1 ),
    "exit must be 1 when CLR_OUTPUT_STYLE is invalid: {out:?}"
  );
  let stderr = String::from_utf8_lossy( &out.stderr );
  assert!(
    stderr.contains( "CLR_OUTPUT_STYLE: invalid value 'bogus'" ),
    "stderr must contain env var validation message. Got:\n{stderr}"
  );
  assert!(
    stderr.contains( "expected: summary, raw" ),
    "stderr must contain expected values. Got:\n{stderr}"
  );
}

// ── EC-13: --output-format stream-json --output-style summary → no --- ────────

/// EC-13: Same fallback as EC-05 — `--output-format stream-json` forwarded via Path A;
/// `render_summary()` called but receives non-JSON stream from fake claude; returns `None`;
/// raw output passed through without summary header.
#[ test ]
fn ec13_stream_json_format_with_summary_style_no_summary()
{
  let out = run_text_claude(
    &[
      "-p", "--max-sessions", "0",
      "--output-format", "stream-json",
      "--output-style", "summary",
      "x",
    ],
  );
  assert!( out.status.success(), "exit must be 0: {out:?}" );
  let stdout = String::from_utf8_lossy( &out.stdout );
  assert!(
    !stdout.contains( "---" ),
    "stdout must NOT contain '---' when render_summary receives non-JSON stream. Got:\n{stdout}"
  );
}

// ── EC-14: Minimal 7-field CLR envelope (no session_id) → stdout contains --- ─

/// EC-14 / IT-2 (BUG-310 regression): `clr -p` with a fake claude emitting a minimal
/// 7-field CLR envelope (no `session_id`, no `usage`, no `total_cost_usd`) must produce
/// a rendered summary containing `---`.
///
/// Verifies the full `clr` execution path end-to-end: Path B auto-injects
/// `--output-format json`, `render_summary()` gates on `"type":"result"` (invariant field),
/// and handles absent `session_id` via `.unwrap_or_default()` instead of propagating `None`
/// (which was the BUG-310 failure mode: `None` → `unwrap_or(out)` → raw JSON on stdout).
#[ test ]
fn ec14_render_summary_minimal_envelope_no_session_id()
{
  let out = run_minimal_claude( &[ "-p", "--max-sessions", "0", "x" ] );
  assert!( out.status.success(), "exit must be 0: {out:?}" );
  let stdout = String::from_utf8_lossy( &out.stdout );
  assert!(
    stdout.contains( "---" ),
    "stdout must contain '---' for minimal CLR envelope without session_id (BUG-310 regression). Got:\n{stdout}"
  );
}

// ── IT-7: Source does NOT contain extract_str( json, "session_id" )? ──────────

/// IT-7: Structural regression guard — verifies that `src/cli/summary.rs` does NOT contain
/// the BUG-310 anti-pattern `extract_str( json, "session_id" )?`.
///
/// This prevents the `?`-on-optional-field gate from being silently re-introduced in future
/// refactors: any such `?` on an optional CLR field returns `None` for any envelope missing
/// that field, restoring the raw-JSON fallback symptom. The only permitted gate is on the
/// invariant `"type"` field.
#[ test ]
fn render_summary_gate_uses_type_not_session_id()
{
  let manifest_dir = env!( "CARGO_MANIFEST_DIR" );
  let src_path = std::path::Path::new( manifest_dir ).join( "src/cli/summary.rs" );
  let source = std::fs::read_to_string( &src_path )
    .expect( "failed to read src/cli/summary.rs" );
  assert!(
    !source.contains( r#"extract_str( json, "session_id" )?"# ),
    "BUG-310 anti-pattern detected: extract_str( json, \"session_id\" )? must not appear in \
    src/cli/summary.rs — gate must use the invariant 'type' field, not the optional 'session_id'"
  );
}
