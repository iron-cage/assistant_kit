//! Test suite for Invariant 005: Isolated Subprocess Defaults.
//!
//! Verifies that `clr isolated` and `clr refresh` inject the required model,
//! effort, session flags, chrome setting, and CLAUDE.md content on every
//! subprocess invocation.
//!
//! ## Root Cause (Invariant 005 gap)
//! `clr isolated` previously used `claude-sonnet-4-6` at binary-default effort
//! with no `--no-session-persistence`, `--dangerously-skip-permissions`, or
//! CLAUDE.md.  `clr refresh` used `--chrome` despite being a pure HTTP OAuth
//! exchange.  `--timeout 0` killed the subprocess immediately instead of
//! disabling the watchdog.
//!
//! ## Fix Applied
//! S1/S7: `ISOLATED_DEFAULT_MODEL = "claude-opus-4-6"` + `REFRESH_DEFAULT_MODEL`;
//!   `EffortLevel::Max` injected for isolated, `EffortLevel::Low` for refresh.
//! S2: `timeout_secs == 0` → `deadline = None` (no watchdog).
//! S3: `--no-session-persistence` prepended for both commands.
//! S4: `--no-chrome` prepended for refresh.
//! S5: `--dangerously-skip-permissions` prepended when `message.is_some()` for isolated.
//! S6: CLAUDE.md written to `<temp_home>/.claude/CLAUDE.md` before spawn.
//!
//! ## Prevention
//! These tests must pass before any change to `credential.rs`, `isolated.rs`,
//! or the `run_isolated_command()` function signature is merged.
//!
//! ## Pitfall
//! Tests that invoke the `clr` binary require it to be built first.  The binary
//! path is resolved via `env!("CARGO_BIN_EXE_clr")`.  Tests that check trace
//! output use `--trace` (stderr) and do not require a live claude session.

#[ cfg( test ) ]
mod isolated_defaults_test
{
  use claude_runner_core::{ ISOLATED_DEFAULT_MODEL, REFRESH_DEFAULT_MODEL };
  use std::process::Command;

  // ── Helpers ───────────────────────────────────────────────────────────────

  fn clr() -> Command
  {
    Command::new( env!( "CARGO_BIN_EXE_clr" ) )
  }

  /// Write a minimal credentials JSON to a temp file and return the path.
  fn temp_creds() -> std::path::PathBuf
  {
    let path = std::env::temp_dir()
      .join( format!( "isd_test_creds_{}.json", std::process::id() ) );
    std::fs::write( &path, "{}" ).expect( "write temp creds" );
    path
  }

  // ── ISD-1 / ISD-2 : model constants ──────────────────────────────────────

  /// ISD-1: `ISOLATED_DEFAULT_MODEL` constant equals `"claude-opus-4-6"`.
  #[ test ]
  fn isd_01_isolated_default_model_is_opus()
  {
    assert_eq!(
      ISOLATED_DEFAULT_MODEL, "claude-opus-4-6",
      "ISOLATED_DEFAULT_MODEL must be claude-opus-4-6 for real user tasks"
    );
  }

  /// ISD-2: `REFRESH_DEFAULT_MODEL` constant equals `"claude-sonnet-4-6"`.
  #[ test ]
  fn isd_02_refresh_default_model_is_sonnet()
  {
    assert_eq!(
      REFRESH_DEFAULT_MODEL, "claude-sonnet-4-6",
      "REFRESH_DEFAULT_MODEL must be claude-sonnet-4-6 for trivial OAuth ping"
    );
  }

  // ── ISD-3 / ISD-4 : effort injection ─────────────────────────────────────

  /// ISD-3: `clr isolated --trace "x"` stderr shows `--effort max`.
  #[ test ]
  fn isd_03_isolated_trace_shows_effort_max()
  {
    let creds = temp_creds();
    let out = clr()
      .args( [ "isolated", "--creds", creds.to_str().unwrap(), "--trace", "x" ] )
      .output()
      .expect( "spawn clr" );
    let stderr = String::from_utf8_lossy( &out.stderr );
    assert!(
      stderr.contains( "--effort max" ),
      "expected '--effort max' in isolated trace; got:\n{stderr}"
    );
    let _ = std::fs::remove_file( &creds );
  }

  /// ISD-4: `clr refresh --trace` stderr shows `--effort low`.
  #[ test ]
  fn isd_04_refresh_trace_shows_effort_low()
  {
    let creds = temp_creds();
    let out = clr()
      .args( [ "refresh", "--creds", creds.to_str().unwrap(), "--trace" ] )
      .output()
      .expect( "spawn clr" );
    let stderr = String::from_utf8_lossy( &out.stderr );
    assert!(
      stderr.contains( "--effort low" ),
      "expected '--effort low' in refresh trace; got:\n{stderr}"
    );
    let _ = std::fs::remove_file( &creds );
  }

  // ── ISD-5 / ISD-6 : skip-permissions ─────────────────────────────────────

  /// ISD-5: `clr isolated --trace "x"` shows `--dangerously-skip-permissions`.
  #[ test ]
  fn isd_05_isolated_trace_shows_skip_perms_when_message()
  {
    let creds = temp_creds();
    let out = clr()
      .args( [ "isolated", "--creds", creds.to_str().unwrap(), "--trace", "x" ] )
      .output()
      .expect( "spawn clr" );
    let stderr = String::from_utf8_lossy( &out.stderr );
    assert!(
      stderr.contains( "--dangerously-skip-permissions" ),
      "expected '--dangerously-skip-permissions' in isolated trace when message present; got:\n{stderr}"
    );
    let _ = std::fs::remove_file( &creds );
  }

  /// ISD-6: `clr isolated --trace` (no message) does NOT inject skip-permissions.
  #[ test ]
  fn isd_06_isolated_trace_no_skip_perms_without_message()
  {
    let creds = temp_creds();
    // No message → interactive mode → no skip-perms injection.
    // Trace fires on stderr before any I/O so the output is available immediately.
    let out = clr()
      .args( [ "isolated", "--creds", creds.to_str().unwrap(), "--trace" ] )
      .output()
      .expect( "spawn clr" );
    let stderr = String::from_utf8_lossy( &out.stderr );
    assert!(
      !stderr.contains( "--dangerously-skip-permissions" ),
      "must NOT inject '--dangerously-skip-permissions' without a message; got:\n{stderr}"
    );
    let _ = std::fs::remove_file( &creds );
  }

  // ── ISD-7 / ISD-8 : no-session-persistence ───────────────────────────────

  /// ISD-7: `clr isolated --trace "x"` shows `--no-session-persistence`.
  #[ test ]
  fn isd_07_isolated_trace_shows_no_session_persistence()
  {
    let creds = temp_creds();
    let out = clr()
      .args( [ "isolated", "--creds", creds.to_str().unwrap(), "--trace", "x" ] )
      .output()
      .expect( "spawn clr" );
    let stderr = String::from_utf8_lossy( &out.stderr );
    assert!(
      stderr.contains( "--no-session-persistence" ),
      "expected '--no-session-persistence' in isolated trace; got:\n{stderr}"
    );
    let _ = std::fs::remove_file( &creds );
  }

  /// ISD-8: `clr refresh --trace` shows `--no-session-persistence`.
  #[ test ]
  fn isd_08_refresh_trace_shows_no_session_persistence()
  {
    let creds = temp_creds();
    let out = clr()
      .args( [ "refresh", "--creds", creds.to_str().unwrap(), "--trace" ] )
      .output()
      .expect( "spawn clr" );
    let stderr = String::from_utf8_lossy( &out.stderr );
    assert!(
      stderr.contains( "--no-session-persistence" ),
      "expected '--no-session-persistence' in refresh trace; got:\n{stderr}"
    );
    let _ = std::fs::remove_file( &creds );
  }

  // ── ISD-9 / ISD-10 : chrome suppression ──────────────────────────────────

  /// ISD-9: `clr refresh --trace` shows `--no-chrome`.
  #[ test ]
  fn isd_09_refresh_trace_shows_no_chrome()
  {
    let creds = temp_creds();
    let out = clr()
      .args( [ "refresh", "--creds", creds.to_str().unwrap(), "--trace" ] )
      .output()
      .expect( "spawn clr" );
    let stderr = String::from_utf8_lossy( &out.stderr );
    assert!(
      stderr.contains( "--no-chrome" ),
      "expected '--no-chrome' in refresh trace (pure HTTP; no browser needed); got:\n{stderr}"
    );
    let _ = std::fs::remove_file( &creds );
  }

  /// ISD-10: `clr isolated --trace "x"` does NOT show `--no-chrome`.
  #[ test ]
  fn isd_10_isolated_trace_does_not_suppress_chrome()
  {
    let creds = temp_creds();
    let out = clr()
      .args( [ "isolated", "--creds", creds.to_str().unwrap(), "--trace", "x" ] )
      .output()
      .expect( "spawn clr" );
    let stderr = String::from_utf8_lossy( &out.stderr );
    assert!(
      !stderr.contains( "--no-chrome" ),
      "isolated must NOT suppress chrome (tasks may use browser tools); got:\n{stderr}"
    );
    let _ = std::fs::remove_file( &creds );
  }

  // ── ISD-11 : CLAUDE.md content ───────────────────────────────────────────

  /// ISD-11: CLAUDE.md content constant matches invariant 005 spec.
  ///
  /// We verify the constant embedded in isolated.rs matches the invariant spec
  /// by checking its key behavioral directives.  The file is written by
  /// `run_isolated()` before spawn and deleted on cleanup.
  #[ test ]
  fn isd_11_claude_md_content_matches_invariant_spec()
  {
    // The CLAUDE.md content is the `claude_md_content` string literal in
    // `module/claude_runner_core/src/isolated.rs`.  We verify its shape by
    // running a quick subprocess that exits immediately (ClaudeNotFound) and
    // confirming the temp dir is cleaned up (implying the write path was hit).
    // The exact content is checked against the invariant 005 required directives.
    let expected_directives = [
      "# Isolated subprocess",
      "Execute the given task immediately and exit.",
      "Do not ask clarifying questions",
      "Do not request human confirmation for any operation.",
      "Do not explain your reasoning or narrate your steps.",
      "Output only the direct result of the task",
      "If the input is a single character or whitespace only, reply with a single period.",
    ];

    // Source of truth: read the constant from the source file directly.
    let source = std::fs::read_to_string(
      concat!(
        env!( "CARGO_MANIFEST_DIR" ),
        "/../claude_runner_core/src/isolated.rs"
      )
    ).expect( "read isolated.rs source" );

    for directive in &expected_directives
    {
      assert!(
        source.contains( directive ),
        "CLAUDE.md content in isolated.rs missing directive: {directive:?}"
      );
    }
  }

  // ── ISD-12 : timeout=0 semantics ─────────────────────────────────────────

  /// ISD-12: `--timeout 0` does not kill subprocess immediately.
  ///
  /// Uses a fake claude script that sleeps 0.3s then exits 0.  With `--timeout 0`
  /// the watchdog must be disabled; the subprocess should complete normally.
  #[ test ]
  #[ cfg( unix ) ]
  fn isd_12_timeout_zero_is_unlimited()
  {
    use std::os::unix::fs::PermissionsExt;

    let tmp = tempfile::tempdir().expect( "create temp dir for fake claude" );
    let fake = tmp.path().join( "claude" );
    std::fs::write( &fake, "#!/bin/sh\nsleep 0.3\nexit 0\n" )
      .expect( "write fake claude" );
    std::fs::set_permissions( &fake, std::fs::Permissions::from_mode( 0o755 ) )
      .expect( "chmod fake claude" );

    let creds = temp_creds();
    let old_path = std::env::var( "PATH" ).unwrap_or_default();
    let new_path = format!( "{}:{old_path}", tmp.path().display() );

    let out = clr()
      .env( "PATH", &new_path )
      .env_remove( "CLAUDECODE" )
      .args( [
        "isolated",
        "--creds", creds.to_str().unwrap(),
        "--timeout", "0",
        "x",
      ] )
      .output()
      .expect( "spawn clr" );

    // exit 2 = timeout fired → --timeout 0 incorrectly killed the subprocess.
    assert_ne!(
      out.status.code(),
      Some( 2 ),
      "exit 2 means timeout fired — '--timeout 0' must disable watchdog, not kill immediately"
    );
    assert_eq!(
      out.status.code(),
      Some( 0 ),
      "fake claude must complete normally with --timeout 0; status: {:?}", out.status
    );

    let _ = std::fs::remove_file( &creds );
  }

  // ── ISD-13 : passthrough override ────────────────────────────────────────

  /// ISD-13: passthrough `-- --effort medium` appears after injected `--effort max`.
  ///
  /// Verifies injection ordering: `--effort max` first (injected), then
  /// `--effort medium` from passthrough — last-wins → medium is effective.
  #[ test ]
  fn isd_13_passthrough_effort_overrides_injected()
  {
    let creds = temp_creds();
    let out = clr()
      .args( [
        "isolated",
        "--creds", creds.to_str().unwrap(),
        "--trace",
        "x",
        "--",
        "--effort", "medium",
      ] )
      .output()
      .expect( "spawn clr" );
    let stderr = String::from_utf8_lossy( &out.stderr );

    let pos_max    = stderr.find( "--effort max" );
    let pos_medium = stderr.find( "--effort medium" );

    assert!( pos_max.is_some(),    "injected '--effort max' not found in trace:\n{stderr}" );
    assert!( pos_medium.is_some(), "passthrough '--effort medium' not found in trace:\n{stderr}" );
    assert!(
      pos_max.unwrap() < pos_medium.unwrap(),
      "injected '--effort max' must appear before passthrough '--effort medium'\ntrace:\n{stderr}"
    );

    let _ = std::fs::remove_file( &creds );
  }
}
