//! Shared test helpers for `claude_runner` integration tests.
//!
//! # Test Matrix
//!
//! | Helper | Used By |
//! |--------|---------|
//! | `run_cli` | `cli_args_test`, `cli_args_ext_test`, `dry_run_test`, `ultrathink_args_test`, `effort_args_test`, `param_edge_cases_test`, `param_extended_flags_test`, `param_group_test`, `execution_mode_test`, `quiet_test`, `ask_command_test`, `user_story_test`, `user_story_creds_isolated_test`, `user_story_output_test`, `user_story_ps_test`, `user_story_kill_test`, `ps_command_test`, `kill_command_test`, `ps_mode_test`, `ps_columns_test`, `ps_wide_test`, `ps_pid_test`, `ps_inspect_test`, `ps_flags_test`, `output_style_test`, `summary_fields_test` |
//! | `run_cli_with_env` | `env_var_test`, `env_var_ext_test`, `invariant_trace_universality_test`, `param_trace_edge_cases_test`, `param_group_test`, `isolated_test`, `user_story_creds_isolated_test`, `user_story_output_test`, `bug_reproducers_239_244_test`, `error_classification_test`, `ps_command_test`, `user_story_ps_test`, `output_style_test`, `summary_fields_test` |
//! | `make_session_dir` | `cli_args_test`, `ultrathink_args_test`, `user_story_test` |
//! | `exit_code` | `refresh_test`, `bug_reproducers_239_244_test`, `user_story_test`, `user_story_creds_isolated_test`, `isolated_test` |
//! | `stderr_str` | `refresh_test`, `bug_reproducers_239_244_test`, `invariant_trace_universality_test`, `error_classification_test`, `user_story_test`, `user_story_creds_isolated_test`, `isolated_correctness_test`, `isolated_test`, `ps_command_test`, `user_story_ps_test`, `kill_command_test`, `user_story_kill_test`, `ps_mode_test`, `ps_columns_test`, `output_format_test` |
//! | `stdout_str` | `refresh_test`, `isolated_correctness_test`, `isolated_test`, `dry_run_test`, `ps_command_test`, `user_story_ps_test`, `kill_command_test`, `user_story_kill_test`, `ps_mode_test`, `ps_columns_test`, `ps_wide_test`, `output_format_test` |
//! | `make_creds_file` | `refresh_test`, `param_trace_edge_cases_test`, `invariant_trace_universality_test`, `user_story_test`, `user_story_creds_isolated_test`, `isolated_correctness_test`, `isolated_test` |
//! | `fake_claude_dir` (unix) | `bug_reproducers_239_244_test`, `error_classification_test`, `execution_mode_test`, `bug_reproducers_247_test`, `exit_code_contract_test`, `output_format_test`, `output_style_test`, `summary_fields_test`, `journal_integration_test`, `param_extended_flags_test` (S89 only) |
//! | `fake_claude_binary_dir` (unix) | `ps_command_test`, `user_story_ps_test`, `kill_command_test`, `user_story_kill_test`, `ps_mode_test`, `ps_columns_test`, `ps_wide_test`, `ps_flags_test` |
//! | `fake_claude` (unix) | `execution_mode_test`, `expect_validation_test` |
//! | `run_with_path` | `execution_mode_test`, `expect_validation_test`, `exit_code_contract_test`, `output_format_test` |
//! | `run_dry` | `user_story_test`, `user_story_creds_isolated_test`, `user_story_output_test`, `dry_run_test` |
//! | `run_ask_dry` | `ask_command_test`, `user_story_creds_isolated_test` |
//! | `spawn_fake_claude` (unix) | `ps_command_test`, `user_story_ps_test`, `kill_command_test`, `user_story_kill_test`, `ps_mode_test`, `ps_columns_test`, `ps_wide_test`, `ps_pid_test`, `ps_inspect_test`, `param_group_test`, `ps_flags_test` |
//! | `spawn_print_claude` (unix) | `ps_command_test`, `user_story_ps_test`, `ps_mode_test`, `ps_columns_test`, `ps_inspect_test`, `param_group_test` |
//! | `run_clr_ps` (unix) | `ps_command_test`, `user_story_ps_test` |
//! | `run_clr_kill` (unix) | `kill_command_test`, `user_story_kill_test` |
//!
//! # Testing Techniques
//!
//! - **`--dry-run`**: Inspect assembled command without spawning Claude subprocess.
//! - **`--trace` (for `isolated`/`refresh`)**: These commands lack `--dry-run`;
//!   use `--trace` to verify the assembled command on stderr.
//! - **`PATH=/nonexistent`**: Force binary-not-found for deterministic failure
//!   testing — trace output fires before subprocess invocation attempt.
//! - **`make_session_dir`**: Create a temp session dir with a dummy `.jsonl` file so
//!   `session_exists()` returns `Some(SessionId)` regardless of the ambient host environment.  Tests that assert
//!   `-c` injection must use `--session-dir <path>` with this helper; otherwise they
//!   are fragile and fail in clean container environments with no prior Claude sessions.

use std::process::Command;

fn assert_container()
{
  let in_container = std::path::Path::new( "/.dockerenv" ).exists()
    || std::path::Path::new( "/run/.containerenv" ).exists()
    || std::env::var( "RUNBOX_CONTAINER" ).as_deref() == Ok( "1" );
  let escaped = std::env::var( "VERB_LAYER" ).as_deref() == Ok( "l0" );
  assert!(
    in_container || escaped,
    "\n\nTests must run inside a container.\n\
     Standard invocation: cd module/claude_profile && ./verb/test\n\
     Host bypass:         VERB_LAYER=l0 cargo nextest run --all-features\n"
  );
}

/// Invoke the `clr` binary with `args`, returning raw `Output` without asserting success.
///
/// Used for both success-path and expected-failure cases — callers check
/// `output.status` or inspect `output.stdout`/`output.stderr` directly.
///
/// `CLR_DIR` and `CLR_SESSION_DIR` are removed from the subprocess environment so that
/// ambient shell values do not affect tests that assert the absence of a `cd` prefix
/// line or `-c` flag (e.g., `s18_dir_absent_from_default_output`).  Tests that
/// explicitly exercise `CLR_DIR`/`CLR_SESSION_DIR` behavior use `run_cli_with_env`
/// instead, which adds those vars explicitly.
///
/// # Panics
///
/// Panics if the `clr` binary cannot be launched (process spawn failure).
#[must_use]
#[inline]
#[allow(dead_code)]
pub fn run_cli( args : &[ &str ] ) -> std::process::Output
{
  assert_container();
  let bin = env!( "CARGO_BIN_EXE_clr" );
  Command::new( bin )
    .args( args )
    .env_remove( "CLR_DIR" )
    .env_remove( "CLR_SESSION_DIR" )
    .output()
    .expect( "Failed to invoke clr binary" )
}

/// Invoke the `clr` binary with `args` and extra environment variables, returning raw `Output`.
///
/// Env vars are injected via `Command::envs()` — no process-global `std::env::set_var`.
/// Safe for concurrent test execution; each subprocess sees only the injected env.
///
/// # Panics
///
/// Panics if the `clr` binary cannot be launched (process spawn failure).
#[must_use]
#[inline]
#[allow(dead_code)]
pub fn run_cli_with_env
(
  args : &[ &str ],
  env  : &[ ( &str, &str ) ],
) -> std::process::Output
{
  assert_container();
  let bin = env!( "CARGO_BIN_EXE_clr" );
  Command::new( bin )
    .args( args )
    .envs( env.iter().copied() )
    .output()
    .expect( "failed to execute clr binary" )
}

/// Create a temp session directory with one dummy `.jsonl` file; returns `(dir, path_string)`.
///
/// The caller must keep the returned `TempDir` alive for the duration of the test —
/// the directory and its contents are deleted when the `TempDir` is dropped.
/// Pass the returned `path_string` as the value of `--session-dir` to force
/// `session_exists()` to return `Some(SessionId)`, making `-c` injection deterministic
/// regardless of the ambient host session state.
///
/// Pitfall: if the caller drops `TempDir` before passing the path to the subprocess,
/// the directory is deleted and `session_exists()` returns `None`.
///
/// # Panics
///
/// Panics if the temp directory or the dummy file cannot be created.
#[must_use]
#[inline]
#[allow(dead_code)]
pub fn make_session_dir() -> ( tempfile::TempDir, String )
{
  let dir = tempfile::TempDir::new().expect( "failed to create temp session dir" );
  std::fs::write( dir.path().join( "00000000-0000-0000-0000-000000000000.jsonl" ), b"{}" )
    .expect( "failed to write dummy session file" );
  let path = dir.path().to_str().expect( "session dir path must be valid UTF-8" ).to_owned();
  ( dir, path )
}

/// Extract the process exit code from a subprocess `Output`.
///
/// Returns `-1` when the process was terminated by a signal (no numeric exit code).
#[must_use]
#[inline]
#[allow(dead_code)]
pub fn exit_code( o : &std::process::Output ) -> i32 { o.status.code().unwrap_or( -1 ) }

/// Extract `stderr` as a UTF-8 string from a subprocess `Output`.
#[must_use]
#[inline]
#[allow(dead_code)]
pub fn stderr_str( o : &std::process::Output ) -> String
{
  String::from_utf8_lossy( &o.stderr ).to_string()
}

/// Extract `stdout` as a UTF-8 string from a subprocess `Output`.
#[must_use]
#[inline]
#[allow(dead_code)]
pub fn stdout_str( o : &std::process::Output ) -> String
{
  String::from_utf8_lossy( &o.stdout ).to_string()
}

/// Write `content` to a new `NamedTempFile` and return it.
///
/// The caller must keep the returned file alive for the duration of the test;
/// dropping it deletes the file on disk.
///
/// # Panics
///
/// Panics if the temp file cannot be created or written.
#[inline]
#[must_use]
#[allow(dead_code)]
pub fn make_creds_file( content : &str ) -> tempfile::NamedTempFile
{
  use std::io::Write as _;
  let mut f = tempfile::NamedTempFile::new().expect( "failed to create temp creds file" );
  f.write_all( content.as_bytes() ).expect( "failed to write creds content" );
  f
}

/// Create a temp dir containing a `claude` shell script with the given body.
///
/// Returns `(TempDir, path_val)` where `path_val` prepends the dir to `$PATH`
/// for injection into subprocess env.  The caller must keep the `TempDir` alive
/// for the duration of the test; dropping it deletes the script.
///
/// # Panics
///
/// Panics if the temp directory, script file, or permissions cannot be set.
#[cfg(unix)]
#[inline]
#[must_use]
#[allow(dead_code)]
pub fn fake_claude_dir( body : &str ) -> ( tempfile::TempDir, String )
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

/// Create a temp dir containing a `claude` symlink to `/bin/sleep`.
///
/// Returns `(TempDir, path_val)` where `path_val` prepends the dir to `$PATH`.
/// A symlink is used instead of a copy to avoid ENOSPC in space-constrained
/// containers and to eliminate ETXTBSY races from concurrent copies.
/// Because `Command::new("claude")` sets `argv[0]` = `"claude"`,
/// `/proc/{pid}/cmdline` shows the basename as `"claude"` — making the spawned
/// process visible to `find_claude_processes()`.  Spawn with `.arg("30")` to
/// keep the process alive long enough for `clr ps` to observe it.
///
/// The caller must keep the `TempDir` alive and `kill()`+`wait()` the child.
///
/// # Panics
///
/// Panics if the temp directory cannot be created or the symlink cannot be made.
#[cfg(unix)]
#[inline]
#[must_use]
#[allow(dead_code)]
pub fn fake_claude_binary_dir() -> ( tempfile::TempDir, String )
{
  let dir  = tempfile::TempDir::new().expect( "tmpdir" );
  let dest = dir.path().join( "claude" );
  std::os::unix::fs::symlink( "/bin/sleep", &dest )
    .expect( "symlink /bin/sleep as claude" );
  let path_val = format!(
    "{}:{}",
    dir.path().display(),
    std::env::var( "PATH" ).unwrap_or_default(),
  );
  ( dir, path_val )
}

/// Spawn a fake `claude` ELF process using the given PATH env; return the `Child` handle.
///
/// Requires `fake_claude_binary_dir()` to have been called first — the PATH must contain
/// a symlink named `claude` pointing to a real ELF binary (shell scripts appear as `sh`
/// in `/proc/{pid}/cmdline` and are invisible to `find_claude_processes()`).
/// The arg `"30"` is passed to the ELF binary (sleep duration) to keep the process alive.
/// The caller must `kill()` + `wait()` the returned child to avoid leaks.
///
/// # Panics
///
/// Panics if the subprocess cannot be spawned after retries.
#[cfg(unix)]
#[inline]
#[must_use]
#[allow(dead_code)]
pub fn spawn_fake_claude( path_val : &str ) -> std::process::Child
{
  assert_container();
  // Retry up to 3 times on ETXTBSY (os error 26 — ExecutableFileBusy).
  // Historically `fake_claude_binary_dir()` used fs::copy which could race with
  // concurrent copies; now uses symlinks, so ETXTBSY should not occur — but the
  // retry is kept as a safety net.
  let mut attempt = 0u32;
  loop
  {
    match std::process::Command::new( "claude" )
      .env( "PATH", path_val )
      .arg( "30" )
      .stdout( std::process::Stdio::null() )
      .stderr( std::process::Stdio::null() )
      .spawn()
    {
      Ok( child ) =>
      {
        std::thread::sleep( core::time::Duration::from_millis( 200 ) );
        return child;
      }
      Err( ref e ) if e.raw_os_error() == Some( 26 ) && attempt < 3 =>
      {
        attempt += 1;
        std::thread::sleep( core::time::Duration::from_millis( 20 * u64::from( attempt ) ) );
      }
      Err( e ) => panic!( "spawn fake claude: {e}" ),
    }
  }
}

/// Spawn a print-mode fake `claude` process (argv contains `--print`).
///
/// Uses `/bin/sh` with `arg0` set to `"claude"` and command string
/// `"sleep 30; :"`.  The `; :` compound prevents the shell from exec-replacing
/// itself with `sleep` (POSIX shells only exec the last *simple* command, not a
/// compound list).  The resulting `/proc/{pid}/cmdline` is:
///
/// ```text
/// ["claude", "-c", "sleep 30; :", "--print"]
/// ```
///
/// `classify_mode()` finds `"--print"` at `args[3]` and returns `"print"`.
/// The `"--print"` token is the script's `$0` (command name for error messages
/// in POSIX `-c` semantics) — it is NOT forwarded to `sleep` as an argument.
///
/// The caller must `kill()` + `wait()` the returned child to avoid leaks.
///
/// # Panics
///
/// Panics if the subprocess cannot be spawned.
#[ cfg( unix ) ]
#[ inline ]
#[ must_use ]
#[ allow( dead_code ) ]
pub fn spawn_print_claude( path_val : &str ) -> std::process::Child
{
  assert_container();
  use std::os::unix::process::CommandExt as _;
  let child = std::process::Command::new( "/bin/sh" )
    .arg0( "claude" )
    .arg( "-c" )
    .arg( "sleep 30; :" )   // "; :" prevents exec-into-sleep (compound list, not simple command)
    .arg( "--print" )       // argv[3] = $0 (script name); classify_mode() finds "--print" here
    .env( "PATH", path_val )
    .stdout( std::process::Stdio::null() )
    .stderr( std::process::Stdio::null() )
    .spawn()
    .expect( "spawn print-mode fake claude" );
  std::thread::sleep( core::time::Duration::from_millis( 200 ) );
  child
}

/// Run `clr ps` with the given PATH env; return the raw `Output`.
///
/// # Panics
///
/// Panics if the subprocess cannot be launched.
#[cfg(unix)]
#[inline]
#[must_use]
#[allow(dead_code)]
pub fn run_clr_ps( path_val : &str ) -> std::process::Output
{
  assert_container();
  let bin = env!( "CARGO_BIN_EXE_clr" );
  std::process::Command::new( bin )
    .arg( "ps" )
    .env( "PATH", path_val )
    .output()
    .expect( "run clr ps" )
}

/// Invoke `clr ask --dry-run` with extra args; assert exit 0 and return stdout as `String`.
///
/// Prepends `["ask", "--dry-run"]` to the given args, invokes the binary, asserts success,
/// and returns the captured stdout.
///
/// # Panics
///
/// Panics if the subprocess cannot be launched or exits non-zero.
#[must_use]
#[inline]
#[allow(dead_code)]
pub fn run_ask_dry( extra_args : &[ &str ] ) -> String
{
  assert_container();
  let bin = env!( "CARGO_BIN_EXE_clr" );
  let mut args = vec![ "ask", "--dry-run" ];
  args.extend_from_slice( extra_args );
  let out = Command::new( bin )
    .args( &args )
    .output()
    .expect( "failed to invoke clr binary" );
  assert!(
    out.status.success(),
    "clr ask --dry-run failed (exit {}): {}",
    out.status.code().unwrap_or( -1 ),
    String::from_utf8_lossy( &out.stderr )
  );
  String::from_utf8_lossy( &out.stdout ).into_owned()
}

/// Create a fake `claude` binary from a full shell script; return `(tempdir, modified PATH)`.
///
/// Unlike `fake_claude_dir`, the caller provides the full script including the shebang
/// (`#!/bin/sh`). The temp dir is prepended to `$PATH` so the fake binary is found first.
/// The caller must keep the returned `TempDir` alive for the duration of the test.
///
/// # Panics
///
/// Panics if the temp directory, script file, or permissions cannot be set.
#[cfg(unix)]
#[inline]
#[must_use]
#[allow(dead_code)]
pub fn fake_claude( script : &str ) -> ( tempfile::TempDir, String )
{
  use std::os::unix::fs::PermissionsExt as _;
  let tmp  = tempfile::tempdir().expect( "Failed to create temp dir" );
  let fake = tmp.path().join( "claude" );
  std::fs::write( &fake, script ).expect( "Failed to write fake claude" );
  std::fs::set_permissions( &fake, std::fs::Permissions::from_mode( 0o755 ) )
    .expect( "Failed to chmod fake claude" );
  let old_path = std::env::var( "PATH" ).unwrap_or_default();
  let new_path = format!( "{}:{old_path}", tmp.path().display() );
  ( tmp, new_path )
}

/// Invoke `clr --dry-run` with `args`; assert exit 0 and return stdout as a `String`.
///
/// Prepends `--dry-run` to the given args, invokes the binary, asserts success,
/// and returns the captured stdout. The caller need not add `--dry-run` themselves.
///
/// `CLR_DIR` and `CLR_SESSION_DIR` are removed for the same reason as `run_cli` — see
/// that function's doc comment for the rationale.
///
/// # Panics
///
/// Panics if the subprocess cannot be launched or exits non-zero.
#[ must_use ]
#[ inline ]
#[ allow( dead_code ) ]
pub fn run_dry( args : &[ &str ] ) -> String
{
  assert_container();
  let bin = env!( "CARGO_BIN_EXE_clr" );
  let mut full = vec![ "--dry-run" ];
  full.extend_from_slice( args );
  let out = Command::new( bin )
    .args( &full )
    .env_remove( "CLR_DIR" )
    .env_remove( "CLR_SESSION_DIR" )
    .output()
    .expect( "Failed to invoke clr binary" );
  assert!(
    out.status.success(),
    "dry-run failed (exit {}): {}",
    out.status.code().unwrap_or( -1 ),
    String::from_utf8_lossy( &out.stderr )
  );
  String::from_utf8_lossy( &out.stdout ).into_owned()
}

/// Invoke `clr` binary with `args` and a custom `PATH`; return raw `Output`.
///
/// Sets only the `PATH` environment variable; all other env vars are inherited.
/// Use this when tests inject a fake `claude` binary via PATH manipulation.
///
/// # Panics
///
/// Panics if the `clr` binary cannot be launched.
#[must_use]
#[inline]
#[allow(dead_code)]
pub fn run_with_path( args : &[ &str ], path : &str ) -> std::process::Output
{
  assert_container();
  let bin = env!( "CARGO_BIN_EXE_clr" );
  Command::new( bin )
    .args( args )
    .env( "PATH", path )
    .output()
    .expect( "Failed to invoke clr binary" )
}

/// Run `clr kill <pid>`; return the raw `Output`.
///
/// `dispatch_kill` reads `/proc` directly — PATH is not needed.  The helper
/// exists for symmetry with `run_clr_ps` and to keep test call sites concise.
///
/// # Panics
///
/// Panics if the subprocess cannot be launched.
#[cfg(unix)]
#[must_use]
#[inline]
#[allow(dead_code)]
pub fn run_clr_kill( pid : u32 ) -> std::process::Output
{
  assert_container();
  let bin = env!( "CARGO_BIN_EXE_clr" );
  Command::new( bin )
    .args( [ "kill", &pid.to_string() ] )
    .output()
    .expect( "run clr kill" )
}
