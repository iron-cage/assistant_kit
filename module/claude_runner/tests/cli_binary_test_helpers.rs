//! Shared test helpers for `claude_runner` integration tests.
//!
//! # Test Matrix
//!
//! | Helper | Used By |
//! |--------|---------|
//! | `run_cli` | `cli_args_test`, `cli_args_ext_test`, `dry_run_test`, `ultrathink_args_test`, `effort_args_test`, `param_edge_cases_test`, `param_extended_flags_test`, `param_group_test`, `execution_mode_test`, `quiet_test`, `ask_command_test`, `user_story_test`, `user_story_creds_isolated_test`, `user_story_output_test`, `user_story_ps_test`, `user_story_kill_test`, `ps_command_test`, `kill_command_test`, `ps_mode_test`, `ps_columns_test`, `ps_wide_test`, `ps_pid_test`, `ps_inspect_test`, `ps_flags_test`, `output_style_test`, `summary_fields_test`, `no_compact_window_test`, `json_config_test` |
//! | `run_cli_with_env` | `env_var_test`, `env_var_ext_test`, `invariant_trace_universality_test`, `param_trace_edge_cases_test`, `param_group_test`, `isolated_test`, `user_story_creds_isolated_test`, `user_story_output_test`, `bug_reproducers_239_244_test`, `error_classification_test`, `ps_command_test`, `user_story_ps_test`, `output_style_test`, `summary_fields_test`, `no_compact_window_test`, `json_config_test`, `config_file_test` |
//! | `run_cli_in_dir` | `config_file_test` |
//! | `make_session_dir` (deprecated for new use) | `cli_args_test`, `ultrathink_args_test`, `user_story_test`, `dry_run_test` |
//! | `make_zero_turn_session_dir` (deprecated for new use) | `execution_mode_test` |
//! | `df` | `session_from_test`, `session_verification_test`, `session_path_resolution_test`, `session_source_isolation_test` |
//! | `make_session_for` | `session_from_test`, `session_verification_test`, `session_path_resolution_test`, `session_source_isolation_test` |
//! | `exit_code` | `refresh_test`, `bug_reproducers_239_244_test`, `user_story_test`, `user_story_creds_isolated_test`, `isolated_test`, `json_config_test`, `config_file_test` |
//! | `stderr_str` | `refresh_test`, `bug_reproducers_239_244_test`, `invariant_trace_universality_test`, `error_classification_test`, `user_story_test`, `user_story_creds_isolated_test`, `isolated_correctness_test`, `isolated_test`, `ps_command_test`, `user_story_ps_test`, `kill_command_test`, `user_story_kill_test`, `ps_mode_test`, `ps_columns_test`, `output_format_test`, `no_compact_window_test`, `json_config_test`, `config_file_test` |
//! | `stdout_str` | `refresh_test`, `isolated_correctness_test`, `isolated_test`, `dry_run_test`, `ps_command_test`, `user_story_ps_test`, `kill_command_test`, `user_story_kill_test`, `ps_mode_test`, `ps_columns_test`, `ps_wide_test`, `output_format_test`, `no_compact_window_test`, `json_config_test`, `config_file_test` |
//! | `make_creds_file` | `refresh_test`, `param_trace_edge_cases_test`, `invariant_trace_universality_test`, `user_story_test`, `user_story_creds_isolated_test`, `isolated_correctness_test`, `isolated_test`, `no_compact_window_test`, `json_config_test` |
//! | `fake_claude_dir` (unix) | `bug_reproducers_239_244_test`, `error_classification_test`, `execution_mode_test`, `bug_reproducers_247_test`, `exit_code_contract_test`, `output_format_test`, `output_style_test`, `summary_fields_test`, `journal_integration_test`, `journal_integration_ext_test`, `param_extended_flags_test` (S89 only), `config_file_test` |
//! | `fake_claude_binary_dir` (unix) | `ps_command_test`, `user_story_ps_test`, `kill_command_test`, `user_story_kill_test`, `ps_mode_test`, `ps_columns_test`, `ps_wide_test`, `ps_flags_test`, `config_file_test` |
//! | `fake_claude` (unix) | `execution_mode_test`, `expect_validation_test` |
//! | `run_with_path` | `execution_mode_test`, `expect_validation_test`, `exit_code_contract_test`, `output_format_test` |
//! | `run_with_path_stdin` | `execution_mode_test` |
//! | `run_with_path_proc` (unix) | `expect_validation_test` |
//! | `make_proc_dir` (unix) | `kill_command_test`, `expect_validation_test`, `config_file_test` |
//! | `run_dry` | `user_story_test`, `user_story_creds_isolated_test`, `user_story_output_test`, `dry_run_test` |
//! | `run_ask_dry` | `ask_command_test`, `user_story_creds_isolated_test` |
//! | `run_topic_dry` | `topic_command_test` |
//! | `spawn_fake_claude` (unix) | `ps_command_test`, `user_story_ps_test`, `kill_command_test`, `user_story_kill_test`, `ps_mode_test`, `ps_columns_test`, `ps_wide_test`, `ps_pid_test`, `ps_inspect_test`, `param_group_test`, `ps_flags_test` |
//! | `spawn_print_claude` (unix) | `ps_command_test`, `user_story_ps_test`, `ps_mode_test`, `ps_columns_test`, `ps_inspect_test`, `param_group_test`, `concurrency_gate_test`, `concurrency_gate_ext2_test`, `config_file_test` |
//! | `spawn_print_claude_for` (unix) | `concurrency_gate_test`, `concurrency_gate_ext_test`, `concurrency_gate_ext2_test`, `concurrency_gate_ext3_test`, `concurrency_gate_deadline_test`, `journal_integration_ext_test`, `config_file_test` |
//! | `run_clr_ps` (unix) | `ps_command_test`, `user_story_ps_test` |
//! | `run_clr_kill` (unix) | `kill_command_test`, `user_story_kill_test` |
//! | `run_isolated` | `isolated_test`, `isolated_plan034_test`, `isolated_plan035_test` |
//! | `find_jsonl_files` (unix) | `journal_integration_test`, `journal_integration_ext_test` |
//! | `read_journal_content` (unix) | `journal_integration_test`, `journal_integration_ext_test` |
//! | `run_with_journal` (unix) | `journal_integration_test`, `journal_integration_ext_test` |
//! | `build_argv_tolerant_sleeper` | `concurrency_gate_test`, `concurrency_gate_ext2_test` |
//! | `slot_owner_pid` | `concurrency_gate_ext_test`, `concurrency_gate_ext2_test` |
//! | `spawn_parked_helper_thread` (unix) | `concurrency_gate_test`, `ps_command_test` |
//! | `wait_for_marker_in_files` | `concurrency_gate_ext_test` |
//!
//! # Testing Techniques
//!
//! - **`--dry-run`**: Inspect assembled command without spawning Claude subprocess.
//!   For `run`/`ask`, dry-run output appears on **stdout**. For `isolated`/`refresh`,
//!   dry-run output appears on **stderr** (R5 added `--dry-run` support to both).
//! - **`--trace` (for `isolated`/`refresh`)**: Verify assembled command on stderr;
//!   subprocess spawn attempt follows (typically fails if `claude` binary absent).
//! - **`PATH=/nonexistent`**: Force binary-not-found for deterministic failure
//!   testing — trace output fires before subprocess invocation attempt.
//! - **`make_session_for`**: Fix(BUG-493) replacement for the retired `--session-dir`
//!   isolation trick. Seeds `<claude_home>/projects/<df(src_dir)>/<uuid>.jsonl` so
//!   `session_exists()` returns `Some(SessionId)` deterministically. Tests that assert
//!   `-c` injection must set `CLAUDE_HOME` to the seeded temp dir and pass
//!   `--from <src_dir>` (or arrange `src_dir` to equal the subprocess's own cwd);
//!   otherwise they are fragile and fail in clean container environments with no
//!   prior Claude sessions, or unstable in dirty ones with leftover host sessions.

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
/// `CLR_NO_COMPACT_WINDOW` is removed for a narrower reason: `077_no_compact_window.md`
/// EC-7 asserts the `CLAUDE_CODE_AUTO_COMPACT_WINDOW` default injection under a
/// *provably* unset opt-out, which the same When/Then as EC-1 can only satisfy if the
/// var's absence is enforced here rather than inherited from whatever the host happens
/// to export. Scrubbing it makes `default_injection_run` deterministic instead of
/// ambient-dependent. Tests that exercise the opt-out use `run_cli_with_env`.
///
/// `HOME` is set to a fixed, empty-by-design path (`/tmp/clr-isolated-home`) so that
/// a host `~/.clr/config.toml` cannot inject `--model` or other preference values into
/// tests that assert a clean default state (Fix(BUG-008) isolation guard). Tests that
/// need a populated `HOME` (e.g., pref-reading tests) use `run_cli_with_env` with an
/// explicit `("HOME", temp_dir)` pair.
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
    .env( "HOME", "/tmp/clr-isolated-home" )
    .env_remove( "CLR_DIR" )
    .env_remove( "CLR_SESSION_DIR" )
    .env_remove( "CLR_NO_COMPACT_WINDOW" )
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

/// Invoke the `clr` binary with `args` inside `dir`, with extra environment variables.
///
/// Mirrors `run_cli_with_env` but additionally sets the subprocess's working directory
/// via `Command::current_dir()` — per-subprocess, safe for concurrent test execution.
/// Used by tests that exercise project-level `.clr.toml` discovery (relative to cwd).
///
/// # Panics
///
/// Panics if the `clr` binary cannot be launched (process spawn failure).
#[must_use]
#[inline]
#[allow(dead_code)]
pub fn run_cli_in_dir
(
  args : &[ &str ],
  dir  : &std::path::Path,
  env  : &[ ( &str, &str ) ],
) -> std::process::Output
{
  assert_container();
  let bin = env!( "CARGO_BIN_EXE_clr" );
  Command::new( bin )
    .args( args )
    .current_dir( dir )
    .envs( env.iter().copied() )
    .output()
    .expect( "failed to execute clr binary" )
}

/// Encode a path using the `Df()` algorithm from `algorithm/001_path_encoding.md`.
///
/// Delegates to the real production encoder rather than reimplementing it —
/// see `Fix(BUG-391)` in `session_from_test.rs` for the regression a hand-rolled
/// duplicate caused (silent divergence from `encode_path()`'s substitution scope
/// the moment a fixture path contained a `.`, e.g. `tempfile::TempDir`'s `.tmp` prefix).
///
/// # Panics
///
/// Panics if `path` cannot be encoded by the production `encode_path()` algorithm.
#[must_use]
#[inline]
#[allow(dead_code)]
pub fn df( path : &str ) -> String
{
  claude_storage_core::encode_path( std::path::Path::new( path ) )
    .expect( "df(): path must encode successfully in test fixtures" )
}

/// Create `<claude_home>/projects/<df(src_dir)>/<uuid>.jsonl` with non-empty content.
///
/// Fix(BUG-493): the deterministic replacement for the old `make_session_dir()` +
/// raw `--session-dir` isolation trick. `--session-dir` is deprecated and inert —
/// `session_exists()` no longer scans it — so forcing a match now requires seeding
/// the storage location `session_from_dir` actually computes: `--from <src_dir>`
/// (or cwd, if `--from` is omitted) resolved through `CLAUDE_HOME`. Callers must
/// set `CLAUDE_HOME` to `claude_home` on the subprocess and pass `--from src_dir`
/// (or arrange `src_dir` to equal the subprocess's own cwd) to make `session_exists()`
/// find the seeded file deterministically, regardless of ambient host session state.
///
/// Returns the `.jsonl` path. The caller must keep the `TempDir` alive.
///
/// # Panics
///
/// Panics if the session directory or file cannot be created.
#[must_use]
#[inline]
#[allow(dead_code)]
pub fn make_session_for( claude_home : &std::path::Path, src_dir : &str, uuid : &str ) -> std::path::PathBuf
{
  let session_dir = claude_home.join( "projects" ).join( df( src_dir ) );
  std::fs::create_dir_all( &session_dir ).expect( "create session dir" );
  let file = session_dir.join( format!( "{uuid}.jsonl" ) );
  std::fs::write( &file, b"{}" ).expect( "write session jsonl" );
  file
}

/// Create `<claude_home>/projects/<df(src_dir)>/<uuid>.jsonl` shaped like a
/// zero-model-turn transcript (BUG-428): structurally qualifies as a resume
/// candidate under `most_recent_session_in_dir()`'s 4 checks (correct extension,
/// no `agent-` prefix, non-zero size, valid UTF-8 stem) but records no model
/// turns — the exact shape claude's real `--resume` logic rejects with
/// `"No conversation found to continue"`.
///
/// Fix(BUG-493): encoded-path counterpart to `make_session_for()`, for BUG-428
/// reproducer tests that specifically need zero-turn content rather than the
/// generic `{}` placeholder — see that function's doc comment for the isolation
/// contract (`CLAUDE_HOME` + `--from src_dir`).
///
/// Returns the `.jsonl` path. The caller must keep the `TempDir` alive.
///
/// # Panics
///
/// Panics if the session directory or file cannot be created.
#[must_use]
#[inline]
#[allow(dead_code)]
pub fn make_zero_turn_session_for( claude_home : &std::path::Path, src_dir : &str, uuid : &str ) -> std::path::PathBuf
{
  let session_dir = claude_home.join( "projects" ).join( df( src_dir ) );
  std::fs::create_dir_all( &session_dir ).expect( "create session dir" );
  let file = session_dir.join( format!( "{uuid}.jsonl" ) );
  std::fs::write( &file, b"{\"type\":\"system\",\"subtype\":\"init\"}\n" )
    .expect( "write zero-turn session jsonl" );
  file
}

/// Create a temp session directory with one dummy `.jsonl` file; returns `(dir, path_string)`.
///
/// The caller must keep the returned `TempDir` alive for the duration of the test —
/// the directory and its contents are deleted when the `TempDir` is dropped.
///
/// Fix(BUG-493): `--session-dir` is deprecated and inert — `session_exists()` no
/// longer scans it, so passing the returned path as `--session-dir` no longer
/// forces `-c` injection. New tests needing deterministic `-c` injection must use
/// `make_session_for()` (seeds the `--from`/cwd-derived storage location instead).
/// Kept only for existing callers not yet migrated; do not add new callers.
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

/// Create a temp session directory with one `.jsonl` file shaped like a zero-model-turn
/// transcript (BUG-428): structurally qualifies as a resume candidate under
/// `most_recent_session_in_dir()`'s 4 checks (correct extension, no `agent-` prefix,
/// non-zero size, valid UTF-8 stem) but records no model turns — the exact shape claude's
/// real `--resume` logic rejects with `"No conversation found to continue"`
/// (`contract/claude_code/docs/version/088_v2_1_187.md:19`). Content mirrors BUG-428's own
/// Minimum Reproducible Example (a lone `system`/`init` line, no `assistant` turn).
///
/// Distinct from `make_session_dir()`: that helper's placeholder `{}` content is never
/// semantically inspected by its callers (only `session_exists()`'s structural checks
/// matter to them) — this one specifically names and documents the zero-turn scenario for
/// BUG-428's own reproducer tests, so a future reader does not have to re-derive why the
/// content looks the way it does.
///
/// The caller must keep the returned `TempDir` alive for the duration of the test.
///
/// # Panics
///
/// Panics if the temp directory or the fixture file cannot be created.
#[must_use]
#[inline]
#[allow(dead_code)]
pub fn make_zero_turn_session_dir() -> ( tempfile::TempDir, String )
{
  let dir = tempfile::TempDir::new().expect( "failed to create temp session dir" );
  std::fs::write(
    dir.path().join( "00000000-0000-0000-0000-000000000001.jsonl" ),
    b"{\"type\":\"system\",\"subtype\":\"init\"}\n",
  )
  .expect( "failed to write zero-turn session file" );
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

/// Build a temp dir containing a `claude` symlink to the `fake_claude_control` ELF
/// binary (a compiled Cargo `[[bin]]` target, not a shell script — see that binary's
/// own module doc comment for why a real ELF is required for process discoverability)
/// and return `(TempDir, PATH value)` for use as `clr query`'s spawned session.
///
/// Unlike `fake_claude_binary_dir()` (which symlinks `/bin/sleep` for plain liveness
/// tests), this fixture speaks the bidirectional control-session wire protocol —
/// required for `clr query`'s daemon, which sends `control_request` envelopes over
/// stdin and expects matching `control_response` envelopes on stdout.
///
/// The caller must keep the `TempDir` alive for as long as any spawned session using it.
///
/// # Panics
///
/// Panics if the temp directory cannot be created or the symlink cannot be made.
#[cfg(unix)]
#[inline]
#[must_use]
#[allow(dead_code)]
pub fn fake_claude_control_binary_dir() -> ( tempfile::TempDir, String )
{
  let dir  = tempfile::TempDir::new().expect( "tmpdir" );
  let dest = dir.path().join( "claude" );
  std::os::unix::fs::symlink( env!( "CARGO_BIN_EXE_fake_claude_control" ), &dest )
    .expect( "symlink fake_claude_control as claude" );
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

/// Spawn a print-mode fake `claude` process that self-exits after `secs` seconds
/// (argv contains `--print`).
///
/// Uses `/bin/sh` with `arg0` set to `"claude"` and command string
/// `"sleep {secs}; :"`.  The `; :` compound prevents the shell from exec-replacing
/// itself with `sleep` (POSIX shells only exec the last *simple* command, not a
/// compound list).  The resulting `/proc/{pid}/cmdline` is:
///
/// ```text
/// ["claude", "-c", "sleep {secs}; :", "--print"]
/// ```
///
/// `classify_mode()` finds `"--print"` at `args[3]` and returns `"print"`.
/// The `"--print"` token is the script's `$0` (command name for error messages
/// in POSIX `-c` semantics) — it is NOT forwarded to `sleep` as an argument.
///
/// A short `secs` value (e.g. 3-5) lets a gate test observe natural release —
/// the process count drops once the kernel removes `/proc/{pid}` on exit, without
/// the caller needing to manually kill it mid-poll.
///
/// The caller must `kill()` + `wait()` the returned child to avoid leaks (safe to
/// call even after natural self-exit — `wait()` still reaps the zombie).
///
/// # Panics
///
/// Panics if the subprocess cannot be spawned.
#[ cfg( unix ) ]
#[ inline ]
#[ must_use ]
#[ allow( dead_code ) ]
pub fn spawn_print_claude_for( path_val : &str, secs : u64 ) -> std::process::Child
{
  assert_container();
  use std::os::unix::process::CommandExt as _;
  let child = std::process::Command::new( "/bin/sh" )
    .arg0( "claude" )
    .arg( "-c" )
    .arg( format!( "sleep {secs}; :" ) )   // "; :" prevents exec-into-sleep (compound list, not simple command)
    .arg( "--print" )       // argv[3] = $0 (script name); classify_mode() finds "--print" here
    .env( "PATH", path_val )
    .stdout( std::process::Stdio::null() )
    .stderr( std::process::Stdio::null() )
    .spawn()
    .expect( "spawn print-mode fake claude" );
  std::thread::sleep( core::time::Duration::from_millis( 200 ) );
  child
}

/// Spawn a print-mode fake `claude` process that sleeps 30 seconds.
///
/// Thin wrapper over [`spawn_print_claude_for`] for callers that only need a
/// long-lived print-mode process (no natural self-expiry within the test).
/// See [`spawn_print_claude_for`] for the full cmdline/classification contract.
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
  spawn_print_claude_for( path_val, 30 )
}

/// Spawn a query-mode fake `claude` process that sleeps 30 seconds (argv contains
/// `--input-format stream-json --output-format stream-json --verbose`, task 418).
///
/// Uses `/bin/sh` with `arg0` set to `"claude"`, same compound-list technique as
/// [`spawn_print_claude_for`] (prevents exec-replacing the shell with `sleep`).
/// The resulting `/proc/{pid}/cmdline` is:
///
/// ```text
/// ["claude", "-c", "sleep 30; :", "--input-format", "stream-json", "--output-format", "stream-json", "--verbose"]
/// ```
///
/// `classify_mode()` finds the adjacent `--input-format`/`stream-json` and
/// `--output-format`/`stream-json` pairs plus a standalone `--verbose` token and
/// returns `"query"` — the exact three-flag shape `clr query`'s dispatch spawns
/// the real subprocess with (see `query.rs`).
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
pub fn spawn_query_claude( path_val : &str ) -> std::process::Child
{
  assert_container();
  use std::os::unix::process::CommandExt as _;
  let child = std::process::Command::new( "/bin/sh" )
    .arg0( "claude" )
    .arg( "-c" )
    .arg( "sleep 30; :" )
    .arg( "--input-format" )
    .arg( "stream-json" )
    .arg( "--output-format" )
    .arg( "stream-json" )
    .arg( "--verbose" )
    .env( "PATH", path_val )
    .stdout( std::process::Stdio::null() )
    .stderr( std::process::Stdio::null() )
    .spawn()
    .expect( "spawn query-mode fake claude" );
  std::thread::sleep( core::time::Duration::from_millis( 200 ) );
  child
}

/// Build a proc isolation dir for `CLR_PROC_DIR`: one `/proc/{pid}` symlink per PID.
///
/// Each symlink points at the real `/proc/{pid}` directory.  When the process exits the
/// kernel removes `/proc/{pid}`, so a broken symlink (ENOENT) causes
/// `find_claude_processes()` to skip that entry — count falls to 0 and `clr ps`
/// terminates cleanly.
///
/// Without `CLR_PROC_DIR`, ambient `claude` processes visible in real `/proc` can cause
/// `RowBuilder::validate_row_length` to panic (exit 101) when tests run in parallel
/// with `ec11_gate_wait_event_emitted_when_gate_blocks` (which spawns its own ELF process).
/// Call `make_proc_dir` AFTER any post-spawn sleep so `/proc/{pid}` exists by the time
/// the dir is used.
///
/// # Panics
///
/// Panics if the temp directory or any symlink cannot be created.
#[ cfg( unix ) ]
#[ inline ]
#[ must_use ]
#[ allow( dead_code ) ]
pub fn make_proc_dir( pids : &[ u32 ] ) -> tempfile::TempDir
{
  let dir = tempfile::TempDir::new().expect( "make_proc_dir" );
  for pid in pids
  {
    std::os::unix::fs::symlink(
      format!( "/proc/{pid}" ),
      dir.path().join( pid.to_string() ),
    ).expect( "proc pid symlink" );
  }
  dir
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

/// Run `clr ps` with PATH env and `CLR_PROC_DIR` proc isolation; return the raw `Output`.
///
/// `proc_dir` must be the path of a dir produced by `make_proc_dir` for the PIDs of all
/// background processes spawned by the test.  Prevents ambient `claude` processes in real
/// `/proc` from reaching `RowBuilder::validate_row_length` and causing a panic (exit 101).
///
/// # Panics
///
/// Panics if the subprocess cannot be launched.
#[ cfg( unix ) ]
#[ inline ]
#[ must_use ]
#[ allow( dead_code ) ]
pub fn run_clr_ps_proc( path_val : &str, proc_dir : &str ) -> std::process::Output
{
  assert_container();
  let bin = env!( "CARGO_BIN_EXE_clr" );
  std::process::Command::new( bin )
    .arg( "ps" )
    .env( "PATH", path_val )
    .env( "CLR_PROC_DIR", proc_dir )
    .output()
    .expect( "run clr ps (proc-isolated)" )
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
    .env( "HOME", "/tmp/clr-isolated-home" )
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

/// Invoke `clr topic --dry-run` with extra args; assert exit 0 and return stdout as `String`.
///
/// Prepends `["topic", "--dry-run"]` to the given args, invokes the binary, asserts success,
/// and returns the captured stdout. Mirrors `run_ask_dry` exactly — `topic` and `ask` share
/// the same dry-run success contract; a deliberately-erroring case (e.g. an unknown flag)
/// must call `run_cli(&["topic", ...])` directly instead, since this helper asserts success.
///
/// # Panics
///
/// Panics if the subprocess cannot be launched or exits non-zero.
#[must_use]
#[inline]
#[allow(dead_code)]
pub fn run_topic_dry( extra_args : &[ &str ] ) -> String
{
  assert_container();
  let bin = env!( "CARGO_BIN_EXE_clr" );
  let mut args = vec![ "topic", "--dry-run" ];
  args.extend_from_slice( extra_args );
  let out = Command::new( bin )
    .args( &args )
    .env( "HOME", "/tmp/clr-isolated-home" )
    .output()
    .expect( "failed to invoke clr binary" );
  assert!(
    out.status.success(),
    "clr topic --dry-run failed (exit {}): {}",
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
    .env( "HOME", "/tmp/clr-isolated-home" )
    .env_remove( "CLR_DIR" )
    .env_remove( "CLR_SESSION_DIR" )
    .env_remove( "CLR_NO_COMPACT_WINDOW" )
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

/// Invoke `clr` binary with `args`, a custom `PATH`, and extra environment variables.
///
/// Mirrors `run_with_path` but additionally injects `env` pairs via `Command::envs()`
/// and scrubs `CLR_DIR`/`CLR_SESSION_DIR`/`CLR_FROM` — used by tests that pin session
/// storage deterministically by setting `CLAUDE_HOME` alongside a fake `claude` binary
/// on `PATH` (see `make_session_for`), where an ambient override var would otherwise
/// change which storage `session_exists()` scans.
///
/// # Panics
///
/// Panics if the `clr` binary cannot be launched.
#[must_use]
#[inline]
#[allow(dead_code)]
pub fn run_with_path_env( args : &[ &str ], path : &str, env : &[ ( &str, &str ) ] ) -> std::process::Output
{
  assert_container();
  let bin = env!( "CARGO_BIN_EXE_clr" );
  Command::new( bin )
    .args( args )
    .env( "PATH", path )
    .envs( env.iter().copied() )
    .env_remove( "CLR_DIR" )
    .env_remove( "CLR_SESSION_DIR" )
    .env_remove( "CLR_FROM" )
    .output()
    .expect( "Failed to invoke clr binary" )
}

/// Invoke `clr` binary with `args`, a custom `PATH`, and piped `stdin` content; return raw `Output`.
///
/// Mirrors `run_with_path` but additionally writes `stdin` to the child's stdin pipe before
/// collecting output — reproduces a shell pipeline (`cat notes.txt | clr run "prompt"`) under
/// `Command`'s piped-stdin API, since the test harness's own inherited stdin cannot be
/// repointed at literal bytes from within a `#[test]` function.
///
/// `stdin` must stay well under the OS pipe buffer size (commonly 64KiB on Linux) — the
/// write happens before `wait_with_output()` drains the child's stdout/stderr pipes, so a
/// larger payload risks the classic parent-writes-while-child-blocks-on-stdout deadlock.
/// Fine for the small fixed strings used in stdin-forwarding regression tests.
///
/// # Panics
///
/// Panics if the `clr` binary cannot be spawned, its stdin pipe cannot be written to, or the
/// process cannot be waited on.
#[must_use]
#[inline]
#[allow(dead_code)]
pub fn run_with_path_stdin( args : &[ &str ], path : &str, stdin : &[ u8 ] ) -> std::process::Output
{
  use std::io::Write as _;
  assert_container();
  let bin = env!( "CARGO_BIN_EXE_clr" );
  let mut child = Command::new( bin )
    .args( args )
    .env( "PATH", path )
    .stdin( std::process::Stdio::piped() )
    .stdout( std::process::Stdio::piped() )
    .stderr( std::process::Stdio::piped() )
    .spawn()
    .expect( "Failed to spawn clr binary" );
  child.stdin.take().expect( "child stdin handle" ).write_all( stdin ).expect( "write stdin" );
  child.wait_with_output().expect( "Failed to wait on clr binary" )
}

/// Invoke `clr` binary with `args`, a custom `PATH`, and `CLR_PROC_DIR` proc isolation.
///
/// Mirrors `run_clr_ps_proc` — combines `run_with_path`'s PATH injection with a
/// `CLR_PROC_DIR` override so `find_claude_processes()` sees only the isolated
/// fixture built by `make_proc_dir`, never the real host `/proc`.  Required for
/// real (non-dry-run) print-mode invocations, which reach `wait_for_session_slot()`
/// in `src/cli/gate.rs` — that function calls `find_claude_processes()` directly,
/// and without `CLR_PROC_DIR` it races against ambient `claude` processes from
/// concurrent test binaries under nextest's parallel execution.
///
/// Also isolates `CLR_GATE_DIR` to a fresh, function-local `TempDir` — without it,
/// `acquire_slot()` (BUG-387's slot-reservation scheme) writes into the shared
/// system-default gate dir, where a low process count (e.g. 0 from an empty
/// `proc_dir`) can collide with another concurrently-running test claiming the
/// same slot index in that same shared directory.
///
/// `proc_dir` must be the path of a dir produced by `make_proc_dir` for the PIDs of
/// any background processes spawned by the test (empty slice is fine when the test
/// expects zero visible Claude processes).
///
/// # Panics
///
/// Panics if the `clr` binary cannot be launched.
#[ cfg( unix ) ]
#[must_use]
#[inline]
#[allow(dead_code)]
pub fn run_with_path_proc( args : &[ &str ], path : &str, proc_dir : &str ) -> std::process::Output
{
  assert_container();
  let bin = env!( "CARGO_BIN_EXE_clr" );
  let gate_dir = tempfile::TempDir::new().expect( "gate dir" );
  Command::new( bin )
    .args( args )
    .env( "PATH", path )
    .env( "CLR_PROC_DIR", proc_dir )
    .env( "CLR_GATE_DIR", gate_dir.path() )
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

/// Invoke `clr isolated <args>` and return raw output.
///
/// Prepends the `"isolated"` subcommand to the caller-supplied arguments and
/// delegates to `run_cli`.  Shared by `isolated_test`, `isolated_plan034_test`,
/// and `isolated_plan035_test` to avoid duplicating the subcommand prefix logic.
#[ must_use ]
#[ inline ]
#[ allow( dead_code ) ]
pub fn run_isolated( args : &[ &str ] ) -> std::process::Output
{
  let mut full = vec![ "isolated" ];
  full.extend_from_slice( args );
  run_cli( &full )
}

/// Poll `child` with `try_wait()` until it exits or `deadline` passes, sleeping
/// 50ms between checks. Never blocks past `deadline` — unlike `.output()`
/// (blocks until natural exit), this lets a test fail fast when a gate-timing
/// override (env var, CLI flag, or config-file key) is not actually taking
/// effect, instead of hanging for however long the real production default
/// (e.g. 30s x 1000 attempts) would otherwise take. Shared by
/// `concurrency_gate_test` and `config_file_test` — both prove a gate-timing
/// override changes real poll behavior via the same bounded-exhaustion pattern.
#[ inline ]
#[ allow( dead_code ) ]
pub fn wait_bounded( child : &mut std::process::Child, deadline : std::time::Instant ) -> Option< std::process::ExitStatus >
{
  while std::time::Instant::now() < deadline
  {
    if let Ok( Some( status ) ) = child.try_wait() { return Some( status ); }
    std::thread::sleep( core::time::Duration::from_millis( 50 ) );
  }
  None
}

/// Poll `paths` every 50ms until any file's content contains `marker`, or
/// `deadline` passes, whichever comes first. Returns `true` once observed,
/// `false` on timeout.
///
/// Used to observe a still-racing subprocess's incremental, file-redirected
/// stderr for a specific message substring instead of guessing a fixed sleep
/// duration long enough for the message to have appeared (Fix(BUG-508): a
/// fixed sleep has no adaptive margin — under genuine host CPU contention a
/// freshly-spawned process can fail to be scheduled enough to print within a
/// guessed window, producing a false-red failure). Shared by
/// `concurrency_gate_ext_test` (T15, T16).
#[ must_use ]
#[ inline ]
#[ allow( dead_code ) ]
pub fn wait_for_marker_in_files( paths : &[ &std::path::Path ], marker : &str, deadline : std::time::Instant ) -> bool
{
  while std::time::Instant::now() < deadline
  {
    for path in paths
    {
      if let Ok( content ) = std::fs::read_to_string( path )
      {
        if content.contains( marker ) { return true; }
      }
    }
    std::thread::sleep( core::time::Duration::from_millis( 50 ) );
  }
  false
}

/// Scan `dir` and return paths to all `*.jsonl` files found (non-recursive).
/// Shared by `journal_integration_test` and `journal_integration_ext_test`.
#[ must_use ]
#[ inline ]
#[ allow( dead_code ) ]
pub fn find_jsonl_files( dir : &std::path::Path ) -> Vec< std::path::PathBuf >
{
  let Ok( rd ) = std::fs::read_dir( dir ) else { return Vec::new() };
  rd.filter_map( core::result::Result::ok )
    .map( | e | e.path() )
    .filter( | p | p.extension().is_some_and( | x | x == "jsonl" ) )
    .collect()
}

/// Read all content from all `*.jsonl` files in `dir`; return concatenated string.
/// Shared by `journal_integration_test` and `journal_integration_ext_test`.
#[ must_use ]
#[ inline ]
#[ allow( dead_code ) ]
pub fn read_journal_content( dir : &std::path::Path ) -> String
{
  find_jsonl_files( dir )
    .iter()
    .map( | p | std::fs::read_to_string( p ).unwrap_or_default() )
    .collect()
}

/// Invoke `clr` in print-mode with a fast-exit fake claude and extra args.
///
/// Clears `CLR_JOURNAL`, `CLR_JOURNAL_DIR`, `_CLR_DEFAULT_TIMEOUT`, and `CLR_TIMEOUT`
/// from the environment, then applies `extra_envs` on top.  Uses `--max-sessions 0` to
/// bypass the gate.  Appends `"x"` as the positional message.
/// Shared by `journal_integration_test` and `journal_integration_ext_test`.
///
/// # Panics
///
/// Panics if the compiled `clr` binary cannot be invoked as a subprocess.
#[ must_use ]
#[ inline ]
#[ allow( dead_code ) ]
pub fn run_with_journal
(
  extra_args : &[ &str ],
  extra_envs : &[ ( &str, &str ) ],
  fake_body  : &str,
) -> ( std::process::Output, tempfile::TempDir )
{
  let ( dir, path ) = fake_claude_dir( fake_body );
  let bin = env!( "CARGO_BIN_EXE_clr" );
  let mut args : Vec< &str > = vec![ "-p", "--max-sessions", "0" ];
  args.extend_from_slice( extra_args );
  args.push( "x" );
  let out = Command::new( bin )
    .args( &args )
    .env( "PATH", &path )
    .env_remove( "CLR_JOURNAL" )
    .env_remove( "CLR_JOURNAL_DIR" )
    .env_remove( "CLR_TIMEOUT" )
    .env_remove( "_CLR_DEFAULT_TIMEOUT" )
    .envs( extra_envs.iter().copied() )
    .output()
    .expect( "failed to invoke clr binary" );
  ( out, dir )
}

/// Compile a tiny real ELF binary named `claude` that ignores all argv and sleeps
/// for `sleep_secs` seconds before exiting.
///
/// Needed because neither existing fake-`claude` fixture fits this test: a
/// shebang shell script (`fake_claude_dir`) shows its *interpreter* as argv[0]
/// in `/proc/{pid}/cmdline`, making it invisible to `find_claude_processes()`'s
/// basename check; and `/bin/sleep` (`fake_claude_binary_dir`) errors out
/// immediately on the non-numeric flags `clr` itself forwards to the dispatched
/// `claude` process (e.g. `-p`). This binary is a real ELF (so the basename
/// check passes) that never inspects `std::env::args()` at all (so it tolerates
/// whatever `clr` forwards) and blocks for a fixed duration (so concurrently
/// racing invocations have an observable overlap window).
///
/// Returns `(TempDir, path_val)` — `path_val` prepends the dir to `$PATH`,
/// mirroring `fake_claude_binary_dir()`'s contract.
///
/// Shared by `concurrency_gate_test` and `concurrency_gate_ext2_test`.
///
/// # Panics
/// Panics if `rustc` is unavailable on `$PATH` or compilation fails.
#[ must_use ]
#[ inline ]
#[ allow( dead_code ) ]
pub fn build_argv_tolerant_sleeper( sleep_secs : u64 ) -> ( tempfile::TempDir, String )
{
  let dir = tempfile::TempDir::new().expect( "tmpdir" );
  let src = dir.path().join( "sleeper.rs" );
  std::fs::write(
    &src,
    format!( "fn main() {{ std::thread::sleep(std::time::Duration::from_secs({sleep_secs})); }}" ),
  ).expect( "write sleeper source" );
  let bin = dir.path().join( "claude" );
  let status = Command::new( "rustc" )
    .arg( "-O" )
    .arg( "-o" ).arg( &bin )
    .arg( &src )
    .stdout( std::process::Stdio::null() )
    .stderr( std::process::Stdio::null() )
    .status()
    .expect( "invoke rustc for T08 fixture" );
  assert!( status.success(), "T08 fixture: rustc failed to compile the argv-tolerant sleeper" );
  let path_val = format!( "{}:{}", dir.path().display(), std::env::var( "PATH" ).unwrap_or_default() );
  ( dir, path_val )
}

/// Extract the `pid` field from a slot-reservation file's JSON content
/// (`{"pid":N,"since":M}` or, since BUG-488, `{"pid":N,"since":M,"starttime":S}`),
/// written by `claim_slot_file()` in `src/cli/gate_slot.rs`. The scan terminates at the
/// first `,` or `}` after the `pid` value, so the optional trailing `starttime`
/// field never affects the result.
///
/// Shared by `concurrency_gate_ext_test`, `concurrency_gate_ext2_test`, and
/// `concurrency_gate_test`.
#[ must_use ]
#[ inline ]
#[ allow( dead_code ) ]
pub fn slot_owner_pid( content : &str ) -> Option< u32 >
{
  let marker = "\"pid\":";
  let start  = content.find( marker )? + marker.len();
  let rest   = &content[ start.. ];
  let end    = rest.find( [ ',', '}' ] )?;
  rest[ ..end ].trim().parse().ok()
}

/// Spawn a parked helper thread and return its kernel thread ID (TID), the
/// `Sender` whose drop releases the park, and the thread's `JoinHandle`.
///
/// The returned TID is a live NON-LEADER thread ID of this test process:
/// `/proc/{tid}/stat` is readable via direct lookup with state ∉ {`Z`}, yet
/// `/proc/{tid}/status` reports `Tgid != tid` and `ls /proc` never lists it.
/// That is exactly the PID-number occupancy shape that masked a dead gate
/// waiter in the wild (BUG-488) — using the test's own thread makes the
/// collision deterministic instead of waiting for a host PID wrap.
///
/// Shared by `concurrency_gate_test` (T42) and `ps_command_test` (IT-47).
///
/// # Panics
///
/// Panics if the helper thread cannot read its own TID via `/proc/thread-self`.
#[ cfg( unix ) ]
#[ must_use ]
#[ inline ]
#[ allow( dead_code ) ]
pub fn spawn_parked_helper_thread() -> ( u32, std::sync::mpsc::Sender< () >, std::thread::JoinHandle< () > )
{
  let ( tid_send, tid_recv )   = std::sync::mpsc::channel();
  let ( park_send, park_recv ) = std::sync::mpsc::channel::< () >();
  let handle = std::thread::spawn( move ||
  {
    // /proc/thread-self (Linux 3.17+) links to <pid>/task/<tid> — its final
    // component is this thread's own TID, with no libc gettid() dependency.
    let tid : u32 = std::fs::read_link( "/proc/thread-self" )
      .ok()
      .and_then( | p | p.file_name().and_then( std::ffi::OsStr::to_str ).and_then( | s | s.parse().ok() ) )
      .expect( "read own TID via /proc/thread-self" );
    tid_send.send( tid ).expect( "send TID to test" );
    let _ = park_recv.recv(); // park until the test drops its Sender
  } );
  let tid = tid_recv.recv().expect( "receive helper thread TID" );
  ( tid, park_send, handle )
}
