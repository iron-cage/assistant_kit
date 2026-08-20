//! Unix-only integration tests.
#![ cfg( unix ) ]
#![ allow( clippy::doc_markdown ) ] // test doc comments use code identifiers in prose
//! Journal Integration Tests — Extended (EC-11..EC-22)
//!
//! Extension of `journal_integration_test.rs` (EC-1–EC-10) covering gate/validation
//! events, error handling, truncation, precedence, and CLI/env duplicate resolution.
//!
//! ## Test Layout
//!
//! - EC-11: Gate blocks → `"type":"gate_wait"` event with `gate_outcome::acquired`
//! - EC-12: Validation retry → `"type":"validation_retry"` event in JSONL
//! - EC-13: Read-only journal dir → subprocess exit preserved; journal errors ignored
//! - EC-14: `--journal-dir <cli>` + `CLR_JOURNAL_DIR=<env>` → file in CLI dir (CLI wins)
//! - EC-15: Stdout > 1 MB at `full` level → field contains `"[truncated at 1MB]"` marker
//! - EC-16: `--dry-run` does NOT create journal directory (BUG-319 regression)
//! - EC-17: `--journal bogus` CLI flag → exit 1 with error message
//! - EC-18: `--journal Full` (wrong case) → exit 1 (case-sensitive)
//! - EC-19: `--journal` as last token (missing value) → exit 1
//! - EC-20: `--journal full --journal meta` (duplicate) → second wins; meta-level JSONL
//! - EC-21: `--journal off --journal-dir <dir>` → no JSONL (off takes precedence)
//! - EC-22: `CLR_JOURNAL=off` + `CLR_JOURNAL_DIR=<dir>` → no JSONL (off via env)

mod cli_binary_test_helpers;
use cli_binary_test_helpers::{ fake_claude_dir, find_jsonl_files, read_journal_content, run_with_journal, spawn_print_claude_for };
use std::process::Command;
use std::os::unix::fs::PermissionsExt;

// ── EC-11: Gate blocks → "type":"gate_wait" emitted ──────────────────────────

/// EC-11: When `--max-sessions 1` and one print-mode `claude` process is already
/// running, the gate blocks for at least one poll cycle and emits a `"gate_wait"`
/// event with `"gate_outcome":"acquired"` once the slot is released.
///
/// Root Cause: gate_wait events only emit when `gate_emitted=true` (i.e. ≥1 full
/// poll cycle elapsed), which requires an actual blocking process to be running.
/// Why Not Caught: no test spawning a real blocking process before invoking clr
/// Fix Applied: Added fake_proc TempDir + symlink /proc/{child_pid}; .env(CLR_PROC_DIR) on clr
///   Command builder — gate now reads synthetic proc dir only (BUG-326)
/// Prevention: assert "type":"gate_wait" + "gate_outcome":"acquired" appear in JSONL
/// Pitfall: `CLR_GATE_POLL_SECS=1` reduces poll interval from 30s to 1s for tests.
/// Fix(BUG-326): `CLR_PROC_DIR` MUST be set on the `clr` Command builder so that
/// `find_claude_processes()` reads only the synthetic proc dir, not real `/proc`.
/// Pitfall: since task 368, the gate counts print-mode processes only (via
/// `classify_mode()`) — the blocker must carry a `--print` marker in its cmdline
/// (`spawn_print_claude_for()`'s `arg0("claude")` + trailing `--print` argument), or
/// the gate excludes it from the count and never blocks. Do NOT use a static
/// `cmdline` file: it persists after child exits, keeping count=1 forever so the
/// gate never releases and `.output()` deadlocks.
#[ cfg( unix ) ]
#[ test ]
#[ doc = "bug_reproducer(BUG-326)" ]
fn ec11_gate_wait_event_emitted_when_gate_blocks()
{
  let jdir   = tempfile::TempDir::new().expect( "jdir" );
  let jdir_s = jdir.path().to_str().expect( "utf-8" ).to_owned();

  // Print-mode blocker (task 368: gate counts print-mode processes only) — holds
  // the gate slot for ~3 seconds; `spawn_print_claude_for()` tags its cmdline with
  // `--print` so `classify_mode()` counts it toward this `-p` invocation's limit.
  let mut child = spawn_print_claude_for( &std::env::var( "PATH" ).unwrap_or_default(), 3 );

  // Brief delay — let the spawned process register in /proc.
  std::thread::sleep( core::time::Duration::from_millis( 300 ) );

  // Fix(BUG-326): synthetic proc dir — gate sees only the blocker, not ambient sessions.
  // Root cause: CLR_PROC_DIR absent → find_claude_processes() reads real /proc →
  //   count=20-24 → max=1 never satisfied → timeout.
  // Pitfall: symlink (not static cmdline) — symlink target vanishes when child exits
  //   (kernel removes /proc/{pid}); ENOENT causes process.rs to skip, count→0, gate releases.
  let fake_proc     = tempfile::TempDir::new().expect( "fake_proc" );
  let fake_proc_str = fake_proc.path().to_str().expect( "fake_proc UTF-8" );
  std::os::unix::fs::symlink(
    format!( "/proc/{}", child.id() ),
    fake_proc.path().join( child.id().to_string() ),
  ).expect( "pid symlink" );

  // Shell-script fake claude for the actual subprocess that clr runs (exits 0).
  let ( _script_dir, script_path ) = fake_claude_dir( "exit 0" );
  let gate_dir = tempfile::TempDir::new().expect( "gate dir" );

  let bin = env!( "CARGO_BIN_EXE_clr" );
  let out = Command::new( bin )
    .args( [
      "-p",
      "--max-sessions",   "1",
      "--journal",        "full",
      "--journal-dir",    &jdir_s,
      "x",
    ] )
    .env( "PATH", &script_path )
    .env( "CLR_PROC_DIR", fake_proc_str )    // BUG-326: isolate gate's proc scan
    .env( "CLR_GATE_DIR", gate_dir.path() )
    .env( "CLR_GATE_POLL_SECS", "1" )
    .env_remove( "CLR_JOURNAL" )
    .env_remove( "CLR_JOURNAL_DIR" )
    .env_remove( "CLR_TIMEOUT" )
    .env_remove( "_CLR_DEFAULT_TIMEOUT" )
    .output()
    .expect( "invoke clr" );

  // Clean up background process.
  let _ = child.wait();

  assert!(
    out.status.success(),
    "exit must be 0 after gate releases. stderr: {}",
    String::from_utf8_lossy( &out.stderr )
  );

  let content = read_journal_content( jdir.path() );
  assert!(
    content.contains( r#""type":"gate_wait""# ),
    "JSONL must contain gate_wait event. Got:\n{content}"
  );
  assert!(
    content.contains( r#""gate_outcome":"acquired""# ),
    "gate_wait event must record outcome=acquired. Got:\n{content}"
  );
}

// ── EC-12: Validation retry fires → "type":"validation_retry" in JSONL ───────

/// EC-12: When `--expect <pattern> --expect-strategy retry --retry-on-validation 1`
/// is set and the first subprocess invocation does not match the pattern, a
/// `"validation_retry"` event is emitted before the re-attempt.
///
/// Root Cause: ValidationRetry events were not emitted before this integration was added.
/// Why Not Caught: no test combining --expect retry with journal recording
/// Fix Applied: emit_validation_retry() called inside the retry loop in apply_expect_validation()
/// Prevention: assert "type":"validation_retry" appears in JSONL when retry fires
/// Pitfall: use --validation-delay 0 to prevent 30s wait between retry attempts
#[ cfg( unix ) ]
#[ test ]
fn ec12_validation_retry_event_emitted_on_expect_mismatch()
{
  let jdir   = tempfile::TempDir::new().expect( "jdir" );
  let jdir_s = jdir.path().to_str().expect( "utf-8" ).to_owned();

  // Counter file: first call outputs "WRONG"; second call outputs "RIGHT".
  let count_dir  = tempfile::TempDir::new().expect( "count dir" );
  let count_file = count_dir.path().join( "count" );
  let count_path = count_file.to_str().expect( "count path utf-8" );

  let script = format!(
    "#!/bin/sh\nif [ -f \"{count_path}\" ]; then printf RIGHT; exit 0; fi\ntouch \"{count_path}\"\nprintf WRONG\nexit 0\n"
  );
  let tmp  = tempfile::TempDir::new().expect( "tmpdir" );
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
      "--max-sessions",        "0",
      "--expect",              "right",
      "--expect-strategy",     "retry",
      "--retry-on-validation", "1",
      "--validation-delay",    "0",
      "--journal",             "full",
      "--journal-dir",         &jdir_s,
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
    "exit must be 0 after validation retry succeeds. stderr: {}",
    String::from_utf8_lossy( &out.stderr )
  );

  let content = read_journal_content( jdir.path() );
  assert!(
    content.contains( r#""type":"validation_retry""# ),
    "JSONL must contain validation_retry event. Got:\n{content}"
  );
}

// ── EC-13: Read-only journal dir → runner exit preserved ─────────────────────

/// EC-13: A read-only journal directory does not abort the runner.  Journal writes
/// are best-effort — permission-denied errors from `append()` are silently ignored.
/// The subprocess exit code is preserved as the runner exit code.
///
/// Root Cause: `emit()` discards `append()` errors via `let _ = w.append(&ev)`.
/// Why Not Caught: no test verifying permission-denied journal write scenario
/// Fix Applied: N/A — existing best-effort contract; test documents the invariant
/// Prevention: assert exit 0 even when journal dir is unwritable; assert no JSONL
/// Pitfall: use mode 555 (not 444) on directory — 555 allows read_dir() listing
/// but prevents file creation; 444 disables traversal, making listing impossible.
#[ cfg( unix ) ]
#[ test ]
fn ec13_readonly_journal_dir_does_not_abort_runner()
{
  let jdir   = tempfile::TempDir::new().expect( "jdir" );
  let jdir_s = jdir.path().to_str().expect( "utf-8" ).to_owned();

  // Make directory read+execute only — new files cannot be created inside it.
  std::fs::set_permissions(
    jdir.path(),
    std::fs::Permissions::from_mode( 0o555 ),
  ).expect( "chmod 555" );

  let ( out, _fake ) = run_with_journal(
    &[ "--journal", "full", "--journal-dir", &jdir_s ],
    &[],
    "exit 0",
  );

  // Restore permissions so TempDir cleanup can delete the directory.
  std::fs::set_permissions(
    jdir.path(),
    std::fs::Permissions::from_mode( 0o755 ),
  ).expect( "chmod 755 restore" );

  assert!(
    out.status.success(),
    "read-only journal dir must not abort the runner. stderr: {}",
    String::from_utf8_lossy( &out.stderr )
  );

  let files = find_jsonl_files( jdir.path() );
  assert!(
    files.is_empty(),
    "no JSONL file should be writable in a read-only dir. found: {files:?}"
  );
}

// ── EC-14: --journal-dir (CLI) wins over CLR_JOURNAL_DIR (env) ───────────────

/// EC-14: When `--journal-dir <cli>` and `CLR_JOURNAL_DIR=<env>` are both present,
/// the CLI flag wins and the JSONL file appears in the CLI-specified directory.
///
/// This validates the 3-tier precedence rule documented in `072_journal.md`:
/// `--journal-dir` > `CLR_JOURNAL_DIR` > `~/.clr/journal/`.
#[ test ]
fn ec14_journal_dir_cli_wins_over_env()
{
  let cli_dir = tempfile::TempDir::new().expect( "cli_dir" );
  let env_dir = tempfile::TempDir::new().expect( "env_dir" );
  let cli_dir_s = cli_dir.path().to_str().expect( "utf-8" ).to_owned();
  let env_dir_s = env_dir.path().to_str().expect( "utf-8" ).to_owned();

  let ( out, _fake ) = run_with_journal(
    &[ "--journal-dir", &cli_dir_s ],
    &[ ( "CLR_JOURNAL_DIR", &env_dir_s ) ],
    "printf cli_wins_test",
  );

  assert!(
    out.status.success(),
    "exit must be 0. stderr: {}",
    String::from_utf8_lossy( &out.stderr )
  );

  // CLI dir must contain the JSONL file.
  let cli_files = find_jsonl_files( cli_dir.path() );
  assert!(
    !cli_files.is_empty(),
    "--journal-dir (CLI) must create JSONL in CLI dir when CLR_JOURNAL_DIR (env) also set. \
     cli_dir: {cli_dir_s}"
  );

  // Env dir must remain empty — CLI wins.
  let env_files = find_jsonl_files( env_dir.path() );
  assert!(
    env_files.is_empty(),
    "--journal-dir (CLI) must override CLR_JOURNAL_DIR (env). \
     Found unexpected files in env_dir: {env_files:?}"
  );
}

// ── EC-15: Stdout > 1 MB → truncation marker in journal event ────────────────

/// EC-15: When the fake claude emits more than 1 MB on stdout, the journal event
/// at `full` level contains a `stdout` field that ends with `"[truncated at 1MB]"`.
///
/// Root Cause: `emit_execution()` used `.chars().take(TRUNCATE)` without appending
///   the truncation marker — callers could not detect whether output was truncated.
/// Why Not Caught: no test verifying >1 MB stdout journal behaviour.
/// Fix Applied: added marker append in `execution.rs:emit_execution()`:
///   `if stdout.chars().count() > TRUNCATE { s.push_str("\n[truncated at 1MB]"); }`
/// Prevention: assert `content.contains("[truncated at 1MB]")` on oversized output.
/// Pitfall: compare via `.chars().count()` (Unicode codepoints) not `.len()` (bytes)
///   so that multibyte sequences are counted correctly at the truncation boundary.
#[ test ]
fn ec15_stdout_over_1mb_has_truncation_marker()
{
  let jdir   = tempfile::TempDir::new().expect( "jdir" );
  let jdir_s = jdir.path().to_str().expect( "utf-8" ).to_owned();

  // Emit 1_100_000 'A' bytes on stdout — exceeds the 1 MB (1_048_576) threshold.
  // `head -c` reads null bytes from /dev/zero; `tr` converts them to 'A'.
  let ( out, _fake ) = run_with_journal(
    &[ "--journal", "full", "--journal-dir", &jdir_s ],
    &[],
    "head -c 1100000 < /dev/zero | tr '\\0' 'A'",
  );

  assert!(
    out.status.success(),
    "exit must be 0. stderr: {}",
    String::from_utf8_lossy( &out.stderr )
  );

  let content = read_journal_content( jdir.path() );
  assert!(
    content.contains( r#""stdout""# ),
    "full level must include stdout field. Got:\n{content}"
  );
  assert!(
    content.contains( "[truncated at 1MB]" ),
    "stdout exceeding 1 MB must end with '[truncated at 1MB]' marker. \
     Content snippet:\n{}",
    &content[ ..content.len().min( 300 ) ],
  );
}

// ── EC-16: --dry-run does NOT create journal directory (BUG-319) ──────────────

/// EC-16: `--dry-run` must not create the journal directory as a side effect.
/// Before BUG-319 fix, `resolve_journal_writer()` was called before the dry-run
/// check, creating `~/.clr/journal/` (or `--journal-dir`) even though no events
/// are emitted.
///
/// ## Root Cause
/// `resolve_journal_writer()` calls `create_dir_all()` unconditionally; it was
/// placed before the dry-run exit in `dispatch_run()`.
/// ## Why Not Caught
/// No test verified that dry-run avoids filesystem side effects for journaling.
/// ## Fix Applied
/// Moved `resolve_journal_writer()` after the `if cli.dry_run { exit(0) }` block.
/// ## Prevention
/// This test asserts the custom `--journal-dir` path does NOT exist after dry-run.
/// ## Pitfall
/// Uses a unique topic directory that does not exist before the test; existence check
/// after dry-run detects the side effect.
#[ test ]
fn ec16_dry_run_does_not_create_journal_directory()
{
  let parent = tempfile::TempDir::new().expect( "parent tmpdir" );
  let jdir   = parent.path().join( "must_not_exist" );
  let jdir_s = jdir.to_str().expect( "utf-8" );

  let bin = env!( "CARGO_BIN_EXE_clr" );
  let out = std::process::Command::new( bin )
    .args( [ "--dry-run", "--journal-dir", jdir_s, "test" ] )
    .env_remove( "CLR_JOURNAL" )
    .env_remove( "CLR_JOURNAL_DIR" )
    .output()
    .expect( "invoke clr dry-run" );

  assert!(
    out.status.success(),
    "dry-run must exit 0. stderr: {}",
    String::from_utf8_lossy( &out.stderr )
  );
  assert!(
    !jdir.exists(),
    "BUG-319 regression: --dry-run must NOT create journal directory. Found: {jdir_s}"
  );
}

// ── EC-17: --journal bogus CLI flag → exit 1 ─────────────────────────────────

/// EC-17: `--journal bogus` (invalid value via CLI flag) exits 1 with an error
/// message identifying the flag and the bad value.
#[ test ]
fn ec17_journal_bogus_cli_flag_exits_1()
{
  let bin = env!( "CARGO_BIN_EXE_clr" );
  let out = std::process::Command::new( bin )
    .args( [ "--dry-run", "--journal", "bogus", "test" ] )
    .env_remove( "CLR_JOURNAL" )
    .env_remove( "CLR_JOURNAL_DIR" )
    .output()
    .expect( "invoke clr" );

  assert_eq!(
    out.status.code(), Some( 1 ),
    "invalid --journal value must exit 1. Got: {:?}",
    out.status.code()
  );
  let stderr = String::from_utf8_lossy( &out.stderr );
  assert!(
    stderr.contains( "--journal" ),
    "error must mention --journal flag. Got:\n{stderr}"
  );
  assert!(
    stderr.contains( "bogus" ),
    "error must include the bad value 'bogus'. Got:\n{stderr}"
  );
}

// ── EC-18: --journal Full (case-sensitive) → exit 1 ──────────────────────────

/// EC-18: `--journal Full` exits 1 because valid values are lowercase only
/// (`full`, `meta`, `off`).  Case sensitivity prevents silent misclassification.
#[ test ]
fn ec18_journal_case_sensitive_exits_1()
{
  let bin = env!( "CARGO_BIN_EXE_clr" );
  for bad in [ "Full", "FULL", "Meta", "META", "Off", "OFF" ]
  {
    let out = std::process::Command::new( bin )
      .args( [ "--dry-run", "--journal", bad, "test" ] )
      .env_remove( "CLR_JOURNAL" )
      .env_remove( "CLR_JOURNAL_DIR" )
      .output()
      .expect( "invoke clr" );

    assert_eq!(
      out.status.code(), Some( 1 ),
      "--journal {bad} must exit 1 (case-sensitive). Got: {:?}",
      out.status.code()
    );
  }
}

// ── EC-19: --journal as last token (missing value) → exit 1 ──────────────────

/// EC-19: `--journal` as the final argument with no following value exits 1
/// with a "requires a value" error.
#[ test ]
fn ec19_journal_missing_value_exits_1()
{
  let bin = env!( "CARGO_BIN_EXE_clr" );
  let out = std::process::Command::new( bin )
    .args( [ "--dry-run", "--journal" ] )
    .env_remove( "CLR_JOURNAL" )
    .env_remove( "CLR_JOURNAL_DIR" )
    .output()
    .expect( "invoke clr" );

  assert_eq!(
    out.status.code(), Some( 1 ),
    "--journal without value must exit 1. Got: {:?}",
    out.status.code()
  );
  let stderr = String::from_utf8_lossy( &out.stderr );
  assert!(
    stderr.contains( "requires a value" ) || stderr.contains( "--journal" ),
    "error must mention --journal or 'requires a value'. Got:\n{stderr}"
  );
}

// ── EC-20: --journal full --journal meta (last wins) → meta-level JSONL ──────

/// EC-20: When `--journal` is specified twice, the last value wins.  Specifying
/// `--journal full --journal meta` results in meta-level journaling (no stdout
/// field in the JSONL event).
#[ cfg( unix ) ]
#[ test ]
fn ec20_journal_duplicate_last_wins()
{
  let jdir   = tempfile::TempDir::new().expect( "jdir" );
  let jdir_s = jdir.path().to_str().expect( "utf-8" ).to_owned();

  let ( out, _fake ) = run_with_journal(
    &[ "--journal", "full", "--journal", "meta", "--journal-dir", &jdir_s ],
    &[],
    "printf duplicate_test",
  );

  assert!(
    out.status.success(),
    "exit must be 0. stderr: {}",
    String::from_utf8_lossy( &out.stderr )
  );

  let content = read_journal_content( jdir.path() );
  assert!(
    content.contains( r#""type":"execution""# ),
    "JSONL must contain execution event. Got:\n{content}"
  );
  assert!(
    !content.contains( r#""stdout""# ),
    "last --journal meta must win; stdout absent in meta mode. Got:\n{content}"
  );
}

// ── EC-21: --journal off --journal-dir <dir> → no JSONL ──────────────────────

/// EC-21: `--journal off` with `--journal-dir <dir>` creates no JSONL file.
/// The `off` level short-circuits `resolve_journal_writer()` before the directory
/// is even created.
#[ test ]
fn ec21_journal_off_with_journal_dir_no_file()
{
  let parent = tempfile::TempDir::new().expect( "parent" );
  let jdir   = parent.path().join( "should_not_appear" );
  let jdir_s = jdir.to_str().expect( "utf-8" );

  let ( dir, path ) = fake_claude_dir( "exit 0" );
  let bin = env!( "CARGO_BIN_EXE_clr" );
  let out = Command::new( bin )
    .args( [ "-p", "--max-sessions", "0", "--journal", "off", "--journal-dir", jdir_s, "x" ] )
    .env( "PATH", &path )
    .env_remove( "CLR_JOURNAL" )
    .env_remove( "CLR_JOURNAL_DIR" )
    .env_remove( "CLR_TIMEOUT" )
    .env_remove( "_CLR_DEFAULT_TIMEOUT" )
    .output()
    .expect( "invoke clr" );

  drop( dir );

  assert!(
    out.status.success(),
    "exit must be 0. stderr: {}",
    String::from_utf8_lossy( &out.stderr )
  );
  // --journal off should never create the directory at all
  assert!(
    !jdir.exists(),
    "--journal off must not create journal dir. Found: {jdir_s}"
  );
}

// ── EC-22: CLR_JOURNAL=off + CLR_JOURNAL_DIR → no JSONL ─────────────────────

/// EC-22: `CLR_JOURNAL=off` via env var with `CLR_JOURNAL_DIR=<dir>` creates no
/// JSONL file.  Mirrors EC-21 but via env vars instead of CLI flags.
#[ test ]
fn ec22_clr_journal_off_env_with_dir_no_file()
{
  let parent = tempfile::TempDir::new().expect( "parent" );
  let jdir   = parent.path().join( "env_off_should_not_appear" );
  let jdir_s = jdir.to_str().expect( "utf-8" );

  let ( dir, path ) = fake_claude_dir( "exit 0" );
  let bin = env!( "CARGO_BIN_EXE_clr" );
  let out = Command::new( bin )
    .args( [ "-p", "--max-sessions", "0", "x" ] )
    .env( "PATH", &path )
    .env( "CLR_JOURNAL", "off" )
    .env( "CLR_JOURNAL_DIR", jdir_s )
    .env_remove( "CLR_TIMEOUT" )
    .env_remove( "_CLR_DEFAULT_TIMEOUT" )
    .output()
    .expect( "invoke clr" );

  drop( dir );

  assert!(
    out.status.success(),
    "exit must be 0. stderr: {}",
    String::from_utf8_lossy( &out.stderr )
  );
  assert!(
    !jdir.exists(),
    "CLR_JOURNAL=off must not create journal dir. Found: {jdir_s}"
  );
}
