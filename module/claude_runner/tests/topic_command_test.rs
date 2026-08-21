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
//!   auto-generated topic slug derived from the message in the `# topic-fork:`
//!   preview line (new topics default to fork mode — no `-slug` directory)
//! - T02: two auto-named calls, same message, same `--dir` — second call's slug
//!   is disambiguated with a `-2` suffix against a pre-existing target directory
//!   (an existing `-slug` dir marks the name taken even though the fresh
//!   disambiguated slug itself starts in fork mode)
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
//! - T09: `bug_reproducer(BUG-541)` — second call to the same topic name (no
//!   `--from`) continues the topic's OWN history even after cwd's most-recent
//!   session identity drifted between the calls
//! - T10: `bug_reproducer(BUG-542)` — a candidate auto-name with no working
//!   directory but surviving session storage is skipped, never selected "fresh"
//! - T11: `bug_reproducer(BUG-543)` — the freshness probe reaches the CANONICAL
//!   storage key through a symlinked base, same as the real run will

mod cli_binary_test_helpers;
use cli_binary_test_helpers::{ exit_code, fake_claude_dir, run_ask_dry, run_cli, run_topic_dry, stderr_str, stdout_str };

/// T01 (IT-1): an auto-generated topic slug derived from the message appears in
/// dry-run output when `--topic` is not explicitly given. New topics default to
/// fork mode, so the slug surfaces in the `# topic-fork:` preview line rather
/// than as a `-slug` directory path.
#[ test ]
fn t01_auto_generated_topic_shown_in_dry_run()
{
  let output = run_topic_dry( &[ "Investigate the flaky concurrency-gate test" ] );
  assert!(
    output.contains( "# topic-fork: topic=investigate" ),
    "topic dry-run must show an auto-generated fork-mode topic slug derived from the message. Got:\n{output}"
  );
}

/// T02 (IT-2): repeated auto-naming with the same message disambiguates via a
/// `-2` counter suffix once the first slug's target directory already exists.
/// The pre-existing `-slug` dir marks the first name taken (freshness signal 1);
/// the disambiguated slug is a brand-new topic and therefore fork-mode, so it
/// shows up in the `# topic-fork:` preview line, not as a directory path.
#[ test ]
fn t02_repeated_auto_naming_disambiguates_via_counter()
{
  let tmp  = tempfile::TempDir::new().expect( "failed to create temp base dir" );
  let base = tmp.path().to_str().expect( "utf8 tempdir path" ).to_string();
  // Short, alnum-only words -> deterministic slug "flaky-gate-test" (no truncation ambiguity).
  let msg = "flaky gate test";

  // Simulate a prior real (non-dry-run) `clr topic` call having already claimed this slug.
  std::fs::create_dir_all( tmp.path().join( "-flaky-gate-test" ) ).expect( "pre-create fixture dir" );

  let second = run_topic_dry( &[ "--dir", &base, msg ] );
  assert!(
    second.contains( "# topic-fork: topic=flaky-gate-test-2 " ),
    "second call with an existing slug dir must disambiguate to flaky-gate-test-2 (fork mode). Got:\n{second}"
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

// ── T04/T05 + T09–T11: session-transplant / storage-probe tests via `clr topic` ──
//
// Mirrors `tests/bug_reproducers_490_492_test.rs`'s own BUG-490 fixture technique
// (CLAUDE_HOME override + `claude_storage_core::encode_path()` + a stubbed `claude`
// executable) — that file already exhaustively covers the transplant mechanism
// itself; T04/T05 confirm it still fires correctly when reached via `clr topic
// --topic NAME` instead of raw `--dir`/`--from`, per task 521 Out of Scope
// ("topic inherits [transplant behavior] as-is"). T09–T11 are the BUG-541/542/543
// reproducers: source-selection drift and auto-name freshness probing against the
// same CLAUDE_HOME-overridden storage fixtures.

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

  /// T09 (BUG-541): a second `clr topic --topic NAME "msg"` call — with `--from`
  /// OMITTED, so the clone source defaults to cwd — must keep continuing the
  /// topic's OWN established history even when cwd (the launching directory, not
  /// the topic directory) has since gained a newer, unrelated session of its own.
  ///
  /// ## Root Cause
  /// `session_from_dir`'s default-to-cwd fallback (`build_claude_command`,
  /// `src/cli/builder.rs`) is blind to `--topic`: it always re-derives the clone
  /// source from the literal launching cwd's own most-recently-modified session,
  /// never from the topic directory's own storage. The first call clones cwd's
  /// then-current session in, as documented — but every later call re-checks cwd
  /// fresh rather than the topic's own copy, so if cwd's most-recent session
  /// identity has since changed (ordinary, expected drift for any actively-used
  /// launch directory), the topic silently transplants that unrelated newer
  /// session in alongside its own history and resumes IT instead — orphaning the
  /// topic's actual accumulated conversation. `docs/cli/command/11_topic.md`
  /// documents the opposite: "every subsequent invocation of that same name finds
  /// the copy already in place and continues it... instead of re-copying".
  ///
  /// ## Why Not Caught
  /// T05 (above) already covers "second call, destination non-empty, never
  /// overwritten" — but it passes an unchanging, explicit `--from src` both
  /// calls, so the source's own most-recent session id never changes between
  /// them; the transplant recomputes the identical source file path either way,
  /// masking the defect. No existing test modeled the launching cwd's own most-
  /// recent session identity changing BETWEEN two calls to the same topic name.
  ///
  /// ## Fix Applied
  /// See `Fix(BUG-541)` in `src/cli/builder.rs`.
  ///
  /// ## Prevention
  /// Any change to `session_from_dir`'s source-selection order must keep a fixture
  /// where the launching cwd's most-recent session identity CHANGES between two
  /// calls to the same target — an unchanging `--from` (T05) cannot see this class.
  ///
  /// ## Pitfall
  /// Assert on `dest_b`'s ABSENCE in topic storage, not on `dest_a`'s content alone —
  /// the buggy transplant copies B alongside A, so A's bytes survive either way.
  // test_kind: bug_reproducer(BUG-541)
  #[ test ]
  fn t09_second_auto_topic_call_ignores_unrelated_source_session_drift()
  {
    container_check();
    let ch   = tempfile::TempDir::new().expect( "claude home" );
    let proj = tempfile::TempDir::new().expect( "launching project dir (acts as cwd)" );
    let proj_canon = std::fs::canonicalize( proj.path() ).expect( "canonicalize proj" );
    let uuid_a = "54109001-1111-2222-3333-444444444444";
    let content_a = b"{\"seed\":\"pre-existing unrelated cwd session A\"}\n";
    make_session( ch.path(), &proj_canon, uuid_a, content_a );

    let effective_dir = proj.path().join( "-541-drift" );
    let tgt_canon_precheck = {
      // Pre-create so the topic storage path can be computed up front; resolve_effective_dir()'s
      // own create_dir_all is idempotent, so this changes nothing about "first call" semantics.
      std::fs::create_dir_all( &effective_dir ).expect( "pre-create effective dir" );
      std::fs::canonicalize( &effective_dir ).expect( "canonicalize effective dir" )
    };
    let topic_storage = ch.path().join( "projects" ).join( df( &tgt_canon_precheck ) );
    let dest_a = topic_storage.join( format!( "{uuid_a}.jsonl" ) );

    // Call 1: first-ever use of this topic name — clones cwd's (proj's) then-current
    // session A in, exactly as documented for a fresh topic.
    let stub_body_1 = format!(
      "printf '%s' '{{\"type\":\"result\",\"subtype\":\"success\",\"is_error\":false,\"result\":\"stub-ok\",\"session_id\":\"{uuid_a}\"}}'"
    );
    let ( _stub_dir_1, stub_path_1 ) = fake_claude_dir( &stub_body_1 );
    let out1 = std::process::Command::new( env!( "CARGO_BIN_EXE_clr" ) )
      .args
      ([
        "topic",
        "--topic", "541-drift",
        "--max-sessions", "0",
        "--journal", "off",
        "start the topic",
      ])
      .current_dir( proj.path() )
      .env( "CLAUDE_HOME", ch.path() )
      .env( "PATH", &stub_path_1 )
      .env_remove( "CLR_DIR" ).env_remove( "CLR_SESSION_DIR" ).env_remove( "CLR_FROM" )
      .output()
      .expect( "invoke clr (call 1)" );
    assert!(
      out1.status.success(),
      "first topic call must succeed. stdout: {}\nstderr: {}",
      String::from_utf8_lossy( &out1.stdout ), String::from_utf8_lossy( &out1.stderr ),
    );
    let cloned_a = std::fs::read( &dest_a ).expect( "session A must be cloned into topic storage" );
    assert_eq!( cloned_a, content_a, "first call must clone cwd's then-current session byte-identically" );

    // Between calls: unrelated cwd-level work leaves a NEWER session B directly in
    // proj's own storage — never touching the topic directory at all.
    std::thread::sleep( core::time::Duration::from_millis( 50 ) );
    let uuid_b = "54109002-1111-2222-3333-444444444444";
    let content_b = b"{\"seed\":\"unrelated later cwd session B, nothing to do with the topic\"}\n";
    make_session( ch.path(), &proj_canon, uuid_b, content_b );
    let dest_b = topic_storage.join( format!( "{uuid_b}.jsonl" ) );

    // Call 2: same topic name, same cwd — must continue the topic's OWN history
    // (session A), never pull in B just because B is now cwd's most-recent.
    let stub_body_2 = format!(
      "printf '%s' '{{\"type\":\"result\",\"subtype\":\"success\",\"is_error\":false,\"result\":\"stub-ok\",\"session_id\":\"{uuid_a}\"}}'"
    );
    let ( _stub_dir_2, stub_path_2 ) = fake_claude_dir( &stub_body_2 );
    let out2 = std::process::Command::new( env!( "CARGO_BIN_EXE_clr" ) )
      .args
      ([
        "topic",
        "--topic", "541-drift",
        "--max-sessions", "0",
        "--journal", "off",
        "continue the topic",
      ])
      .current_dir( proj.path() )
      .env( "CLAUDE_HOME", ch.path() )
      .env( "PATH", &stub_path_2 )
      .env_remove( "CLR_DIR" ).env_remove( "CLR_SESSION_DIR" ).env_remove( "CLR_FROM" )
      .output()
      .expect( "invoke clr (call 2)" );
    assert!(
      out2.status.success(),
      "second topic call must succeed. stdout: {}\nstderr: {}",
      String::from_utf8_lossy( &out2.stdout ), String::from_utf8_lossy( &out2.stderr ),
    );

    assert!(
      !dest_b.exists(),
      "second call must NOT transplant cwd's unrelated newer session B into the \
       topic's own storage — the topic's continuity must not depend on cwd drift. \
       Found: {}",
      dest_b.display(),
    );
    let a_after = std::fs::read( &dest_a ).expect( "session A must still be present in topic storage" );
    assert_eq!(
      a_after, content_a,
      "topic's own session A must remain the one continued, untouched by cwd drift"
    );
  }

  /// T10 (BUG-542): a candidate auto-name whose WORKING directory does not exist but
  /// whose SESSION STORAGE already holds a session (e.g. the directory was deleted
  /// after use, or storage predates any `clr topic` use of that exact path) must
  /// never be auto-selected as "fresh" — the namer must skip it and disambiguate to
  /// the next candidate instead.
  ///
  /// ## Root Cause
  /// `disambiguate_slug` (`src/cli/topic.rs`) judged a candidate free purely by
  /// `topic_dir(base, name).exists()`. Session storage lives under
  /// `~/.claude/projects/`, entirely independent of the working directory's own
  /// filesystem lifetime, so a candidate with no working directory but a real,
  /// unrelated session already in storage was wrongly judged "fresh". Combined with
  /// `builder.rs`'s Fix(BUG-541) (deliberately authoritative once a target has ANY
  /// qualifying session of its own), the "fresh" name would then silently resume
  /// that orphaned, unrelated history on first use instead of starting clean.
  ///
  /// ## Why Not Caught
  /// T02 (the only prior disambiguation test) pre-creates the candidate's WORKING
  /// directory — no fixture ever modeled the inverse state this bug lives in:
  /// storage present, directory absent.
  ///
  /// ## Fix Applied
  /// See `Fix(BUG-542)` in `src/cli/topic.rs` (`name_is_taken`).
  ///
  /// ## Prevention
  /// Freshness has two independent signals (directory existence, session storage);
  /// every disambiguation fixture must state which signal it exercises — this test
  /// pins the storage-only signal, T02 the directory-only one.
  ///
  /// ## Pitfall
  /// Anchor `cd`-line assertions with the trailing `\n` — `-orphan-topic` is itself
  /// a string prefix of the correct `-orphan-topic-2`, so an unanchored `contains`
  /// on the shorter form false-positives against the longer. And session storage
  /// under `~/.claude/projects/` outlives its working directory's own deletion —
  /// `rm -rf`'ing a topic dir does not touch its storage, so any freshness check
  /// keyed on directory existence alone is silently wrong the moment a directory
  /// is ever removed and its name reused.
  // test_kind: bug_reproducer(BUG-542)
  #[ test ]
  fn t10_auto_naming_skips_candidate_with_orphaned_session_storage()
  {
    container_check();
    let ch   = tempfile::TempDir::new().expect( "claude home" );
    let base = tempfile::TempDir::new().expect( "topic base dir" );

    // Simulate a topic whose working directory was deleted after use but whose
    // session storage survived: the storage exists, the working directory does not.
    let orphan_dir = base.path().join( "-orphan-topic" );
    let uuid = "54209001-1111-2222-3333-444444444444";
    make_session( ch.path(), &orphan_dir, uuid, b"{\"seed\":\"orphaned unrelated history\"}\n" );
    assert!( !orphan_dir.exists(), "fixture setup: working directory must NOT exist on disk" );

    let out = std::process::Command::new( env!( "CARGO_BIN_EXE_clr" ) )
      .args
      ([
        "topic",
        "--dry-run",
        "--dir", base.path().to_str().expect( "utf-8" ),
        "orphan topic",
      ])
      .env( "CLAUDE_HOME", ch.path() )
      .env_remove( "CLR_DIR" ).env_remove( "CLR_SESSION_DIR" ).env_remove( "CLR_FROM" )
      .output()
      .expect( "invoke clr" );
    assert!(
      out.status.success(),
      "dry-run must succeed. stderr: {}",
      String::from_utf8_lossy( &out.stderr ),
    );
    let stdout = String::from_utf8_lossy( &out.stdout ).into_owned();

    // Trailing `\n` boundary matters: "-orphan-topic" is itself a string prefix of
    // the correct "-orphan-topic-2", so an unanchored `contains` on the shorter form
    // would false-positive against the longer, correctly-disambiguated line.
    let taken_cd = format!( "cd {}\n", orphan_dir.display() );
    let free_cd  = format!( "cd {}\n", base.path().join( "-orphan-topic-2" ).display() );
    assert!(
      !stdout.contains( &taken_cd ),
      "auto-naming must NOT select a candidate whose storage already has an orphaned \
       session. Got:\n{stdout}"
    );
    assert!(
      stdout.contains( &free_cd ),
      "auto-naming must disambiguate past the orphaned candidate to -orphan-topic-2. Got:\n{stdout}"
    );
  }

  /// T11 (BUG-543): an auto-name candidate whose orphaned session storage lives
  /// under a symlinked `--dir` base must still be detected — the freshness probe
  /// must resolve the SAME canonical storage key the real run would use, not the
  /// symlink's own literal path.
  ///
  /// ## Root Cause
  /// `fs::canonicalize()` fails for any path whose leaf does not exist — true of every
  /// auto-name candidate by definition — so `name_is_taken()`'s probe always resolved
  /// candidates through `physical_abs()`'s LEXICAL fallback
  /// (`cwd.join(raw).components().collect()` — symlinked ancestors kept, `..` kept),
  /// while the real run canonicalizes after `resolve_effective_dir()`'s
  /// `create_dir_all`. Divergent storage keys under symlinked/`..` bases: the probe
  /// missed the orphaned storage BUG-542 exists to detect, re-selected the "fresh"
  /// name, and the run resumed the orphaned history.
  ///
  /// ## Why Not Caught
  /// T10's fixture base is a plain `TempDir` — canonical by construction, so the
  /// lexical and canonical encodings coincide and the probe worked. No fixture routed
  /// the base through a symlink.
  ///
  /// ## Fix Applied
  /// See `Fix(BUG-543)` in `src/cli/builder.rs` (`canonicalize_deepest_prefix`) — the
  /// fallback now canonicalizes the deepest existing prefix and appends the
  /// nonexistent tail literally, matching what `create_dir_all` + claude's physical
  /// getcwd will yield.
  ///
  /// ## Prevention
  /// Pins the symlinked-base probe next to T10's canonical-base probe so the two
  /// resolution paths cannot drift apart again.
  ///
  /// ## Pitfall
  /// The seeded storage key MUST be derived from the REAL (canonical) base, never from
  /// the symlink — seeding under the symlink's own lexical encoding would let the
  /// buggy probe "find" it and the test would pass for the wrong reason.
  // test_kind: bug_reproducer(BUG-543)
  #[ test ]
  fn t11_auto_naming_probes_storage_through_symlinked_base()
  {
    container_check();
    let ch = tempfile::TempDir::new().expect( "claude home" );
    let real_base = tempfile::TempDir::new().expect( "real topic base dir" );
    let real_base_canon = std::fs::canonicalize( real_base.path() ).expect( "canonicalize real base" );

    // Convenience symlink standing in for the `--dir` base, mirroring `~/proj -> /data/projects`.
    // Lives inside its OWN fresh TempDir (never a fixed shared name) so repeated/parallel runs
    // never collide on a leftover symlink from a prior invocation.
    let link_parent = tempfile::TempDir::new().expect( "symlink parent dir" );
    let link_base = link_parent.path().join( "topic-base-link" );
    std::os::unix::fs::symlink( &real_base_canon, &link_base ).expect( "create symlink base" );

    // Seed the orphaned session under the CANONICAL path — the working directory was
    // deleted after use, its storage survives, keyed by the real (non-symlink) path.
    let orphan_dir_canon = real_base_canon.join( "-orphan-topic" );
    let uuid = "54309001-1111-2222-3333-444444444444";
    make_session( ch.path(), &orphan_dir_canon, uuid, b"{\"seed\":\"orphaned unrelated history\"}\n" );
    assert!( !orphan_dir_canon.exists(), "fixture setup: working directory must NOT exist on disk" );

    let out = std::process::Command::new( env!( "CARGO_BIN_EXE_clr" ) )
      .args
      ([
        "topic",
        "--dry-run",
        "--dir", link_base.to_str().expect( "utf-8" ),
        "orphan topic",
      ])
      .env( "CLAUDE_HOME", ch.path() )
      .env_remove( "CLR_DIR" ).env_remove( "CLR_SESSION_DIR" ).env_remove( "CLR_FROM" )
      .output()
      .expect( "invoke clr" );
    assert!(
      out.status.success(),
      "dry-run must succeed. stderr: {}",
      String::from_utf8_lossy( &out.stderr ),
    );
    let stdout = String::from_utf8_lossy( &out.stdout ).into_owned();

    // The dry-run's `cd` line echoes the effective dir built from the literal `--dir`
    // input (dry-run never creates anything, so it can't canonicalize the not-yet-
    // existing candidate) — assert against the symlink-relative form actually printed,
    // matching T10's trailing-`\n` boundary discipline (the orphaned name is a string
    // prefix of the correctly-disambiguated one).
    let taken_cd = format!( "cd {}\n", link_base.join( "-orphan-topic" ).display() );
    let free_cd  = format!( "cd {}\n", link_base.join( "-orphan-topic-2" ).display() );
    assert!(
      !stdout.contains( &taken_cd ),
      "BUG-543: auto-naming must NOT select a candidate whose CANONICAL storage already \
       has an orphaned session, even when probed through a symlinked base. Got:\n{stdout}"
    );
    assert!(
      stdout.contains( &free_cd ),
      "BUG-543: auto-naming must disambiguate past the orphaned candidate to \
       -orphan-topic-2 when probed through a symlinked base. Got:\n{stdout}"
    );
  }
}
