//! `topic` Subcommand Integration Tests
//!
//! ## Purpose
//!
//! Verify that `clr topic` behaves exactly like `clr ask`/`clr run`, with one
//! addition: when `--topic` is not explicitly given, a directory-name slug
//! derived from the message is auto-injected as `--topic`, disambiguated with a
//! `-2`, `-3`, ... counter suffix against what already exists on disk. An explicit
//! `--topic` bypasses slug generation entirely and makes `topic` byte-identical
//! to `ask`.
//!
//! ## Strategy
//!
//! T01–T03, T06–T08 invoke `clr topic --dry-run` (via `run_topic_dry`) and compare
//! against `clr ask --dry-run` (via `run_ask_dry`) — no real Claude invocation.
//! T04/T05 (`#[cfg(unix)]`) prove the session-transplant clone/continue transition
//! against a real `.jsonl` file with a real (non-dry-run) subprocess spawn, per
//! task 521's AF1 requirement — a stubbed `claude` executable (`fake_claude_dir`)
//! stands in for the real binary, and `CLAUDE_HOME` is overridden so the fixture
//! never touches the host's real session storage.
//!
//! ## Corner Cases Covered (mirrors `tests/docs/cli/command/11_topic.md` IT-1..IT-8)
//!
//! - T01: `clr topic --dry-run "msg"` (no `--topic`) — dry-run output shows an
//!   auto-generated `--topic` path derived from the message
//! - T02: two auto-named calls, same message, same `--dir` — second call's slug
//!   is disambiguated with a `-2` suffix against a pre-existing target directory
//! - T03: `clr topic --topic NAME "msg" --dry-run` == `clr ask --topic NAME
//!   "msg" --dry-run` (byte-identical, per IT-3's own spec wording)
//! - T04: first `clr topic --topic NAME "msg"` call (real, non-dry-run) with a
//!   qualifying source session — session-transplant clone fires, `.jsonl` copied
//! - T05: second call, same NAME, destination already non-empty — no re-copy
//!   (pre-existing, possibly-diverged destination content is never overwritten)
//! - T06: `clr topic --not-a-real-flag "msg"` — unknown flag rejected, exit 1
//! - T07: `clr topic help` / `--help` / `-h` — topic-specific help text, exit 0
//!   (positional-`help` intercept per the BUG-249 pattern every subcommand
//!   dispatcher delegating to `dispatch_run` must repeat independently)
//! - T08: `clr topic --topic NAME --effort high "msg" --dry-run` == the same
//!   `ask` invocation — `--effort high` passthrough is visible and unbroken
//!   (AF2: a parameter that would visibly change dry-run output if broken)

mod cli_binary_test_helpers;
use cli_binary_test_helpers::{ exit_code, fake_claude_dir, run_ask_dry, run_cli, run_topic_dry, stderr_str, stdout_str };

/// T01 (IT-1): auto-generated `--topic` path appears in dry-run output when
/// `--topic` is not explicitly given.
#[ test ]
fn t01_auto_generated_topic_shown_in_dry_run()
{
  let output = run_topic_dry( &[ "Investigate the flaky concurrency-gate test" ] );
  let sep = std::path::MAIN_SEPARATOR;
  assert!(
    output.contains( &format!( "{sep}-investigate" ) ),
    "topic dry-run must show an auto-generated --topic path derived from the message. Got:\n{output}"
  );
}

/// T02 (IT-2): repeated auto-naming with the same message disambiguates via a
/// `-2` counter suffix once the first slug's target directory already exists.
#[ test ]
fn t02_repeated_auto_naming_disambiguates_via_counter()
{
  let tmp  = tempfile::TempDir::new().expect( "failed to create temp base dir" );
  let base = tmp.path().to_str().expect( "utf8 tempdir path" ).to_string();
  // Short, alnum-only words -> deterministic slug "flaky-gate-test" (no truncation ambiguity).
  let msg = "flaky gate test";

  // Simulate a prior real (non-dry-run) `clr topic` call having already claimed this slug.
  std::fs::create_dir_all( tmp.path().join( "-flaky-gate-test" ) ).expect( "pre-create fixture dir" );

  let sep    = std::path::MAIN_SEPARATOR;
  let second = run_topic_dry( &[ "--dir", &base, msg ] );
  assert!(
    second.contains( &format!( "{sep}-flaky-gate-test-2" ) ),
    "second call with an existing slug dir must disambiguate to -flaky-gate-test-2. Got:\n{second}"
  );
}

/// T03 (IT-3): explicit `--topic` bypasses slug generation — `topic` produces
/// byte-identical dry-run output to `ask` given the same trailing arguments.
#[ test ]
fn t03_explicit_topic_matches_ask_byte_for_byte()
{
  let topic_out = run_topic_dry( &[ "--topic", "auth-refactor", "q" ] );
  let ask_out   = run_ask_dry( &[ "--topic", "auth-refactor", "q" ] );
  assert_eq!(
    topic_out, ask_out,
    "topic --topic NAME must produce identical dry-run output to ask --topic NAME.\ntopic:\n{topic_out}\nask:\n{ask_out}"
  );
}

/// T06 (IT-6): an unknown flag is rejected with exit 1 and a stderr diagnostic
/// naming the unknown option — same contract as `ask`/`run`. Uses `run_cli`
/// directly (not `run_topic_dry`, which asserts success) since this case must fail.
#[ test ]
fn t06_unknown_flag_exits_1()
{
  let out = run_cli( &[ "topic", "--not-a-real-flag", "msg" ] );
  assert_eq!( exit_code( &out ), 1, "unknown flag must exit 1. stderr: {}", stderr_str( &out ) );
  assert!(
    stderr_str( &out ).contains( "unknown option" ),
    "stderr must name the unknown option. Got:\n{}", stderr_str( &out )
  );
}

/// T07 (IT-7): all three help-invocation forms — positional `help`, `--help`,
/// `-h` — print topic-specific help text and exit 0.
///
/// ## Fix Applied
///
/// Regression coverage for the BUG-249 pattern: every subcommand dispatcher that
/// delegates to `dispatch_run` must independently intercept the positional `help`
/// token before parsing, or `clr topic help` would be sent to Claude as a literal
/// message instead of printing help.
#[ test ]
fn t07_positional_and_flag_help_forms_print_topic_help_and_exit_0()
{
  for args in [ &[ "topic", "help" ][ .. ], &[ "topic", "--help" ][ .. ], &[ "topic", "-h" ][ .. ] ]
  {
    let out = run_cli( args );
    assert_eq!( exit_code( &out ), 0, "{args:?} must exit 0. stderr: {}", stderr_str( &out ) );
    let stdout = stdout_str( &out );
    assert!(
      stdout.contains( "auto-generated" ) && stdout.contains( "clr topic" ),
      "{args:?} must print topic-specific help text (distinct from the generic top-level help). Got:\n{stdout}"
    );
  }
}

/// T08 (IT-8): `--effort high` passes through `topic` identically to `ask` —
/// AF2 requires a parameter that visibly changes dry-run output if broken, so
/// this asserts full equivalence (not just a substring match) against `ask`.
#[ test ]
fn t08_effort_high_passthrough_matches_ask_dry_run()
{
  let topic_out = run_topic_dry( &[ "--topic", "effort-check", "--effort", "high", "msg" ] );
  let ask_out   = run_ask_dry( &[ "--topic", "effort-check", "--effort", "high", "msg" ] );
  assert_eq!(
    topic_out, ask_out,
    "topic --effort high must pass through identically to ask --effort high.\ntopic:\n{topic_out}\nask:\n{ask_out}"
  );
  assert!(
    topic_out.contains( "high" ),
    "dry-run output must visibly reflect --effort high (AF2: not a no-op flag). Got:\n{topic_out}"
  );
}

// ── T04/T05: real session-transplant clone/continue via `clr topic` ───────────
//
// Mirrors `tests/bug_reproducers_490_492_test.rs`'s own BUG-490 fixture technique
// (CLAUDE_HOME override + `claude_storage_core::encode_path()` + a stubbed `claude`
// executable) — that file already exhaustively covers the transplant mechanism
// itself; these two tests confirm it still fires correctly when reached via `clr
// topic --topic NAME` instead of raw `--dir`/`--from`, per task 521 Out of Scope
// ("topic inherits [transplant behavior] as-is").

#[ cfg( unix ) ]
mod transplant
{
  use super::{ fake_claude_dir, stderr_str };

  /// Container guard (mirrors the private `assert_container` in `cli_binary_test_helpers`).
  fn container_check()
  {
    let in_container = std::path::Path::new( "/.dockerenv" ).exists()
      || std::path::Path::new( "/run/.containerenv" ).exists()
      || std::env::var( "RUNBOX_CONTAINER" ).as_deref() == Ok( "1" );
    let escaped = std::env::var( "VERB_LAYER" ).as_deref() == Ok( "l0" );
    assert!(
      in_container || escaped,
      "\n\nTests must run inside a container.\n\
       Host bypass: VERB_LAYER=l0 cargo nextest run --all-features\n"
    );
  }

  /// Encode a path using the production `Df()` encoder (BUG-391 precedent: never hand-roll).
  fn df( path : &std::path::Path ) -> String
  {
    claude_storage_core::encode_path( path )
      .expect( "df(): path must encode successfully in test fixtures" )
  }

  /// Create `<claude_home>/projects/<df(project_dir)>/<uuid>.jsonl` with the given content.
  ///
  /// Returns the `.jsonl` path. The caller must keep the `TempDir` alive.
  fn make_session( claude_home : &std::path::Path, project_dir : &std::path::Path, uuid : &str, content : &[ u8 ] )
    -> std::path::PathBuf
  {
    let storage = claude_home.join( "projects" ).join( df( project_dir ) );
    std::fs::create_dir_all( &storage ).expect( "create session storage dir" );
    let file = storage.join( format!( "{uuid}.jsonl" ) );
    std::fs::write( &file, content ).expect( "write session jsonl" );
    file
  }

  /// T04 (IT-4): first `clr topic --topic NAME "msg"` call, a qualifying source
  /// session present at `--from`, real (non-dry-run) invocation — the session-
  /// transplant plan fires and the source `.jsonl` is copied byte-identically
  /// into the target's own storage before the (stubbed) subprocess spawns.
  #[ test ]
  fn t04_first_explicit_topic_call_clones_session()
  {
    container_check();
    let ch   = tempfile::TempDir::new().expect( "claude home" );
    let src  = tempfile::TempDir::new().expect( "source project" );
    let base = tempfile::TempDir::new().expect( "target base dir" );
    let src_canon = std::fs::canonicalize( src.path() ).expect( "canonicalize source" );
    let uuid = "52104004-1111-2222-3333-444444444444";
    let content = b"{\"seed\":\"topic clone source\"}\n";
    let src_jsonl = make_session( ch.path(), &src_canon, uuid, content );

    // Pre-create the empty effective dir (base/-NAME) so it can be canonicalized up
    // front — resolve_effective_dir()'s own create_dir_all is idempotent, so this
    // changes nothing about "first call, no existing session" semantics.
    let effective_dir = base.path().join( "-521-topic-clone" );
    std::fs::create_dir_all( &effective_dir ).expect( "pre-create effective dir" );
    let tgt_canon = std::fs::canonicalize( &effective_dir ).expect( "canonicalize effective dir" );
    let dest_dir  = ch.path().join( "projects" ).join( df( &tgt_canon ) );
    let dest_file = dest_dir.join( format!( "{uuid}.jsonl" ) );

    let stub_body = format!(
      "printf '%s' '{{\"type\":\"result\",\"subtype\":\"success\",\"is_error\":false,\"result\":\"stub-ok\",\"session_id\":\"{uuid}\"}}'"
    );
    let ( _stub_dir, stub_path_val ) = fake_claude_dir( &stub_body );

    let out = std::process::Command::new( env!( "CARGO_BIN_EXE_clr" ) )
      .args
      ([
        "topic",
        "--dir", base.path().to_str().expect( "utf-8" ),
        "--topic", "521-topic-clone",
        "--from", src.path().to_str().expect( "utf-8" ),
        "--max-sessions", "0",
        "--journal", "off",
        "clone this session please",
      ])
      .env( "CLAUDE_HOME", ch.path() )
      .env( "PATH", &stub_path_val )
      .env_remove( "CLR_DIR" ).env_remove( "CLR_SESSION_DIR" ).env_remove( "CLR_FROM" )
      .output()
      .expect( "invoke clr" );
    assert!(
      out.status.success(),
      "real topic run must succeed. stdout: {}\nstderr: {}",
      String::from_utf8_lossy( &out.stdout ), String::from_utf8_lossy( &out.stderr ),
    );

    let copied = std::fs::read( &dest_file ).expect( "transplanted file must exist in target storage" );
    assert_eq!( copied, content, "transplanted file must be byte-identical to the source session" );
    let src_after = std::fs::read( &src_jsonl ).expect( "source must still exist" );
    assert_eq!( src_after, content, "source session must never be modified by a clone run" );
  }

  /// T05 (IT-5): a second `clr topic --topic NAME "msg"` call against the same
  /// NAME never overwrites the (possibly-diverged) destination already there —
  /// proof that continuation, not re-copy, is used on repeat use.
  #[ test ]
  fn t05_second_explicit_topic_call_never_recopies_existing_destination()
  {
    container_check();
    let ch   = tempfile::TempDir::new().expect( "claude home" );
    let src  = tempfile::TempDir::new().expect( "source project" );
    let base = tempfile::TempDir::new().expect( "target base dir" );
    let src_canon = std::fs::canonicalize( src.path() ).expect( "canonicalize source" );
    let uuid = "52105005-1111-2222-3333-444444444444";
    make_session( ch.path(), &src_canon, uuid, b"{\"seed\":\"original\"}\n" );

    let effective_dir = base.path().join( "-521-topic-continue" );
    std::fs::create_dir_all( &effective_dir ).expect( "pre-create effective dir" );
    let tgt_canon = std::fs::canonicalize( &effective_dir ).expect( "canonicalize effective dir" );
    // Pre-place a diverged prior clone under the SAME uuid in the target's own storage —
    // simulates "first call already happened and the session has since moved on".
    let diverged = b"{\"seed\":\"original\"}\n{\"turn\":\"topic-local divergence\"}\n";
    let dest_file = make_session( ch.path(), &tgt_canon, uuid, diverged );

    let stub_body = format!(
      "printf '%s' '{{\"type\":\"result\",\"subtype\":\"success\",\"is_error\":false,\"result\":\"stub-ok\",\"session_id\":\"{uuid}\"}}'"
    );
    let ( _stub_dir, stub_path_val ) = fake_claude_dir( &stub_body );

    let out = std::process::Command::new( env!( "CARGO_BIN_EXE_clr" ) )
      .args
      ([
        "topic",
        "--dir", base.path().to_str().expect( "utf-8" ),
        "--topic", "521-topic-continue",
        "--from", src.path().to_str().expect( "utf-8" ),
        "--max-sessions", "0",
        "--journal", "off",
        "--trace",
        "continue the clone lineage",
      ])
      .env( "CLAUDE_HOME", ch.path() )
      .env( "PATH", &stub_path_val )
      .env_remove( "CLR_DIR" ).env_remove( "CLR_SESSION_DIR" ).env_remove( "CLR_FROM" )
      .output()
      .expect( "invoke clr" );
    assert!(
      out.status.success(),
      "real topic run must succeed. stderr: {}",
      String::from_utf8_lossy( &out.stderr ),
    );

    let after = std::fs::read( &dest_file ).expect( "dest must still exist" );
    assert_eq!( after, diverged, "existing destination (diverged prior clone) must never be overwritten" );
    assert!(
      stderr_str( &out ).contains( " -c \"" ),
      "second call must continue via -c, not clone fresh. Got:\n{}", stderr_str( &out )
    );
  }
}
