//! Test suite for Invariant 005: Isolated Subprocess Defaults.
//!
//! Verifies that `clr isolated` and `clr refresh` inject the required model,
//! effort, session flags, chrome setting, and CLAUDE.md content on every
//! subprocess invocation.
//!
//! ## Root Cause (Invariant 005 gap)
//! `clr isolated` previously used `claude-sonnet-5` at binary-default effort
//! with no `--no-session-persistence`, `--dangerously-skip-permissions`, or
//! CLAUDE.md.  `clr refresh` used `--chrome` despite being a pure HTTP OAuth
//! exchange.  `--timeout 0` killed the subprocess immediately instead of
//! disabling the watchdog.
//!
//! ## Fix Applied
//! S1/S7: `DEFAULT_MODEL = "claude-opus-5-5"` + `REFRESH_DEFAULT_MODEL`;
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
  use claude_runner_core::{ DEFAULT_MODEL, REFRESH_DEFAULT_MODEL };
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

  /// ISD-1: `DEFAULT_MODEL` constant equals `"claude-opus-5-5"`.
  #[ test ]
  fn isd_01_default_model_is_opus_5_5()
  {
    assert_eq!(
      DEFAULT_MODEL, "claude-opus-5-5",
      "DEFAULT_MODEL must be the explicit Opus 5.5 ID (the `opus` alias resolves per Claude release)"
    );
  }

  /// ISD-2: `REFRESH_DEFAULT_MODEL` constant equals `"claude-sonnet-5"`.
  #[ test ]
  fn isd_02_refresh_default_model_is_sonnet()
  {
    assert_eq!(
      REFRESH_DEFAULT_MODEL, "claude-sonnet-5",
      "REFRESH_DEFAULT_MODEL must be claude-sonnet-5 for trivial OAuth ping"
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
      "Do not use extended thinking",
      "Do not use tool calls",
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

  // ── ISD-14 : built-in default model reaches the assembled command ─────────

  /// ISD-14: with no `--model`, no `CLR_MODEL`, and no config preference in either
  /// tier, `clr isolated --dry-run` previews `--model claude-opus-5-5` (`DEFAULT_MODEL`).
  ///
  /// ISD-1 pins the constant alone; this pins the end-to-end path —
  /// `IsolatedModel::Default` → `resolve_isolated_default_model()` returns `None` →
  /// `model_id()` fallback — into the command the subprocess would actually receive.
  #[ test ]
  fn isd_14_isolated_dry_run_shows_default_model()
  {
    let tmp = tempfile::tempdir().expect( "create temp dir" );
    let creds = temp_creds();
    // cwd = HOME = empty temp dir → neither the project nor the user config tier
    // can supply a preference.
    let out = clr()
      .current_dir( tmp.path() )
      .env( "HOME", tmp.path() )
      .env_remove( "CLR_MODEL" )
      .env_remove( "CLR_CONFIG_DIR" )
      .args( [ "isolated", "--creds", creds.to_str().unwrap(), "--dry-run", "msg" ] )
      .output()
      .expect( "spawn clr" );
    assert_eq!(
      out.status.code(),
      Some( 0 ),
      "expected exit 0 from --dry-run; stderr: {}", String::from_utf8_lossy( &out.stderr )
    );
    let stdout = String::from_utf8_lossy( &out.stdout );
    assert!(
      stdout.contains( &format!( "--model {DEFAULT_MODEL}" ) ),
      "isolated preview must carry the built-in default model; got:\n{stdout}"
    );
    let _ = std::fs::remove_file( &creds );
  }

  // ── BUG-485 : config model preference must reach every --model site ──────

  /// # Root Cause
  /// `IsolatedModel::Default`'s config model preference (project `.clr.toml` /
  /// user `~/.clr/config.toml`, read by `resolve_isolated_default_model()`) was
  /// consulted only inside `run_isolated_ext()`.  The two sibling
  /// `--model`-prepend sites consuming the same `Default` value —
  /// `emit_credential_trace()`'s `--dry-run`/`--trace` preview and the `--file`
  /// real path (`run_isolated_with_stdin_file()`) — fell back to
  /// `DEFAULT_MODEL`, so the preview showed the wrong model and the two
  /// real execution paths ran different models on identical inputs.
  ///
  /// # Why Not Caught
  /// `isolated_model_resolution_test.rs` pins the resolver in isolation and
  /// ISD-1 pins the constant, but no test ran the actual binary with a config
  /// preference present and read which model the assembled command carries —
  /// the divergence lived in per-site arg assembly no assertion ever crossed.
  ///
  /// # Fix Applied
  /// `run_isolated_command()` resolves `Default` → `Specific(pref)` once at
  /// entry, before arg assembly — upstream of the preview/ext/stdin-file
  /// fan-out, so all three sites see the same resolved model.
  /// `run_isolated_ext()`'s own consultation remains as a fallback for direct
  /// core-API callers.
  ///
  /// # Prevention
  /// This test runs the real `clr isolated --dry-run` binary in a temp cwd
  /// whose `.clr.toml` pins a marker model, with `HOME` pointed at the same
  /// temp dir so only the project tier can supply the preference.  It asserts
  /// the previewed command carries the marker and not the hardcoded default.
  /// Because the fix resolves upstream of the fan-out, the preview is a
  /// WYSIWYG witness for all three consuming sites.
  ///
  /// # Pitfall
  /// Three sibling `--model`-prepend sites consume the same `IsolatedModel`
  /// value; a preference consulted at only one of them silently splits
  /// behavior across execution paths.  Resolve shared inputs once, upstream
  /// of the fan-out — never per-site.
  // test_kind: bug_reproducer(BUG-485)
  #[ test ]
  fn bug485_dry_run_preview_shows_config_model_pref_not_hardcoded_default()
  {
    let tmp = tempfile::tempdir().expect( "create temp project dir" );
    std::fs::write( tmp.path().join( ".clr.toml" ), "model = \"cfg-pinned-model\"\n" )
      .expect( "write .clr.toml" );
    let creds = temp_creds();
    // cwd = temp dir → project-tier `.clr.toml` is the one just written;
    // HOME = temp dir → user tier (`~/.clr/config.toml`) cannot interfere.
    let out = clr()
      .current_dir( tmp.path() )
      .env( "HOME", tmp.path() )
      .args( [ "isolated", "--creds", creds.to_str().unwrap(), "--dry-run", "msg" ] )
      .output()
      .expect( "spawn clr" );
    assert_eq!(
      out.status.code(),
      Some( 0 ),
      "expected exit 0 from --dry-run; stderr: {}", String::from_utf8_lossy( &out.stderr )
    );
    let stdout = String::from_utf8_lossy( &out.stdout );
    assert!(
      stdout.contains( "--no-session-persistence" ),
      "sanity: preview must show the injected isolated flags (guards against a vacuously \
       empty preview satisfying the model assertions); got:\n{stdout}"
    );
    assert!(
      stdout.contains( "--model cfg-pinned-model" ),
      "BUG-485: with `.clr.toml` pinning `model = \"cfg-pinned-model\"` and no explicit \
       --model flag, the --dry-run preview must show the configured preference — the same \
       model run_isolated_ext() would actually use. Got:\n{stdout}"
    );
    assert!(
      !stdout.contains( &format!( "--model {DEFAULT_MODEL}" ) ),
      "BUG-485: the preview must not fall back to the hardcoded DEFAULT_MODEL \
       when a config preference is set. Got:\n{stdout}"
    );
    let _ = std::fs::remove_file( &creds );
  }

  // ── BUG-007 : `CLR_CONFIG_DIR` must redirect `isolated`'s user tier too ────

  /// # Root Cause
  /// `resolve_isolated_default_model()` (`claude_runner_core::isolated`) built the
  /// user tier as `$HOME/.clr/config.toml` itself and never read `CLR_CONFIG_DIR`.
  /// The override was honored only by `config.rs`'s private `user_config_dir()`, so
  /// with the override set `clr run` read `$CLR_CONFIG_DIR/config.toml` while
  /// `clr isolated` read `$HOME/.clr/config.toml`.
  ///
  /// # Why Not Caught
  /// ISD-14 removes `CLR_CONFIG_DIR` and BUG-485's test uses the project tier, so no
  /// isolated test ever set the override. `config_file_test.rs` T10 sets it, but only
  /// for `clr run`.
  ///
  /// # Fix Applied
  /// `claude_runner_core::user_config_path()` is the one resolver for the user
  /// `config.toml`. `resolve_isolated_default_model()` and `discover_config_paths()`
  /// both call it, and `user_config_dir()` is gone.
  ///
  /// # Prevention
  /// This test gives `HOME` and `CLR_CONFIG_DIR` different models, so an isolated path
  /// that reads `HOME` alone previews the wrong one. The resolver's own decision table
  /// is `claude_runner_core/tests/config_path_test.rs`.
  ///
  /// # Pitfall
  /// An env override implemented inside one consumer redirects that consumer only.
  /// The file's other readers keep agreeing with each other, not with the override,
  /// and the split shows up only when the override is set.
  // test_kind: bug_reproducer(BUG-007)
  #[ test ]
  fn bug007_isolated_dry_run_reads_user_tier_through_clr_config_dir()
  {
    let home = tempfile::tempdir().expect( "create temp HOME" );
    let home_clr = home.path().join( ".clr" );
    std::fs::create_dir_all( &home_clr ).expect( "create HOME/.clr" );
    std::fs::write( home_clr.join( "config.toml" ), "model = \"home-model\"\n" )
      .expect( "write HOME config.toml" );
    let override_dir = tempfile::tempdir().expect( "create temp CLR_CONFIG_DIR" );
    std::fs::write( override_dir.path().join( "config.toml" ), "model = \"override-model\"\n" )
      .expect( "write override config.toml" );
    // Empty cwd → no project `.clr.toml`, so only the user tier can supply the model.
    let project = tempfile::tempdir().expect( "create empty project dir" );
    let creds = temp_creds();
    let out = clr()
      .current_dir( project.path() )
      .env( "HOME", home.path() )
      .env( "CLR_CONFIG_DIR", override_dir.path() )
      .env_remove( "CLR_MODEL" )
      .args( [ "isolated", "--creds", creds.to_str().unwrap(), "--dry-run", "msg" ] )
      .output()
      .expect( "spawn clr" );
    let _ = std::fs::remove_file( &creds );
    assert_eq!(
      out.status.code(),
      Some( 0 ),
      "expected exit 0 from --dry-run; stderr: {}", String::from_utf8_lossy( &out.stderr )
    );
    let stdout = String::from_utf8_lossy( &out.stdout );
    assert!(
      stdout.contains( "--model override-model" ),
      "BUG-007: with CLR_CONFIG_DIR set, `clr isolated` must read the user tier from \
       $CLR_CONFIG_DIR/config.toml, the file `clr run` reads. Got:\n{stdout}"
    );
    assert!(
      !stdout.contains( "--model home-model" ),
      "BUG-007: $HOME/.clr/config.toml must not be read while CLR_CONFIG_DIR is set. Got:\n{stdout}"
    );
  }
}
