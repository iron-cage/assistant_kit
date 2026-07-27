//! Tests for `stdin_file` and `unset_claudecode` fields on `ClaudeCommand`.
//!
//! Covers feature specs:
//! - [`005_stdin_file.md`](docs/feature/005_stdin_file.md) — FT-1 through FT-6
//! - [`006_unset_claudecode.md`](docs/feature/006_unset_claudecode.md) — FT-1 through FT-5

use claude_runner_core::ClaudeCommand;

// ── 005: stdin_file ─────────────────────────────────────────────────────────

// T01 / FT-1: with_stdin_file(path) → path appears in dry-run describe output
#[ test ]
fn t01_with_stdin_file_sets_field()
{
  let cmd = ClaudeCommand::new()
    .with_stdin_file( std::path::PathBuf::from( "/tmp/t01_stdin.txt" ) )
    .with_dry_run( true );
  let output = cmd.execute().expect( "dry-run must succeed" );
  assert!(
    output.stdout.contains( "< /tmp/t01_stdin.txt" ),
    "describe output must contain '< /tmp/t01_stdin.txt'. Got: {}",
    output.stdout
  );
}

// T02 / FT-2: No stdin_file → describe output contains no stdin reference
#[ test ]
fn t02_stdin_file_absent_by_default()
{
  let cmd = ClaudeCommand::new()
    .with_dry_run( true );
  let output = cmd.execute().expect( "dry-run must succeed" );
  assert!(
    !output.stdout.contains( "< " ),
    "describe output must NOT contain '< ' when stdin_file is None. Got: {}",
    output.stdout
  );
}

// T03 / FT-3: Nonexistent file path → execute() returns Err with path in message
#[ test ]
fn t03_stdin_file_err_on_nonexistent()
{
  let cmd = ClaudeCommand::new()
    .with_stdin_file( std::path::PathBuf::from( "/tmp/nonexistent_zzzz_t03.txt" ) );
  let result = cmd.execute();
  assert!( result.is_err(), "execute() must return Err for nonexistent file" );
  let err_msg = result.unwrap_err().to_string();
  assert!(
    err_msg.contains( "/tmp/nonexistent_zzzz_t03.txt" ),
    "error must contain file path. Got: {err_msg}"
  );
}

// T04 / FT-4: dry_run + nonexistent path → execute() returns Ok (file not opened)
#[ test ]
fn t04_stdin_file_dry_run_skips_open()
{
  let cmd = ClaudeCommand::new()
    .with_stdin_file( std::path::PathBuf::from( "/tmp/nonexistent_zzzz_t04.txt" ) )
    .with_dry_run( true );
  let result = cmd.execute();
  assert!( result.is_ok(), "dry-run must return Ok even with nonexistent file" );
}

// T05 / FT-5: Last-write wins on repeated with_stdin_file
#[ test ]
fn t05_stdin_file_last_call_wins()
{
  let cmd = ClaudeCommand::new()
    .with_stdin_file( std::path::PathBuf::from( "/tmp/path_a.txt" ) )
    .with_stdin_file( std::path::PathBuf::from( "/tmp/path_b.txt" ) )
    .with_dry_run( true );
  let output = cmd.execute().expect( "dry-run must succeed" );
  assert!(
    output.stdout.contains( "/tmp/path_b.txt" ),
    "describe must show path_b. Got: {}",
    output.stdout
  );
  assert!(
    !output.stdout.contains( "/tmp/path_a.txt" ),
    "describe must NOT show path_a. Got: {}",
    output.stdout
  );
}

// ── 006: unset_claudecode ───────────────────────────────────────────────────

// T06 / FT-1 (006): Default new() → CLAUDECODE marked for removal
#[ test ]
fn t06_unset_claudecode_default_true()
{
  let cmd = ClaudeCommand::new()
    .build_command_for_test();
  let has_removal = cmd.get_envs().any( | ( key, val ) |
    key == "CLAUDECODE" && val.is_none()
  );
  assert!(
    has_removal,
    "default ClaudeCommand must mark CLAUDECODE for removal via env_remove"
  );
}

// T07 / FT-2 (006): with_unset_claudecode(false) → CLAUDECODE NOT removed
#[ test ]
fn t07_with_unset_claudecode_false_keeps_env()
{
  let cmd = ClaudeCommand::new()
    .with_unset_claudecode( false )
    .build_command_for_test();
  let has_removal = cmd.get_envs().any( | ( key, val ) |
    key == "CLAUDECODE" && val.is_none()
  );
  assert!(
    !has_removal,
    "with_unset_claudecode(false) must NOT mark CLAUDECODE for removal"
  );
}

// T08 / FT-3 (006): Explicit with_unset_claudecode(true) → same as default
#[ test ]
fn t08_env_remove_called_when_default()
{
  let cmd = ClaudeCommand::new()
    .with_unset_claudecode( true )
    .build_command_for_test();
  let has_removal = cmd.get_envs().any( | ( key, val ) |
    key == "CLAUDECODE" && val.is_none()
  );
  assert!(
    has_removal,
    "explicit with_unset_claudecode(true) must mark CLAUDECODE for removal"
  );
}

// T09 / FT-4 (006): env_remove wired in build_command — visible via build_command_for_test
//
// Distinct from T06 (FT-1): T06 asserts the behavioral outcome (CLAUDECODE removed);
// T09 asserts the wiring location — calling build_command_for_test() directly (not
// execute()) proves env_remove is in build_command(), not deferred to execute().
#[ test ]
fn t09_env_remove_wired_in_build_command()
{
  // Call build_command_for_test() directly — NOT execute()
  let cmd = ClaudeCommand::new()
    .build_command_for_test();
  let has_removal = cmd.get_envs().any( | ( key, val ) |
    key == "CLAUDECODE" && val.is_none()
  );
  assert!(
    has_removal,
    "build_command_for_test() must already show CLAUDECODE removal — \
     proves env_remove is wired in build_command(), not deferred to execute()"
  );
}

// T10 / 005 boundary: existing temp file + non-dry-run → error is NOT about file-open
#[ test ]
fn t10_stdin_file_with_real_file_no_open_error()
{
  let tmp = tempfile::NamedTempFile::new().expect( "create temp file" );
  std::fs::write( tmp.path(), "hello stdin" ).expect( "write temp" );
  let cmd = ClaudeCommand::new()
    .with_stdin_file( tmp.path().to_path_buf() );
  let result = cmd.execute();
  // The execute() may fail (no claude binary), but the error must NOT be about file-open
  match result
  {
    Ok( _ ) => {} // unlikely without claude, but fine
    Err( e ) =>
    {
      let msg = e.to_string();
      assert!(
        !msg.contains( "cannot open stdin file" ),
        "error must NOT be about file open for existing file. Got: {msg}"
      );
    }
  }
}

// T11 / FT-6 (005): execute_interactive() with nonexistent path → Err with path
#[ test ]
fn t11_stdin_file_interactive_err_on_nonexistent()
{
  let cmd = ClaudeCommand::new()
    .with_stdin_file( std::path::PathBuf::from( "/tmp/nonexistent_zzzz_t11.txt" ) );
  let result = cmd.execute_interactive();
  assert!( result.is_err(), "execute_interactive() must return Err for nonexistent file" );
  let err_msg = result.unwrap_err().to_string();
  assert!(
    err_msg.contains( "/tmp/nonexistent_zzzz_t11.txt" ),
    "error must contain file path. Got: {err_msg}"
  );
}

// T12 / FT-5 (006): Last-write wins on repeated with_unset_claudecode
#[ test ]
fn t12_unset_claudecode_last_write_wins()
{
  let cmd = ClaudeCommand::new()
    .with_unset_claudecode( false )
    .with_unset_claudecode( true )
    .build_command_for_test();
  let has_removal = cmd.get_envs().any( | ( key, val ) |
    key == "CLAUDECODE" && val.is_none()
  );
  assert!(
    has_removal,
    "last with_unset_claudecode(true) must win — CLAUDECODE must be marked for removal"
  );
}

// T13 / 005 regression: describe_compact() with stdin_file must start with "env -u CLAUDECODE"
//
// Root Cause: describe() emits "< path" as a separate last line when it is pushed to
//   `lines` instead of `parts`. describe_compact() = lines().last() would then return
//   "< path" rather than the invocation line.
// Why Not Caught: T01 uses contains("< path") which passes even if "< path" is the
//   ONLY content; it does not assert that the invocation line precedes it.
// Fix Applied: "< path" is pushed to `parts` (inline with the invocation) before
//   lines.push(parts.join(" ")), so it always appears on the same invocation line.
// Prevention: This test asserts describe_compact() starts_with("env -u CLAUDECODE") when
//   stdin_file is set, catching any future regression where "< path" moves to a
//   separate line and becomes the last() line.
// Pitfall: contains("< path") alone is insufficient to guard inline placement;
//   always pair with starts_with("env -u CLAUDECODE") in describe_compact assertions.
//
// Source: feature/005_stdin_file.md
#[ test ]
fn t13_stdin_file_describe_compact_on_claude_line()
{
  let compact = ClaudeCommand::new()
    .with_stdin_file( std::path::PathBuf::from( "/tmp/t13_stdin.txt" ) )
    .describe_compact();
  // Fix(BUG-246): default unset_claudecode=true → starts with "env -u CLAUDECODE"
  assert!(
    compact.starts_with( "env -u CLAUDECODE" ),
    "describe_compact() must start with 'env -u CLAUDECODE' when stdin_file is set \
     ('< path' must be inline on the invocation line, not the last separate line). Got: {compact}"
  );
  assert!(
    compact.contains( "< /tmp/t13_stdin.txt" ),
    "describe_compact() must contain stdin redirect inline. Got: {compact}"
  );
}

// T14 / 005 corner: execute_interactive() + dry_run + nonexistent → Ok (file not opened)
//
// Root Cause: If the stdin_file open were before the dry_run guard in execute_interactive(),
//   a nonexistent path would return Err even in dry-run mode.
// Why Not Caught: T04 covers execute() + dry_run + nonexistent, but execute_interactive()
//   has its own code path with a separate dry_run guard.
// Fix Applied: execute_interactive() checks dry_run and returns early before any file open.
// Prevention: This test verifies the dry_run guard is present in the interactive path,
//   mirroring T04's coverage for the non-interactive path.
// Pitfall: Assume both execute() and execute_interactive() have independent guards;
//   verifying one does not guarantee the other.
//
// Source: feature/005_stdin_file.md
#[ test ]
fn t14_stdin_file_interactive_dry_run_skips_open()
{
  let cmd = ClaudeCommand::new()
    .with_stdin_file( std::path::PathBuf::from( "/tmp/nonexistent_zzzz_t14.txt" ) )
    .with_dry_run( true );
  let result = cmd.execute_interactive();
  assert!(
    result.is_ok(),
    "execute_interactive() with dry_run=true must return Ok even with nonexistent file"
  );
}

// ── BUG-424: stdin_content spawn-method wiring (FT-9 through FT-15) ────────────

// Return a temp dir containing a fake `claude` shell script and the augmented PATH value.
//
// The returned `TempDir` must be kept alive for the duration of the test — dropping it
// removes the directory and makes the fake binary inaccessible. Private per-file copy:
// Rust compiles each `tests/*.rs` file as its own binary, so this ~15-line helper (already
// proven in `bug_243_test.rs:40`) is duplicated locally rather than shared — not worth
// extracting to a module for the 4 call sites below.
#[ cfg( unix ) ]
fn fake_claude_dir( body : &str ) -> ( tempfile::TempDir, String )
{
  use std::os::unix::fs::PermissionsExt as _;
  let dir  = tempfile::TempDir::new().expect( "tmpdir" );
  let path = dir.path().join( "claude" );
  let script = format!( "#!/bin/sh\n{body}\n" );
  std::fs::write( &path, script.as_bytes() ).expect( "write fake-claude" );
  std::fs::set_permissions( &path, std::fs::Permissions::from_mode( 0o755 ) )
    .expect( "chmod fake-claude" );
  let path_val = format!(
    "{}:{}",
    dir.path().display(),
    std::env::var( "PATH" ).unwrap_or_default(),
  );
  ( dir, path_val )
}

// T15 / FT-9: stdin_content in describe output
#[ test ]
fn t15_stdin_content_describe_output()
{
  let cmd = ClaudeCommand::new()
    .with_stdin_content( b"hello".to_vec() )
    .with_dry_run( true );
  let output = cmd.execute().expect( "dry-run must succeed" );
  assert!(
    output.stdout.contains( "<piped stdin, 5 bytes>" ),
    "describe output must show byte count. Got: {}", output.stdout
  );
}

// T16 / FT-10: stdin_file takes priority over stdin_content in describe output
#[ test ]
fn t16_stdin_file_priority_over_stdin_content_describe()
{
  let cmd = ClaudeCommand::new()
    .with_stdin_file( std::path::PathBuf::from( "/tmp/t16_stdin.txt" ) )
    .with_stdin_content( b"ignored".to_vec() )
    .with_dry_run( true );
  let output = cmd.execute().expect( "dry-run must succeed" );
  assert!(
    output.stdout.contains( "< /tmp/t16_stdin.txt" ),
    "describe output must show stdin_file path when both are set. Got: {}", output.stdout
  );
  assert!(
    !output.stdout.contains( "piped stdin" ),
    "describe output must NOT show piped-stdin byte count when stdin_file wins. Got: {}", output.stdout
  );
}

// T17 / FT-11: execute() delivers stdin_content bytes to the real subprocess's stdin
#[ cfg( unix ) ]
#[ test ]
#[ allow( unsafe_code ) ]
fn t17_stdin_content_delivered_via_execute()
{
  let ( _dir, path_val ) = fake_claude_dir( "cat" );
  let orig_path = std::env::var( "PATH" ).unwrap_or_default();
  // SAFETY: single-threaded test binary (nextest isolates each test into its own process);
  // no other test reads PATH concurrently.
  unsafe { std::env::set_var( "PATH", &path_val ); }
  let cmd = ClaudeCommand::new()
    .with_stdin_content( b"t17_marker".to_vec() );
  let result = cmd.execute();
  // SAFETY: restoring the original PATH value; same invariant as the `set_var` above —
  // single-threaded test binary, no concurrent PATH access.
  unsafe { std::env::set_var( "PATH", &orig_path ); }
  let output = result.expect( "execute must succeed with fake claude" );
  assert!(
    output.stdout.contains( "t17_marker" ),
    "execute() must deliver stdin_content bytes to subprocess stdin. Got: {}", output.stdout
  );
}

// T18 / FT-12: execute_interactive() delivers stdin_content bytes to the real subprocess's stdin
//
// execute_interactive() inherits the parent's stdout (TTY passthrough) instead of capturing
// it, so the fake claude script redirects its own stdin echo to a temp file instead of stdout —
// sidesteps the inherited-stdout problem entirely.
#[ cfg( unix ) ]
#[ test ]
#[ allow( unsafe_code ) ]
fn t18_stdin_content_delivered_via_execute_interactive()
{
  let out_file = tempfile::NamedTempFile::new().expect( "create output capture file" );
  let out_path = out_file.path().display().to_string();
  let ( _dir, path_val ) = fake_claude_dir( &format!( "cat > {out_path}" ) );
  let orig_path = std::env::var( "PATH" ).unwrap_or_default();
  // SAFETY: single-threaded test binary (nextest isolates each test into its own process);
  // no other test reads PATH concurrently.
  unsafe { std::env::set_var( "PATH", &path_val ); }
  let cmd = ClaudeCommand::new()
    .with_stdin_content( b"t18_marker".to_vec() );
  let result = cmd.execute_interactive();
  // SAFETY: restoring the original PATH value; same invariant as the `set_var` above —
  // single-threaded test binary, no concurrent PATH access.
  unsafe { std::env::set_var( "PATH", &orig_path ); }
  assert!( result.is_ok(), "execute_interactive must succeed with fake claude" );
  let received = std::fs::read_to_string( &out_path ).expect( "read captured output" );
  assert!(
    received.contains( "t18_marker" ),
    "execute_interactive() must deliver stdin_content bytes to subprocess stdin. Got: {received}"
  );
}

// T19 / FT-13: spawn_piped() delivers stdin_content bytes to the real subprocess's stdin
#[ cfg( unix ) ]
#[ test ]
#[ allow( unsafe_code ) ]
fn t19_stdin_content_delivered_via_spawn_piped()
{
  let ( _dir, path_val ) = fake_claude_dir( "cat" );
  let orig_path = std::env::var( "PATH" ).unwrap_or_default();
  // SAFETY: single-threaded test binary (nextest isolates each test into its own process);
  // no other test reads PATH concurrently.
  unsafe { std::env::set_var( "PATH", &path_val ); }
  let cmd = ClaudeCommand::new()
    .with_stdin_content( b"t19_marker".to_vec() );
  let child = cmd.spawn_piped();
  // SAFETY: restoring the original PATH value; same invariant as the `set_var` above —
  // single-threaded test binary, no concurrent PATH access.
  unsafe { std::env::set_var( "PATH", &orig_path ); }
  let output = child.expect( "spawn_piped must succeed" )
    .wait_with_output().expect( "wait for child" );
  let stdout = String::from_utf8_lossy( &output.stdout );
  assert!(
    stdout.contains( "t19_marker" ),
    "spawn_piped() must deliver stdin_content bytes to subprocess stdin. Got: {stdout}"
  );
}

// T20 / FT-14: spawn_tty() delivers stdin_content bytes to the real subprocess's stdin
//
// spawn_tty() inherits the parent's stdout (TTY passthrough), so the fake claude script
// redirects its stdin echo to a temp file — same technique as T18.
#[ cfg( unix ) ]
#[ test ]
#[ allow( unsafe_code ) ]
fn t20_stdin_content_delivered_via_spawn_tty()
{
  let out_file = tempfile::NamedTempFile::new().expect( "create output capture file" );
  let out_path = out_file.path().display().to_string();
  let ( _dir, path_val ) = fake_claude_dir( &format!( "cat > {out_path}" ) );
  let orig_path = std::env::var( "PATH" ).unwrap_or_default();
  // SAFETY: single-threaded test binary (nextest isolates each test into its own process);
  // no other test reads PATH concurrently.
  unsafe { std::env::set_var( "PATH", &path_val ); }
  let cmd = ClaudeCommand::new()
    .with_stdin_content( b"t20_marker".to_vec() );
  let child = cmd.spawn_tty();
  // SAFETY: restoring the original PATH value; same invariant as the `set_var` above —
  // single-threaded test binary, no concurrent PATH access.
  unsafe { std::env::set_var( "PATH", &orig_path ); }
  let mut child = child.expect( "spawn_tty must succeed" );
  child.wait().expect( "wait for child" );
  let received = std::fs::read_to_string( &out_path ).expect( "read captured output" );
  assert!(
    received.contains( "t20_marker" ),
    "spawn_tty() must deliver stdin_content bytes to subprocess stdin. Got: {received}"
  );
}

// T21 / FT-15: dry-run skips tempfile materialization when stdin_content is set
#[ test ]
fn t21_stdin_content_dry_run_skips_materialization()
{
  let cmd = ClaudeCommand::new()
    .with_stdin_content( b"unused".to_vec() )
    .with_dry_run( true );
  let result = cmd.execute();
  assert!(
    result.is_ok(),
    "dry-run must return Ok even with stdin_content set (no tempfile materialization attempted)"
  );
}
