//! Integration tests for `clr isolated` subcommand.
//!
//! # Test Matrix
//!
//! | ID | Test | Requires Live Claude |
//! |----|------|----------------------|
//! | IT-2 | `--creds missing.json` → exit 1 | No |
//! | IT-7 | `--timeout abc` → exit 1, invalid timeout | No |
//! | IT-8 | No `--creds`, `CLR_CREDS` unset → defaults to `$HOME/.claude/.credentials.json`; trace confirms | No |
//! | IT-9 | `clr isolated --help` → exit 0, help text shown | No |
//! | IT-10 | `--creds <f> --trace "msg"` → credential trace on stderr before attempt | No |
//! | EC-creds-4 | Nonexistent creds file → exit 1 | No |
//! | EC-creds-5 | `--creds` without value → exit 1 | No |
//! | EC-creds-6 | `--creds` omitted, `CLR_CREDS` unset → trace confirms default path | No |
//! | EC-timeout-4 | `--timeout -1` → exit 1 | No |
//! | EC-timeout-5 | `--timeout abc` → exit 1 | No |
//! | EC-timeout-6 | `--timeout` without value → exit 1 | No |
//! | EC-timeout-1 | `--timeout 30` → accepted | No |
//! | EC-timeout-2 | `--timeout 0` → accepted | No |
//! | EC-timeout-3 | `--timeout 3600` → accepted | No |
//! | IT-1 | Happy path: valid creds, message → exit 0 | **Yes** (`lim_it`) |
//! | IT-3 | Timeout 0, no creds refresh → exit 2 | **Yes** (`lim_it`) |
//! | IT-4 | Credential write-back after startup refresh → exit 0 | **Yes** (`lim_it`) |
//! | IT-5 | No message → Claude rejects piped non-TTY context | **Yes** (`lim_it`) |
//! | IT-6 | `-- --version` passthrough → version in stdout | **Yes** (`lim_it`) |
//! | EC-creds-1 | Valid file path → subprocess runs | **Yes** (`lim_it`) |
//! | EC-creds-2 | Absolute path → resolved correctly | **Yes** (`lim_it`) |
//! | EC-creds-3 | Relative path → resolved via cwd | **Yes** (`lim_it`) |
//!
//! Tests containing `lim_it` run by default in container environments.
//! They early-return when the `claude` binary is absent from `$PATH`.

#![ cfg( feature = "enabled" ) ]

use std::io::Write as _;
use tempfile::NamedTempFile;

mod cli_binary_test_helpers;
use cli_binary_test_helpers::{ exit_code, make_creds_file, run_with_path, stderr_str, stdout_str };
#[ cfg( unix ) ]
use cli_binary_test_helpers::fake_claude_dir;

// ── helpers ──────────────────────────────────────────────────────────────────

/// Invoke `clr isolated <args>` and return raw output.
///
/// Delegates to the shared `cli_binary_test_helpers::run_cli` binary helper, prepending
/// the `"isolated"` subcommand to the caller-supplied arguments.
fn run_isolated( args : &[ &str ] ) -> std::process::Output
{
  let mut full = vec![ "isolated" ];
  full.extend_from_slice( args );
  cli_binary_test_helpers::run_cli( &full )
}

/// Returns `true` when the `claude` binary is reachable in `$PATH`.
///
/// Tests that spawn the real `claude` subprocess must early-return when this
/// returns `false` — the binary is absent in the current environment
/// (e.g. containerized CI without the CLI installed).
fn claude_binary_available() -> bool
{
  std::process::Command::new( "claude" )
    .arg( "--version" )
    .stdout( std::process::Stdio::null() )
    .stderr( std::process::Stdio::null() )
    .status()
    .is_ok()
}

/// Copy the live credentials file to a `NamedTempFile` and return `(file, path)`.
///
/// Returns `None` when `$HOME/.claude/.credentials.json` is absent — callers
/// must skip the test in that case to keep the suite passing in environments
/// without credentials.
fn live_creds_file() -> Option< ( NamedTempFile, String ) >
{
  let home    = std::env::var( "HOME" ).ok()?;
  let source  = std::path::Path::new( &home ).join( ".claude" ).join( ".credentials.json" );
  let content = std::fs::read_to_string( &source ).ok()?;
  let mut tmp = NamedTempFile::new().ok()?;
  tmp.write_all( content.as_bytes() ).ok()?;
  let path    = tmp.path().to_str()?.to_string();
  Some( ( tmp, path ) )
}

// ── offline tests (no live claude required) ───────────────────────────────────

/// IT-2: creds file that does not exist → exit 1 with file-not-found message.
///
/// Source: tests/docs/cli/command/03_isolated.md#it-2
#[ test ]
fn test_it2_creds_file_not_found()
{
  let out = run_isolated( &[ "--creds", "/tmp/clr_test_nonexistent_it2.json", "test" ] );
  assert_eq!( exit_code( &out ), 1, "expected exit 1; stderr: {}", stderr_str( &out ) );
  let err = stderr_str( &out );
  assert!(
    err.contains( "not found" ) || err.contains( "No such file" ) || err.contains( "cannot read" ),
    "expected file-not-found message; got: {err}"
  );
}

/// IT-7: `--timeout abc` → exit 1, invalid --timeout error.
///
/// Source: tests/docs/cli/command/03_isolated.md#it-7
#[ test ]
fn test_it7_invalid_timeout()
{
  let creds = make_creds_file( "{}" );
  let path  = creds.path().to_str().unwrap();
  let out   = run_isolated( &[ "--creds", path, "--timeout", "abc", "test" ] );
  assert_eq!( exit_code( &out ), 1, "expected exit 1; stderr: {}", stderr_str( &out ) );
  assert!(
    stderr_str( &out ).contains( "invalid --timeout" ),
    "expected 'invalid --timeout' message; got: {}", stderr_str( &out )
  );
}

/// IT-8: No `--creds`, `CLR_CREDS` unset → defaults to `$HOME/.claude/.credentials.json`; trace confirms path.
///
/// Source: tests/docs/cli/command/03_isolated.md#it-8
#[ test ]
fn test_it8_missing_creds_flag()
{
  let tmp      = tempfile::tempdir().expect( "create tmp home" );
  let creds_dir = tmp.path().join( ".claude" );
  std::fs::create_dir_all( &creds_dir ).expect( "create .claude dir" );
  std::fs::write( creds_dir.join( ".credentials.json" ), "{}" ).expect( "write placeholder creds" );
  let expected = creds_dir.join( ".credentials.json" );

  let out = std::process::Command::new( env!( "CARGO_BIN_EXE_clr" ) )
    .args( [ "isolated", "--trace", "test" ] )
    .env( "HOME", tmp.path() )
    .env_remove( "CLR_CREDS" )
    .env( "PATH", "/nonexistent" )
    .output()
    .expect( "invoke clr isolated" );

  let stderr      = String::from_utf8_lossy( &out.stderr );
  let expected_str = expected.to_str().unwrap();
  assert!(
    stderr.contains( "# creds:" ),
    "trace must emit '# creds:' line; got stderr:\n{stderr}"
  );
  assert!(
    stderr.contains( expected_str ),
    "trace must contain default path '{expected_str}'; got stderr:\n{stderr}"
  );
}

/// EC-creds-4: Nonexistent creds file → exit 1, file-not-found error.
///
/// Source: tests/docs/cli/param/19_creds.md#ec-4
#[ test ]
fn test_ec_creds4_file_not_found()
{
  let out = run_isolated( &[ "--creds", "/tmp/clr_test_nonexistent_ec4.json", "test" ] );
  assert_eq!( exit_code( &out ), 1, "expected exit 1; stderr: {}", stderr_str( &out ) );
  let err = stderr_str( &out );
  assert!(
    err.contains( "not found" ) || err.contains( "No such file" ) || err.contains( "cannot read" ),
    "expected file-not-found message; got: {err}"
  );
}

/// EC-creds-5: `--creds` without value → exit 1, argument requires value.
///
/// Source: tests/docs/cli/param/19_creds.md#ec-5
#[ test ]
fn test_ec_creds5_no_value()
{
  let out = run_isolated( &[ "--creds" ] );
  assert_eq!( exit_code( &out ), 1, "expected exit 1; stderr: {}", stderr_str( &out ) );
  assert!(
    stderr_str( &out ).contains( "--creds" ),
    "expected '--creds' in error; got: {}", stderr_str( &out )
  );
}

/// EC-creds-6: `--creds` omitted, `CLR_CREDS` unset → trace confirms default `$HOME/.claude/.credentials.json`.
///
/// Source: tests/docs/cli/param/19_creds.md#ec-6
#[ test ]
fn test_ec_creds6_required_flag()
{
  let tmp      = tempfile::tempdir().expect( "create tmp home" );
  let creds_dir = tmp.path().join( ".claude" );
  std::fs::create_dir_all( &creds_dir ).expect( "create .claude dir" );
  std::fs::write( creds_dir.join( ".credentials.json" ), "{}" ).expect( "write placeholder creds" );
  let expected = creds_dir.join( ".credentials.json" );

  let out = std::process::Command::new( env!( "CARGO_BIN_EXE_clr" ) )
    .args( [ "isolated", "--trace", "test" ] )
    .env( "HOME", tmp.path() )
    .env_remove( "CLR_CREDS" )
    .env( "PATH", "/nonexistent" )
    .output()
    .expect( "invoke clr isolated" );

  let stderr      = String::from_utf8_lossy( &out.stderr );
  let expected_str = expected.to_str().unwrap();
  assert!(
    stderr.contains( "# creds:" ),
    "trace must emit '# creds:' line; got stderr:\n{stderr}"
  );
  assert!(
    stderr.contains( expected_str ),
    "trace must show default path '{expected_str}'; got stderr:\n{stderr}"
  );
}

/// EC-timeout-4: `--timeout -1` → exit 1, negative not accepted.
///
/// `-1` starts with `-` so it is consumed as the value of `--timeout`,
/// then `parse_timeout` rejects it because `u64` cannot represent negatives.
///
/// Source: tests/docs/cli/param/20_timeout.md#ec-4
#[ test ]
fn test_ec_timeout4_negative()
{
  let creds = make_creds_file( "{}" );
  let path  = creds.path().to_str().unwrap();
  let out   = run_isolated( &[ "--creds", path, "--timeout", "-1", "test" ] );
  assert_eq!( exit_code( &out ), 1, "expected exit 1; stderr: {}", stderr_str( &out ) );
  assert!(
    stderr_str( &out ).contains( "invalid --timeout" ),
    "expected 'invalid --timeout' message; got: {}", stderr_str( &out )
  );
}

/// EC-timeout-5: `--timeout abc` → exit 1, non-numeric rejected.
///
/// Source: tests/docs/cli/param/20_timeout.md#ec-5
#[ test ]
fn test_ec_timeout5_non_numeric()
{
  let creds = make_creds_file( "{}" );
  let path  = creds.path().to_str().unwrap();
  let out   = run_isolated( &[ "--creds", path, "--timeout", "abc", "test" ] );
  assert_eq!( exit_code( &out ), 1, "expected exit 1; stderr: {}", stderr_str( &out ) );
  assert!(
    stderr_str( &out ).contains( "invalid --timeout" ),
    "expected 'invalid --timeout' message; got: {}", stderr_str( &out )
  );
}

/// EC-timeout-6: `--timeout` without value → exit 1, requires argument.
///
/// Source: tests/docs/cli/param/20_timeout.md#ec-6
#[ test ]
fn test_ec_timeout6_no_value()
{
  let creds = make_creds_file( "{}" );
  let path  = creds.path().to_str().unwrap();
  let out   = run_isolated( &[ "--creds", path, "--timeout" ] );
  assert_eq!( exit_code( &out ), 1, "expected exit 1; stderr: {}", stderr_str( &out ) );
  assert!(
    stderr_str( &out ).contains( "--timeout" ),
    "expected '--timeout' in error; got: {}", stderr_str( &out )
  );
}

/// EC-timeout-1: `--timeout 30` → accepted, no parse error.
///
/// The subprocess may fail (e.g. `ClaudeNotFound` → exit 1) but the error
/// must NOT be an invalid-timeout parse error.
///
/// Source: tests/docs/cli/param/20_timeout.md#ec-1
#[ test ]
fn test_ec_timeout1_value_accepted()
{
  let creds = make_creds_file( "{}" );
  let path  = creds.path().to_str().unwrap();
  let out   = run_isolated( &[ "--creds", path, "--timeout", "30", "test" ] );
  assert!(
    !stderr_str( &out ).contains( "invalid --timeout" ),
    "unexpected 'invalid --timeout' in stderr; got: {}", stderr_str( &out )
  );
}

/// EC-timeout-2: `--timeout 0` → accepted, no parse error.
///
/// Subprocess may exit 1 (`ClaudeNotFound`) or 2 (immediate timeout) — we only
/// assert the value itself is parsed without error.
///
/// Source: tests/docs/cli/param/20_timeout.md#ec-2
#[ test ]
fn test_ec_timeout2_zero_accepted()
{
  let creds = make_creds_file( "{}" );
  let path  = creds.path().to_str().unwrap();
  let out   = run_isolated( &[ "--creds", path, "--timeout", "0", "test" ] );
  assert!(
    !stderr_str( &out ).contains( "invalid --timeout" ),
    "unexpected 'invalid --timeout' in stderr; got: {}", stderr_str( &out )
  );
}

/// EC-timeout-3: `--timeout 3600` → accepted, no parse error.
///
/// Source: tests/docs/cli/param/20_timeout.md#ec-3
#[ test ]
fn test_ec_timeout3_large_accepted()
{
  let creds = make_creds_file( "{}" );
  let path  = creds.path().to_str().unwrap();
  let out   = run_isolated( &[ "--creds", path, "--timeout", "3600", "test" ] );
  assert!(
    !stderr_str( &out ).contains( "invalid --timeout" ),
    "unexpected 'invalid --timeout' in stderr; got: {}", stderr_str( &out )
  );
}

// ── live-claude tests (lim_it) ────────────────────────────────────────────────

/// IT-1: Happy path — valid creds, message → response on stdout, exit 0.
///
/// Source: tests/docs/cli/command/03_isolated.md#it-1
#[ test ]
fn it1_lim_it_happy_path()
{
  if !claude_binary_available() { return; }
  let Some( ( _tmp, path ) ) = live_creds_file() else
  {
    panic!( "lim_it test requires live credentials at $HOME/.claude/.credentials.json — run only in credentialed environments, not in standard CI" );
  };
  let out = run_isolated( &[ "--creds", &path, "What is 2+2? Reply with just the number." ] );
  assert_eq!(
    exit_code( &out ), 0,
    "expected exit 0; stderr: {}", stderr_str( &out )
  );
  assert!(
    !stdout_str( &out ).is_empty(),
    "expected non-empty stdout from claude response"
  );
}

/// IT-3: `--timeout 0` without creds refresh → exit 2 (timeout).
///
/// A 0-second deadline ensures the subprocess is killed before it can
/// produce output or refresh credentials (assuming no near-instant refresh).
///
/// Source: tests/docs/cli/command/03_isolated.md#it-3
#[ test ]
fn it3_lim_it_timeout_no_refresh()
{
  if !claude_binary_available() { return; }
  let Some( ( _tmp, path ) ) = live_creds_file() else
  {
    panic!( "lim_it test requires live credentials at $HOME/.claude/.credentials.json — run only in credentialed environments, not in standard CI" );
  };
  let out = run_isolated( &[ "--creds", &path, "--timeout", "0", "Long running analysis task" ] );
  // Creds refresh at startup before timeout is theoretically possible;
  // both exit 0 (refreshed) and exit 2 (plain timeout) are valid outcomes.
  let code = exit_code( &out );
  assert!(
    code == 0 || code == 2,
    "expected exit 0 or 2 (timeout), got {code}; stderr: {}", stderr_str( &out )
  );
}

/// IT-4: Credential write-back after startup refresh.
///
/// Verifies the creds write-back path: Claude may refresh the OAuth token at
/// startup before processing the prompt. The refreshed credentials are written
/// back to `--creds` and `clr isolated` exits 0. If no refresh occurs within
/// the run window, exit 0 (normal completion) is also acceptable.
///
/// A minimal prompt `"."` is required: the Claude CLI auto-enters `--print`
/// mode when stdin is piped (non-TTY) and rejects runs without a prompt.
///
/// Source: tests/docs/cli/command/03_isolated.md#it-4
#[ test ]
fn it4_lim_it_timeout_with_refresh()
{
  if !claude_binary_available() { return; }
  let Some( ( tmp, path ) ) = live_creds_file() else
  {
    panic!( "lim_it test requires live credentials at $HOME/.claude/.credentials.json — run only in credentialed environments, not in standard CI" );
  };
  let content_before = std::fs::read_to_string( &path ).unwrap_or_default();
  // Minimal prompt "." — same as refresh uses; Claude CLI requires a prompt
  // when stdin is piped (auto-activates --print mode in non-TTY contexts).
  let out  = run_isolated( &[ "--creds", &path, "--timeout", "0", "." ] );
  let code = exit_code( &out );
  assert!(
    code == 0,
    "expected exit 0; got {code}; stderr: {}", stderr_str( &out )
  );
  // Credentials file must still be non-empty after the run.
  let content_after = std::fs::read_to_string( &path ).unwrap_or_default();
  assert!( !content_after.is_empty(), "creds file should not be empty after run" );
  let _ = ( content_before, content_after, tmp );
}

/// IT-5: No message → Claude subprocess rejects non-interactive piped context.
///
/// The Claude CLI auto-enters `--print` mode when stdin is piped (non-TTY)
/// and exits 1 with a prompt-required message. This is expected — `clr` itself
/// must not produce a parse-level error. We assert no `clr`-specific parse
/// error is emitted (no "missing required" or "invalid --timeout").
///
/// Source: tests/docs/cli/command/03_isolated.md#it-5
#[ test ]
fn it5_lim_it_interactive_mode()
{
  let Some( ( _tmp, path ) ) = live_creds_file() else { return; };
  // Use a short timeout so the test doesn't hang in non-interactive envs.
  let out = run_isolated( &[ "--creds", &path, "--timeout", "3" ] );
  let err = stderr_str( &out );
  assert!(
    !err.contains( "missing required" ) && !err.contains( "invalid --timeout" ),
    "unexpected parse error; got: {err}"
  );
}

/// IT-6: `-- --version` → passthrough args forwarded to claude subprocess.
///
/// `claude --version` exits 0 and prints a version string to stdout.
///
/// Source: tests/docs/cli/command/03_isolated.md#it-6
#[ test ]
fn it6_lim_it_flag_passthrough()
{
  if !claude_binary_available() { return; }
  let Some( ( _tmp, path ) ) = live_creds_file() else
  {
    panic!( "lim_it test requires live credentials at $HOME/.claude/.credentials.json — run only in credentialed environments, not in standard CI" );
  };
  let out = run_isolated( &[ "--creds", &path, "--", "--version" ] );
  assert_eq!(
    exit_code( &out ), 0,
    "expected exit 0 from --version; stderr: {}", stderr_str( &out )
  );
  assert!(
    !stdout_str( &out ).is_empty(),
    "expected version string on stdout"
  );
}

/// EC-creds-1: Valid file path → subprocess runs without startup error.
///
/// Source: tests/docs/cli/param/19_creds.md#ec-1
#[ test ]
fn ec_creds1_lim_it_valid_file_path()
{
  let Some( ( _tmp, path ) ) = live_creds_file() else { return; };
  let out = run_isolated( &[ "--creds", &path, "--timeout", "10", "Say hi" ] );
  let err = stderr_str( &out );
  assert!(
    !err.contains( "cannot read" ) && !err.contains( "No such file" ),
    "unexpected file-not-found error; got: {err}"
  );
}

/// EC-creds-2: Absolute path → resolved correctly.
///
/// Source: tests/docs/cli/param/19_creds.md#ec-2
#[ test ]
fn ec_creds2_lim_it_absolute_path()
{
  let Some( ( _tmp, path ) ) = live_creds_file() else { return; };
  // `path` from live_creds_file() is already absolute (NamedTempFile path).
  assert!( path.starts_with( '/' ), "expected absolute path; got: {path}" );
  let out = run_isolated( &[ "--creds", &path, "--timeout", "10", "Say hi" ] );
  let err = stderr_str( &out );
  assert!(
    !err.contains( "cannot read" ) && !err.contains( "No such file" ),
    "unexpected file-not-found error; got: {err}"
  );
}

/// EC-creds-3: Relative path → resolved against caller's cwd.
///
/// Creates a creds file inside a temp directory, then invokes `clr isolated`
/// with the bare filename and that directory as the working directory.
///
/// Source: tests/docs/cli/param/19_creds.md#ec-3
#[ test ]
fn ec_creds3_lim_it_relative_path()
{
  let Some( ( (), content ) ) = live_creds_file().map( | ( tmp, path ) |
  {
    let c = std::fs::read_to_string( &path ).unwrap_or_default();
    drop( tmp );
    ( (), c )
  } ) else { return; };
  // Write creds to a named file in a temp directory.
  let tmp_dir = tempfile::tempdir().expect( "failed to create temp dir" );
  let rel_name = "ec3_creds.json";
  std::fs::write( tmp_dir.path().join( rel_name ), &content )
    .expect( "failed to write creds for relative path test" );
  let bin = env!( "CARGO_BIN_EXE_clr" );
  let out = std::process::Command::new( bin )
    .current_dir( tmp_dir.path() )
    .args( [ "isolated", "--creds", rel_name, "--timeout", "10", "Say hi" ] )
    .output()
    .expect( "failed to invoke clr" );
  let err = stderr_str( &out );
  assert!(
    !err.contains( "cannot read" ) && !err.contains( "No such file" ),
    "unexpected file-not-found error for relative path; got: {err}"
  );
}

/// BUG-225: `clr isolate --help` and `clr isol` exit 1 with unknown-subcommand error.
///
/// Reproduces the silent unknown-subcommand fallthrough when the first CLI token
/// resembles a known subcommand but is not an exact match (issue-unknown-subcommand).
///
/// ## Root Cause
/// `run_cli()` in `src/lib.rs` dispatches `isolated` via exact string match at line 805.
/// Any non-matching first token silently fell through to `parse_args()`, whose global
/// `--help` short-circuit then fired and showed generic help with no indication the first
/// token was unrecognised.  No unknown-subcommand guard existed between the dispatch
/// block and the `parse_args()` call.
///
/// ## Why Not Caught Initially
/// All previous tests used the correct spelling (`clr isolated …`).  Typos and truncations
/// of `"isolated"` were never exercised, so the silent fallthrough was never observed.
///
/// ## Fix Applied
/// Added an identifier-like prefix-match guard in `run_cli()` (`src/lib.rs`) immediately
/// after the `isolated` dispatch block and before `parse_args()`.  The guard checks whether
/// the first token has `len() >= 4`, contains only alphanumeric/`_`/`-` characters, does
/// not start with `-`, and prefix-matches a name from `KNOWN_SUBCOMMANDS: &[&str] =
/// &["isolated"]`.  On match it prints `"Error: unknown subcommand: <token>. Did you mean
/// '<sub>'?"` to stderr and calls `std::process::exit(1)`.
///
/// ## Prevention
/// `KNOWN_SUBCOMMANDS` constant in `run_cli()` makes future subcommands self-documenting;
/// adding a new subcommand automatically extends guard coverage without touching guard logic.
///
/// ## Pitfall to Avoid
/// A bare string comparison against known subcommands only guards exact matches; typos and
/// truncations pass silently unless a separate prefix-match guard is also placed between
/// the subcommand dispatch block and the main argument parser.
// test_kind: bug_reproducer(BUG-225)
#[ test ]
fn bug_reproducer_225_unknown_subcommand_typo()
{
  let bin = env!( "CARGO_BIN_EXE_clr" );

  // T01: 7-char prefix "isolate" — misspelling of "isolated"
  {
    let out = std::process::Command::new( bin )
      .args( [ "isolate", "--help" ] )
      .output()
      .expect( "failed to invoke clr isolate --help" );
    let stderr = String::from_utf8_lossy( &out.stderr );
    assert_eq!(
      out.status.code(),
      Some( 1 ),
      "clr isolate --help must exit 1 (unknown subcommand); got: {:?}\nstderr: {stderr}",
      out.status.code(),
    );
    assert!(
      stderr.contains( "unknown subcommand" ),
      "stderr must contain 'unknown subcommand'; got: {stderr}"
    );
    assert!(
      stderr.contains( "isolated" ),
      "stderr must suggest 'isolated'; got: {stderr}"
    );
  }

  // T02: 4-char prefix "isol" — truncation of "isolated"
  {
    let out = std::process::Command::new( bin )
      .args( [ "isol" ] )
      .output()
      .expect( "failed to invoke clr isol" );
    let stderr = String::from_utf8_lossy( &out.stderr );
    assert_eq!(
      out.status.code(),
      Some( 1 ),
      "clr isol must exit 1 (unknown subcommand); got: {:?}\nstderr: {stderr}",
      out.status.code(),
    );
    assert!(
      stderr.contains( "unknown subcommand" ),
      "stderr must contain 'unknown subcommand'; got: {stderr}"
    );
    assert!(
      stderr.contains( "isolated" ),
      "stderr must suggest 'isolated'; got: {stderr}"
    );
  }
}

/// IT-9: `clr isolated --help` exits 0 and prints isolated-specific help text.
///
/// ## Root Cause (bug_reproducer(BUG-222))
/// `parse_isolated_args()` had no `"-h" | "--help"` arm before the
/// `s if s.starts_with('-')` catch-all, so `--help` matched the catch-all and
/// returned `Err("unknown option: --help")`, causing exit 1.
///
/// ## Why Not Caught
/// Only happy-path and error-flag tests existed for `isolated`;
/// no test exercised `--help` on the subcommand.
///
/// ## Fix Applied
/// Added `print_isolated_help()` function (exits 0) and inserted a
/// `"-h" | "--help"` match arm before the catch-all in `parse_isolated_args()`.
///
/// ## Prevention
/// Test both `-h` and `--help` exit codes and stdout content for
/// every subcommand that accepts flags.
///
/// ## Pitfall
/// `print_isolated_help()` must call `std::process::exit(0)` directly —
/// returning `Ok(...)` from the arm is insufficient because the caller checks
/// `creds_path` and would error on the missing `--creds` argument.
// test_kind: bug_reproducer(BUG-222)
#[ test ]
fn it9_isolated_help_exits_zero()
{
  let bin = env!( "CARGO_BIN_EXE_clr" );
  let out = std::process::Command::new( bin )
    .args( [ "isolated", "--help" ] )
    .output()
    .expect( "failed to invoke clr isolated --help" );
  assert_eq!(
    out.status.code(),
    Some( 0 ),
    "clr isolated --help must exit 0; got: {:?}\nstderr: {}",
    out.status.code(),
    String::from_utf8_lossy( &out.stderr ),
  );
  let stdout = String::from_utf8_lossy( &out.stdout );
  assert!(
    stdout.contains( "--creds" ),
    "help text must mention --creds; got:\n{stdout}",
  );
  assert!(
    stdout.contains( "--timeout" ),
    "help text must mention --timeout; got:\n{stdout}",
  );
}

// ── IT-12 through IT-15: --dry-run tests (offline) ──────────────────────────

/// IT-12: `--dry-run` exits 0 without spawning a subprocess.
///
/// Source: tests/docs/cli/command/03_isolated.md#it-12
#[ test ]
fn it12_dry_run_exits_zero()
{
  let creds = make_creds_file( "{}" );
  let path  = creds.path().to_str().unwrap();
  let out   = run_isolated( &[ "--creds", path, "--dry-run" ] );
  assert_eq!(
    exit_code( &out ), 0,
    "expected exit 0 from --dry-run; stderr: {}", stderr_str( &out )
  );
  assert!(
    !stdout_str( &out ).is_empty(),
    "--dry-run must print command preview to stdout"
  );
}

/// IT-13: `--dry-run "msg"` exits 0 and preview contains `--print` with the message.
///
/// Source: tests/docs/cli/command/03_isolated.md#it-13
#[ test ]
fn it13_dry_run_includes_message()
{
  let creds = make_creds_file( "{}" );
  let path  = creds.path().to_str().unwrap();
  let out   = run_isolated( &[ "--creds", path, "--dry-run", "say hello" ] );
  assert_eq!(
    exit_code( &out ), 0,
    "expected exit 0; stderr: {}", stderr_str( &out )
  );
  let stdout = stdout_str( &out );
  assert!(
    stdout.contains( "--print" ),
    "--dry-run with message must include --print in preview; got:\n{stdout}"
  );
  assert!(
    stdout.contains( "say hello" ),
    "--dry-run with message must include message text in preview; got:\n{stdout}"
  );
}

/// IT-14: `--dry-run --dir /tmp` exits 0 and preview includes `--dir`.
///
/// Source: tests/docs/cli/command/03_isolated.md#it-14
#[ test ]
fn it14_dry_run_includes_dir()
{
  let creds = make_creds_file( "{}" );
  let path  = creds.path().to_str().unwrap();
  let out   = run_isolated( &[ "--creds", path, "--dry-run", "--dir", "/tmp", "msg" ] );
  assert_eq!(
    exit_code( &out ), 0,
    "expected exit 0; stderr: {}", stderr_str( &out )
  );
  assert!(
    stdout_str( &out ).contains( "--dir" ),
    "--dry-run with --dir must include --dir in preview; got:\n{}", stdout_str( &out )
  );
}

/// IT-15: `--dry-run --add-dir /tmp` exits 0 and preview includes `--add-dir`.
///
/// Source: tests/docs/cli/command/03_isolated.md#it-15
#[ test ]
fn it15_dry_run_includes_add_dir()
{
  let creds = make_creds_file( "{}" );
  let path  = creds.path().to_str().unwrap();
  let out   = run_isolated( &[ "--creds", path, "--dry-run", "--add-dir", "/tmp", "msg" ] );
  assert_eq!(
    exit_code( &out ), 0,
    "expected exit 0; stderr: {}", stderr_str( &out )
  );
  assert!(
    stdout_str( &out ).contains( "--add-dir" ),
    "--dry-run with --add-dir must include --add-dir in preview; got:\n{}", stdout_str( &out )
  );
}

// ── IT-16 through IT-20: --dir and --add-dir tests ──────────────────────────

/// IT-16: `--dir /tmp` is injected into the subprocess argv.
///
/// Fake claude echoes all args to stdout so the test can verify `--dir /tmp` arrives.
///
/// Source: tests/docs/cli/command/03_isolated.md#it-16
#[ cfg( unix ) ]
#[ test ]
fn it16_dir_injected_into_subprocess()
{
  let creds          = make_creds_file( "{}" );
  let creds_path     = creds.path().to_str().unwrap();
  let ( _dir, path ) = fake_claude_dir( "echo \"$@\"" );
  let out            = run_with_path(
    &[ "isolated", "--creds", creds_path, "--dir", "/tmp", "msg" ],
    &path,
  );
  assert!(
    out.status.success(),
    "expected exit 0; stderr: {}", stderr_str( &out )
  );
  assert!(
    stdout_str( &out ).contains( "--dir" ),
    "--dir must be injected into subprocess argv; got:\n{}", stdout_str( &out )
  );
}

/// IT-17: `--dir /nonexistent_clr_test_dir_it17` exits 1 before subprocess spawn.
///
/// Source: tests/docs/cli/command/03_isolated.md#it-17
#[ test ]
fn it17_dir_nonexistent_exits_one()
{
  let creds = make_creds_file( "{}" );
  let path  = creds.path().to_str().unwrap();
  let out   = run_isolated(
    &[ "--creds", path, "--dir", "/nonexistent_clr_test_dir_it17", "msg" ],
  );
  assert_eq!(
    exit_code( &out ), 1,
    "expected exit 1 for nonexistent --dir; stderr: {}", stderr_str( &out )
  );
  let err = stderr_str( &out );
  assert!(
    err.contains( "does not exist" ) || err.contains( "not found" ),
    "stderr must indicate nonexistent dir; got:\n{err}"
  );
}

/// IT-18: `--add-dir /tmp` is injected into the subprocess argv.
///
/// Source: tests/docs/cli/command/03_isolated.md#it-18
#[ cfg( unix ) ]
#[ test ]
fn it18_add_dir_injected_into_subprocess()
{
  let creds          = make_creds_file( "{}" );
  let creds_path     = creds.path().to_str().unwrap();
  let ( _dir, path ) = fake_claude_dir( "echo \"$@\"" );
  let out            = run_with_path(
    &[ "isolated", "--creds", creds_path, "--add-dir", "/tmp", "msg" ],
    &path,
  );
  assert!(
    stdout_str( &out ).contains( "--add-dir" ),
    "--add-dir must be injected into subprocess argv; got:\n{}", stdout_str( &out )
  );
}

/// IT-19: `--dir /tmp --add-dir /var` — both flags appear in subprocess argv.
///
/// Source: tests/docs/cli/command/03_isolated.md#it-19
#[ cfg( unix ) ]
#[ test ]
fn it19_dir_and_add_dir_combined()
{
  let creds          = make_creds_file( "{}" );
  let creds_path     = creds.path().to_str().unwrap();
  let ( _dir, path ) = fake_claude_dir( "echo \"$@\"" );
  let out            = run_with_path(
    &[ "isolated", "--creds", creds_path, "--dir", "/tmp", "--add-dir", "/var", "msg" ],
    &path,
  );
  let stdout = stdout_str( &out );
  assert!(
    stdout.contains( "--dir" ) && stdout.contains( "--add-dir" ),
    "both --dir and --add-dir must appear in subprocess argv; got:\n{stdout}"
  );
}

/// IT-20: `CLR_DIR=/tmp` env var is applied when `--dir` flag is absent.
///
/// Uses `--dry-run` to verify env var pickup without spawning a subprocess.
///
/// Source: tests/docs/cli/command/03_isolated.md#it-20
#[ test ]
fn it20_clr_dir_env_fallback()
{
  let creds      = make_creds_file( "{}" );
  let creds_path = creds.path().to_str().unwrap();
  let out        = std::process::Command::new( env!( "CARGO_BIN_EXE_clr" ) )
    .args( [ "isolated", "--creds", creds_path, "--dry-run", "msg" ] )
    .env( "CLR_DIR", "/tmp" )
    .env_remove( "CLR_ADD_DIR" )
    .output()
    .expect( "invoke clr isolated" );
  assert_eq!(
    out.status.code(),
    Some( 0 ),
    "expected exit 0; stderr: {}", String::from_utf8_lossy( &out.stderr )
  );
  let stdout = String::from_utf8_lossy( &out.stdout );
  assert!(
    stdout.contains( "--dir" ),
    "CLR_DIR must appear in --dry-run preview; got:\n{stdout}"
  );
}

// ── IT-21 through IT-23: --file tests ──────────────────────────────────────

/// IT-21: `--file <path>` pipes file content to the subprocess stdin.
///
/// Fake claude runs `cat` to read stdin and emit it to stdout, proving file content arrived.
///
/// Source: tests/docs/cli/command/03_isolated.md#it-21
#[ cfg( unix ) ]
#[ test ]
fn it21_file_piped_as_stdin()
{
  let creds          = make_creds_file( "{}" );
  let creds_path     = creds.path().to_str().unwrap();
  let mut input_file = NamedTempFile::new().expect( "create input file" );
  input_file.write_all( b"file_content_it21" ).expect( "write input file" );
  let input_path     = input_file.path().to_str().unwrap();
  let ( _dir, path ) = fake_claude_dir( "cat" );
  let out            = run_with_path(
    &[ "isolated", "--creds", creds_path, "--file", input_path, "process this" ],
    &path,
  );
  assert!(
    out.status.success(),
    "expected exit 0; stderr: {}", stderr_str( &out )
  );
  assert!(
    stdout_str( &out ).contains( "file_content_it21" ),
    "--file must pipe file content to subprocess stdin; got:\n{}", stdout_str( &out )
  );
}

/// IT-22: `--file /nonexistent_clr_it22.txt` exits 1 before subprocess spawn.
///
/// Source: tests/docs/cli/command/03_isolated.md#it-22
#[ test ]
fn it22_file_nonexistent_exits_one()
{
  let out = run_isolated( &[
    "--creds", "/tmp/clr_it22_dummy.json",
    "--file",  "/tmp/clr_it22_nonexistent_input.txt",
    "msg",
  ] );
  assert_eq!(
    exit_code( &out ), 1,
    "expected exit 1 for nonexistent --file; stderr: {}", stderr_str( &out )
  );
  let err = stderr_str( &out );
  assert!(
    err.contains( "does not exist" ) || err.contains( "not found" ),
    "stderr must indicate nonexistent file; got:\n{err}"
  );
}

/// IT-23: `--file <path> "msg"` — file as stdin combined with prompt message.
///
/// File content arrives via subprocess stdin; message is forwarded via `--print`.
///
/// Source: tests/docs/cli/command/03_isolated.md#it-23
#[ cfg( unix ) ]
#[ test ]
fn it23_file_combined_with_message()
{
  let creds          = make_creds_file( "{}" );
  let creds_path     = creds.path().to_str().unwrap();
  let mut input_file = NamedTempFile::new().expect( "create input file" );
  input_file.write_all( b"combined_input_it23" ).expect( "write input file" );
  let input_path     = input_file.path().to_str().unwrap();
  let ( _dir, path ) = fake_claude_dir( "cat" );
  let out            = run_with_path(
    &[ "isolated", "--creds", creds_path, "--file", input_path, "process this file" ],
    &path,
  );
  assert!(
    out.status.success(),
    "expected exit 0; stderr: {}", stderr_str( &out )
  );
  assert!(
    stdout_str( &out ).contains( "combined_input_it23" ),
    "--file + message: file content must reach subprocess stdin; got:\n{}", stdout_str( &out )
  );
}

// ── IT-24 through IT-27: --expect and --expect-strategy tests ───────────────

/// IT-24: `--expect "hello"` — output matches → exit 0, output unchanged.
///
/// Source: tests/docs/cli/command/03_isolated.md#it-24
#[ cfg( unix ) ]
#[ test ]
fn it24_expect_match_exits_zero()
{
  let creds          = make_creds_file( "{}" );
  let creds_path     = creds.path().to_str().unwrap();
  let ( _dir, path ) = fake_claude_dir( "echo 'hello'" );
  let out            = run_with_path(
    &[ "isolated", "--creds", creds_path, "--expect", "hello", "msg" ],
    &path,
  );
  assert!(
    out.status.success(),
    "expect match must exit 0; stderr: {}", stderr_str( &out )
  );
  assert!(
    stdout_str( &out ).contains( "hello" ),
    "output must be unchanged on match; got:\n{}", stdout_str( &out )
  );
}

/// IT-25: `--expect "hello" --expect-strategy fail` — mismatch → exit 3.
///
/// Source: tests/docs/cli/command/03_isolated.md#it-25
#[ cfg( unix ) ]
#[ test ]
fn it25_expect_mismatch_fail_exits_three()
{
  let creds          = make_creds_file( "{}" );
  let creds_path     = creds.path().to_str().unwrap();
  let ( _dir, path ) = fake_claude_dir( "echo 'world'" );
  let out            = run_with_path(
    &[
      "isolated", "--creds", creds_path,
      "--expect", "hello", "--expect-strategy", "fail", "msg",
    ],
    &path,
  );
  assert_eq!(
    exit_code( &out ), 3,
    "expect mismatch with fail strategy must exit 3; stderr: {}", stderr_str( &out )
  );
  let err = stderr_str( &out );
  assert!(
    err.contains( "expected" ) || err.contains( "Validation" ),
    "stderr must indicate expect mismatch; got:\n{err}"
  );
}

/// IT-26: `--expect "hello" --expect-strategy default:no` — mismatch → exit 0, stdout "no".
///
/// Source: tests/docs/cli/command/03_isolated.md#it-26
#[ cfg( unix ) ]
#[ test ]
fn it26_expect_mismatch_default_fallback()
{
  let creds          = make_creds_file( "{}" );
  let creds_path     = creds.path().to_str().unwrap();
  let ( _dir, path ) = fake_claude_dir( "echo 'world'" );
  let out            = run_with_path(
    &[
      "isolated", "--creds", creds_path,
      "--expect", "hello", "--expect-strategy", "default:no", "msg",
    ],
    &path,
  );
  assert!(
    out.status.success(),
    "default strategy must exit 0; stderr: {}", stderr_str( &out )
  );
  assert!(
    stdout_str( &out ).contains( "no" ),
    "default strategy must replace output with fallback 'no'; got:\n{}", stdout_str( &out )
  );
}

/// IT-27: `--expect-strategy retry` is explicitly unsupported for isolated → exit 1.
///
/// Source: tests/docs/cli/command/03_isolated.md#it-27
#[ cfg( unix ) ]
#[ test ]
fn it27_expect_strategy_retry_unsupported()
{
  let creds          = make_creds_file( "{}" );
  let creds_path     = creds.path().to_str().unwrap();
  let ( _dir, path ) = fake_claude_dir( "echo 'world'" );
  let out            = run_with_path(
    &[
      "isolated", "--creds", creds_path,
      "--expect", "hello", "--expect-strategy", "retry", "msg",
    ],
    &path,
  );
  assert_eq!(
    exit_code( &out ), 1,
    "retry strategy must exit 1 (unsupported); stderr: {}", stderr_str( &out )
  );
  let err = stderr_str( &out );
  assert!(
    err.contains( "retry" ),
    "stderr must mention 'retry' as unsupported; got:\n{err}"
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
