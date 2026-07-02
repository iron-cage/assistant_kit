//! Tests for `--no-compact-window` Flag and `CLAUDE_CODE_AUTO_COMPACT_WINDOW` Injection
//!
//! ## Purpose
//!
//! Verify that `CLAUDE_CODE_AUTO_COMPACT_WINDOW=200000` is injected by default for all four
//! running commands (`run`, `ask`, `isolated`, `refresh`) and that `--no-compact-window` /
//! `CLR_NO_COMPACT_WINDOW` correctly suppresses it. All 12 functions use `--dry-run` to
//! inspect the assembled subprocess environment without spawning Claude Code.
//!
//! ## Spec Coverage
//!
//! | Spec | Cases Covered |
//! |------|---------------|
//! | `tests/docs/cli/param_group/06_running_commands.md` | RC-3, RC-4, RC-5, RC-6, RC-7 |
//! | `tests/docs/cli/env_param/03_auto_compact_window.md` | acw:EC-1..EC-5, acw:EC-7..EC-9 |
//! | `tests/docs/cli/param/075_no_compact_window.md` | ncw:EC-1..EC-8 |
//!
//! ## Exclusions
//!
//! 15 of the 27 spec cases are excluded: RC-1/RC-2 (cross-invocation dry-run vs trace
//! equality), acw:EC-6/ncw:EC-9 (trace-output variant requiring PATH manipulation), RC-8
//! (journaling, covered by `journal_integration_test.rs`), RC-9 (timeout semantics, covered
//! by `timeout_test.rs`), plus 9 incidental cases already verified from other spec angles.

mod cli_binary_test_helpers;
use cli_binary_test_helpers::{ run_cli, run_cli_with_env, stdout_str, stderr_str, make_creds_file };

// RC-3/run, acw:EC-1, ncw:EC-1
#[ test ]
fn default_injection_run()
{
  let output = stdout_str( &run_cli( &[ "--dry-run", "t" ] ) );
  assert!(
    output.contains( "CLAUDE_CODE_AUTO_COMPACT_WINDOW=200000" ),
    "Default injection must be present in run dry-run stdout. Got:\n{output}"
  );
}

// RC-4, acw:EC-2, ncw:EC-2
#[ test ]
fn flag_suppresses_for_run()
{
  let output = stdout_str( &run_cli( &[ "--no-compact-window", "--dry-run", "t" ] ) );
  assert!(
    !output.contains( "CLAUDE_CODE_AUTO_COMPACT_WINDOW" ),
    "--no-compact-window must suppress injection for run. Got:\n{output}"
  );
}

// RC-7, acw:EC-3, ncw:EC-6
#[ test ]
fn env_one_suppresses_for_run()
{
  let output = stdout_str( &run_cli_with_env(
    &[ "--dry-run", "t" ],
    &[ ( "CLR_NO_COMPACT_WINDOW", "1" ) ],
  ) );
  assert!(
    !output.contains( "CLAUDE_CODE_AUTO_COMPACT_WINDOW" ),
    "CLR_NO_COMPACT_WINDOW=1 must suppress injection. Got:\n{output}"
  );
}

// acw:EC-4
#[ test ]
fn env_true_suppresses_for_run()
{
  let output = stdout_str( &run_cli_with_env(
    &[ "--dry-run", "t" ],
    &[ ( "CLR_NO_COMPACT_WINDOW", "true" ) ],
  ) );
  assert!(
    !output.contains( "CLAUDE_CODE_AUTO_COMPACT_WINDOW" ),
    "CLR_NO_COMPACT_WINDOW=true must suppress injection. Got:\n{output}"
  );
}

// acw:EC-9
#[ test ]
fn env_zero_does_not_suppress()
{
  let output = stdout_str( &run_cli_with_env(
    &[ "--dry-run", "t" ],
    &[ ( "CLR_NO_COMPACT_WINDOW", "0" ) ],
  ) );
  assert!(
    output.contains( "CLAUDE_CODE_AUTO_COMPACT_WINDOW=200000" ),
    "CLR_NO_COMPACT_WINDOW=0 (falsy) must NOT suppress injection. Got:\n{output}"
  );
}

// acw:EC-5 / ncw:EC-8 — dry-run WYSIWYG fidelity (var present).
//
// Body is intentionally identical to default_injection_run. The dry-run output is
// What-You-See-Is-What-the-subprocess-Gets; checking that the env var IS present in
// dry-run output tests this fidelity directly. A separate named function makes this
// dry-run fidelity an explicit, named coverage point independent of the injection test.
#[ test ]
fn dry_run_shows_var_when_active()
{
  let output = stdout_str( &run_cli( &[ "--dry-run", "t" ] ) );
  assert!(
    output.contains( "CLAUDE_CODE_AUTO_COMPACT_WINDOW=200000" ),
    "dry-run must reveal injected env var when active (WYSIWYG fidelity). Got:\n{output}"
  );
}

// acw:EC-5 / ncw:EC-8 — dry-run WYSIWYG fidelity (var absent).
//
// Body is intentionally identical to flag_suppresses_for_run. Dry-run must omit the
// suppressed var — same WYSIWYG property applied to the suppressed case. A separate
// named function makes suppression-fidelity an explicit, named coverage point.
#[ test ]
fn dry_run_shows_no_var_when_suppressed()
{
  let output = stdout_str( &run_cli( &[ "--no-compact-window", "--dry-run", "t" ] ) );
  assert!(
    !output.contains( "CLAUDE_CODE_AUTO_COMPACT_WINDOW" ),
    "dry-run must omit suppressed env var (WYSIWYG fidelity). Got:\n{output}"
  );
}

// ncw:EC-5
#[ test ]
fn flag_suppresses_for_ask()
{
  let output = stdout_str( &run_cli( &[ "ask", "--no-compact-window", "--dry-run", "t" ] ) );
  assert!(
    !output.contains( "CLAUDE_CODE_AUTO_COMPACT_WINDOW" ),
    "--no-compact-window must suppress injection for ask. Got:\n{output}"
  );
}

// acw:EC-7, RC-3/isolated
#[ test ]
fn default_injection_isolated()
{
  let creds = make_creds_file( "{}" );
  let tmp_path = creds.path().to_str().unwrap();
  let output = stderr_str( &run_cli( &[ "isolated", "--creds", tmp_path, "--dry-run" ] ) );
  assert!(
    output.contains( "CLAUDE_CODE_AUTO_COMPACT_WINDOW=200000" ),
    "Default injection must be present in isolated dry-run stderr. Got:\n{output}"
  );
}

// RC-5, ncw:EC-3
#[ test ]
fn flag_suppresses_for_isolated()
{
  let creds = make_creds_file( "{}" );
  let tmp_path = creds.path().to_str().unwrap();
  let output = stderr_str( &run_cli( &[
    "isolated", "--creds", tmp_path, "--no-compact-window", "--dry-run",
  ] ) );
  assert!(
    !output.contains( "CLAUDE_CODE_AUTO_COMPACT_WINDOW" ),
    "--no-compact-window must suppress injection for isolated. Got:\n{output}"
  );
}

// acw:EC-8, RC-3/refresh
#[ test ]
fn default_injection_refresh()
{
  let creds = make_creds_file( "{}" );
  let tmp_path = creds.path().to_str().unwrap();
  let output = stderr_str( &run_cli( &[ "refresh", "--creds", tmp_path, "--dry-run" ] ) );
  assert!(
    output.contains( "CLAUDE_CODE_AUTO_COMPACT_WINDOW=200000" ),
    "Default injection must be present in refresh dry-run stderr. Got:\n{output}"
  );
}

// RC-6, ncw:EC-4
#[ test ]
fn flag_suppresses_for_refresh()
{
  let creds = make_creds_file( "{}" );
  let tmp_path = creds.path().to_str().unwrap();
  let output = stderr_str( &run_cli( &[
    "refresh", "--creds", tmp_path, "--no-compact-window", "--dry-run",
  ] ) );
  assert!(
    !output.contains( "CLAUDE_CODE_AUTO_COMPACT_WINDOW" ),
    "--no-compact-window must suppress injection for refresh. Got:\n{output}"
  );
}
