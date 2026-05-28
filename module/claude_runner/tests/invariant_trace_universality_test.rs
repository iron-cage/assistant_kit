//! Invariant tests — Trace Universality (INV-004)
//!
//! Verifies that every subprocess-executing `clr` command accepts `--trace`
//! and emits diagnostics to stderr before invocation.
//!
//! Source: `tests/docs/invariant/004_trace_universality.md`

use std::io::Write as _;
use tempfile::NamedTempFile;

mod cli_binary_test_helpers;

// ── helper ───────────────────────────────────────────────────────────────────

/// Write `content` to a new `NamedTempFile` and return it.
///
/// Caller must keep the returned handle alive; dropping it deletes the file.
/// RAII: `creds` binding must outlive the `run_cli(...)` call.
fn make_creds_file( content : &str ) -> NamedTempFile
{
  let mut f = NamedTempFile::new().expect( "failed to create temp creds file" );
  f.write_all( content.as_bytes() ).expect( "failed to write creds content" );
  f
}

fn stderr_str( o : &std::process::Output ) -> String
{
  String::from_utf8_lossy( &o.stderr ).to_string()
}

// ── tests ─────────────────────────────────────────────────────────────────────

/// IT-1: `clr --trace "Fix bug"` (run) → stderr contains env+command before invocation.
///
/// Trace fires before subprocess attempt; exit is non-zero because PATH=/nonexistent
/// prevents claude from being found.  PATH is restricted to avoid hanging: without it,
/// an installed claude binary starts an interactive session and the test never completes.
///
/// Source: tests/docs/invariant/004_trace_universality.md#it-1
#[ test ]
fn it_01_run_trace_stderr_output()
{
  // PATH=/nonexistent: trace fires first, then spawn fails immediately (claude not found).
  // Without this, an installed claude binary would open an interactive TTY and hang forever.
  let out    = cli_binary_test_helpers::run_cli_with_env(
    &[ "--trace", "Fix bug" ],
    &[ ( "PATH", "/nonexistent" ) ],
  );
  let stderr = stderr_str( &out );
  assert!(
    stderr.contains( "CLAUDE_CODE_MAX_OUTPUT_TOKENS=200000" ),
    "run --trace must emit CLAUDE_CODE_MAX_OUTPUT_TOKENS=200000 on stderr. Got:\n{stderr}"
  );
  assert!(
    stderr.contains( "--dangerously-skip-permissions" ),
    "run --trace must emit --dangerously-skip-permissions on stderr. Got:\n{stderr}"
  );
}

/// IT-2: `clr ask --trace "What is X?"` → stderr contains ask-default env+command.
///
/// Trace fires before subprocess attempt; exit 1 (claude absent) is acceptable.
///
/// Cross-invariant confirmation: verifies `ask` trace from the invariant test file's perspective.
/// Not a duplicate of IT-9 (`ask_command_test.rs`) — these two tests cover the same behavior
/// from different test entry points (command coverage vs. invariant coverage).
///
/// Source: tests/docs/invariant/004_trace_universality.md#it-2
#[ test ]
fn it_02_ask_trace_stderr_output()
{
  let out    = cli_binary_test_helpers::run_cli( &[ "ask", "--trace", "What is X?" ] );
  let stderr = stderr_str( &out );
  assert!(
    stderr.contains( "CLAUDE_CODE_MAX_OUTPUT_TOKENS=16384" ),
    "ask --trace must emit CLAUDE_CODE_MAX_OUTPUT_TOKENS=16384 on stderr. Got:\n{stderr}"
  );
  assert!(
    stderr.contains( "--effort high" ),
    "ask --trace must emit --effort high on stderr. Got:\n{stderr}"
  );
  let code = out.status.code().unwrap_or( -1 );
  assert!( code == 0 || code == 1, "expected exit 0 or 1 (trace before invoke); got {code}" );
}

/// IT-3: `clr isolated --creds <f> --trace "Fix bug"` → stderr has credential trace.
///
/// Uses `NamedTempFile` so `read_to_string(creds_path)` succeeds and
/// `emit_credential_trace` is reached BEFORE the subprocess attempt
/// (`emit_credential_trace` fires before `run_isolated()` in `run_isolated_command`).
///
/// Source: tests/docs/invariant/004_trace_universality.md#it-3
#[ test ]
fn it_03_isolated_trace_stderr_output()
{
  let creds  = make_creds_file( "{}" );
  let path   = creds.path().to_str().expect( "temp path is valid UTF-8" );
  let out    = cli_binary_test_helpers::run_cli( &[ "isolated", "--creds", path, "--trace", "Fix bug" ] );
  let stderr = stderr_str( &out );
  assert!(
    stderr.contains( "# clr isolated" ),
    "isolated --trace must emit '# clr isolated' on stderr. Got:\n{stderr}"
  );
  assert!(
    stderr.contains( "# creds:" ),
    "isolated --trace must emit '# creds:' on stderr. Got:\n{stderr}"
  );
  assert!(
    stderr.contains( "# timeout: 30s" ),
    "isolated --trace must emit '# timeout: 30s' (default from parse_isolated_args) on stderr. Got:\n{stderr}"
  );
  let code = out.status.code().unwrap_or( -1 );
  assert!( code == 0 || code == 1, "expected exit 0 or 1 (trace before invoke); got {code}" );
}

/// IT-4: `clr refresh --creds <f> --trace` → stderr has credential trace with 45s timeout.
///
/// Uses `NamedTempFile` so `read_to_string(creds_path)` succeeds and
/// `emit_credential_trace` is reached BEFORE the subprocess attempt
/// (`emit_credential_trace` fires before `run_isolated()` in `run_isolated_command`).
///
/// Source: tests/docs/invariant/004_trace_universality.md#it-4
#[ test ]
fn it_04_refresh_trace_stderr_output()
{
  let creds  = make_creds_file( "{}" );
  let path   = creds.path().to_str().expect( "temp path is valid UTF-8" );
  let out    = cli_binary_test_helpers::run_cli( &[ "refresh", "--creds", path, "--trace" ] );
  let stderr = stderr_str( &out );
  assert!(
    stderr.contains( "# clr refresh" ),
    "refresh --trace must emit '# clr refresh' on stderr. Got:\n{stderr}"
  );
  assert!(
    stderr.contains( "# creds:" ),
    "refresh --trace must emit '# creds:' on stderr. Got:\n{stderr}"
  );
  assert!(
    stderr.contains( "# timeout: 45s" ),
    "refresh --trace must emit '# timeout: 45s' (default from parse_refresh_args) on stderr. Got:\n{stderr}"
  );
  let code = out.status.code().unwrap_or( -1 );
  assert!( code == 0 || code == 1, "expected exit 0 or 1 (trace before invoke); got {code}" );
}

/// IT-5: Static — `"--trace"` appears ≥ 3× in `src/cli.rs` (one per parse function).
///
/// Verifies that `parse_args()`, `parse_isolated_args()`, and `parse_refresh_args()`
/// all register the `--trace` flag.  Reads the actual source file at runtime via
/// `CARGO_MANIFEST_DIR` (baked Docker image source — ensure image is current before running).
///
/// Source: tests/docs/invariant/004_trace_universality.md#it-5
#[ test ]
fn it_05_static_trace_universality()
{
  let manifest_dir = env!( "CARGO_MANIFEST_DIR" );
  let cli_path     = format!( "{manifest_dir}/src/cli.rs" );
  let cli_rs       = std::fs::read_to_string( &cli_path )
    .unwrap_or_else( | e | panic!( "failed to read {cli_path}: {e}" ) );
  let count = cli_rs.matches( "\"--trace\"" ).count();
  assert!(
    count >= 3,
    "expected '\"--trace\"' to appear ≥ 3 times in src/cli.rs \
     (parse_args, parse_isolated_args, parse_refresh_args); got {count}"
  );
}
