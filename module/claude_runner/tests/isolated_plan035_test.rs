//! Integration tests for `clr isolated` subcommand — Plan 035.
//!
//! # Test Matrix
//!
//! | ID | Test | Requires Live Claude |
//! |----|------|----------------------|
//! | IT-29 | `--output-file <PATH>` tees subprocess output to a file | No (fake claude) |
//! | IT-30 | `--strip-fences` strips outermost markdown code fences | No (fake claude) |
//! | IT-31 | `--output-style summary` renders CLR envelope as key:value pairs | No (fake claude) |
//! | IT-32 | `--summary-fields minimal` limits rendered fields | No (fake claude) |
//! | IT-33 | `CLR_OUTPUT_FILE` env var fallback applies when flag absent | No (fake claude) |
//! | IT-34 | `CLR_STRIP_FENCES=1` env var fallback strips fences | No (fake claude) |
//! | IT-35 | `CLR_OUTPUT_STYLE=summary` env var fallback | No (fake claude) |
//! | IT-36 | `CLR_SUMMARY_FIELDS=minimal` env var fallback | No (fake claude) |
//! | IT-37 | `CLR_JOURNAL=bogus` exits 1 with error message | No |
//! | IT-10 | `--creds <f> --trace "msg"` → credential trace on stderr | No |

#![ cfg( feature = "enabled" ) ]

mod cli_binary_test_helpers;
use cli_binary_test_helpers::{ exit_code, make_creds_file, run_isolated, run_with_path, stderr_str, stdout_str };
#[ cfg( unix ) ]
use cli_binary_test_helpers::fake_claude_dir;

// ── IT-29 through IT-36: output params ──────────────────────────────────────

/// IT-29: `--output-file <PATH>` tees subprocess output to a file.
///
/// Fake claude echoes known text; test verifies both stdout and file content.
///
/// Source: tests/docs/cli/command/03_isolated.md#it-29
#[ cfg( unix ) ]
#[ test ]
fn it29_output_file_tees_to_disk()
{
  let creds          = make_creds_file( "{}" );
  let creds_path     = creds.path().to_str().unwrap();
  let ( _dir, path ) = fake_claude_dir( "echo 'output_test_it29'" );
  let out_file       = tempfile::NamedTempFile::new().expect( "create output file" );
  let out_path       = out_file.path().to_str().unwrap();
  let out            = run_with_path(
    &[ "isolated", "--creds", creds_path, "--output-file", out_path, "msg" ],
    &path,
  );
  assert!(
    out.status.success(),
    "expected exit 0; stderr: {}", stderr_str( &out )
  );
  assert!(
    stdout_str( &out ).contains( "output_test_it29" ),
    "stdout must still contain output; got:\n{}", stdout_str( &out )
  );
  let file_content = std::fs::read_to_string( out_file.path() ).expect( "read output file" );
  assert!(
    file_content.contains( "output_test_it29" ),
    "output file must contain subprocess output; got:\n{file_content}"
  );
}

/// IT-30: `--strip-fences` strips outermost markdown code fences from output.
///
/// Source: tests/docs/cli/command/03_isolated.md#it-30
#[ cfg( unix ) ]
#[ test ]
fn it30_strip_fences_removes_outer_fences()
{
  let creds          = make_creds_file( "{}" );
  let creds_path     = creds.path().to_str().unwrap();
  let ( _dir, path ) = fake_claude_dir( "printf '```python\\nprint(42)\\n```\\n'" );
  let out            = run_with_path(
    &[ "isolated", "--creds", creds_path, "--strip-fences", "msg" ],
    &path,
  );
  assert!(
    out.status.success(),
    "expected exit 0; stderr: {}", stderr_str( &out )
  );
  let stdout = stdout_str( &out );
  assert!(
    stdout.contains( "print(42)" ),
    "stripped output must contain inner code; got:\n{stdout}"
  );
  assert!(
    !stdout.contains( "```" ),
    "stripped output must not contain fence markers; got:\n{stdout}"
  );
}

/// IT-31: `--output-style summary` renders CLR envelope as key:value pairs.
///
/// Fake claude emits a valid CLR JSON envelope with `"type":"result"`.
/// `render_summary()` gating on `"type":"result"` is mandatory per invariant/008.
///
/// Source: tests/docs/cli/command/03_isolated.md#it-31
#[ cfg( unix ) ]
#[ test ]
fn it31_output_style_summary_renders_envelope()
{
  let creds          = make_creds_file( "{}" );
  let creds_path     = creds.path().to_str().unwrap();
  let json           = r#"{"type":"result","session_id":"s1","cost_usd":0.01,"duration_ms":500,"result":"hello world"}"#;
  let script         = format!( "printf '{json}'" );
  let ( _dir, path ) = fake_claude_dir( &script );
  let out            = run_with_path(
    &[ "isolated", "--creds", creds_path, "--output-style", "summary", "msg" ],
    &path,
  );
  assert!(
    out.status.success(),
    "expected exit 0; stderr: {}", stderr_str( &out )
  );
  let stdout = stdout_str( &out );
  assert!(
    stdout.contains( "---" ),
    "summary output must contain separator; got:\n{stdout}"
  );
  assert!(
    stdout.contains( "result" ),
    "summary output must contain result field; got:\n{stdout}"
  );
}

/// IT-32: `--summary-fields minimal` limits rendered fields.
///
/// Source: tests/docs/cli/command/03_isolated.md#it-32
#[ cfg( unix ) ]
#[ test ]
fn it32_summary_fields_minimal()
{
  let creds          = make_creds_file( "{}" );
  let creds_path     = creds.path().to_str().unwrap();
  let json           = r#"{"type":"result","session_id":"s1","cost_usd":0.05,"duration_ms":1234,"result":"content","model":"opus","num_turns":3}"#;
  let script         = format!( "printf '{json}'" );
  let ( _dir, path ) = fake_claude_dir( &script );
  let out            = run_with_path(
    &[
      "isolated", "--creds", creds_path,
      "--output-style", "summary", "--summary-fields", "minimal", "msg",
    ],
    &path,
  );
  assert!(
    out.status.success(),
    "expected exit 0; stderr: {}", stderr_str( &out )
  );
  let stdout = stdout_str( &out );
  assert!(
    stdout.contains( "result" ),
    "minimal profile must include result; got:\n{stdout}"
  );
  // minimal profile excludes verbose fields like num_turns
  assert!(
    !stdout.contains( "num_turns" ),
    "minimal profile must not include num_turns; got:\n{stdout}"
  );
}

/// IT-33: `CLR_OUTPUT_FILE` env var fallback applies when flag is absent.
///
/// Source: tests/docs/cli/command/03_isolated.md#it-33
#[ cfg( unix ) ]
#[ test ]
fn it33_clr_output_file_env_fallback()
{
  let creds          = make_creds_file( "{}" );
  let creds_path     = creds.path().to_str().unwrap();
  let ( _dir, path ) = fake_claude_dir( "echo 'env_file_it33'" );
  let out_file       = tempfile::NamedTempFile::new().expect( "create output file" );
  let out_path       = out_file.path().to_str().unwrap();
  let out            = std::process::Command::new( env!( "CARGO_BIN_EXE_clr" ) )
    .args( [ "isolated", "--creds", creds_path, "msg" ] )
    .env( "PATH", &path )
    .env( "CLR_OUTPUT_FILE", out_path )
    .env_remove( "CLR_STRIP_FENCES" )
    .env_remove( "CLR_OUTPUT_STYLE" )
    .env_remove( "CLR_SUMMARY_FIELDS" )
    .output()
    .expect( "invoke clr" );
  assert!(
    out.status.success(),
    "expected exit 0; stderr: {}", String::from_utf8_lossy( &out.stderr )
  );
  let content = std::fs::read_to_string( out_file.path() ).expect( "read output file" );
  assert!(
    content.contains( "env_file_it33" ),
    "CLR_OUTPUT_FILE must tee output to file; got:\n{content}"
  );
}

/// IT-34: `CLR_STRIP_FENCES=1` env var fallback strips fences.
///
/// Source: tests/docs/cli/command/03_isolated.md#it-34
#[ cfg( unix ) ]
#[ test ]
fn it34_clr_strip_fences_env_fallback()
{
  let creds          = make_creds_file( "{}" );
  let creds_path     = creds.path().to_str().unwrap();
  let ( _dir, path ) = fake_claude_dir( "printf '```\\nstripped_it34\\n```\\n'" );
  let out            = std::process::Command::new( env!( "CARGO_BIN_EXE_clr" ) )
    .args( [ "isolated", "--creds", creds_path, "msg" ] )
    .env( "PATH", &path )
    .env( "CLR_STRIP_FENCES", "1" )
    .env_remove( "CLR_OUTPUT_FILE" )
    .env_remove( "CLR_OUTPUT_STYLE" )
    .env_remove( "CLR_SUMMARY_FIELDS" )
    .output()
    .expect( "invoke clr" );
  assert!(
    out.status.success(),
    "expected exit 0; stderr: {}", String::from_utf8_lossy( &out.stderr )
  );
  let stdout = String::from_utf8_lossy( &out.stdout );
  assert!(
    stdout.contains( "stripped_it34" ),
    "output must contain inner content; got:\n{stdout}"
  );
  assert!(
    !stdout.contains( "```" ),
    "CLR_STRIP_FENCES=1 must strip fences; got:\n{stdout}"
  );
}

/// IT-35: `CLR_OUTPUT_STYLE=summary` env var fallback.
///
/// Source: tests/docs/cli/command/03_isolated.md#it-35
#[ cfg( unix ) ]
#[ test ]
fn it35_clr_output_style_env_fallback()
{
  let creds          = make_creds_file( "{}" );
  let creds_path     = creds.path().to_str().unwrap();
  let json           = r#"{"type":"result","session_id":"s2","cost_usd":0.02,"result":"env_style_it35"}"#;
  let script         = format!( "printf '{json}'" );
  let ( _dir, path ) = fake_claude_dir( &script );
  let out            = std::process::Command::new( env!( "CARGO_BIN_EXE_clr" ) )
    .args( [ "isolated", "--creds", creds_path, "msg" ] )
    .env( "PATH", &path )
    .env( "CLR_OUTPUT_STYLE", "summary" )
    .env_remove( "CLR_OUTPUT_FILE" )
    .env_remove( "CLR_STRIP_FENCES" )
    .env_remove( "CLR_SUMMARY_FIELDS" )
    .output()
    .expect( "invoke clr" );
  assert!(
    out.status.success(),
    "expected exit 0; stderr: {}", String::from_utf8_lossy( &out.stderr )
  );
  let stdout = String::from_utf8_lossy( &out.stdout );
  assert!(
    stdout.contains( "---" ),
    "CLR_OUTPUT_STYLE=summary must render summary; got:\n{stdout}"
  );
}

/// IT-36: `CLR_SUMMARY_FIELDS=minimal` env var fallback.
///
/// Source: tests/docs/cli/command/03_isolated.md#it-36
#[ cfg( unix ) ]
#[ test ]
fn it36_clr_summary_fields_env_fallback()
{
  let creds          = make_creds_file( "{}" );
  let creds_path     = creds.path().to_str().unwrap();
  let json           = r#"{"type":"result","session_id":"s3","cost_usd":0.03,"result":"env_fields_it36","model":"opus","num_turns":5}"#;
  let script         = format!( "printf '{json}'" );
  let ( _dir, path ) = fake_claude_dir( &script );
  let out            = std::process::Command::new( env!( "CARGO_BIN_EXE_clr" ) )
    .args( [ "isolated", "--creds", creds_path, "msg" ] )
    .env( "PATH", &path )
    .env( "CLR_OUTPUT_STYLE", "summary" )
    .env( "CLR_SUMMARY_FIELDS", "minimal" )
    .env_remove( "CLR_OUTPUT_FILE" )
    .env_remove( "CLR_STRIP_FENCES" )
    .output()
    .expect( "invoke clr" );
  assert!(
    out.status.success(),
    "expected exit 0; stderr: {}", String::from_utf8_lossy( &out.stderr )
  );
  let stdout = String::from_utf8_lossy( &out.stdout );
  assert!(
    stdout.contains( "result" ),
    "CLR_SUMMARY_FIELDS=minimal must include result; got:\n{stdout}"
  );
  assert!(
    !stdout.contains( "num_turns" ),
    "CLR_SUMMARY_FIELDS=minimal must exclude num_turns; got:\n{stdout}"
  );
}

// ── IT-37: CLR_JOURNAL=invalid → exit 1 ──────────────────────────────────────

/// IT-37: `CLR_JOURNAL=bogus` env var with `clr isolated` exits 1 and names the
/// invalid value in the error message.
///
/// ## Root Cause
///
/// `apply_isolated_env_vars()` applied `CLR_JOURNAL` directly via `env_str()` without
/// validation — an invalid value was silently accepted and treated as meta level.
///
/// ## Why Not Caught
///
/// No test asserted that `CLR_JOURNAL=bogus` would be rejected by the isolated path;
/// only `run`/`ask` had a test for this (EC-9 in `journal_integration_test.rs`).
///
/// ## Fix Applied
///
/// `apply_isolated_env_vars()` now validates `CLR_JOURNAL` against `"full" | "meta" | "off"`
/// and returns `Err` with the same message format as `apply_env_vars()` in `env.rs`.
///
/// ## Prevention
///
/// Assert `CLR_JOURNAL=bogus clr isolated …` exits 1 and names the env var in stderr.
///
/// ## Pitfall
///
/// The `--creds` flag must point to a readable file — `apply_isolated_env_vars()` runs
/// after arg parsing; validation fires before the creds-path check, so providing a valid
/// (but empty) creds file ensures the env var error is the first exit point.
///
/// Source: tests/docs/cli/command/03_isolated.md#it-37
#[ test ]
fn it37_clr_journal_invalid_value_exits_1()
{
  let creds   = make_creds_file( "{}" );
  let creds_s = creds.path().to_str().expect( "utf-8" );
  let bin     = env!( "CARGO_BIN_EXE_clr" );
  let out     = std::process::Command::new( bin )
    .args( [ "isolated", "--creds", creds_s, "--dry-run", "x" ] )
    .env( "CLR_JOURNAL", "bogus" )
    .env_remove( "CLR_JOURNAL_DIR" )
    .output()
    .expect( "failed to invoke clr isolated" );
  assert_eq!(
    exit_code( &out ),
    1,
    "CLR_JOURNAL=bogus must cause isolated to exit 1. Got: {:?}\nstderr: {}",
    out.status.code(),
    stderr_str( &out ),
  );
  let stderr = stderr_str( &out );
  assert!(
    stderr.contains( "CLR_JOURNAL" ),
    "error must mention CLR_JOURNAL. Got:\n{stderr}"
  );
  assert!(
    stderr.to_lowercase().contains( "invalid" ),
    "error must describe the value as invalid. Got:\n{stderr}"
  );
}

/// IT-10: `--creds <f> --trace "msg"` → credential trace lines on stderr before subprocess attempt.
///
/// `emit_credential_trace` fires before `run_isolated()` in `run_isolated_command`,
/// so `# clr isolated`, `# creds:`, and `# timeout: 30s` appear on stderr even when
/// claude is absent.  Uses `NamedTempFile` so the creds file is readable when
/// `read_to_string` is called inside `clr`.
///
/// Source: tests/docs/cli/command/003_isolated.md#it-10
#[ test ]
fn it10_isolated_trace_stderr_output()
{
  let creds = make_creds_file( "{}" );
  let path  = creds.path().to_str().expect( "temp path is valid UTF-8" );
  let out   = run_isolated( &[ "--creds", path, "--trace", "Fix bug" ] );
  let err   = stderr_str( &out );
  assert!(
    err.contains( "# clr isolated" ),
    "isolated --trace must emit '# clr isolated' on stderr. Got:\n{err}"
  );
  assert!(
    err.contains( "# creds:" ),
    "isolated --trace must emit '# creds:' on stderr. Got:\n{err}"
  );
  assert!(
    err.contains( "# timeout: 30s" ),
    "isolated --trace must emit '# timeout: 30s' (default) on stderr. Got:\n{err}"
  );
  let code = out.status.code().unwrap_or( -1 );
  assert!( code == 0 || code == 1, "expected exit 0 or 1 (trace before invoke); got {code}" );
}
