//! `clr run --interactive` vs. a daemon-hosted session in the same directory.
//!
//! # What is covered, and what is not
//!
//! `docs/feature/008_interactive_handoff.md` names three outcomes for a pre-spawn
//! probe: no daemon, no match, and an idle match that gets released. All three are
//! daemon-interaction logic this crate owns, and are covered here against a real
//! (non-mocked) `claude_daemon_core::Daemon` — the same in-process technique
//! `ps_flags_test.rs`'s IT-55 uses, via `cli_binary_test_helpers::spawn_real_daemon`.
//!
//! The busy-refuse branch is not exercised here: forcing a hosted session's `busy`
//! flag to `true` means writing a transcript with a genuinely open turn, which is
//! `claude_storage_core`'s own turn-detection surface, not this feature's. It is
//! covered instead by `docs/feature/008_interactive_handoff.md`'s own manual
//! verification recipe, against a real `claude` — the same deferral already
//! accepted for the resume mechanism's terminal behaviour in
//! `claude_daemon_core/docs/feature/009_session_resume.md`.
//!
//! The fake `claude` (`/bin/sleep` from `fake_claude_binary_dir()`) errors
//! instantly on every argv shape this feature ever produces — no numeric
//! duration is ever among them — so every case here completes in well under a
//! second with no bounded-timeout machinery needed.
//!
//! ## Coverage
//!
//! | TC | Scenario | Expected |
//! |----|----------|----------|
//! | IH-1 | No daemon running | Proceeds unchanged; no daemon started to ask |
//! | IH-2 | Daemon running, nothing hosted for the target dir | Proceeds unchanged |
//! | IH-3 | Daemon running, idle session hosted for the target dir | Session is released |

mod cli_binary_test_helpers;
use cli_binary_test_helpers::{ fake_claude_binary_dir, run_cli_with_env, spawn_real_daemon };
use claude_daemon_core::{ client, Request };

/// Run `clr run --interactive --dir <target>` against an injected `HOME`/`PATH`.
fn run_interactive_in( home : &std::path::Path, path_val : &str, target : &std::path::Path ) -> std::process::Output
{
  let home_str   = home.to_str().expect( "home path is not UTF-8" );
  let target_str = target.to_str().expect( "target dir is not UTF-8" );
  run_cli_with_env(
    &[ "run", "--interactive", "--dir", target_str ],
    &[ ( "HOME", home_str ), ( "PATH", path_val ) ],
  )
}

/// IH-1: with no daemon reachable, the pre-spawn probe finds nothing and the
/// command proceeds unchanged — and, per the probe-don't-start contract
/// (`docs/cli/command/15_sessions.md`), never starts a daemon just to ask it.
#[ test ]
fn ih1_no_daemon_running_proceeds_unchanged()
{
  let ( _bin_dir, path_val ) = fake_claude_binary_dir();
  let home   = tempfile::TempDir::new().expect( "home tempdir" );
  let target = tempfile::TempDir::new().expect( "target tempdir" );

  let _out = run_interactive_in( home.path(), &path_val, target.path() );

  let runtime = home.path().join( ".claude" ).join( "-daemon" );
  assert!(
    !runtime.join( "daemon.sock" ).exists(),
    "an interactive spawn with nothing hosted must not itself start a daemon at {}",
    runtime.display()
  );
}

/// IH-2: a daemon is running, but hosts nothing for the target directory — the
/// probe succeeds, the cwd match fails, and the command proceeds unchanged.
/// The daemon must still be alive and answering afterward: finding no match is
/// not an error and must not disturb it.
#[ test ]
fn ih2_daemon_running_but_nothing_hosted_for_target_dir_proceeds_unchanged()
{
  let ( _bin_dir, path_val ) = fake_claude_binary_dir();
  let home   = tempfile::TempDir::new().expect( "home tempdir" );
  let target = tempfile::TempDir::new().expect( "target tempdir" );
  let ( socket, shutdown ) = spawn_real_daemon( home.path(), &path_val );

  let _out = run_interactive_in( home.path(), &path_val, target.path() );

  let alive = client::call( &socket, &Request::ListSessions ).is_ok();
  shutdown();

  assert!( alive, "the daemon must still answer after a no-match probe" );
}

/// IH-3: a daemon hosts an idle session for exactly the target directory — the
/// probe finds it, and because it is idle (freshly spawned, never marked busy)
/// the handoff releases it for real: `Shutdown` reaches the daemon, and the
/// session is gone from `list_sessions` afterward.
#[ test ]
fn ih3_daemon_running_with_idle_session_for_target_dir_releases_it()
{
  let ( _bin_dir, path_val ) = fake_claude_binary_dir();
  let home   = tempfile::TempDir::new().expect( "home tempdir" );
  let target = tempfile::TempDir::new().expect( "target tempdir" );
  let ( socket, shutdown ) = spawn_real_daemon( home.path(), &path_val );

  let spawn_result = client::call( &socket, &Request::Spawn { cwd : target.path().to_path_buf(), prompt : None } )
    .expect( "spawn request failed" );
  assert!(
    spawn_result[ "session_id" ].as_str().is_some(),
    "IH-3: spawn must report a session_id, got {spawn_result:?}"
  );

  let _out = run_interactive_in( home.path(), &path_val, target.path() );

  let list_after = client::call( &socket, &Request::ListSessions ).expect( "list_sessions failed" );
  shutdown();

  let still_hosted = list_after.as_array().is_some_and( | rows | !rows.is_empty() );
  assert!(
    !still_hosted,
    "IH-3: the idle session for the target dir must have been released. list_sessions: {list_after:?}"
  );
}
