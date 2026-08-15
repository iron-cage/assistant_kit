//! Unix-only integration tests.
#![ cfg( unix ) ]
#![ allow( clippy::doc_markdown ) ] // test doc comments use code identifiers in prose
//! Journal Integration Tests (EC-1..EC-10)
//!
//! ## Purpose
//!
//! Verify that `--journal`/`--journal-dir`/`CLR_JOURNAL`/`CLR_JOURNAL_DIR` control
//! journal file creation, level filtering, and event emission at each lifecycle point.
//!
//! EC-11–EC-22 live in `journal_integration_ext_test.rs`.
//!
//! ## Test Layout
//!
//! - EC-1: `--journal off` → no JSONL file written
//! - EC-2: `--journal full` → JSONL with `"type":"execution"` and stdout/stderr fields
//! - EC-3: `--journal meta` → JSONL without stdout/stderr fields
//! - EC-4: `--journal-dir <dir>` only (level defaults to "full") → JSONL in custom dir
//! - EC-5: `CLR_JOURNAL=meta` env → meta-level JSONL
//! - EC-6: `CLR_JOURNAL_DIR=<dir>` env → JSONL written to env-specified dir
//! - EC-7: Retry fires → `"type":"retry"` event in JSONL
//! - EC-8: Timeout fires → `"type":"timeout"` event in JSONL
//! - EC-9: `CLR_JOURNAL=invalid` → exit 1; stderr contains error message
//! - EC-10: No `--journal-dir` + `HOME=<tmpdir>` → JSONL at `~/.clr/journal/`

mod cli_binary_test_helpers;
use cli_binary_test_helpers::{ fake_claude_dir, find_jsonl_files, read_journal_content, run_with_journal };
use std::process::Command;
use std::os::unix::fs::PermissionsExt;

// ── EC-1: --journal off → no JSONL written ────────────────────────────────────

/// EC-1: `--journal off` suppresses journal output entirely; no JSONL file created.
#[ test ]
fn ec1_journal_off_no_file_written()
{
  let jdir = tempfile::TempDir::new().expect( "tmpdir" );
  let jdir_s = jdir.path().to_str().expect( "utf-8" ).to_owned();

  let ( out, _fake ) = run_with_journal(
    &[ "--journal", "off", "--journal-dir", &jdir_s ],
    &[],
    "printf done\nexit 0",
  );

  assert!(
    out.status.success(),
    "exit must be 0. stderr: {}",
    String::from_utf8_lossy( &out.stderr )
  );
  let files = find_jsonl_files( jdir.path() );
  assert!(
    files.is_empty(),
    "--journal off must produce no JSONL files; found: {files:?}"
  );
}

// ── EC-2: --journal full → execution event with stdout/stderr ────────────────

/// EC-2: `--journal full` writes a JSONL file containing an `"execution"` event
/// with `stdout` and `stderr` fields (level "full" includes output).
#[ test ]
fn ec2_journal_full_execution_event_with_output()
{
  let jdir = tempfile::TempDir::new().expect( "tmpdir" );
  let jdir_s = jdir.path().to_str().expect( "utf-8" ).to_owned();

  let ( out, _fake ) = run_with_journal(
    &[ "--journal", "full", "--journal-dir", &jdir_s ],
    &[],
    "printf journal_test_output",
  );

  assert!(
    out.status.success(),
    "exit must be 0. stderr: {}",
    String::from_utf8_lossy( &out.stderr )
  );

  let content = read_journal_content( jdir.path() );
  assert!(
    !content.is_empty(),
    "--journal full must write a JSONL file. journal dir: {jdir_s}"
  );
  assert!(
    content.contains( r#""type":"execution""# ),
    "JSONL must contain execution event. Got:\n{content}"
  );
  assert!(
    content.contains( r#""exit_code":0"# ),
    "execution event must record exit_code 0. Got:\n{content}"
  );
  assert!(
    content.contains( r#""stdout""# ),
    "full level must include stdout field. Got:\n{content}"
  );
}

// ── EC-3: --journal meta → execution event without stdout/stderr ──────────────

/// EC-3: `--journal meta` writes an `"execution"` event but omits `stdout`/`stderr`
/// to keep the journal compact for high-throughput use cases.
#[ test ]
fn ec3_journal_meta_omits_output_fields()
{
  let jdir = tempfile::TempDir::new().expect( "tmpdir" );
  let jdir_s = jdir.path().to_str().expect( "utf-8" ).to_owned();

  let ( out, _fake ) = run_with_journal(
    &[ "--journal", "meta", "--journal-dir", &jdir_s ],
    &[],
    "printf meta_test_output",
  );

  assert!(
    out.status.success(),
    "exit must be 0. stderr: {}",
    String::from_utf8_lossy( &out.stderr )
  );

  let content = read_journal_content( jdir.path() );
  assert!(
    content.contains( r#""type":"execution""# ),
    "meta level must still write execution event. Got:\n{content}"
  );
  assert!(
    !content.contains( r#""stdout""# ),
    "meta level must omit stdout field. Got:\n{content}"
  );
  assert!(
    !content.contains( r#""stderr""# ),
    "meta level must omit stderr field. Got:\n{content}"
  );
}

// ── EC-4: --journal-dir only → default level is "full" ───────────────────────

/// EC-4: `--journal-dir <dir>` without explicit `--journal` defaults to level "full"
/// and writes the journal to the specified directory.
#[ test ]
fn ec4_journal_dir_only_defaults_to_full()
{
  let jdir = tempfile::TempDir::new().expect( "tmpdir" );
  let jdir_s = jdir.path().to_str().expect( "utf-8" ).to_owned();

  let ( out, _fake ) = run_with_journal(
    &[ "--journal-dir", &jdir_s ],
    &[],
    "printf dir_test",
  );

  assert!(
    out.status.success(),
    "exit must be 0. stderr: {}",
    String::from_utf8_lossy( &out.stderr )
  );

  let files = find_jsonl_files( jdir.path() );
  assert!(
    !files.is_empty(),
    "--journal-dir must produce a JSONL file with default level full. dir: {jdir_s}"
  );
  let content = read_journal_content( jdir.path() );
  assert!(
    content.contains( r#""type":"execution""# ),
    "JSONL must contain execution event. Got:\n{content}"
  );
  // Full level: stdout field present when subprocess emitted output
  assert!(
    content.contains( r#""stdout""# ),
    "default full level must include stdout when subprocess has output. Got:\n{content}"
  );
}

// ── EC-5: CLR_JOURNAL=meta env → meta-level JSONL ────────────────────────────

/// EC-5: `CLR_JOURNAL=meta` env var controls journal level; stdout/stderr absent.
#[ test ]
fn ec5_clr_journal_env_meta()
{
  let jdir = tempfile::TempDir::new().expect( "tmpdir" );
  let jdir_s = jdir.path().to_str().expect( "utf-8" ).to_owned();

  let ( out, _fake ) = run_with_journal(
    &[ "--journal-dir", &jdir_s ],
    &[ ( "CLR_JOURNAL", "meta" ) ],
    "printf env_meta_output",
  );

  assert!(
    out.status.success(),
    "exit must be 0. stderr: {}",
    String::from_utf8_lossy( &out.stderr )
  );

  let content = read_journal_content( jdir.path() );
  assert!(
    content.contains( r#""type":"execution""# ),
    "CLR_JOURNAL=meta must still emit execution event. Got:\n{content}"
  );
  assert!(
    !content.contains( r#""stdout""# ),
    "CLR_JOURNAL=meta must omit stdout field. Got:\n{content}"
  );
  assert!(
    !content.contains( r#""stderr""# ),
    "CLR_JOURNAL=meta must omit stderr field. Got:\n{content}"
  );
}

// ── EC-6: CLR_JOURNAL_DIR env → JSONL in env-specified dir ───────────────────

/// EC-6: `CLR_JOURNAL_DIR=<dir>` env var redirects the journal to that directory
/// when no `--journal-dir` CLI flag is present.
#[ test ]
fn ec6_clr_journal_dir_env()
{
  let jdir = tempfile::TempDir::new().expect( "tmpdir" );
  let jdir_s = jdir.path().to_str().expect( "utf-8" ).to_owned();

  let ( out, _fake ) = run_with_journal(
    &[],  // no --journal-dir CLI flag
    &[ ( "CLR_JOURNAL_DIR", &jdir_s ) ],
    "printf dir_env_test",
  );

  assert!(
    out.status.success(),
    "exit must be 0. stderr: {}",
    String::from_utf8_lossy( &out.stderr )
  );

  let files = find_jsonl_files( jdir.path() );
  assert!(
    !files.is_empty(),
    "CLR_JOURNAL_DIR must redirect journal to the specified dir. dir: {jdir_s}"
  );
  let content = read_journal_content( jdir.path() );
  assert!(
    content.contains( r#""type":"execution""# ),
    "JSONL must contain execution event. Got:\n{content}"
  );
}

// ── EC-7: Retry fires → "type":"retry" event in JSONL ────────────────────────

/// EC-7: When `--retry-on-transient 1 --transient-delay 0` is set and the fake claude
/// exits 2 on the first attempt (classified as Transient/RateLimit), a `"retry"` event
/// is emitted before the successful second attempt.
///
/// Root Cause: retry events not emitted before implementing journal integration
/// Why Not Caught: no test asserting retry events in JSONL output
/// Fix Applied: emit_retry() called before each sleep/re-attempt in run_print_mode()
/// Prevention: assert "type":"retry" appears before "type":"execution" in JSONL
/// Pitfall: --transient-delay 0 is required; default 30s delay makes test hang
#[ cfg( unix ) ]
#[ test ]
fn ec7_retry_event_emitted_on_transient_failure()
{
  let jdir = tempfile::TempDir::new().expect( "jdir" );
  let jdir_s = jdir.path().to_str().expect( "utf-8" ).to_owned();

  // Counter file: first invocation exits 2; second exits 0.
  let count_dir = tempfile::TempDir::new().expect( "count dir" );
  let count_file = count_dir.path().join( "count" );
  let count_path = count_file.to_str().expect( "count path utf-8" );

  let script = format!(
    "#!/bin/sh\nif [ -f \"{count_path}\" ]; then exit 0; fi\ntouch \"{count_path}\"\nexit 2\n"
  );
  let tmp = tempfile::TempDir::new().expect( "tmpdir" );
  let fake = tmp.path().join( "claude" );
  std::fs::write( &fake, script.as_bytes() ).expect( "write fake claude" );
  std::fs::set_permissions( &fake, std::fs::Permissions::from_mode( 0o755 ) )
    .expect( "chmod" );
  let old_path = std::env::var( "PATH" ).unwrap_or_default();
  let new_path = format!( "{}:{old_path}", tmp.path().display() );

  let bin = env!( "CARGO_BIN_EXE_clr" );
  let out = Command::new( bin )
    .args( [
      "-p",
      "--retry-on-transient", "1",
      "--transient-delay",    "0",
      "--max-sessions",       "0",
      "--journal",            "full",
      "--journal-dir",        &jdir_s,
      "x",
    ] )
    .env( "PATH", &new_path )
    .env_remove( "CLR_JOURNAL" )
    .env_remove( "CLR_JOURNAL_DIR" )
    .env_remove( "CLR_TIMEOUT" )
    .env_remove( "_CLR_DEFAULT_TIMEOUT" )
    .output()
    .expect( "invoke clr" );

  assert!(
    out.status.success(),
    "exit must be 0 after retry succeeds. stderr: {}",
    String::from_utf8_lossy( &out.stderr )
  );

  let content = read_journal_content( jdir.path() );
  assert!(
    content.contains( r#""type":"retry""# ),
    "JSONL must contain a retry event when transient retry fires. Got:\n{content}"
  );
  assert!(
    content.contains( r#""type":"execution""# ),
    "JSONL must contain final execution event after retry succeeds. Got:\n{content}"
  );
  assert!(
    content.contains( r#""error_class":"Transient""# ),
    "retry event must record Transient error class. Got:\n{content}"
  );
}

// ── EC-8: Timeout fires → "type":"timeout" event in JSONL ────────────────────

/// EC-8: When `_CLR_DEFAULT_TIMEOUT=2` and the subprocess hangs, the watchdog kills
/// it and emits a `"timeout"` event before exiting with code 4.
///
/// Root Cause: timeout events not emitted before implementing journal integration
/// Why Not Caught: no test asserting timeout events in JSONL output
/// Fix Applied: emit_timeout() called in poll_timeout() before exit(4)
/// Prevention: assert "type":"timeout" in JSONL after watchdog fires
/// Pitfall: must use --retry-override 0 to prevent retry loop from firing after timeout
#[ cfg( unix ) ]
#[ test ]
fn ec8_timeout_event_emitted_on_watchdog_fire()
{
  let jdir = tempfile::TempDir::new().expect( "jdir" );
  let jdir_s = jdir.path().to_str().expect( "utf-8" ).to_owned();

  let tmp  = tempfile::TempDir::new().expect( "tmpdir" );
  let fake = tmp.path().join( "claude" );

  // Fake claude sleeps indefinitely — killed by 2s watchdog
  std::fs::write( &fake, b"#!/bin/sh\nsleep 300\n" ).expect( "write fake claude" );
  std::fs::set_permissions( &fake, std::fs::Permissions::from_mode( 0o755 ) )
    .expect( "chmod" );

  let old_path = std::env::var( "PATH" ).unwrap_or_default();
  let new_path = format!( "{}:{old_path}", tmp.path().display() );

  let bin = env!( "CARGO_BIN_EXE_clr" );
  let out = Command::new( bin )
    .args( [
      "-p",
      "--retry-override", "0",   // no retry — one attempt only
      "--max-sessions",   "0",
      "--journal",        "full",
      "--journal-dir",    &jdir_s,
      "x",
    ] )
    .env( "PATH", &new_path )
    .env( "_CLR_DEFAULT_TIMEOUT", "2" )
    .env_remove( "CLR_JOURNAL" )
    .env_remove( "CLR_JOURNAL_DIR" )
    .env_remove( "CLR_TIMEOUT" )
    .output()
    .expect( "invoke clr" );

  assert_eq!(
    out.status.code(),
    Some( 4 ),
    "exit must be 4 when watchdog fires. Got: {:?}",
    out.status.code()
  );

  let content = read_journal_content( jdir.path() );
  assert!(
    content.contains( r#""type":"timeout""# ),
    "JSONL must contain a timeout event when watchdog kills the subprocess. Got:\n{content}"
  );
  assert!(
    content.contains( r#""exit_code":4"# ),
    "timeout event must record exit_code 4. Got:\n{content}"
  );
}

// ── EC-9: CLR_JOURNAL=invalid → exit 1 with error ────────────────────────────

/// EC-9: `CLR_JOURNAL` with an invalid level value exits 1 and prints an error
/// message identifying the env var and the bad value.
#[ test ]
fn ec9_clr_journal_invalid_value_exits_1()
{
  let bin = env!( "CARGO_BIN_EXE_clr" );
  let out = Command::new( bin )
    .args( [ "--dry-run", "x" ] )
    .env( "CLR_JOURNAL", "bogus" )
    .env_remove( "CLR_JOURNAL_DIR" )
    .output()
    .expect( "invoke clr" );

  assert_eq!(
    out.status.code(),
    Some( 1 ),
    "invalid CLR_JOURNAL must exit 1. Got: {:?}",
    out.status.code()
  );
  let stderr = String::from_utf8_lossy( &out.stderr );
  assert!(
    stderr.contains( "CLR_JOURNAL" ),
    "error must mention CLR_JOURNAL. Got:\n{stderr}"
  );
  assert!(
    stderr.to_lowercase().contains( "invalid" ),
    "error must describe the value as invalid. Got:\n{stderr}"
  );
}

// ── EC-10: Default dir = ~/.clr/journal/ when no --journal-dir ───────────────

/// EC-10: Without `--journal-dir` or `CLR_JOURNAL_DIR`, the journal is written to
/// `$HOME/.clr/journal/`.  Verified by setting `HOME` to a temp dir and confirming
/// the JSONL appears under `<tmpdir>/.clr/journal/`.
///
/// Pitfall: must clear `CLR_JOURNAL_DIR` from env — ambient value would override HOME fallback.
#[ cfg( unix ) ]
#[ test ]
fn ec10_default_journal_dir_is_home_clr_journal()
{
  let fake_home = tempfile::TempDir::new().expect( "fake home" );
  let home_s    = fake_home.path().to_str().expect( "utf-8" ).to_owned();

  let ( out, _fake ) = {
    let ( dir, path ) = fake_claude_dir( "printf home_test" );
    let bin = env!( "CARGO_BIN_EXE_clr" );
    let out = Command::new( bin )
      .args( [ "-p", "--max-sessions", "0", "x" ] )
      .env( "PATH", &path )
      .env( "HOME", &home_s )
      .env_remove( "CLR_JOURNAL" )
      .env_remove( "CLR_JOURNAL_DIR" )
      .env_remove( "CLR_TIMEOUT" )
      .env_remove( "_CLR_DEFAULT_TIMEOUT" )
      .output()
      .expect( "invoke clr" );
    ( out, dir )
  };

  assert!(
    out.status.success(),
    "exit must be 0. stderr: {}",
    String::from_utf8_lossy( &out.stderr )
  );

  let expected_dir = fake_home.path().join( ".clr" ).join( "journal" );
  let files = find_jsonl_files( &expected_dir );
  assert!(
    !files.is_empty(),
    "default journal must appear at ~/.clr/journal/ (HOME={home_s}). \
     Expected dir: {expected_dir:?}. Files found: {files:?}"
  );
  let content = read_journal_content( &expected_dir );
  assert!(
    content.contains( r#""type":"execution""# ),
    "default-dir journal must contain execution event. Got:\n{content}"
  );
}

