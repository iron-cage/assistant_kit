//! BUG-539 reproducer: interactive journal events never carry `duration_ms`.
//!
//! # Root Cause (BUG-539)
//!
//! `emit_interactive()` received only the exit code — `run_interactive()` never
//! captured a start instant, so no elapsed time could cross the call boundary at
//! either completion site (the blocking `execute_interactive()` path and the
//! `spawn_tty()`/`try_wait()` polling path). The schema was ready the whole time:
//! `EventFields.duration_ms` exists in `claude_journal` and the viewer renders it;
//! only the interactive emission path never set it, violating feature 002 AC-012
//! ("Interactive session events include session duration") on every session.
//!
//! # Why Not Caught (BUG-539)
//!
//! The journal integration suites (`journal_integration_test` EC-1..EC-10,
//! `journal_integration_ext_test` EC-11..EC-22) drive print-mode paths only —
//! no test ever ran `--interactive` through the journal, so AC-012 had no test
//! row and the missing field was invisible to the suite. It surfaced only during
//! the 2026-08-20 quota-burn reconstruction, when a per-event-type key-set audit
//! of the live journal showed 26/26 `interactive` events without any duration key
//! and session lengths had to be inferred from transcript file mtimes.
//!
//! # Fix Applied (BUG-539)
//!
//! `run_interactive()` captures `std::time::Instant::now()` at entry and both
//! completion sites pass `started.elapsed()` milliseconds into
//! `emit_interactive()`, whose signature gains a `duration_ms : u64` parameter
//! stored into `ev.fields.duration_ms`. The timeout event path is unchanged —
//! a timed-out session dies at a known bound (`timeout_secs`) by construction.
//!
//! # Prevention (BUG-539)
//!
//! Every acceptance criterion that promises a journal field must have a test
//! asserting that field on the exact event type it names — AC-012 existed in
//! `docs/feature/002_journaling_integration.md` for the whole life of the
//! interactive path with zero coverage. These reproducers pin both interactive
//! completion sites so a refactor cannot drop the field from one of them.
//!
//! # Pitfall (BUG-539)
//!
//! Duration must come from a monotonic `Instant` captured at function entry —
//! never from subtracting event `ts` wall-clock strings, and never by reusing
//! the polling path's `deadline` instant (that is `start + timeout`, not the
//! start). Interactive sessions run for hours: keep the field `u64` millis.

#![ cfg( unix ) ]

mod cli_binary_test_helpers;
use cli_binary_test_helpers::{ fake_claude_dir, read_journal_content };
use std::process::Command;

/// Run `clr --interactive` against an instantly-exiting fake `claude`, journaling
/// into a private temp dir; return the raw output plus both live temp dirs.
///
/// Stdin comes from `Command::output()`'s null pipe (non-TTY), so absent
/// `--timeout` the session resolves through `default_print_timeout()` — `0` in
/// production since TSK-503 — and takes the blocking `execute_interactive()`
/// path; an explicit `--timeout` forces the `spawn_tty()`/`try_wait()` polling
/// path instead. `HOME` points at the fixture-empty isolation path so a host
/// `~/.clr/config.toml` cannot inject a `timeout` preference and flip the branch
/// under test.
fn run_interactive_with_journal( extra_args : &[ &str ] )
-> ( std::process::Output, tempfile::TempDir, tempfile::TempDir )
{
  let ( fake_dir, path ) = fake_claude_dir( "exit 0" );
  let journal_dir = tempfile::TempDir::new().expect( "journal tmpdir" );
  let jd = journal_dir.path().display().to_string();
  let bin = env!( "CARGO_BIN_EXE_clr" );
  let mut args : Vec< &str > = vec!
  [
    "--interactive", "--max-sessions", "0",
    "--journal", "full", "--journal-dir", &jd,
  ];
  args.extend_from_slice( extra_args );
  args.push( "x" );
  let out = Command::new( bin )
    .args( &args )
    .env( "PATH", &path )
    .env( "HOME", "/tmp/clr-isolated-home" )
    .env_remove( "CLR_JOURNAL" )
    .env_remove( "CLR_JOURNAL_DIR" )
    .env_remove( "CLR_TIMEOUT" )
    .env_remove( "_CLR_DEFAULT_TIMEOUT" )
    .env_remove( "CLR_DIR" )
    .output()
    .expect( "failed to invoke clr binary" );
  ( out, fake_dir, journal_dir )
}

/// Extract the sole `"type":"interactive"` line from the journal content.
fn interactive_line( journal : &str ) -> String
{
  let lines : Vec< &str > = journal
    .lines()
    .filter( | l | l.contains( r#""type":"interactive""# ) )
    .collect();
  assert_eq!( lines.len(), 1, "expected exactly one interactive event, journal:\n{journal}" );
  ( *lines.first().expect( "checked len above" ) ).to_owned()
}

/// Parse the numeric value following `"duration_ms":` on the event line.
fn duration_ms_value( line : &str ) -> u64
{
  let key = r#""duration_ms":"#;
  let start = line.find( key ).expect( "duration_ms key present (asserted by caller)" ) + key.len();
  let digits : String = line[ start.. ].chars().take_while( char::is_ascii_digit ).collect();
  digits.parse().expect( "duration_ms value must be an unsigned integer" )
}

/// AC-012: the blocking (`timeout == 0`) interactive completion site emits
/// `duration_ms` on its journal event (BUG-539).
#[ doc = "bug_reproducer(BUG-539)" ]
#[ test ]
fn bug_539_blocking_path_interactive_event_carries_duration_ms()
{
  let ( out, _fake, journal_dir ) = run_interactive_with_journal( &[] );
  assert!( out.status.success(), "clr must exit 0, stderr: {}", String::from_utf8_lossy( &out.stderr ) );
  let journal = read_journal_content( journal_dir.path() );
  let line = interactive_line( &journal );
  assert!(
    line.contains( r#""duration_ms":"# ),
    "BUG-539: blocking-path interactive event lacks duration_ms — AC-012 violated: {line}",
  );
  // Fake claude exits instantly; anything over 60s means a wrong clock source.
  assert!( duration_ms_value( &line ) < 60_000, "implausible duration on instant-exit session: {line}" );
}

/// AC-012: the timeout-polling (`timeout > 0`) interactive completion site emits
/// `duration_ms` on its journal event (BUG-539).
#[ doc = "bug_reproducer(BUG-539)" ]
#[ test ]
fn bug_539_timeout_path_interactive_event_carries_duration_ms()
{
  let ( out, _fake, journal_dir ) = run_interactive_with_journal( &[ "--timeout", "30" ] );
  assert!( out.status.success(), "clr must exit 0, stderr: {}", String::from_utf8_lossy( &out.stderr ) );
  let journal = read_journal_content( journal_dir.path() );
  let line = interactive_line( &journal );
  assert!(
    line.contains( r#""duration_ms":"# ),
    "BUG-539: timeout-path interactive event lacks duration_ms — AC-012 violated: {line}",
  );
  assert!( duration_ms_value( &line ) < 60_000, "implausible duration on instant-exit session: {line}" );
}
