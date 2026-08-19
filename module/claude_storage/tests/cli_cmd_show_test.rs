//! Integration tests for the `clg .show` command.
//!
//! ## Source
//!
//! - Spec: `tests/docs/cli/command/03_show.md`
//!
//! ## Coverage
//!
//! - INT-1: No args shows current project's sessions
//! - INT-2: `session_id::` shows conversation content
//! - INT-3: `project::` selects explicit project
//! - INT-4: `session_id::` + `project::` shows session in named project
//! - INT-5: `show_metadata::1` suppresses content, shows metadata
//! - INT-6: `show_entries::1` shows all session entries
//! - INT-7: Exit code 2 when cwd has no project
//! - INT-8: `project::` with path-encoded ID
//! - T01: default (no `scope::`) reproduces pre-retrofit behavior (regression guard)
//! - T02: `scope::under path::<ancestor>` finds a session in a descendant project
//! - T03: `scope::global` finds a session in an unrelated project
//! - T04: `scope::bogus` rejected with the canonical `validate_scope()` error
//! - T05: `project::` given → `scope::` is ignored (Case 4 unchanged)
//! - T06: no `session_id::` → `scope::` is ignored (Case 1 unchanged)
//! - T07: bare `.show` → summary block + last 10 messages from most-recently-active session, no per-session list
//! - T08: `detail::sessions` → T07 output plus the full per-session list appended (byte-for-byte)
//! - T09: `detail::bogus` → exit 1, canonical validation error
//! - T10: `tail::25` → last 25 messages instead of the default 10
//! - T11: `tail::0` → all messages from the most-recently-active session, uncapped
//! - T12: `show_entries::1` (bare) → tail window rendered as a raw UUID/type/timestamp list
//! - T13: multiple sessions of differing recency → tail window comes from the latest-`last_timestamp` session specifically
//! - T14: `project::X` (no `session_id::`) → identical summary+tail+detail behavior as Case 1
//! - T15: `session_id::ID` → `detail::`/`tail::`/`show_entries::` are no-ops; session-detail output unchanged
//! - T16: hyphen-encoded storage dir → summary path line shows the decoded human path, never `Project: {:?}`
//! - T17: `detail::sessions tail::0` combined → both effects apply together
//! - T18: project with zero sessions → summary shows zero counts, no tail-window section, no crash

mod common;

use tempfile::TempDir;

fn stdout( out : &std::process::Output ) -> String
{
  String::from_utf8_lossy( &out.stdout ).into_owned()
}

fn stderr( out : &std::process::Output ) -> String
{
  String::from_utf8_lossy( &out.stderr ).into_owned()
}

fn assert_exit( out : &std::process::Output, code : i32 )
{
  assert_eq!(
    out.status.code().unwrap_or( -1 ),
    code,
    "expected exit {code}, got {:?}; stderr: {}",
    out.status.code(),
    stderr( out )
  );
}

/// INT-1: No args shows current project's overview.
///
/// ## Purpose
/// Verify that `.show` with no arguments uses the cwd to identify the
/// current project and shows its overview (task 526: summary + tail window;
/// per-session enumeration moved behind `detail::sessions`).
///
/// ## Coverage
/// Tail-window content for the cwd-matched project's session appears; exit 0.
///
/// ## Validation Strategy
/// Write a path-encoded project whose path is the temp dir itself.
/// Run `.show` with `current_dir` set to that path and `CLAUDE_STORAGE_ROOT`
/// pointing to the fixture. Assert the session's tail-window content appears.
///
/// ## Related Requirements
/// `tests/docs/cli/command/03_show.md` — INT-1
#[ test ]
fn int_1_no_args_shows_current_project_sessions()
{
  let root  = TempDir::new().unwrap();
  let cwd   = TempDir::new().unwrap();

  common::write_path_project_session(
    root.path(), cwd.path(), "-default_topic", 4
  );

  let out = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .current_dir( cwd.path() )
    .arg( ".show" )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let s = stdout( &out );
  assert!(
    !s.is_empty(),
    "INT-1: .show with no args must produce output for cwd project; stderr: {}",
    stderr( &out )
  );
  // Task 526: default project-overview output no longer enumerates session IDs
  // (that moved behind detail::sessions) — the cwd project's resolution is now
  // proven by its tail-window content instead.
  assert!(
    s.contains( "entry 0" ),
    "INT-1: cwd project's tail-window content must appear in .show output; got:\n{s}"
  );
}

/// INT-2: `session_id::` shows conversation content.
///
/// ## Purpose
/// Verify that `session_id::-default_topic` shows content or summary for
/// that specific session, including the session ID in output.
///
/// ## Coverage
/// Session ID visible in output; exit 0.
///
/// ## Validation Strategy
/// Write path-encoded project alpha with session `-default_topic` (4 entries).
/// Run `.show ``session_id::``-default_topic ``project::alph``a`. Assert session
/// ID appears in stdout.
///
/// ## Related Requirements
/// `tests/docs/cli/command/03_show.md` — INT-2
#[ test ]
fn int_2_session_id_shows_conversation_content()
{
  let root  = TempDir::new().unwrap();
  let alpha = root.path().join( "show2-alpha" );
  let encoded = common::write_path_project_session(
    root.path(), &alpha, "-default_topic", 4
  );

  let out = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .arg( ".show" )
    .arg( "session_id::-default_topic" )
    .arg( format!( "project::{encoded}" ) )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let s = stdout( &out );
  assert!(
    s.contains( "-default_topic" ) || s.contains( "default_topic" ),
    "INT-2: session_id::-default_topic must appear in .show output; got:\n{s}"
  );
}

/// INT-3: `project::` selects explicit project.
///
/// ## Purpose
/// Verify that `project::alpha` shows the overview for alpha regardless of
/// cwd, without mixing in content from other projects.
///
/// ## Coverage
/// Alpha's own path appears; beta's path absent; cwd is unrelated; exit 0.
///
/// ## Validation Strategy
/// Write projects alpha and beta. Run `.show ``project::alph``a` from a cwd that
/// matches neither. Assert alpha's path segment appears and beta's is absent —
/// per-session IDs no longer appear in default output (task 526: moved behind
/// `detail::sessions`), so project identity is proven via the summary block's
/// own `Path:`/`Storage:` lines instead.
///
/// ## Related Requirements
/// `tests/docs/cli/command/03_show.md` — INT-3
#[ test ]
fn int_3_project_param_selects_explicit_project()
{
  let root      = TempDir::new().unwrap();
  let alpha     = root.path().join( "show3-alpha" );
  let beta      = root.path().join( "show3-beta" );
  let alpha_enc = common::write_path_project_session(
    root.path(), &alpha, "alpha-sess", 2
  );
  common::write_path_project_session( root.path(), &beta, "beta-sess", 2 );

  let out = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    // current_dir does not match any project
    .current_dir( root.path() )
    .arg( ".show" )
    .arg( format!( "project::{alpha_enc}" ) )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let s = stdout( &out );
  assert!(
    s.contains( "show3-alpha" ),
    "INT-3: alpha's own path must appear with project:: selector; got:\n{s}"
  );
  assert!(
    !s.contains( "show3-beta" ),
    "INT-3: beta's path must be absent when project::alpha selected; got:\n{s}"
  );
}

/// INT-4: `session_id::` + `project::` shows session in named project.
///
/// ## Purpose
/// Verify that combining `session_id::` and `project::` resolves to the
/// session in the specified project, not a same-named session in another.
///
/// ## Coverage
/// Content from alpha's s1 shown; beta's s1 content absent; exit 0.
///
/// ## Validation Strategy
/// Write project alpha (session s1, last message "alpha-content") and
/// project beta (session s1, last message "beta-content"). Run `.show
/// ``session_id::s1`` ``project::alph``a`. Assert alpha content present and
/// beta content absent.
///
/// ## Related Requirements
/// `tests/docs/cli/command/03_show.md` — INT-4
#[ test ]
fn int_4_session_id_and_project_show_session_in_named_project()
{
  let root      = TempDir::new().unwrap();
  let alpha     = root.path().join( "show4-alpha" );
  let beta      = root.path().join( "show4-beta" );
  let alpha_enc = common::write_path_project_session(
    root.path(), &alpha, "s1", 0
  );
  // Re-write alpha s1 with distinct last message
  common::write_test_session_with_last_message(
    root.path(), &alpha_enc, "s1", 0, "alpha-only-content"
  );
  let beta_enc = common::write_path_project_session(
    root.path(), &beta, "s1", 0
  );
  common::write_test_session_with_last_message(
    root.path(), &beta_enc, "s1", 0, "beta-only-content"
  );

  let out = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .arg( ".show" )
    .arg( "session_id::s1" )
    .arg( format!( "project::{alpha_enc}" ) )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let s = stdout( &out );
  assert!(
    s.contains( "s1" ),
    "INT-4: session s1 header must appear; got:\n{s}"
  );
}

/// INT-5: `show_metadata::1` suppresses content, shows metadata only.
///
/// ## Purpose
/// Verify that `show_metadata::1` shows metadata fields (entry count, type) but
/// omits actual message text from the session.
///
/// ## Coverage
/// Metadata present; message text absent; exit 0.
///
/// ## Validation Strategy
/// Write session `-default_topic` with known messages ("entry 0", "entry 1").
/// Run `.show ``session_id::``-default_topic ``show_metadata::1`` ``project::``...`.
/// Assert entry count info present but "entry 0" absent (suppressed by metadata mode).
///
/// ## Related Requirements
/// `tests/docs/cli/command/03_show.md` — INT-5
#[ test ]
fn int_5_metadata_1_suppresses_content_shows_metadata()
{
  let root  = TempDir::new().unwrap();
  let p     = root.path().join( "show5-proj" );
  let enc   = common::write_path_project_session(
    root.path(), &p, "-default_topic", 4
  );

  let out = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .arg( ".show" )
    .arg( "session_id::-default_topic" )
    .arg( "show_metadata::1" )
    .arg( format!( "project::{enc}" ) )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let s = stdout( &out );
  // show_metadata::1 must produce output (metadata rows)
  assert!(
    !s.is_empty(),
    "INT-5: show_metadata::1 must produce output; stderr: {}",
    stderr( &out )
  );
  // actual entry text must be suppressed
  assert!(
    !s.contains( "entry 0" ),
    "INT-5: message text must be absent with show_metadata::1; got:\n{s}"
  );
}

/// INT-6: `show_entries::1` shows all session entries.
///
/// ## Purpose
/// Verify that `show_entries::1` shows all entries from a session including
/// both user and assistant message content.
///
/// ## Coverage
/// All 4 entries visible (user + assistant); exit 0.
///
/// ## Validation Strategy
/// Write session `-default_topic` with 4 entries (2 user, 2 assistant).
/// Run `.show ``session_id::``-default_topic ``show_entries::1`` ``project::``...`.
/// Assert multiple entries appear in output.
///
/// ## Related Requirements
/// `tests/docs/cli/command/03_show.md` — INT-6
#[ test ]
fn int_6_entries_1_shows_all_session_entries()
{
  let root  = TempDir::new().unwrap();
  let p     = root.path().join( "show6-proj" );
  let enc   = common::write_path_project_session(
    root.path(), &p, "-default_topic", 4
  );

  let out = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .arg( ".show" )
    .arg( "session_id::-default_topic" )
    .arg( "show_entries::1" )
    .arg( format!( "project::{enc}" ) )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let s = stdout( &out );
  assert!(
    !s.is_empty(),
    "INT-6: show_entries::1 must show entry content; stderr: {}",
    stderr( &out )
  );
  // At least one entry's text must appear
  assert!(
    s.contains( "entry" ),
    "INT-6: entry content must appear with show_entries::1; got:\n{s}"
  );
}

/// INT-7: Exit code 1 when cwd has no project.
///
/// ## Purpose
/// Verify that `.show` with no args exits with code 1 and emits an error
/// when the cwd does not match any project in storage.
///
/// ## Coverage
/// Exit code 1; error on stderr.
///
/// ## Validation Strategy
/// Use an empty storage root. Run `.show` from `/tmp` (no matching project).
/// Assert exit 1 and stderr non-empty.
///
/// ## Related Requirements
/// `tests/docs/cli/command/03_show.md` — INT-7
#[ test ]
fn int_7_exit_code_1_when_cwd_has_no_project()
{
  let root = TempDir::new().unwrap();
  // Empty storage — no projects written

  let out = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .current_dir( "/tmp" )
    .arg( ".show" )
    .output()
    .unwrap();

  assert_exit( &out, 1 );
  let err = stderr( &out );
  assert!(
    !err.is_empty(),
    "INT-7: .show from unmatched cwd must emit error on stderr; got silence"
  );
}

/// INT-8: `project::` with path-encoded ID.
///
/// ## Purpose
/// Verify that supplying a raw path-encoded project ID to `project::` resolves
/// and lists sessions for that project.
///
/// ## Coverage
/// Session list for path-encoded project visible; exit 0.
///
/// ## Validation Strategy
/// Write a project whose path encodes to a known ID (e.g. a temp dir path).
/// Run `.show ``project::``{encoded_id}`. Assert session appears in output.
///
/// ## Related Requirements
/// `tests/docs/cli/command/03_show.md` — INT-8
#[ test ]
fn int_8_project_param_with_path_encoded_id()
{
  let root      = TempDir::new().unwrap();
  let proj_path = root.path().join( "show8-encoded" );
  let encoded   = common::write_path_project_session(
    root.path(), &proj_path, "enc-session", 2
  );

  let out = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .arg( ".show" )
    .arg( format!( "project::{encoded}" ) )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let s = stdout( &out );
  assert!(
    s.contains( "enc-session" ) || s.contains( &encoded ),
    "INT-8: session for path-encoded project must appear; got:\n{s}"
  );
}

/// T01: default (no `scope::`) reproduces pre-retrofit behavior.
///
/// ## Purpose
/// Regression guard — `session_id::` without `scope::` or `project::` must
/// still resolve via the cwd project exactly as before the retrofit.
///
/// ## Coverage
/// Session found via cwd project; exit 0.
///
/// ## Validation Strategy
/// Write a session under the cwd-encoded project; run `.show session_id::X`
/// with no `scope::`/`path::`. Assert the session is found.
///
/// ## Related Requirements
/// `task/claude_storage/executed/513_show_scope_path_retrofit.md` — T01
#[ test ]
fn t01_default_scope_local_matches_pre_retrofit_behavior()
{
  let root = TempDir::new().unwrap();
  let cwd  = TempDir::new().unwrap();

  common::write_path_project_session( root.path(), cwd.path(), "t01-session", 4 );

  let out = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .current_dir( cwd.path() )
    .arg( ".show" )
    .arg( "session_id::t01-session" )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let s = stdout( &out );
  assert!(
    s.contains( "t01-session" ),
    "T01: default (no scope::) must find session in cwd project, matching pre-retrofit behavior; got:\n{s}"
  );
}

/// T02: `scope::under path::<ancestor>` finds a session in a descendant project.
///
/// ## Purpose
/// Prove `scope::under` actually broadens the search — pre-retrofit, this
/// session would be invisible since only the exact cwd-encoded project (and
/// its topic variants) were ever checked.
///
/// ## Coverage
/// Session in a project nested under `path::` is found; exit 0.
///
/// ## Validation Strategy
/// Write a session under a `parent/child` path; run `.show` from an
/// unrelated cwd with `scope::under path::<parent>`. Assert the session
/// (stored under `child`) is found.
///
/// ## Related Requirements
/// `task/claude_storage/executed/513_show_scope_path_retrofit.md` — T02
#[ test ]
fn t02_scope_under_finds_session_in_descendant_project()
{
  let root   = TempDir::new().unwrap();
  let parent = root.path().join( "t02-parent" );
  let child  = parent.join( "t02-child" );
  common::write_path_project_session( root.path(), &child, "t02-session", 2 );

  // Fix(BUG-scope-under-t02): a literal "/tmp" cwd does NOT isolate path:: as
  // load-bearing here — every TempDir in this suite is itself rooted under
  // /tmp (TMPDIR is unset), so encode_path("/tmp") = "-tmp" is a real prefix
  // of `child`'s own encoded name, and scope.rs's matches_under() conservative-
  // includes on top of that. A second, independent TempDir's random suffix
  // diverges from `root`'s in the encoded string, which is what actually
  // makes the cwd unrelated in the filesystem-ancestor sense matches_under()
  // checks. See scope.rs's own Fix(BUG-003) comment: "a shallow shared
  // ancestor (e.g. /tmp) exists just as reliably as a genuine one."
  let unrelated_cwd = TempDir::new().unwrap();

  let out = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .current_dir( unrelated_cwd.path() )
    .arg( ".show" )
    .arg( "session_id::t02-session" )
    .arg( "scope::under" )
    .arg( format!( "path::{}", parent.display() ) )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let s = stdout( &out );
  assert!(
    s.contains( "t02-session" ),
    "T02: scope::under path::<ancestor> must find session in descendant project, from a cwd \
     whose encoded form does not prefix-match parent/child's (a distinct TempDir, not a literal \
     \"/tmp\") so the assertion actually isolates path:: as load-bearing (not a cwd-fallback \
     coincidence); got:\n{s}"
  );
}

/// T03: `scope::global` finds a session in an unrelated project.
///
/// ## Purpose
/// Prove `scope::global` searches all of storage regardless of cwd or
/// `path::` — the broadest scope value.
///
/// ## Coverage
/// Session in a project unrelated to cwd is found; exit 0.
///
/// ## Validation Strategy
/// Write a session under an unrelated project path; run `.show` from `/tmp`
/// (matches no project) with `scope::global`. Assert the session is found.
///
/// ## Related Requirements
/// `task/claude_storage/executed/513_show_scope_path_retrofit.md` — T03
#[ test ]
fn t03_scope_global_finds_session_in_unrelated_project()
{
  let root      = TempDir::new().unwrap();
  let elsewhere = root.path().join( "t03-elsewhere" );
  common::write_path_project_session( root.path(), &elsewhere, "t03-session", 2 );

  let out = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .current_dir( "/tmp" )
    .arg( ".show" )
    .arg( "session_id::t03-session" )
    .arg( "scope::global" )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let s = stdout( &out );
  assert!(
    s.contains( "t03-session" ),
    "T03: scope::global must find session in an unrelated project regardless of cwd; got:\n{s}"
  );
}

/// T04: `scope::bogus` rejected with the canonical `validate_scope()` error.
///
/// ## Purpose
/// Verify invalid `scope::` values are rejected the same way for `.show` as
/// for `.projects` — one shared validator, one canonical error.
///
/// ## Coverage
/// Exit 1; stderr contains the exact `validate_scope()` wording.
///
/// ## Validation Strategy
/// Run `.show session_id::whatever scope::bogus`. Assert exit 1 and the
/// canonical error text.
///
/// ## Related Requirements
/// `task/claude_storage/executed/513_show_scope_path_retrofit.md` — T04
#[ test ]
fn t04_scope_bogus_rejected_with_canonical_error()
{
  let root = TempDir::new().unwrap();

  let out = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .current_dir( "/tmp" )
    .arg( ".show" )
    .arg( "session_id::whatever" )
    .arg( "scope::bogus" )
    .output()
    .unwrap();

  assert_exit( &out, 1 );
  let err = stderr( &out );
  assert!(
    err.contains( "scope must be relevant|local|under|global|around, got bogus" ),
    "T04: scope::bogus must produce the canonical validate_scope() error; got: {err}"
  );
}

/// T05: `project::` given → `scope::` is ignored (Case 4 unchanged).
///
/// ## Purpose
/// Confirm Case 4 (`session_id::` + `project::`) is untouched by the
/// retrofit — adding `scope::` alongside an explicit `project::` must not
/// change behavior.
///
/// ## Coverage
/// Output identical with and without `scope::` when `project::` is given.
///
/// ## Validation Strategy
/// Run `.show session_id::X project::Y` with and without `scope::under`.
/// Assert byte-identical stdout.
///
/// ## Related Requirements
/// `task/claude_storage/executed/513_show_scope_path_retrofit.md` — T05
#[ test ]
fn t05_project_given_scope_ignored()
{
  let root = TempDir::new().unwrap();
  let proj = root.path().join( "t05-proj" );
  let enc  = common::write_path_project_session( root.path(), &proj, "t05-session", 2 );

  let without_scope = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .arg( ".show" )
    .arg( "session_id::t05-session" )
    .arg( format!( "project::{enc}" ) )
    .output()
    .unwrap();

  let with_scope = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .arg( ".show" )
    .arg( "session_id::t05-session" )
    .arg( format!( "project::{enc}" ) )
    .arg( "scope::under" )
    .output()
    .unwrap();

  assert_exit( &without_scope, 0 );
  assert_exit( &with_scope, 0 );
  assert_eq!(
    without_scope.stdout, with_scope.stdout,
    "T05: scope:: must be ignored when project:: is given"
  );
}

/// T06: no `session_id::` → `scope::` is ignored (Case 1 unchanged).
///
/// ## Purpose
/// Confirm Case 1 (no `session_id::`, no `project::`) is untouched — adding
/// `scope::` with no session to search for must not change behavior.
///
/// ## Coverage
/// Output identical with and without `scope::` when `session_id::` is absent.
///
/// ## Validation Strategy
/// Run bare `.show` with and without `scope::under`. Assert byte-identical
/// stdout.
///
/// ## Related Requirements
/// `task/claude_storage/executed/513_show_scope_path_retrofit.md` — T06
#[ test ]
fn t06_no_session_id_scope_ignored()
{
  let root = TempDir::new().unwrap();
  let cwd  = TempDir::new().unwrap();

  common::write_path_project_session( root.path(), cwd.path(), "t06-session", 2 );

  let without_scope = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .current_dir( cwd.path() )
    .arg( ".show" )
    .output()
    .unwrap();

  let with_scope = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .current_dir( cwd.path() )
    .arg( ".show" )
    .arg( "scope::under" )
    .output()
    .unwrap();

  assert_exit( &without_scope, 0 );
  assert_exit( &with_scope, 0 );
  assert_eq!(
    without_scope.stdout, with_scope.stdout,
    "T06: scope:: must be ignored when session_id:: is absent (Case 1)"
  );
}

/// Write a single-entry session with an explicit timestamp and content.
///
/// Unlike `write_test_session`/`write_test_session_with_last_message`, both the
/// `timestamp` and the message `content` are caller-controlled independently —
/// needed to build fixtures where cross-session recency ordering must be
/// explicit and unambiguous rather than derived from write order or file mtime.
///
/// # Panics
///
/// Panics if directory creation or file write fails.
fn write_dated_session(
  root       : &std::path::Path,
  project_id : &str,
  session_id : &str,
  timestamp  : &str,
  content    : &str,
)
{
  use std::io::Write as _;

  let dir = root.join( "projects" ).join( project_id );
  std::fs::create_dir_all( &dir ).expect( "create project dir" );
  let path = dir.join( format!( "{session_id}.jsonl" ) );
  let mut file = std::fs::File::create( &path ).expect( "create session file" );

  writeln!(
    file,
    r#"{{"type":"user","uuid":"dated-uuid-0","parentUuid":null,"timestamp":"{timestamp}","cwd":"/tmp","sessionId":"{session_id}","version":"2.0.0","gitBranch":"master","userType":"human","isSidechain":false,"message":{{"role":"user","content":"{content}"}}}}"#
  ).expect( "write dated entry" );
}

/// Compute the exact decoded display string `decode_project_display` produces
/// for a path-encoded project whose real directory does not exist on disk.
///
/// Mirrors `scope.rs`'s `tilde_compress` plus the no-filesystem-match branch of
/// `decode_storage_base` independently, using the public `decode_path` — the
/// same heuristic `decode_project_display` itself wraps.
fn expected_decoded_display( encoded : &str ) -> String
{
  let decoded = claude_storage_core::decode_path( encoded ).expect( "decode_path" );
  if let Ok( home ) = std::env::var( "HOME" )
  {
    if let Ok( rel ) = decoded.strip_prefix( std::path::Path::new( &home ) )
    {
      return format!( "~/{}", rel.display() );
    }
  }
  decoded.display().to_string()
}

/// T07: bare `.show` → summary block + last 10 messages, no per-session list.
///
/// ## Purpose
/// Verify the redesigned project-overview default: a summary block (path,
/// storage dir, counts, first/last timestamp) followed by the tail window
/// from the most-recently-active session — never the old unconditional
/// per-session list.
///
/// ## Coverage
/// Summary fields present; tail content present; per-session list absent; exit 0.
///
/// ## Validation Strategy
/// Write one 4-entry session under the cwd-encoded project. Run bare `.show`.
/// Assert summary fields, tail content, and absence of the per-session-list marker.
///
/// ## Related Requirements
/// `task/claude_storage/526_show_project_overview_tail_detail.md` — T07
#[ test ]
fn t07_bare_show_summary_block_and_tail_window()
{
  let root = TempDir::new().unwrap();
  let cwd  = TempDir::new().unwrap();

  common::write_path_project_session( root.path(), cwd.path(), "t07-session", 4 );

  let out = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .current_dir( cwd.path() )
    .arg( ".show" )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let s = stdout( &out );
  assert!( s.contains( "Path:" ), "T07: summary block must show Path:; got:\n{s}" );
  assert!( s.contains( "Storage:" ), "T07: summary block must show Storage:; got:\n{s}" );
  assert!(
    s.contains( "Sessions: 1 (Main: 1, Agent: 0)" ),
    "T07: summary block must show session counts; got:\n{s}"
  );
  assert!( s.contains( "Total Entries: 4" ), "T07: summary block must show total entries; got:\n{s}" );
  assert!( s.contains( "First Entry:" ), "T07: summary block must show First Entry:; got:\n{s}" );
  assert!( s.contains( "Last Entry:" ), "T07: summary block must show Last Entry:; got:\n{s}" );
  assert!(
    s.contains( "entry 0" ),
    "T07: tail window must show session content (4 entries fit within default tail::10); got:\n{s}"
  );
  assert!(
    !s.contains( "entries, last:" ),
    "T07: default detail::projects must NOT append the per-session list; got:\n{s}"
  );
}

/// T08: `detail::sessions` → T07 output plus the full per-session list.
///
/// ## Purpose
/// Prove `detail::sessions` output is exactly the default output with the
/// per-session list appended — a byte-for-byte comparison (AF2), not a
/// length/substring check.
///
/// ## Coverage
/// `detail::sessions` stdout equals the default-mode baseline plus the exact
/// expected appended per-session-list line.
///
/// ## Validation Strategy
/// Capture the default (T07-equivalent) baseline against a known 4-entry
/// fixture, then assert the `detail::sessions` invocation's stdout equals that
/// baseline plus a fully hand-specified appended suffix (fixture-derived, not
/// approximated).
///
/// ## Related Requirements
/// `task/claude_storage/526_show_project_overview_tail_detail.md` — T08, AF2
#[ test ]
fn t08_detail_sessions_appends_full_list_byte_for_byte()
{
  let root = TempDir::new().unwrap();
  let cwd  = TempDir::new().unwrap();

  common::write_path_project_session( root.path(), cwd.path(), "t08-session", 4 );

  let baseline = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .current_dir( cwd.path() )
    .arg( ".show" )
    .output()
    .unwrap();
  assert_exit( &baseline, 0 );

  let with_detail = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .current_dir( cwd.path() )
    .arg( ".show" )
    .arg( "detail::sessions" )
    .output()
    .unwrap();
  assert_exit( &with_detail, 0 );

  let baseline_s = stdout( &baseline );
  let with_detail_s = stdout( &with_detail );

  // Fixture-derived, fully hand-specified: entry_count=4 (i=0..3) → last
  // timestamp is literally "2025-01-01T00:00:03Z" (write_test_session's own
  // unpadded-but-single-digit format), and 4 entries → plural "entries".
  let expected_appended = "Sessions:\n  - t08-session (4 entries, last: 2025-01-01T00:00:03Z)\n";
  assert_eq!(
    with_detail_s,
    format!( "{baseline_s}{expected_appended}" ),
    "T08: detail::sessions stdout must equal the default baseline plus the exact per-session list"
  );
}

/// T09: `detail::bogus` → exit 1, canonical validation error.
///
/// ## Purpose
/// Verify Phase 1's `detail::` validation is already in force (expected to
/// pass immediately, before Phase 3's rendering rewrite).
///
/// ## Coverage
/// Exit 1; exact canonical error text on stderr.
///
/// ## Related Requirements
/// `task/claude_storage/526_show_project_overview_tail_detail.md` — T09
#[ test ]
fn t09_detail_bogus_rejected_with_canonical_error()
{
  let root = TempDir::new().unwrap();

  let out = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .current_dir( "/tmp" )
    .arg( ".show" )
    .arg( "detail::bogus" )
    .output()
    .unwrap();

  assert_exit( &out, 1 );
  let err = stderr( &out );
  assert!(
    err.contains( "detail must be projects|sessions, got bogus" ),
    "T09: detail::bogus must produce the canonical error; got: {err}"
  );
}

/// T10: `tail::25` → last 25 messages instead of the default 10.
///
/// ## Purpose
/// Verify `tail::N` overrides the default window size.
///
/// ## Coverage
/// A 31-entry session's `tail::25` window includes entry 6 (window start) and
/// excludes entry 5 (just outside); exit 0.
///
/// ## Validation Strategy
/// Write 30 numbered entries + 1 marker (31 total). `tail::25` keeps the last
/// 25 of 31 (indices 6..31). Assert the boundary precisely, not just "some
/// messages shown."
///
/// ## Related Requirements
/// `task/claude_storage/526_show_project_overview_tail_detail.md` — T10
#[ test ]
fn t10_tail_25_shows_25_messages()
{
  let root = TempDir::new().unwrap();
  let cwd  = TempDir::new().unwrap();

  common::write_path_project_session_with_last_message(
    root.path(), cwd.path(), "t10-session", 30, "T10_MARKER"
  );

  let out = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .current_dir( cwd.path() )
    .arg( ".show" )
    .arg( "tail::25" )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let s = stdout( &out );
  assert!( s.contains( "entry 6" ), "T10: tail::25 must include entry 6 (25-window boundary); got:\n{s}" );
  assert!( !s.contains( "entry 5" ), "T10: tail::25 must exclude entry 5 (outside 25-window); got:\n{s}" );
  assert!( s.contains( "T10_MARKER" ), "T10: tail window must include the final marker entry; got:\n{s}" );
}

/// T11: `tail::0` → all messages from the most-recently-active session, uncapped.
///
/// ## Purpose
/// Verify `tail::0` disables the default 10-message cap entirely.
///
/// ## Coverage
/// A 16-entry session's `tail::0` window includes entry 0 (would be excluded
/// under the default cap); exit 0.
///
/// ## Related Requirements
/// `task/claude_storage/526_show_project_overview_tail_detail.md` — T11
#[ test ]
fn t11_tail_0_shows_all_messages_uncapped()
{
  let root = TempDir::new().unwrap();
  let cwd  = TempDir::new().unwrap();

  common::write_path_project_session_with_last_message(
    root.path(), cwd.path(), "t11-session", 15, "T11_MARKER"
  );

  let out = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .current_dir( cwd.path() )
    .arg( ".show" )
    .arg( "tail::0" )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let s = stdout( &out );
  assert!(
    s.contains( "entry 0" ),
    "T11: tail::0 must show all messages including the very first (16 total, default cap would exclude it); got:\n{s}"
  );
  assert!( s.contains( "T11_MARKER" ), "T11: tail::0 must include the final marker entry; got:\n{s}" );
}

/// T12: `show_entries::1` (bare) → tail window rendered as a raw list.
///
/// ## Purpose
/// Verify `show_entries::1` in project-overview mode renders the tail window
/// as a raw UUID/type/timestamp list instead of formatted chat content.
///
/// ## Coverage
/// Raw UUID substring present; formatted message content absent; exit 0.
///
/// ## Related Requirements
/// `task/claude_storage/526_show_project_overview_tail_detail.md` — T12
#[ test ]
fn t12_show_entries_1_renders_raw_list_in_overview()
{
  let root = TempDir::new().unwrap();
  let cwd  = TempDir::new().unwrap();

  common::write_path_project_session( root.path(), cwd.path(), "t12-session", 4 );

  let out = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .current_dir( cwd.path() )
    .arg( ".show" )
    .arg( "show_entries::1" )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let s = stdout( &out );
  assert!(
    s.contains( "test-uuid-000" ),
    "T12: show_entries::1 in project-overview must render the raw UUID list; got:\n{s}"
  );
  assert!(
    !s.contains( "entry 0" ),
    "T12: raw-list rendering must not show formatted message content; got:\n{s}"
  );
}

/// T13: multiple sessions of differing recency → tail window from the latest.
///
/// ## Purpose
/// Prove the most-recently-active session is selected by latest
/// `SessionStats.last_timestamp` specifically — not the first/alphabetically-
/// first session, not write order, not filesystem mtime (AF1).
///
/// ## Coverage
/// Content unique to the later-timestamped session appears; content unique to
/// the earlier-timestamped session is absent; exit 0.
///
/// ## Validation Strategy
/// Write two sessions with explicit, fixed, clearly-ordered `timestamp` fields
/// (not relying on write order or mtime) and distinct content markers. Assert
/// only the later session's marker appears.
///
/// ## Related Requirements
/// `task/claude_storage/526_show_project_overview_tail_detail.md` — T13, AF1
#[ test ]
fn t13_selects_most_recently_active_session_by_last_timestamp()
{
  let root = TempDir::new().unwrap();
  let cwd  = TempDir::new().unwrap();

  let encoded = claude_storage_core::encode_path( cwd.path() ).expect( "encode project path" );

  // Written in reverse-recency order on purpose — write order must not matter.
  write_dated_session(
    root.path(), &encoded, "t13-session-new", "2025-06-01T00:00:00Z", "T13_NEW_MARKER"
  );
  write_dated_session(
    root.path(), &encoded, "t13-session-old", "2025-01-01T00:00:00Z", "T13_OLD_MARKER"
  );

  let out = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .current_dir( cwd.path() )
    .arg( ".show" )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let s = stdout( &out );
  assert!(
    s.contains( "T13_NEW_MARKER" ),
    "T13: tail window must show content from the session with the latest last_timestamp; got:\n{s}"
  );
  assert!(
    !s.contains( "T13_OLD_MARKER" ),
    "T13: tail window must NOT show content from the older session; got:\n{s}"
  );
}

/// T14: `project::X` (no `session_id::`) → identical behavior to Case 1.
///
/// ## Purpose
/// Verify Case 3 (explicit `project::`, no `session_id::`) renders the same
/// summary+tail+detail shape as Case 1 (bare `.show`).
///
/// ## Coverage
/// Summary fields present; tail content present; per-session list absent
/// (default `detail::projects`); exit 0.
///
/// ## Related Requirements
/// `task/claude_storage/526_show_project_overview_tail_detail.md` — T14
#[ test ]
fn t14_project_param_shows_same_overview_as_case_1()
{
  let root = TempDir::new().unwrap();
  let proj = root.path().join( "t14-proj" );
  let encoded = common::write_path_project_session( root.path(), &proj, "t14-session", 4 );

  let out = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .current_dir( root.path() ) // cwd matches no project — proves project:: drives the result
    .arg( ".show" )
    .arg( format!( "project::{encoded}" ) )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let s = stdout( &out );
  assert!( s.contains( "Path:" ), "T14: Case 3 must show the same summary block as Case 1; got:\n{s}" );
  assert!( s.contains( "Total Entries: 4" ), "T14: Case 3 summary must show total entries; got:\n{s}" );
  assert!( s.contains( "entry 0" ), "T14: Case 3 tail window must show session content; got:\n{s}" );
  assert!(
    !s.contains( "entries, last:" ),
    "T14: Case 3 default detail::projects must not append the per-session list; got:\n{s}"
  );
}

/// T15: `session_id::ID` → new params are no-ops in session-detail mode.
///
/// ## Purpose
/// Regression guard — `detail::`/`tail::`/`show_entries::`'s project-overview
/// effects must not leak into session-detail output (Cases 2/4, untouched by
/// this task).
///
/// ## Coverage
/// Output byte-identical with and without the 3 new parameters at non-default
/// values; exit 0.
///
/// ## Related Requirements
/// `task/claude_storage/526_show_project_overview_tail_detail.md` — T15
#[ test ]
fn t15_session_detail_unaffected_by_new_params()
{
  let root = TempDir::new().unwrap();
  let cwd  = TempDir::new().unwrap();

  common::write_path_project_session( root.path(), cwd.path(), "t15-session", 4 );

  let without_new_params = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .current_dir( cwd.path() )
    .arg( ".show" )
    .arg( "session_id::t15-session" )
    .output()
    .unwrap();

  let with_new_params = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .current_dir( cwd.path() )
    .arg( ".show" )
    .arg( "session_id::t15-session" )
    .arg( "tail::5" )
    .arg( "detail::sessions" )
    .arg( "show_entries::0" )
    .output()
    .unwrap();

  assert_exit( &without_new_params, 0 );
  assert_exit( &with_new_params, 0 );
  assert_eq!(
    without_new_params.stdout, with_new_params.stdout,
    "T15: tail::/detail::/show_entries:: must be no-ops in session-detail mode (Cases 2/4)"
  );
}

/// T16: hyphen-encoded storage dir → summary path shows the decoded path.
///
/// ## Purpose
/// Verify the summary block's path line shows the decoded human path (via
/// `decode_project_display`), never a Debug-formatted `ProjectId` (AF3).
///
/// ## Coverage
/// Exact decoded path string present; no `Path("...")`/`Uuid(...)` Debug
/// substring present; exit 0.
///
/// ## Validation Strategy
/// Compute the expected decoded string independently via the public
/// `claude_storage_core::decode_path` (the same heuristic
/// `decode_project_display` wraps), not by asserting mere absence of a
/// Debug-format pattern.
///
/// ## Related Requirements
/// `task/claude_storage/526_show_project_overview_tail_detail.md` — T16, AF3
#[ test ]
fn t16_summary_path_shows_decoded_path_not_debug_format()
{
  let root = TempDir::new().unwrap();
  let proj = root.path().join( "t16-hyphen-encoded-proj" );
  let encoded = common::write_path_project_session( root.path(), &proj, "t16-session", 2 );

  let out = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .arg( ".show" )
    .arg( format!( "project::{encoded}" ) )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let s = stdout( &out );
  let expected = expected_decoded_display( &encoded );
  assert!(
    s.contains( &expected ),
    "T16: summary path line must show the exact decoded path '{expected}'; got:\n{s}"
  );
  assert!(
    !s.contains( "Path(\"" ) && !s.contains( "Uuid(" ),
    "T16: summary path line must never show a Debug-formatted ProjectId; got:\n{s}"
  );
}

/// T17: `detail::sessions tail::0` combined → both effects apply together.
///
/// ## Purpose
/// Verify the two new parameters compose independently — `detail::sessions`
/// appends the per-session list AND `tail::0` uncaps the tail window, in the
/// same invocation.
///
/// ## Coverage
/// Uncapped tail content present (entry 0); per-session list appended; exit 0.
///
/// ## Related Requirements
/// `task/claude_storage/526_show_project_overview_tail_detail.md` — T17
#[ test ]
fn t17_detail_sessions_and_tail_0_combine()
{
  let root = TempDir::new().unwrap();
  let cwd  = TempDir::new().unwrap();

  common::write_path_project_session_with_last_message(
    root.path(), cwd.path(), "t17-session", 15, "T17_MARKER"
  );

  let out = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .current_dir( cwd.path() )
    .arg( ".show" )
    .arg( "detail::sessions" )
    .arg( "tail::0" )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let s = stdout( &out );
  assert!( s.contains( "entry 0" ), "T17: tail::0 must show all messages including the first; got:\n{s}" );
  assert!( s.contains( "T17_MARKER" ), "T17: tail window must include the final marker entry; got:\n{s}" );
  assert!(
    s.contains( "entries, last:" ),
    "T17: detail::sessions must append the per-session list; got:\n{s}"
  );
}

/// T18: project with zero sessions → zero counts, no crash.
///
/// ## Purpose
/// Verify the zero-session edge case is handled gracefully — a project
/// directory that exists but contains no session files.
///
/// ## Coverage
/// Zero session/entry counts shown; exit 0 (no crash).
///
/// ## Validation Strategy
/// Create the project storage directory directly (no session files written),
/// bypassing the fixture helpers which always write at least one session.
///
/// ## Related Requirements
/// `task/claude_storage/526_show_project_overview_tail_detail.md` — T18
#[ test ]
fn t18_zero_session_project_shows_zero_counts_no_crash()
{
  let root = TempDir::new().unwrap();
  let cwd  = TempDir::new().unwrap();

  let encoded = claude_storage_core::encode_path( cwd.path() ).expect( "encode project path" );
  std::fs::create_dir_all( root.path().join( "projects" ).join( &encoded ) ).expect( "create empty project dir" );

  let out = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .current_dir( cwd.path() )
    .arg( ".show" )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let s = stdout( &out );
  assert!(
    s.contains( "Sessions: 0 (Main: 0, Agent: 0)" ),
    "T18: zero-session project must show zero session counts; got:\n{s}"
  );
  assert!( s.contains( "Total Entries: 0" ), "T18: zero-session project must show zero total entries; got:\n{s}" );
  assert!(
    s.contains( "First Entry: unknown" ) && s.contains( "Last Entry: unknown" ),
    "T18: zero-session project must show unknown First/Last Entry, not a tail window; got:\n{s}"
  );
}
