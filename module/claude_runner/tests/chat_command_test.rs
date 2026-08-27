//! `clr chat` — argument handling and the things it refuses to do.
//!
//! # What is covered, and what is not
//!
//! Almost every case here is one the command can settle *before* it needs a
//! session: argument parsing, help, and the ordering guarantee that argument
//! errors are reported without a daemon being started to discover them. The
//! exception is CH-10, which starts a real daemon — because whether `--session`
//! is honoured or silently ignored is only visible from *which* request fails.
//!
//! A chat that actually completes is not a CLI test. It needs a real `claude`
//! on `PATH`, answering on a real terminal, over a real model call — that is an
//! end-to-end concern, and the layers under it are tested where they live: the
//! terminal in `claude_pty_core`, the send/read cycle in `claude_daemon_core`'s
//! `serve_test.rs`, the rendering of what comes back in `claude_terminal_core`'s
//! `render_test.rs`, and the answer read out of the transcript in
//! `claude_storage_core`'s `transcript_answer_test.rs`.
//!
//! The ordering guarantee (CH-6, CH-7) is the one worth stating out loud: a
//! command that auto-starts a daemon must not start one on the way to reporting
//! a typo.
//!
//! ## Specification References
//!
//! - `docs/cli/command/14_chat.md` — command contract
//!
//! ## Coverage
//!
//! | TC | Scenario | Expected |
//! |----|----------|----------|
//! | CH-1 | `clr chat help` | Usage, every option, exit 0 |
//! | CH-2 | No message | Says a message is needed, exit 1 |
//! | CH-3 | Unknown option | Names it, exit 1 |
//! | CH-4 | `--timeout` with a non-number | Names the bad value, exit 1 |
//! | CH-5 | A flag with its value missing | Names the flag, exit 1 |
//! | CH-6 | Two bare arguments | Suggests quoting, exit 1 |
//! | CH-7 | Argument error | No daemon started to discover it |
//! | CH-8 | `clr chatt` | Typo guard suggests `chat` |
//! | CH-9 | The phrase the onboarding hint keys on | Still the daemon's own wording |
//! | CH-10 | `--session <ID>` against a real daemon | Fails at `send`, not at `spawn` |

mod cli_binary_test_helpers;
use cli_binary_test_helpers::{ DaemonGuard, exit_code, run_cli, run_cli_with_env, stderr_str, stdout_str };

/// Run `clr chat` against an injected home.
fn chat_in( home : &std::path::Path, args : &[ &str ] ) -> std::process::Output
{
  let home = home.to_str().expect( "home path is not UTF-8" );
  let mut full = vec![ "chat" ];
  full.extend_from_slice( args );
  run_cli_with_env( &full, &[ ( "HOME", home ) ] )
}

/// CH-1: help documents every option the command takes.
#[ test ]
fn ch1_help_lists_every_option()
{
  let home = tempfile::tempdir().expect( "tempdir" );
  let out = chat_in( home.path(), &[ "help" ] );

  assert_eq!( exit_code( &out ), 0, "help must succeed" );
  let stdout = stdout_str( &out );
  for flag in [ "--dir", "--session", "--timeout", "--raw" ]
  {
    assert!( stdout.contains( flag ), "help must document {flag}. Got:\n{stdout}" );
  }
}

/// CH-2: a chat with nothing to say is a mistake, and says which one.
#[ test ]
fn ch2_missing_message_is_rejected()
{
  let home = tempfile::tempdir().expect( "tempdir" );
  let out = chat_in( home.path(), &[] );

  assert_eq!( exit_code( &out ), 1, "no message must fail" );
  let stderr = stderr_str( &out );
  assert!( stderr.contains( "needs a message" ), "must say what is missing. Got:\n{stderr}" );
}

/// CH-3: an option that does not exist is named rather than ignored.
#[ test ]
fn ch3_unknown_option_is_rejected()
{
  let home = tempfile::tempdir().expect( "tempdir" );
  let out = chat_in( home.path(), &[ "hello", "--loudly" ] );

  assert_eq!( exit_code( &out ), 1, "an unknown option must fail" );
  assert!
  (
    stderr_str( &out ).contains( "--loudly" ),
    "the rejection must name the option. Got:\n{}",
    stderr_str( &out )
  );
}

/// CH-4: a timeout that is not a number is caught, with the value quoted back.
#[ test ]
fn ch4_non_numeric_timeout_is_rejected()
{
  let home = tempfile::tempdir().expect( "tempdir" );
  let out = chat_in( home.path(), &[ "hello", "--timeout", "soon" ] );

  assert_eq!( exit_code( &out ), 1, "a non-numeric timeout must fail" );
  assert!
  (
    stderr_str( &out ).contains( "soon" ),
    "the rejection must quote the bad value. Got:\n{}",
    stderr_str( &out )
  );
}

/// CH-5: a flag at the very end, with nothing after it, is not silently ignored.
#[ test ]
fn ch5_flag_without_value_is_rejected()
{
  let home = tempfile::tempdir().expect( "tempdir" );
  let out = chat_in( home.path(), &[ "hello", "--session" ] );

  assert_eq!( exit_code( &out ), 1, "a dangling flag must fail" );
  assert!
  (
    stderr_str( &out ).contains( "--session" ),
    "the rejection must name the flag. Got:\n{}",
    stderr_str( &out )
  );
}

/// CH-6: two bare words mean the quotes were forgotten.
///
/// Chatting about only the first word would be the worst outcome: it succeeds,
/// costs a model call, and answers a question nobody asked.
#[ test ]
fn ch6_unquoted_message_is_rejected()
{
  let home = tempfile::tempdir().expect( "tempdir" );
  let out = chat_in( home.path(), &[ "say", "hello" ] );

  assert_eq!( exit_code( &out ), 1, "an unquoted message must fail" );
  assert!
  (
    stderr_str( &out ).contains( "quote it" ),
    "the rejection must suggest quoting. Got:\n{}",
    stderr_str( &out )
  );
}

/// CH-7: arguments are settled before anything is started.
///
/// The ordering that makes CH-2..CH-6 cheap. `clr chat` auto-starts a daemon,
/// which is right when there is a chat to have and wrong on the way to
/// reporting a typo.
#[ test ]
fn ch7_argument_errors_start_no_daemon()
{
  let home = tempfile::tempdir().expect( "tempdir" );
  let out = chat_in( home.path(), &[ "hello", "--loudly" ] );
  assert_eq!( exit_code( &out ), 1 );

  let runtime = home.path().join( ".claude" ).join( "-daemon" );
  assert!
  (
    !runtime.join( "daemon.sock" ).exists(),
    "a rejected argument must not have started a daemon at {}",
    runtime.display()
  );
}

/// CH-8: a near-miss on the subcommand is caught by the typo guard.
#[ test ]
fn ch8_typo_suggests_chat()
{
  let out = run_cli( &[ "chatt" ] );

  assert_eq!( exit_code( &out ), 1, "an unknown subcommand must fail" );
  assert!
  (
    stderr_str( &out ).contains( "chat" ),
    "the guard must suggest 'chat'. Got:\n{}",
    stderr_str( &out )
  );
}

/// CH-9: the onboarding hint's trigger phrase is still the daemon's own wording.
///
/// `chat` decides whether to print the "answer the first-run prompt" hint by
/// looking for a substring of the daemon's error text — the only signal it has,
/// since the failure arrives as prose inside `Error::Remote`. That is a coupling
/// across a crate boundary that nothing else would notice breaking: reword
/// `Error::NoRegistration`'s `Display` and the hint quietly stops appearing,
/// leaving the error it exists to explain exactly as opaque as before.
///
/// So the phrase is pinned here rather than left implicit. If this fails, the
/// fix is to update the needle in `src/cli/chat.rs` to match — not to revert the
/// wording.
#[ test ]
fn ch9_the_hint_phrase_matches_the_daemon_error()
{
  let rendered = claude_daemon_core::Error::NoRegistration { pid : 4321 }.to_string();

  assert!
  (
    rendered.contains( "never registered a conversation id" ),
    "the phrase `chat` keys its onboarding hint on is gone from the daemon's error. Got:\n{rendered}"
  );
}

/// CH-10: `--session` addresses the daemon directly instead of resolving a directory.
///
/// The only case in this file that needs a real daemon, and it earns one: an
/// ignored `--session` is invisible from the outside. `chat` would fall through
/// to matching on the working directory, find nothing, and *spawn* a session —
/// which is a different failure with a different message, and on a machine with
/// a working `claude` is not a failure at all. Naming a session that cannot
/// exist is what separates the two: reaching the daemon fails at `send`, and
/// ignoring the flag fails at `spawn`.
///
/// The id is a well-formed UUID the daemon has never seen, so the rejection
/// comes from the session table rather than from parsing.
#[ test ]
fn ch10_session_flag_reaches_the_daemon()
{
  let home = tempfile::tempdir().expect( "tempdir" );
  let _guard = DaemonGuard::for_home( home.path() );

  let started = run_cli_with_env
  (
    &[ "daemon", "start" ],
    &[ ( "HOME", home.path().to_str().expect( "home path is not UTF-8" ) ) ],
  );
  assert_eq!( exit_code( &started ), 0, "daemon must start. stderr:\n{}", stderr_str( &started ) );

  let out = chat_in( home.path(), &[ "hello", "--session", "00000000-0000-4000-8000-000000000000" ] );
  let stderr = stderr_str( &out );

  assert_eq!( exit_code( &out ), 1, "an unknown session must fail. stderr:\n{stderr}" );
  assert!
  (
    stderr.contains( "would not take the message" ),
    "the failure must come from `send`, which means `--session` reached the daemon. Got:\n{stderr}"
  );
  assert!
  (
    !stderr.contains( "would not start" ),
    "`--session` was ignored and a spawn was attempted instead. Got:\n{stderr}"
  );
}
