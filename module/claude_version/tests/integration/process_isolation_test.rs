//! Process isolation tests.
//!
//! Verifies that `.version.guard` does not deliver kill signals to running
//! Claude processes. The guard's only interaction with running processes is
//! via `hot_swap_binary()` (unlink); `send_kill_signals()` is reachable only
//! through the explicit `.processes.kill` user command.
//!
//! # Test Matrix
//!
//! | # | Scenario | Expected |
//! |---|----------|----------|
//! | IT-1 | `.version.guard version::stable` with live claude-named process | Dummy process survives guard run (no kill signals) |

// IT-1: `.version.guard` does not kill running processes.
//
// ## Root Cause
//
// `send_kill_signals()` and `processes_kill_routine()` are isolated to the
// `.processes.kill` explicit user command. No automatic path (guard, install,
// daemon) reaches them. This is a precautionary regression tripwire — the
// isolation invariant is expected to hold without any code change; the task
// formalizes it as a tested and documented guarantee.
//
// ## Why Not Caught
//
// No prior integration test verified that a process running during a guard
// invocation survived the guard exit path. The invariant was correct but
// undocumented and untested.
//
// ## Fix Applied
//
// Isolation doc comments added to `send_kill_signals()` and
// `processes_kill_routine()` in `src/commands/process.rs` naming the
// exact caller/callee relationship. This test creates a structural
// regression tripwire for the guard execution path.
//
// ## Prevention
//
// This test catches any future wiring of kill functions from guard/install
// paths. `kill -0 PID` exits 0 if alive, non-0 if dead — a dead process
// here means a kill signal was sent, causing an immediate, loud test failure.
//
// ## Pitfall
//
// Without the `~/.local/bin/claude` symlink pointing to a semver-named file,
// `get_installed_version()` falls back to running `claude --version` via PATH,
// which finds the sleep dummy and either hangs or returns garbage — causing
// the guard to call `perform_install` and trigger a network request.
#[ test ]
fn version_guard_does_not_kill()
{
  // ── Process-detection setup ─────────────────────────────────────────────
  // Create a temp directory with `claude` → /usr/bin/sleep.  Spawning
  // `Command::new("claude")` with this dir first in PATH creates a process
  // whose /proc/{pid}/cmdline argv[0] is "claude" — exactly what
  // find_claude_processes() matches.
  let tmp_bin = tempfile::TempDir::new().unwrap();
  let tmp_bin_dir = tmp_bin.path();

  let sleep_bin = if std::path::Path::new( "/usr/bin/sleep" ).exists()
  {
    "/usr/bin/sleep"
  }
  else
  {
    "/bin/sleep"
  };
  std::os::unix::fs::symlink( sleep_bin, tmp_bin_dir.join( "claude" ) ).unwrap();

  let orig_path = std::env::var( "PATH" ).unwrap_or_default();
  let aug_path  = format!( "{}:{}", tmp_bin_dir.display(), orig_path );

  // Spawn a dummy claude process (sleep 300) that find_claude_processes() will detect.
  let mut dummy = std::process::Command::new( "claude" )
    .arg( "300" )
    .env( "PATH", &aug_path )
    .stdout( std::process::Stdio::null() )
    .stderr( std::process::Stdio::null() )
    .spawn()
    .expect( "failed to spawn dummy claude process" );
  let dummy_pid = dummy.id();

  // ── Version-check setup ─────────────────────────────────────────────────
  // Resolve stable alias to its pinned semver (compile-time constant; stays
  // in sync with VERSION_ALIASES automatically — no hardcoded string needed).
  let stable_ver = claude_version_core::version::VERSION_ALIASES
    .iter()
    .find( | a | a.name == "stable" )
    .map( | a | a.value )
    .expect( "stable alias not found in VERSION_ALIASES" );

  // Create an isolated HOME with ~/.local/bin/claude → stable_ver file.
  // get_version_from_symlink() reads $HOME/.local/bin/claude symlink and
  // extracts the version from the target filename — this makes
  // get_installed_version() return stable_ver immediately, so the guard
  // finds "already installed" and short-circuits without calling
  // perform_install (which would trigger a network request).
  let temp_home = tempfile::TempDir::new().unwrap();
  let local_bin = temp_home.path().join( ".local" ).join( "bin" );
  std::fs::create_dir_all( &local_bin ).unwrap();

  // Version-named target file (content irrelevant; only filename matters).
  std::fs::write( local_bin.join( stable_ver ), "" ).unwrap();
  // Relative symlink: .local/bin/claude → 2.1.78 (or whichever stable_ver is).
  std::os::unix::fs::symlink( stable_ver, local_bin.join( "claude" ) ).unwrap();

  // Empty settings.json so guard does not fall back to real ~/.claude.
  let claude_dir = temp_home.path().join( ".claude" );
  std::fs::create_dir_all( &claude_dir ).unwrap();
  std::fs::write( claude_dir.join( "settings.json" ), "{}" ).unwrap();

  let home_str = temp_home.path().to_str().unwrap();

  // ── Run the guard ──────────────────────────────────────────────────────
  // Invoke `clv .version.guard version::stable` as a real subprocess.
  // Guard resolves stable → stable_ver, finds it already installed via
  // symlink, and exits 0 without calling perform_install or send_kill_signals.
  let status = std::process::Command::new( env!( "CARGO_BIN_EXE_clv" ) )
    .args( [ ".version.guard", "version::stable" ] )
    .env( "HOME", home_str )
    .env( "PATH", &aug_path )
    .stdout( std::process::Stdio::null() )
    .stderr( std::process::Stdio::null() )
    .status()
    .expect( "failed to spawn guard subprocess" );

  assert!( status.success(), "guard must exit 0; got: {status:?}" );

  // ── Liveness assertion ─────────────────────────────────────────────────
  // `kill -0 PID` exits 0 if the process is alive, non-0 if dead.
  // A dead process here means the guard sent kill signals — isolation violated.
  let alive = std::process::Command::new( "kill" )
    .args( [ "-0", &dummy_pid.to_string() ] )
    .status()
    .expect( "failed to run kill -0" )
    .success();

  // Clean up before asserting so the dummy never leaks on test failure.
  let _ = dummy.kill();
  let _ = dummy.wait();

  assert!( alive, "guard sent kill signals to PID {dummy_pid} — isolation invariant violated" );
}
