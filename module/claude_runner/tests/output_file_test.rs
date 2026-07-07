//! `--output-file` Parameter Integration Tests
//!
//! ## Purpose
//!
//! Verify EC-1 through EC-6 from `tests/docs/cli/param/029_output_file.md`:
//! tee behavior, dry-run skip, write error, env var, CLI-wins, help text.
//!
//! ## Test Layout
//!
//! - EC-2, EC-5, EC-6: parser / dry-run — no subprocess required
//! - EC-1, EC-3, EC-4: require live subprocess — guarded by `#[cfg(feature = "enabled")]`
//!
//! ## Corner Cases Covered
//!
//! - EC-1: live run tee — file created with same content as stdout
//! - EC-2: no `--output-file` → no file artifact (dry-run)
//! - EC-3: non-writable path → exit 1, stderr contains path and OS error
//! - EC-4: `--output-file` + `--strip-fences` → same stripped content in file and stdout
//! - EC-5: `--dry-run` with `--output-file` → no file created
//! - EC-6: `clr --help` lists `--output-file`

mod cli_binary_test_helpers;
use cli_binary_test_helpers::{ run_cli, run_cli_with_env };
// Fix(BUG-316): Root cause: Command imported without cfg gate but used only in
// #[cfg(unix)] test fns (ec1/ec3/ec4); on Windows those tests are absent so the
// import is unused → -W unused-imports warning.  Pitfall: gate the import with
// the same cfg that gates all its usage sites.
#[ cfg( unix ) ]
use std::process::Command;

// ── EC-2: Default — no file artifact ────────────────────────────────────────

/// EC-2: Without `--output-file` a dry-run leaves no file on disk.
#[ test ]
fn ec2_no_output_file_no_artifact()
{
  let out = run_cli( &[ "--dry-run", "task" ] );
  assert!( out.status.success(), "dry-run must exit 0: {:?}", out.status.code() );
  // No file path was given — nothing to check beyond exit 0 and no unexpected files.
  // This test primarily guards that the field defaults to None without error.
}

// ── EC-5: --dry-run skips file write ────────────────────────────────────────

/// EC-5: `--output-file` + `--dry-run` → file NOT created.
#[ test ]
fn ec5_dry_run_skips_output_file_write()
{
  let unique = format!( "/tmp/clr_ec5_should_not_exist_{}.txt", std::process::id() );
  // Ensure file does not exist before the test
  let _ = std::fs::remove_file( &unique );

  let out = run_cli( &[ "--dry-run", "--output-file", &unique, "task" ] );
  assert!( out.status.success(), "dry-run must exit 0. stderr: {}", String::from_utf8_lossy( &out.stderr ) );
  assert!(
    !std::path::Path::new( &unique ).exists(),
    "dry-run must NOT create the output file at {unique}"
  );
}

// ── EC-6: --help lists --output-file ────────────────────────────────────────

/// EC-6: `clr --help` output contains `--output-file`.
#[ test ]
fn ec6_help_lists_output_file()
{
  let out = run_cli( &[ "--help" ] );
  assert!( out.status.success(), "clr --help must exit 0" );
  let stdout = String::from_utf8_lossy( &out.stdout );
  assert!(
    stdout.contains( "--output-file" ),
    "`clr --help` must list --output-file. Got:\n{stdout}"
  );
}

/// EC-6b: `clr ask --help` also lists `--output-file`.
#[ test ]
fn ec6b_ask_help_lists_output_file()
{
  let out = run_cli( &[ "ask", "--help" ] );
  assert!( out.status.success(), "clr ask --help must exit 0" );
  let stdout = String::from_utf8_lossy( &out.stdout );
  assert!(
    stdout.contains( "--output-file" ),
    "`clr ask --help` must list --output-file. Got:\n{stdout}"
  );
}

// ── CLR_OUTPUT_FILE env var accepted (dry-run) ───────────────────────────────

/// Env var `CLR_OUTPUT_FILE` is accepted — dry-run exits 0 without writing.
#[ test ]
fn clr_output_file_env_var_accepted_in_dry_run()
{
  let unique = format!( "/tmp/clr_envtest_should_not_exist_{}.txt", std::process::id() );
  let _ = std::fs::remove_file( &unique );

  let out = run_cli_with_env(
    &[ "--dry-run", "task" ],
    &[ ( "CLR_OUTPUT_FILE", &unique ) ],
  );
  assert!( out.status.success(), "CLR_OUTPUT_FILE + dry-run must exit 0. stderr: {}", String::from_utf8_lossy( &out.stderr ) );
  assert!(
    !std::path::Path::new( &unique ).exists(),
    "dry-run must NOT create the output file even when CLR_OUTPUT_FILE is set"
  );
}

// ── EC-4: --output-file + --strip-fences → same stripped content ────────────

/// EC-4: `--output-file` + `--strip-fences` → stripped content identical in file and stdout.
///
/// Uses a fake `claude` that returns a fenced code block; verifies neither file nor
/// stdout retains fence markers after stripping.
#[ cfg( unix ) ]
#[ test ]
fn ec4_strip_fences_and_output_file_same_content()
{
  use std::os::unix::fs::PermissionsExt;

  let tmp = tempfile::tempdir().expect( "create temp dir" );
  let fake = tmp.path().join( "claude" );
  // fake claude outputs a fenced code block
  std::fs::write( &fake, "#!/bin/sh\nprintf '```\\nsome code\\n```\\n'" )
    .expect( "write fake claude" );
  std::fs::set_permissions( &fake, std::fs::Permissions::from_mode( 0o755 ) )
    .expect( "chmod fake claude" );

  let out_path = tmp.path().join( "out.txt" );
  let out_path_str = out_path.to_str().expect( "path is utf-8" );
  let old_path = std::env::var( "PATH" ).unwrap_or_default();
  let new_path = format!( "{}:{old_path}", tmp.path().display() );
  let bin = env!( "CARGO_BIN_EXE_clr" );

  let out = Command::new( bin )
    .args( [ "-p", "--max-sessions", "0", "--strip-fences", "--output-file", out_path_str, "task" ] )
    .env( "PATH", &new_path )
    .output()
    .expect( "invoke clr" );

  assert!(
    out.status.success(),
    "exit must be 0. stderr: {}",
    String::from_utf8_lossy( &out.stderr )
  );
  let stdout  = String::from_utf8_lossy( &out.stdout ).to_string();
  let on_disk = std::fs::read_to_string( &out_path ).expect( "output file must exist after run" );

  assert_eq!( stdout, on_disk, "file and stdout must be equal after strip-fences" );
  assert!(
    !stdout.contains( "```" ),
    "output must not contain fence markers. Got: {stdout:?}"
  );
  assert!(
    stdout.trim().contains( "some code" ),
    "stripped content must include inner text. Got: {stdout:?}"
  );
}

// ── EC-1: tee behavior — file created with same content as stdout ─────────────

/// EC-1: tee behavior — file created with same content as stdout.
///
/// Uses a fake `claude` that outputs a known string; verifies file content equals stdout.
/// No real claude subprocess required.
#[ cfg( unix ) ]
#[ test ]
fn ec1_tee_file_content_equals_stdout()
{
  use std::os::unix::fs::PermissionsExt;

  let tmp = tempfile::tempdir().expect( "create temp dir" );
  let fake = tmp.path().join( "claude" );
  std::fs::write( &fake, "#!/bin/sh\nprintf 'hello_tee_test_content'" )
    .expect( "write fake claude" );
  std::fs::set_permissions( &fake, std::fs::Permissions::from_mode( 0o755 ) )
    .expect( "chmod fake claude" );

  let out_file = tmp.path().join( "out.txt" );
  let out_path = out_file.to_str().expect( "path is utf-8" );
  let old_path = std::env::var( "PATH" ).unwrap_or_default();
  let new_path = format!( "{}:{old_path}", tmp.path().display() );
  let bin = env!( "CARGO_BIN_EXE_clr" );

  let out = Command::new( bin )
    .args( [ "-p", "--max-sessions", "0", "--output-file", out_path, "task" ] )
    .env( "PATH", &new_path )
    .output()
    .expect( "invoke clr" );

  assert!(
    out.status.success(),
    "exit must be 0. stderr: {}",
    String::from_utf8_lossy( &out.stderr )
  );
  let stdout  = String::from_utf8_lossy( &out.stdout ).to_string();
  let on_disk = std::fs::read_to_string( &out_file ).expect( "output file must exist after run" );
  assert_eq!( stdout, on_disk, "file content must equal stdout" );
  assert!( !stdout.is_empty(), "captured output must be non-empty" );
}

// ── EC-3: non-writable path → exit 1 ─────────────────────────────────────────

/// EC-3: non-writable path → exit 1, stderr contains path.
///
/// Uses a fake `claude` that exits 0; verifies clr exits 1 and names the bad path on stderr
/// when the output file cannot be created.  No real claude subprocess required.
#[ cfg( unix ) ]
#[ test ]
fn ec3_nonwritable_path_exits_1()
{
  use std::os::unix::fs::PermissionsExt;

  let tmp = tempfile::tempdir().expect( "create temp dir" );
  let fake = tmp.path().join( "claude" );
  std::fs::write( &fake, "#!/bin/sh\nprintf 'output'" )
    .expect( "write fake claude" );
  std::fs::set_permissions( &fake, std::fs::Permissions::from_mode( 0o755 ) )
    .expect( "chmod fake claude" );

  let old_path = std::env::var( "PATH" ).unwrap_or_default();
  let new_path = format!( "{}:{old_path}", tmp.path().display() );
  let bad_path = "/nonexistent_dir_clr_ec3/out.txt";
  let bin = env!( "CARGO_BIN_EXE_clr" );

  let out = Command::new( bin )
    .args( [ "-p", "--max-sessions", "0", "--output-file", bad_path, "task" ] )
    .env( "PATH", &new_path )
    .output()
    .expect( "invoke clr" );

  assert_eq!(
    out.status.code(),
    Some( 1 ),
    "non-writable path must exit 1. Got: {:?}", out.status.code()
  );
  let stderr = String::from_utf8_lossy( &out.stderr );
  assert!(
    stderr.contains( bad_path ),
    "stderr must contain the bad path. Got:\n{stderr}"
  );
}
