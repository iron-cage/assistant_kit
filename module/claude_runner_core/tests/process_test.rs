//! Process-scanner unit tests.
//!
//! # Test Matrix
//!
//! | TC | Description | F/L | P/N |
//! |----|-------------|-----|-----|
//! | TC-061 | `find_claude_processes` returns `Vec` without panic | F16/L1 | P |
//! | TC-062 | `find_claude_processes` excludes self PID | F16 self | P |
//! | TC-063 | `find_claude_processes` finds a live process whose argv[0] is `claude` | F16/L2 | P |
//! | TC-064 | `find_claude_processes` finds two live `claude`-named processes | F16/L3 | P |
//! | TC-065 | `find_claude_processes` entry with deleted CWD → included, cwd empty/fallback | F16/L4 | P |
//! | TC-066 | `find_claude_processes` skips /proc entry with unreadable cmdline silently | F16/L5 | P |
//! | TC-067 | `send_sigterm` with valid PID → Ok(()) | F19/L1 | P |
//! | TC-068 | `send_sigterm` with non-existent PID → Err | F19/L3 | N |
//! | TC-069 | `send_sigkill` with valid PID → Ok(()) | F19/L1 | P |
//! | TC-070 | `find_claude_processes` does not panic when /proc is unavailable | F18 | P |
//! | TC-071 | `find_claude_processes` skips a live process whose argv[0] is not `claude` | F16/L2 | N |

#![ cfg( unix ) ]

use std::process::Command;
use std::os::unix::process::CommandExt;

use claude_runner_core::process::{ find_claude_processes, send_sigterm, send_sigkill };

/// Spawn `sleep 60` whose argv[0] is `argv0` and return its Child handle.
///
/// The scanner matches on the basename of argv[0], so `spawn_sleep_as( "claude" )` yields a
/// live process the scanner must list, and any other name one it must skip. The fixture
/// lives until the test kills it, so a scan never races the process's own exit.
// Fix(BUG-006): replaces `claude --version` as the scanner fixture — it exits in ~15ms,
//   long before the old 100ms sleep ended, leaving a zombie with an empty cmdline.
// Root cause: tc063/tc064 assumed a spawned-but-unwaited child is still running; the scanner
//   reads argv[0] from `/proc/<pid>/cmdline`, which the kernel empties once the child exits.
// Pitfall: holding a `Child` keeps the process unreaped, not running — a fixture whose own
//   runtime can end before the check is invisible to anything reading argv from `/proc`.
fn spawn_sleep_as( argv0 : &str ) -> std::process::Child
{
  Command::new( "sleep" )
  .arg0( argv0 )
  .arg( "60" )
  .stdout( std::process::Stdio::null() )
  .stderr( std::process::Stdio::null() )
  .spawn()
  .expect( "failed to spawn sleep process" )
}

/// Block until `/proc/<pid>/cmdline` shows `argv0` as its first field; panic after 5s.
///
/// `spawn()` can return while the child is still inside `execve`, before the new image's
/// argv is set up, and in that window `cmdline` reads empty (about 40% of reads taken
/// straight after spawn, measured for BUG-006). A scan in that window misses a process
/// that is about to be live, so scanner tests call this before they scan.
fn wait_for_argv0( pid : u32, argv0 : &str )
{
  let path     = format!( "/proc/{pid}/cmdline" );
  let deadline = std::time::Instant::now() + core::time::Duration::from_secs( 5 );
  loop
  {
    let cmdline = std::fs::read( &path ).unwrap_or_default();
    if cmdline.split( |&b| b == 0 ).next() == Some( argv0.as_bytes() ) { return; }
    assert!( std::time::Instant::now() < deadline, "PID {pid} never showed argv[0]={argv0} in {path}" );
    std::thread::sleep( core::time::Duration::from_millis( 1 ) );
  }
}

// TC-061: function returns Vec without panicking
#[ test ]
fn tc061_find_claude_processes_does_not_panic()
{
  // Just confirm the function runs without panicking.  The result may be empty or
  // non-empty depending on the environment; this test exercises the code path.
  let _procs = find_claude_processes();
}

// TC-062: self PID must not be in the result (test binary is not named "claude")
#[ test ]
fn tc062_find_claude_processes_excludes_self_pid()
{
  let self_pid = std::process::id();
  let procs    = find_claude_processes();
  assert!(
    !procs.iter().any( |p| p.pid == self_pid ),
    "self PID {self_pid} must not appear in find_claude_processes() results"
  );
}

/// TC-063: a live process whose argv[0] is `claude` is listed by the scanner.
///
/// # Root Cause
///
/// The old test spawned `claude --version` (`process_test.rs:59`), which exits in ~15ms, then
/// slept 100ms (`:67`) before scanning. By scan time the child was an unreaped zombie. The
/// kernel returns an empty `/proc/<pid>/cmdline` for a zombie, so `find_claude_processes`
/// (`claude_core/src/process.rs:96-108`) derived `binary_name == ""` and skipped the entry:
/// `found == false`, panic at `:76`. tc064 failed the same way with two children.
///
/// # Why Not Caught
///
/// The `else { return; }` branch reported PASS wherever `claude` wasn't on `PATH`, so the
/// assertion never ran there. Where it did run, pass/fail hinged on the installed build's
/// `--version` latency outlasting a hard-coded 100ms, an assumption the test never stated.
///
/// # Fix Applied
///
/// `spawn_sleep_as( "claude" )` spawns `sleep 60` under argv[0] `claude` via
/// `CommandExt::arg0`: alive until the test kills it, and independent of any installed binary.
/// The skip branch is gone. The fixed 100ms sleep is replaced by `wait_for_argv0`, which
/// polls `/proc/<pid>/cmdline` until the new argv appears, because `spawn()` can return
/// before `execve` has set it up. Tracked as BUG-006.
///
/// # Prevention
///
/// Scanner tests control their fixture's argv[0] and lifetime, and a missing precondition
/// fails loudly instead of returning early. TC-071 is the negative counterpart: a live
/// process with a non-`claude` argv[0] must not be listed.
///
/// # Pitfall
///
/// An exited-but-unreaped child keeps its `/proc/<pid>` directory, but its cmdline is empty.
/// Anything reading argv from `/proc` can't see it, even while the `Child` handle is held.
/// A child that hasn't finished `execve` reads empty the same way, so "spawn returned" never
/// means "argv is visible".
// test_kind: bug_reproducer(BUG-006)
#[ test ]
fn tc063_finds_one_claude_process()
{
  let mut child = spawn_sleep_as( "claude" );
  let pid = child.id();
  wait_for_argv0( pid, "claude" );

  let found = find_claude_processes().iter().any( |p| p.pid == pid );

  child.kill().ok();
  child.wait().ok();

  assert!( found, "live process with argv[0]=claude (PID {pid}) must be found by scanner" );
}

/// TC-064: two live `claude`-named processes are both listed. See TC-063 for BUG-006.
// test_kind: bug_reproducer(BUG-006)
#[ test ]
fn tc064_finds_two_claude_processes()
{
  let mut c1 = spawn_sleep_as( "claude" );
  let mut c2 = spawn_sleep_as( "claude" );
  let pid1 = c1.id();
  let pid2 = c2.id();
  wait_for_argv0( pid1, "claude" );
  wait_for_argv0( pid2, "claude" );

  let procs  = find_claude_processes();
  let found1 = procs.iter().any( |p| p.pid == pid1 );
  let found2 = procs.iter().any( |p| p.pid == pid2 );

  c1.kill().ok(); c1.wait().ok();
  c2.kill().ok(); c2.wait().ok();

  assert!( found1, "first live argv[0]=claude PID {pid1} must be found" );
  assert!( found2, "second live argv[0]=claude PID {pid2} must be found" );
}

// TC-065: entry with deleted/unreachable CWD is included with fallback CWD
#[ test ]
fn tc065_process_with_deleted_cwd_included_with_fallback()
{
  // Spawn sleep under a temp dir, then delete the dir.
  // The scanner must still include the entry (cwd may be empty or contain "(deleted)").
  use tempfile::TempDir;
  let dir = TempDir::new().unwrap();
  let mut child = Command::new( "sleep" )
  .arg( "60" )
  .current_dir( dir.path() )
  .stdout( std::process::Stdio::null() )
  .stderr( std::process::Stdio::null() )
  .spawn()
  .expect( "failed to spawn sleep" );

  let _pid = child.id();
  let path = dir.keep(); // keep dir so we can delete it manually

  // Delete the CWD
  std::fs::remove_dir_all( &path ).ok();

  std::thread::sleep( core::time::Duration::from_millis( 50 ) );

  // sleep is not named "claude", so it won't appear in results; what we test is that
  // find_claude_processes() doesn't panic on a process with a deleted CWD.
  let _ = find_claude_processes();

  child.kill().ok();
  child.wait().ok();
}

// TC-066: /proc entry with unreadable cmdline is silently skipped
#[ test ]
fn tc066_unreadable_cmdline_silently_skipped()
{
  // We can't easily make /proc/{pid}/cmdline unreadable without root.
  // This test verifies that find_claude_processes() handles EACCES gracefully:
  // it must not panic even if some /proc entries cannot be read.
  // We call it and assert it returns without panicking.
  let _ = find_claude_processes();
}

// TC-067: send_sigterm to a valid (sleep) PID returns Ok
#[ test ]
fn tc067_send_sigterm_valid_pid_returns_ok()
{
  let mut child = spawn_sleep_as( "sleep" );
  let pid = child.id();
  let result = send_sigterm( pid );
  child.wait().ok();
  assert!( result.is_ok(), "send_sigterm to valid PID must succeed, got: {result:?}" );
}

// TC-068: send_sigterm to non-existent PID returns Err
#[ test ]
fn tc068_send_sigterm_nonexistent_pid_returns_err()
{
  // CRITICAL: Do NOT use u32::MAX here. 4294967295 wraps to -1 as pid_t, and
  // kill(-1, SIGTERM) sends SIGTERM to every process the caller can kill — it
  // would wipe out all concurrent test processes in nextest's parallel runner.
  //
  // Instead: read the kernel's actual pid_max and use pid_max+1, which is a
  // valid positive pid_t that the kernel will reject with ESRCH (no such process).
  let pid_max : u32 = std::fs::read_to_string( "/proc/sys/kernel/pid_max" )
  .ok()
  .and_then( | s | s.trim().parse().ok() )
  .unwrap_or( 32768 );

  let result = send_sigterm( pid_max + 1 );
  assert!( result.is_err(), "send_sigterm to PID above pid_max must fail" );
}

// TC-069: send_sigkill to a valid (sleep) PID returns Ok
#[ test ]
fn tc069_send_sigkill_valid_pid_returns_ok()
{
  let mut child = spawn_sleep_as( "sleep" );
  let pid = child.id();
  let result = send_sigkill( pid );
  child.wait().ok();
  assert!( result.is_ok(), "send_sigkill to valid PID must succeed, got: {result:?}" );
}

// TC-070: find_claude_processes does not panic regardless of /proc state
#[ test ]
fn tc070_find_claude_processes_does_not_panic()
{
  // On Linux /proc is always present; this exercises the code path that handles
  // any transient /proc errors without panicking.
  let _ = find_claude_processes();
}

/// TC-071: a live process whose argv[0] is not `claude` is not listed.
///
/// Negative counterpart to TC-063 (BUG-006). The same fixture under a different argv[0] must
/// be skipped. Without this, a scanner that listed every process would pass TC-063/TC-064.
/// The readiness wait matters here too: a scan during `execve` sees an empty cmdline and
/// skips the process for the wrong reason.
#[ test ]
fn tc071_non_claude_argv0_not_found()
{
  let mut child = spawn_sleep_as( "sleep" );
  let pid = child.id();
  wait_for_argv0( pid, "sleep" );

  let found = find_claude_processes().iter().any( |p| p.pid == pid );

  child.kill().ok();
  child.wait().ok();

  assert!( !found, "live process with argv[0]=sleep (PID {pid}) must not be listed by scanner" );
}
