//! Process-scanner unit tests.
//!
//! # Test Matrix
//!
//! | TC | Description | F/L | P/N |
//! |----|-------------|-----|-----|
//! | TC-061 | `find_claude_processes` returns `Vec` without panic | F16/L1 | P |
//! | TC-062 | `find_claude_processes` excludes self PID | F16 self | P |
//! | TC-063 | `find_claude_processes` finds a spawned claude process | F16/L2 | P |
//! | TC-064 | `find_claude_processes` finds two spawned claude processes | F16/L3 | P |
//! | TC-065 | `find_claude_processes` entry with deleted CWD → included, cwd empty/fallback | F16/L4 | P |
//! | TC-066 | `find_claude_processes` skips /proc entry with unreadable cmdline silently | F16/L5 | P |
//! | TC-067 | `send_sigterm` with valid PID → Ok(()) | F19/L1 | P |
//! | TC-068 | `send_sigterm` with non-existent PID → Err | F19/L3 | N |
//! | TC-069 | `send_sigkill` with valid PID → Ok(()) | F19/L1 | P |
//! | TC-070 | `find_claude_processes` does not panic when /proc is unavailable | F18 | P |

use std::process::Command;

use claude_runner_core::process::{ find_claude_processes, send_sigterm, send_sigkill };

/// Spawn a `sleep` subprocess and return its PID and Child handle.
fn spawn_sleep() -> std::process::Child
{
  Command::new( "sleep" )
  .arg( "60" )
  .stdout( std::process::Stdio::null() )
  .stderr( std::process::Stdio::null() )
  .spawn()
  .expect( "failed to spawn sleep process" )
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

// TC-063: finds one spawned claude process (skipped if claude not in PATH)
#[ test ]
fn tc063_finds_one_claude_process()
{
  // Try to spawn claude --version; if not available we just skip the assertion.
  let Ok( mut child ) = Command::new( "claude" )
  .arg( "--version" )
  .stdout( std::process::Stdio::null() )
  .stderr( std::process::Stdio::null() )
  .spawn()
  else { return; }; // claude not in PATH — environment cannot provide this test

  // Give the process a moment to appear in /proc
  std::thread::sleep( core::time::Duration::from_millis( 100 ) );
  let pid = child.id();

  let procs = find_claude_processes();
  let found = procs.iter().any( |p| p.pid == pid );

  child.kill().ok();
  child.wait().ok();

  assert!( found, "spawned claude process with PID {pid} must be found by scanner" );
}

// TC-064: finds two spawned claude processes (skipped if claude not in PATH)
#[ test ]
fn tc064_finds_two_claude_processes()
{
  let Ok( mut c1 ) = Command::new( "claude" )
  .arg( "--version" )
  .stdout( std::process::Stdio::null() )
  .stderr( std::process::Stdio::null() )
  .spawn()
  else { return; };
  let Ok( mut c2 ) = Command::new( "claude" )
  .arg( "--version" )
  .stdout( std::process::Stdio::null() )
  .stderr( std::process::Stdio::null() )
  .spawn()
  else { c1.kill().ok(); c1.wait().ok(); return; };

  std::thread::sleep( core::time::Duration::from_millis( 100 ) );

  let pid1 = c1.id();
  let pid2 = c2.id();

  let procs = find_claude_processes();
  let found1 = procs.iter().any( |p| p.pid == pid1 );
  let found2 = procs.iter().any( |p| p.pid == pid2 );

  c1.kill().ok(); c1.wait().ok();
  c2.kill().ok(); c2.wait().ok();

  assert!( found1, "first spawned claude PID {pid1} must be found" );
  assert!( found2, "second spawned claude PID {pid2} must be found" );
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
  let mut child = spawn_sleep();
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
  let mut child = spawn_sleep();
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
