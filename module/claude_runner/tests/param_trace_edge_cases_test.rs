//! Trace Parameter Edge Case Tests
//!
//! ## Purpose
//!
//! Cover trace-mode spec test cases from `tests/docs/cli/param/013_trace.md`
//! and `tests/docs/cli/command/` not exercised by other test files.
//!
//! ## Strategy
//!
//! All tests invoke the compiled binary via `env!("CARGO_BIN_EXE_clr")`.
//! Basic trace tests set PATH to `/nonexistent` so execution fails deterministically.
//! Credential trace tests write a temp `NamedTempFile` for `isolated`/`refresh` subcommands.
//!
//! ## Spec Coverage
//!
//! trace:
//! - S04: `--trace "msg"` without `--dry-run` → stderr has command, exit 1 (`01_run.md` IT-5, `013_trace.md` EC-1, `011_dry_run.md` EC-2)
//! - S05: `--trace --dry-run` no message → stdout preview, stderr empty (`013_trace.md` EC-4)
//! - S06: `--trace "msg"` stderr contains env vars and command (`013_trace.md` EC-6)
//! - S58: `isolated --creds <f> --trace "msg"` → `# clr isolated`, `# creds:`, `# timeout: 30s` on stderr (`013_trace.md` EC-7)
//! - S59: `refresh --creds <f> --trace` → `# clr refresh`, `# creds:`, `# timeout: 45s` on stderr (`013_trace.md` EC-8)
//! - S60: `isolated --creds /nonexistent --trace "msg"` → trace fires on stderr before creds-read failure (bug reproducer)

mod cli_binary_test_helpers;
use cli_binary_test_helpers::{ make_creds_file, run_cli };

// S04: --trace without --dry-run → stderr has command; exit 1 (claude absent)
#[ test ]
fn s04_trace_without_dry_run_echoes_command_to_stderr()
{
  let out = cli_binary_test_helpers::run_cli_with_env( &[ "--trace", "Fix bug" ], &[ ( "PATH", "/nonexistent" ) ] );
  assert!(
    !out.status.success(),
    "--trace without --dry-run must fail (claude not found)"
  );
  let stderr = String::from_utf8_lossy( &out.stderr );
  assert!(
    stderr.contains( "claude" ),
    "--trace must echo assembled command to stderr before invocation attempt. Got:\n{stderr}"
  );
}

// S05: --trace --dry-run without message → stdout has preview; stderr is empty
#[ test ]
fn s05_trace_with_dry_run_no_message_stderr_empty()
{
  let out = run_cli( &[ "--trace", "--dry-run" ] );
  assert!( out.status.success(), "--trace --dry-run must exit 0. stderr: {}", String::from_utf8_lossy( &out.stderr ) );
  let stdout = String::from_utf8_lossy( &out.stdout );
  assert!(
    stdout.contains( "claude" ),
    "--dry-run output must appear on stdout. Got:\n{stdout}"
  );
  assert!(
    out.stderr.is_empty(),
    "--trace must not fire when --dry-run wins (stderr must be empty). Got:\n{}",
    String::from_utf8_lossy( &out.stderr )
  );
}

// S06: --trace (no --dry-run) stderr includes env vars and command
#[ test ]
fn s06_trace_stderr_includes_env_vars_and_command()
{
  let out = cli_binary_test_helpers::run_cli_with_env( &[ "--trace", "Fix bug" ], &[ ( "PATH", "/nonexistent" ) ] );
  assert!( !out.status.success(), "must fail (claude absent)" );
  let stderr = String::from_utf8_lossy( &out.stderr );
  assert!(
    stderr.contains( "claude" ),
    "trace stderr must include assembled command. Got:\n{stderr}"
  );
  assert!(
    stderr.contains( "CLAUDE_CODE_MAX_OUTPUT_TOKENS=" ),
    "trace stderr must include env vars. Got:\n{stderr}"
  );
}

// S58: `isolated --creds <f> --trace "msg"` → credential trace format on stderr (`013_trace.md` EC-7)
#[ test ]
fn s58_isolated_trace_credential_format()
{
  let creds = make_creds_file( "{}" );
  let path  = creds.path().to_str().expect( "temp path is valid UTF-8" );
  let out   = run_cli( &[ "isolated", "--creds", path, "--trace", "Fix bug" ] );
  let err   = String::from_utf8_lossy( &out.stderr );
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
  assert!( code == 0 || code == 1, "expected exit 0 or 1 (trace fires before invoke); got {code}" );
}

// S60: `isolated --creds /nonexistent --trace "msg"` → trace fires even when creds file is missing
//
// ## Root Cause (bug_reproducer(BUG-223))
//
// `run_isolated_command` read the credentials file first (exiting 1 on failure),
// then emitted trace.  This meant `--trace` produced no output when the creds
// file was absent — the opposite of `run_refresh_command`, which emits trace
// BEFORE calling `run_isolated_command` and therefore always fires trace.
//
// ## Why Not Caught
//
// All trace tests (IT-10, EC-7, S58) used a readable `NamedTempFile` for creds,
// so the creds-read always succeeded and the ordering bug was invisible.
//
// ## Fix Applied
//
// Reordered `run_isolated_command`: build args first, emit trace, THEN read creds.
// Trace now fires before any I/O that may exit early, matching `run_refresh_command`.
//
// ## Prevention
//
// When adding a new trace-emitting command: emit trace before any validation step
// that may exit early (e.g. file reads, argument validation).  The invariant
// "trace fires before subprocess launch" extends to "trace fires before any early
// exit" so that --trace is always useful for diagnosing failures.
//
// ## Pitfall
//
// Do not test `--trace` with only a happy-path creds file.  Also verify that
// trace fires when creds are absent — otherwise the ordering bug can regress silently.
// test_kind: bug_reproducer(BUG-223)
#[ test ]
fn s60_isolated_trace_fires_even_with_missing_creds()
{
  let missing = "/tmp/s60_nonexistent_creds_file_that_must_not_exist.json";
  // Precondition: the file really must not exist.
  assert!(
    !std::path::Path::new( missing ).exists(),
    "precondition: {missing} must not exist for this test to be valid"
  );
  let out = run_cli( &[ "isolated", "--creds", missing, "--trace", "Fix bug" ] );
  let err = String::from_utf8_lossy( &out.stderr );
  // Trace must appear on stderr BEFORE any creds-related error.
  assert!(
    err.contains( "# clr isolated" ),
    "isolated --trace must emit '# clr isolated' on stderr even when creds file is absent. Got:\n{err}"
  );
  assert!(
    err.contains( "# creds:" ),
    "isolated --trace must emit '# creds:' on stderr even when creds file is absent. Got:\n{err}"
  );
  assert!(
    err.contains( "# timeout: 30s" ),
    "isolated --trace must emit '# timeout: 30s' on stderr even when creds file is absent. Got:\n{err}"
  );
  // After trace, creds-read fails → exit 1.
  assert_eq!(
    out.status.code(),
    Some( 1 ),
    "isolated with missing creds must exit 1; got {:?}",
    out.status.code()
  );
}

// S59: `refresh --creds <f> --trace` → credential trace format on stderr with 45s timeout (`013_trace.md` EC-8)
#[ test ]
fn s59_refresh_trace_credential_format()
{
  let creds = make_creds_file( "{}" );
  let path  = creds.path().to_str().expect( "temp path is valid UTF-8" );
  let out   = run_cli( &[ "refresh", "--creds", path, "--trace" ] );
  let err   = String::from_utf8_lossy( &out.stderr );
  assert!(
    err.contains( "# clr refresh" ),
    "refresh --trace must emit '# clr refresh' on stderr. Got:\n{err}"
  );
  assert!(
    err.contains( "# creds:" ),
    "refresh --trace must emit '# creds:' on stderr. Got:\n{err}"
  );
  assert!(
    err.contains( "# timeout: 45s" ),
    "refresh --trace must emit '# timeout: 45s' (distinct from isolated's 30s) on stderr. Got:\n{err}"
  );
  let code = out.status.code().unwrap_or( -1 );
  assert!( code == 0 || code == 1, "expected exit 0 or 1 (trace fires before invoke); got {code}" );
}
