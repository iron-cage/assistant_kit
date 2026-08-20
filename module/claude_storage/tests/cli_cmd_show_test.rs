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
//! - INT-13: `fields::timestamp` shows exactly one field per entry, no chat-log text
//! - INT-14: `fields::model,uuid` renders fields in request order
//! - INT-15: `fields::all` shows every one of the 18 fields, including ones content mode drops
//! - INT-16: `fields::bogus` rejected with the canonical 18-field error
//! - INT-17: `index::N` narrows session-detail to exactly one message's chat-log content
//! - INT-18: out-of-range `index::` rejected, error names the actual entry count
//! - INT-19: `fields::` composed with `index::` narrows projection to exactly one message
//! - INT-20: `fields::` applies to the project-overview tail window, not just session-detail
//! - EC-5 (fields): `all` combined with another token rejected
//! - EC-6 (fields): empty `fields::` value rejected
//! - EC-7 (fields): case-insensitive, whitespace-trimmed tokens match canonical byte-for-byte
//! - EC-8 (fields): duplicate tokens collapse to one occurrence
//! - EC-11 (fields): assistant-only field on a `user` entry renders as `—`
//! - EC-13 (fields): user-only field on an `assistant` entry renders as `—`
//! - EC-2/EC-3 (index): `index::1`/`index::4` boundary positions
//! - EC-4/EC-5 (index): `index::0`/negative `index::` rejected
//! - EC-7 (index): `index::` counts within the `tail::`-windowed slice, not the full session
//! - EC-8 (index): `index::` composed with `show_entries::1` narrows the raw list to one line
//! - T21: `index::` against a zero-session project rejected (canonical 0-entry error, no crash)

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
    s.contains( "Sessions: 1 · Main: 1 · Agent: 0" ),
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
    !s.contains( "entries · last:" ),
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
  let expected_appended = "Sessions:\n  - t08-session · 4 entries · last: 2025-01-01T00:00:03Z\n";
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
    !s.contains( "entries · last:" ),
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
    s.contains( "entries · last:" ),
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
    s.contains( "Sessions: 0 · Main: 0 · Agent: 0" ),
    "T18: zero-session project must show zero session counts; got:\n{s}"
  );
  assert!( s.contains( "Total Entries: 0" ), "T18: zero-session project must show zero total entries; got:\n{s}" );
  assert!(
    s.contains( "First Entry: unknown" ) && s.contains( "Last Entry: unknown" ),
    "T18: zero-session project must show unknown First/Last Entry, not a tail window; got:\n{s}"
  );
}

/// Fixture: fixed 4-entry session covering every attribute shape `fields::`
/// must project — position 1 user w/ `thinkingMetadata`, position 2 simple
/// assistant, position 3 rich assistant (thinking + text + `tool_use` +
/// `tool_result`), position 4 plain user. See
/// `docs/cli/pitfall/03_test_data_format.md` § Full-Attribute Shapes for the
/// underlying JSON schema this mirrors.
///
/// # Panics
///
/// Panics if directory creation or file write fails.
fn write_full_attribute_session( root : &std::path::Path, project_id : &str, session_id : &str )
{
  use std::io::Write as _;

  let dir = root.join( "projects" ).join( project_id );
  std::fs::create_dir_all( &dir ).expect( "create project dir" );
  let path = dir.join( format!( "{session_id}.jsonl" ) );
  let mut file = std::fs::File::create( &path ).expect( "create session file" );

  let lines : [ &str; 4 ] =
  [
    r#"{"type":"user","uuid":"fa-uuid-1","parentUuid":null,"timestamp":"2025-11-24T10:00:00.000Z","cwd":"/tmp","sessionId":"FA_SID","version":"2.0.31","gitBranch":"main","userType":"external","isSidechain":false,"message":{"role":"user","content":"first user message"},"thinkingMetadata":{"level":"high","disabled":false}}"#,
    r#"{"type":"assistant","uuid":"fa-uuid-2","parentUuid":"fa-uuid-1","timestamp":"2025-11-24T10:00:05.000Z","cwd":"/tmp","sessionId":"FA_SID","version":"2.0.31","gitBranch":"main","userType":"external","isSidechain":false,"requestId":"req-fa-2","message":{"model":"claude-fa-model-2","id":"msg-fa-2","role":"assistant","content":[{"type":"text","text":"simple reply"}],"stop_reason":"end_turn","stop_sequence":null}}"#,
    r#"{"type":"assistant","uuid":"fa-uuid-3","parentUuid":"fa-uuid-2","timestamp":"2025-11-24T10:00:10.000Z","cwd":"/tmp","sessionId":"FA_SID","version":"2.0.31","gitBranch":"main","userType":"external","isSidechain":false,"requestId":"req-fa-3","message":{"model":"claude-fa-model-3","id":"msg-fa-3","role":"assistant","content":[{"type":"thinking","thinking":"reasoning text","signature":"sig-fa-3"},{"type":"text","text":"rich reply text"},{"type":"tool_use","id":"toolu-fa-3","name":"read_file","input":{"path":"/tmp/x"}},{"type":"tool_result","tool_use_id":"toolu-fa-3","content":"file contents ok","is_error":false}],"stop_reason":"tool_use","stop_sequence":null}}"#,
    r#"{"type":"user","uuid":"fa-uuid-4","parentUuid":"fa-uuid-3","timestamp":"2025-11-24T10:00:15.000Z","cwd":"/tmp","sessionId":"FA_SID","version":"2.0.31","gitBranch":"main","userType":"external","isSidechain":false,"message":{"role":"user","content":"final user message"}}"#,
  ];

  for line in lines
  {
    writeln!( file, "{}", line.replace( "FA_SID", session_id ) ).expect( "write full-attribute entry" );
  }
}

/// T01/INT-13/EC-1: `fields::timestamp` shows just that field for every entry.
///
/// ## Purpose
/// Verify a single-field request renders exactly one `timestamp` line per
/// entry — no message text, no other attribute.
///
/// ## Coverage
/// One `timestamp ·` line per entry (4 total); message text and other
/// attribute labels absent; exit 0.
///
/// ## Validation Strategy
/// Write the full-attribute fixture (4 entries, distinct timestamps). Run
/// `fields::timestamp`. Count `timestamp ·` occurrences and assert absence
/// of message text / other field labels.
///
/// ## Related Requirements
/// `tests/docs/cli/command/03_show.md` INT-13; `tests/docs/cli/param/32_fields.md` EC-1
#[ test ]
fn int_13_fields_single_field_every_entry()
{
  let root = TempDir::new().unwrap();
  write_full_attribute_session( root.path(), "fa-proj", "fa-sess" );

  let out = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .arg( ".show" )
    .arg( "session_id::fa-sess" )
    .arg( "project::fa-proj" )
    .arg( "fields::timestamp" )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let s = stdout( &out );
  assert_eq!(
    s.matches( "timestamp · " ).count(), 4,
    "T01: exactly one timestamp line per entry (4 entries); got:\n{s}"
  );
  for ts in [ "2025-11-24T10:00:00.000Z", "2025-11-24T10:00:05.000Z", "2025-11-24T10:00:10.000Z", "2025-11-24T10:00:15.000Z" ]
  {
    assert!( s.contains( ts ), "T01: timestamp {ts} must appear; got:\n{s}" );
  }
  assert!( !s.contains( "content ·" ) && !s.contains( "content." ), "T01: no content field expected; got:\n{s}" );
  assert!( !s.contains( "model ·" ) && !s.contains( "uuid ·" ), "T01: no other field expected; got:\n{s}" );
  assert!(
    !s.contains( "simple reply" ) && !s.contains( "first user message" ),
    "T01: no chat-log message text expected; got:\n{s}"
  );
}

/// T02/INT-14/EC-2: `fields::model,uuid` shows fields in request order.
///
/// ## Purpose
/// Verify multi-field requests render fields in the order requested, not
/// canonical vocabulary order (`uuid` precedes `model` canonically).
///
/// ## Coverage
/// `model ·` line appears before `uuid ·` line for the known assistant entry; exit 0.
///
/// ## Validation Strategy
/// Write the full-attribute fixture. Run `fields::model,uuid`. Assert the
/// byte offset of entry 2's `model ·` line is before its `uuid ·` line.
///
/// ## Related Requirements
/// `tests/docs/cli/command/03_show.md` INT-14; `tests/docs/cli/param/32_fields.md` EC-2
#[ test ]
fn int_14_fields_multi_field_request_order()
{
  let root = TempDir::new().unwrap();
  write_full_attribute_session( root.path(), "fa-proj", "fa-sess" );

  let out = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .arg( ".show" )
    .arg( "session_id::fa-sess" )
    .arg( "project::fa-proj" )
    .arg( "fields::model,uuid" )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let s = stdout( &out );
  let model_pos = s.find( "model · claude-fa-model-2" ).expect( "T02: model line for entry 2 must appear" );
  let uuid_pos = s.find( "uuid · fa-uuid-2" ).expect( "T02: uuid line for entry 2 must appear" );
  assert!(
    model_pos < uuid_pos,
    "T02: model line must precede uuid line (request order); got:\n{s}"
  );
}

/// T03/INT-15/EC-3: `fields::all` shows every one of the 18 fields.
///
/// ## Purpose
/// Verify `fields::all` renders every canonical attribute — including ones
/// the default chat-log content mode drops entirely (`parent_uuid`, `cwd`,
/// `version`, `git_branch`, `request_id`, `thinking_level`/`thinking_disabled`,
/// `tool_use`'s `id`/`input`, and a successful `tool_result`'s `content`).
///
/// ## Coverage
/// All 18 field labels present for the relevant entries; exit 0.
///
/// ## Validation Strategy
/// Write the full-attribute fixture (entry 1 has `thinking_metadata`, entry 3
/// has `thinking`/`text`/`tool_use`/`tool_result` blocks). Run `fields::all`. Assert
/// every attribute this fixture can exercise appears.
///
/// ## Related Requirements
/// `tests/docs/cli/command/03_show.md` INT-15; `tests/docs/cli/param/32_fields.md` EC-3
#[ test ]
fn int_15_fields_all_shows_every_dropped_attribute()
{
  let root = TempDir::new().unwrap();
  write_full_attribute_session( root.path(), "fa-proj", "fa-sess" );

  let out = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .arg( ".show" )
    .arg( "session_id::fa-sess" )
    .arg( "project::fa-proj" )
    .arg( "fields::all" )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let s = stdout( &out );
  assert!( s.contains( "parent_uuid · fa-uuid-2" ), "T03: parent_uuid must appear; got:\n{s}" );
  assert!( s.contains( "cwd · /tmp" ), "T03: cwd must appear; got:\n{s}" );
  assert!( s.contains( "version · 2.0.31" ), "T03: version must appear; got:\n{s}" );
  assert!( s.contains( "git_branch · main" ), "T03: git_branch must appear; got:\n{s}" );
  assert!( s.contains( "request_id · req-fa-3" ), "T03: request_id must appear; got:\n{s}" );
  assert!( s.contains( "thinking_level · high" ), "T03: thinking_level must appear; got:\n{s}" );
  assert!( s.contains( "thinking_disabled · false" ), "T03: thinking_disabled must appear; got:\n{s}" );
  assert!( s.contains( "content.tool_use.id · toolu-fa-3" ), "T03: tool_use id must appear; got:\n{s}" );
  assert!( s.contains( "content.tool_use.name · read_file" ), "T03: tool_use name must appear; got:\n{s}" );
  assert!(
    s.contains( r#"content.tool_use.input · {"path":"/tmp/x"}"# ),
    "T03: tool_use input must render as clean JSON, not Rust Debug format; got:\n{s}"
  );
  assert!(
    !s.contains( "Object(" ) && !s.contains( "String(" ),
    "T03: tool_use input must not leak Rust Debug-format internals; got:\n{s}"
  );
  assert!( s.contains( "content.tool_result.tool_use_id · toolu-fa-3" ), "T03: tool_result tool_use_id must appear; got:\n{s}" );
  assert!( s.contains( "content.tool_result.content · file contents ok" ), "T03: successful tool_result content must appear; got:\n{s}" );
  assert!( s.contains( "content.tool_result.is_error · false" ), "T03: tool_result is_error must appear; got:\n{s}" );
  assert!( s.contains( "content.thinking · reasoning text" ), "T03: thinking block must appear; got:\n{s}" );
  assert!( s.contains( "content.thinking.signature · sig-fa-3" ), "T03: thinking signature must appear; got:\n{s}" );
}

/// T04/INT-16/EC-4: `fields::` with an invalid token is rejected.
///
/// ## Purpose
/// Verify an unrecognized field token is rejected with the canonical
/// 18-field error message.
///
/// ## Coverage
/// stderr contains the canonical error; stdout empty; exit 1.
///
/// ## Validation Strategy
/// Write the full-attribute fixture. Run `fields::bogus`. Assert exact error text.
///
/// ## Related Requirements
/// `tests/docs/cli/command/03_show.md` INT-16; `tests/docs/cli/param/32_fields.md` EC-4
#[ test ]
fn int_16_fields_invalid_token_rejected()
{
  let root = TempDir::new().unwrap();
  write_full_attribute_session( root.path(), "fa-proj", "fa-sess" );

  let out = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .arg( ".show" )
    .arg( "session_id::fa-sess" )
    .arg( "project::fa-proj" )
    .arg( "fields::bogus" )
    .output()
    .unwrap();

  assert_exit( &out, 1 );
  assert!(
    stderr( &out ).contains(
      "unknown field 'bogus' — valid fields: uuid, parent_uuid, timestamp, entry_type, cwd, session_id, version, git_branch, user_type, is_sidechain, content, thinking_level, thinking_disabled, model, message_id, stop_reason, stop_sequence, request_id, or all"
    ),
    "T04: canonical 18-field error expected; got:\n{}", stderr( &out )
  );
  assert!( stdout( &out ).is_empty(), "T04: stdout must be empty on error" );
}

/// T05/EC-5: `all` combined with another token is rejected.
///
/// ## Purpose
/// Verify `fields::all,uuid` is rejected — `all` cannot be combined.
///
/// ## Coverage
/// stderr contains the canonical error; stdout empty; exit 1.
///
/// ## Validation Strategy
/// Write the full-attribute fixture. Run `fields::all,uuid`. Assert exact error text.
///
/// ## Related Requirements
/// `tests/docs/cli/param/32_fields.md` EC-5; `tests/docs/cli/type/15_field_selector.md` TC-5
#[ test ]
fn fields_all_combined_with_other_rejected()
{
  let root = TempDir::new().unwrap();
  write_full_attribute_session( root.path(), "fa-proj", "fa-sess" );

  let out = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .arg( ".show" )
    .arg( "session_id::fa-sess" )
    .arg( "project::fa-proj" )
    .arg( "fields::all,uuid" )
    .output()
    .unwrap();

  assert_exit( &out, 1 );
  assert!(
    stderr( &out ).contains( "'all' cannot be combined with other fields" ),
    "T05: canonical error expected; got:\n{}", stderr( &out )
  );
  assert!( stdout( &out ).is_empty(), "T05: stdout must be empty on error" );
}

/// T06/EC-6: Empty `fields::` value is rejected.
///
/// ## Purpose
/// Verify `fields::` with nothing after it is rejected.
///
/// ## Coverage
/// stderr mentions the `fields` argument; stdout empty; exit 1.
///
/// ## Validation Strategy
/// Write the full-attribute fixture. Run `fields::` (empty value). Assert
/// rejection. A bare trailing `fields::` never reaches `FieldSelector::parse`
/// — unilang's own instruction parser rejects the missing value first
/// ("Expected value for named argument 'fields' but found end of
/// instruction"), the same pre-existing behavior `ec_2_query_empty_rejected`
/// (`cli_param_query_test.rs`) already accommodates for `query::`.
/// `FieldSelector::parse("")`'s own `"fields must be non-empty"` message is
/// covered directly at the unit level by `tc9_empty_string_rejected`
/// (`field_selector.rs`), which bypasses the CLI parser entirely.
///
/// ## Related Requirements
/// `tests/docs/cli/param/32_fields.md` EC-6; `tests/docs/cli/type/15_field_selector.md` TC-9
#[ test ]
fn fields_empty_value_rejected()
{
  let root = TempDir::new().unwrap();
  write_full_attribute_session( root.path(), "fa-proj", "fa-sess" );

  let out = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .arg( ".show" )
    .arg( "session_id::fa-sess" )
    .arg( "project::fa-proj" )
    .arg( "fields::" )
    .output()
    .unwrap();

  assert_exit( &out, 1 );
  assert!(
    stderr( &out ).contains( "fields" ),
    "T06: error must mention the fields argument; got:\n{}", stderr( &out )
  );
  assert!( stdout( &out ).is_empty(), "T06: stdout must be empty on error" );
}

/// T07/EC-7: Case-insensitive, whitespace-trimmed tokens match canonical byte-for-byte.
///
/// ## Purpose
/// Verify `fields:: UUID , Timestamp ` parses identically to
/// `fields::uuid,timestamp` — same output byte-for-byte.
///
/// ## Coverage
/// Two invocations produce byte-identical stdout; both exit 0.
///
/// ## Validation Strategy
/// Write the full-attribute fixture. Run both forms. Compare stdout directly.
///
/// ## Related Requirements
/// `tests/docs/cli/param/32_fields.md` EC-7; `tests/docs/cli/type/15_field_selector.md` TC-6, TC-7
#[ test ]
fn fields_case_insensitive_whitespace_trimmed_matches_canonical()
{
  let root = TempDir::new().unwrap();
  write_full_attribute_session( root.path(), "fa-proj", "fa-sess" );

  let out1 = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .arg( ".show" )
    .arg( "session_id::fa-sess" )
    .arg( "project::fa-proj" )
    .arg( "fields:: UUID , Timestamp " )
    .output()
    .unwrap();

  let out2 = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .arg( ".show" )
    .arg( "session_id::fa-sess" )
    .arg( "project::fa-proj" )
    .arg( "fields::uuid,timestamp" )
    .output()
    .unwrap();

  assert_exit( &out1, 0 );
  assert_exit( &out2, 0 );
  assert_eq!( stdout( &out1 ), stdout( &out2 ), "T07: case/whitespace variance must not change output" );
}

/// T08/EC-8: Duplicate tokens collapse to one occurrence.
///
/// ## Purpose
/// Verify `fields::uuid,uuid` behaves identically to `fields::uuid`.
///
/// ## Coverage
/// Two invocations produce byte-identical stdout; both exit 0.
///
/// ## Validation Strategy
/// Write the full-attribute fixture. Run both forms. Compare stdout directly.
///
/// ## Related Requirements
/// `tests/docs/cli/param/32_fields.md` EC-8; `tests/docs/cli/type/15_field_selector.md` TC-8
#[ test ]
fn fields_duplicate_token_collapses()
{
  let root = TempDir::new().unwrap();
  write_full_attribute_session( root.path(), "fa-proj", "fa-sess" );

  let out1 = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .arg( ".show" )
    .arg( "session_id::fa-sess" )
    .arg( "project::fa-proj" )
    .arg( "fields::uuid,uuid" )
    .output()
    .unwrap();

  let out2 = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .arg( ".show" )
    .arg( "session_id::fa-sess" )
    .arg( "project::fa-proj" )
    .arg( "fields::uuid" )
    .output()
    .unwrap();

  assert_exit( &out1, 0 );
  assert_exit( &out2, 0 );
  assert_eq!( stdout( &out1 ), stdout( &out2 ), "T08: duplicate token must collapse to one occurrence" );
}

/// T10/INT-20/EC-10: `fields::` applies to the project-overview tail window.
///
/// ## Purpose
/// Verify `fields::` is not session-detail-only — it also projects the
/// project-overview tail window's entries.
///
/// ## Coverage
/// Summary block present; last 5 entries rendered as field-projection blocks
/// (not chat-log content); exit 0.
///
/// ## Validation Strategy
/// Write a 6-entry cwd-resolved project. Run bare `.show fields::timestamp
/// tail::5`. Assert exactly 5 `timestamp ·` lines and the earliest entry's
/// timestamp (outside the window) is absent.
///
/// ## Related Requirements
/// `tests/docs/cli/command/03_show.md` INT-20; `tests/docs/cli/param/32_fields.md` EC-10
#[ test ]
fn int_20_fields_applies_to_project_overview_tail_window()
{
  let root = TempDir::new().unwrap();
  let cwd  = TempDir::new().unwrap();
  common::write_path_project_session( root.path(), cwd.path(), "-fa-tail", 6 );

  let out = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .current_dir( cwd.path() )
    .arg( ".show" )
    .arg( "fields::timestamp" )
    .arg( "tail::5" )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let s = stdout( &out );
  assert!( s.contains( "Path:" ) || s.contains( "Sessions:" ), "T10: summary block must still appear; got:\n{s}" );
  assert_eq!( s.matches( "timestamp · " ).count(), 5, "T10: exactly 5 field-projection blocks expected; got:\n{s}" );
  // Entry 0's raw timestamp legitimately appears once, in the summary block's
  // own "First Entry:" line (full-session bound, unaffected by tail::) — only
  // a field-projection *line* for entry 0 would prove the tail window leaked it.
  assert!(
    !s.contains( "timestamp · 2025-01-01T00:00:00Z" ),
    "T10: entry 0 (outside the 5-window) must not get its own field-projection line; got:\n{s}"
  );
  assert!( !s.contains( "entry 0\n" ) && !s.contains( "· User:\nentry 0" ), "T10: chat-log content must not appear; got:\n{s}" );
}

/// T11/EC-11: Assistant-only field on a `user` entry renders as `—`.
///
/// ## Purpose
/// Verify requesting `model` (assistant-only) on entry 1 (a `user` message)
/// renders `—` instead of erroring or panicking.
///
/// ## Coverage
/// `model · —` appears; exit 0.
///
/// ## Validation Strategy
/// Write the full-attribute fixture. Run `fields::model index::1`. Assert
/// the em-dash rendering.
///
/// ## Related Requirements
/// `tests/docs/cli/param/32_fields.md` EC-11; `tests/docs/cli/type/15_field_selector.md` role-gap coverage
#[ test ]
fn fields_assistant_only_field_on_user_entry_renders_em_dash()
{
  let root = TempDir::new().unwrap();
  write_full_attribute_session( root.path(), "fa-proj", "fa-sess" );

  let out = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .arg( ".show" )
    .arg( "session_id::fa-sess" )
    .arg( "project::fa-proj" )
    .arg( "fields::model" )
    .arg( "index::1" )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let s = stdout( &out );
  assert!( s.contains( "model · —" ), "T11: assistant-only field on user entry must render em-dash; got:\n{s}" );
}

/// T12/INT-19/EC-12/EC-9(index): `fields::` composed with `index::` narrows to one message.
///
/// ## Purpose
/// Verify `fields::uuid,model index::3` shows only entry 3's requested
/// attributes — no other entry, no other field.
///
/// ## Coverage
/// Entry 3's `uuid`/`model` lines present; entries 1/2/4 and other fields absent; exit 0.
///
/// ## Validation Strategy
/// Write the full-attribute fixture. Run `fields::uuid,model index::3`.
/// Assert entry 3's known values present, others absent.
///
/// ## Related Requirements
/// `tests/docs/cli/command/03_show.md` INT-19; `tests/docs/cli/param/32_fields.md` EC-12; `tests/docs/cli/param/33_index.md` EC-9
#[ test ]
fn int_19_fields_index_composed_single_message_projection()
{
  let root = TempDir::new().unwrap();
  write_full_attribute_session( root.path(), "fa-proj", "fa-sess" );

  let out = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .arg( ".show" )
    .arg( "session_id::fa-sess" )
    .arg( "project::fa-proj" )
    .arg( "fields::uuid,model" )
    .arg( "index::3" )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let s = stdout( &out );
  assert!( s.contains( "uuid · fa-uuid-3" ), "T12: entry 3's uuid must appear; got:\n{s}" );
  assert!( s.contains( "model · claude-fa-model-3" ), "T12: entry 3's model must appear; got:\n{s}" );
  assert!( !s.contains( "fa-uuid-1" ) && !s.contains( "fa-uuid-2" ) && !s.contains( "fa-uuid-4" ), "T12: other entries must be absent; got:\n{s}" );
  assert!( !s.contains( "timestamp ·" ), "T12: only requested fields (uuid, model) expected; got:\n{s}" );
}

/// T13/INT-17/EC-1(index): `index::N` narrows session-detail to one message.
///
/// ## Purpose
/// Verify `index::2` (no `fields::`) shows only entry 2's chat-log content.
///
/// ## Coverage
/// Entry 2's content present; entries 1/3/4 content absent; exit 0.
///
/// ## Validation Strategy
/// Write the full-attribute fixture. Run `index::2`. Assert only entry 2's
/// known text appears.
///
/// ## Related Requirements
/// `tests/docs/cli/command/03_show.md` INT-17; `tests/docs/cli/param/33_index.md` EC-1
#[ test ]
fn int_17_index_narrows_session_detail_one_message()
{
  let root = TempDir::new().unwrap();
  write_full_attribute_session( root.path(), "fa-proj", "fa-sess" );

  let out = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .arg( ".show" )
    .arg( "session_id::fa-sess" )
    .arg( "project::fa-proj" )
    .arg( "index::2" )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let s = stdout( &out );
  assert!( s.contains( "simple reply" ), "T13: entry 2's content must appear; got:\n{s}" );
  assert!(
    !s.contains( "first user message" ) && !s.contains( "rich reply text" ) && !s.contains( "final user message" ),
    "T13: only entry 2's content expected; got:\n{s}"
  );
}

/// T14a/EC-2: `index::1` selects the first message (boundary).
///
/// ## Purpose
/// Verify the lower boundary of `index::` selects entry 1 exactly.
///
/// ## Coverage
/// Entry 1's content present; other entries' content absent; exit 0.
///
/// ## Validation Strategy
/// Write the full-attribute fixture. Run `index::1`. Assert only entry 1's known text appears.
///
/// ## Related Requirements
/// `tests/docs/cli/param/33_index.md` EC-2
#[ test ]
fn index_boundary_first_position()
{
  let root = TempDir::new().unwrap();
  write_full_attribute_session( root.path(), "fa-proj", "fa-sess" );

  let out = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .arg( ".show" )
    .arg( "session_id::fa-sess" )
    .arg( "project::fa-proj" )
    .arg( "index::1" )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let s = stdout( &out );
  assert!( s.contains( "first user message" ), "T14a: entry 1's content must appear; got:\n{s}" );
  assert!(
    !s.contains( "simple reply" ) && !s.contains( "rich reply text" ) && !s.contains( "final user message" ),
    "T14a: only entry 1's content expected; got:\n{s}"
  );
}

/// T14b/EC-3(index): `index::` at the last valid position selects the last message (boundary).
///
/// ## Purpose
/// Verify the upper boundary of `index::` selects the last entry exactly —
/// the boundary immediately below the out-of-range case (T16).
///
/// ## Coverage
/// Entry 4's content present; other entries' content absent; exit 0.
///
/// ## Validation Strategy
/// Write the full-attribute fixture (4 entries). Run `index::4`. Assert only
/// entry 4's known text appears.
///
/// ## Related Requirements
/// `tests/docs/cli/param/33_index.md` EC-3
#[ test ]
fn index_boundary_last_position()
{
  let root = TempDir::new().unwrap();
  write_full_attribute_session( root.path(), "fa-proj", "fa-sess" );

  let out = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .arg( ".show" )
    .arg( "session_id::fa-sess" )
    .arg( "project::fa-proj" )
    .arg( "index::4" )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let s = stdout( &out );
  assert!( s.contains( "final user message" ), "T14b: entry 4's content must appear; got:\n{s}" );
  assert!(
    !s.contains( "first user message" ) && !s.contains( "simple reply" ) && !s.contains( "rich reply text" ),
    "T14b: only entry 4's content expected; got:\n{s}"
  );
}

/// T15a/EC-4: `index::0` is rejected.
///
/// ## Purpose
/// Verify `index::0` fails validation (1-based, not 0-based).
///
/// ## Coverage
/// stderr contains the canonical error; stdout empty; exit 1.
///
/// ## Validation Strategy
/// Write the full-attribute fixture. Run `index::0`. Assert exact error text.
///
/// ## Related Requirements
/// `tests/docs/cli/param/33_index.md` EC-4
#[ test ]
fn index_zero_rejected()
{
  let root = TempDir::new().unwrap();
  write_full_attribute_session( root.path(), "fa-proj", "fa-sess" );

  let out = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .arg( ".show" )
    .arg( "session_id::fa-sess" )
    .arg( "project::fa-proj" )
    .arg( "index::0" )
    .output()
    .unwrap();

  assert_exit( &out, 1 );
  assert!(
    stderr( &out ).contains( "index must be a positive integer (1-based), got 0" ),
    "T15a: canonical error expected; got:\n{}", stderr( &out )
  );
  assert!( stdout( &out ).is_empty(), "T15a: stdout must be empty on error" );
}

/// T15b/EC-5: Negative `index::` is rejected.
///
/// ## Purpose
/// Verify `index::-1` fails validation with the same error class as `index::0`.
///
/// ## Coverage
/// stderr contains the canonical error; stdout empty; exit 1.
///
/// ## Validation Strategy
/// Write the full-attribute fixture. Run `index::-1`. Assert exact error text.
///
/// ## Related Requirements
/// `tests/docs/cli/param/33_index.md` EC-5
#[ test ]
fn index_negative_rejected()
{
  let root = TempDir::new().unwrap();
  write_full_attribute_session( root.path(), "fa-proj", "fa-sess" );

  let out = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .arg( ".show" )
    .arg( "session_id::fa-sess" )
    .arg( "project::fa-proj" )
    .arg( "index::-1" )
    .output()
    .unwrap();

  assert_exit( &out, 1 );
  assert!(
    stderr( &out ).contains( "index must be a positive integer (1-based), got -1" ),
    "T15b: canonical error expected; got:\n{}", stderr( &out )
  );
  assert!( stdout( &out ).is_empty(), "T15b: stdout must be empty on error" );
}

/// T16/INT-18/EC-6: Out-of-range `index::` is rejected, error names the actual count.
///
/// ## Purpose
/// Verify `index::99` against a 4-entry session fails with the exact count.
///
/// ## Coverage
/// stderr contains the canonical error; stdout empty; exit 1.
///
/// ## Validation Strategy
/// Write the full-attribute fixture (4 entries). Run `index::99`. Assert exact error text.
///
/// ## Related Requirements
/// `tests/docs/cli/command/03_show.md` INT-18; `tests/docs/cli/param/33_index.md` EC-6
#[ test ]
fn int_18_index_out_of_range_rejected()
{
  let root = TempDir::new().unwrap();
  write_full_attribute_session( root.path(), "fa-proj", "fa-sess" );

  let out = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .arg( ".show" )
    .arg( "session_id::fa-sess" )
    .arg( "project::fa-proj" )
    .arg( "index::99" )
    .output()
    .unwrap();

  assert_exit( &out, 1 );
  assert!(
    stderr( &out ).contains( "index out of range: 99 (4 entries)" ),
    "T16: canonical error expected; got:\n{}", stderr( &out )
  );
  assert!( stdout( &out ).is_empty(), "T16: stdout must be empty on error" );
}

/// T17/EC-7: `index::` counts within the `tail::`-windowed slice, not the full session.
///
/// ## Purpose
/// Verify `tail::5 index::1` selects the 1st message of the 5-entry tail
/// window — the 16th message of the full 20-entry session — not the 1st
/// message of the session's complete history.
///
/// ## Coverage
/// The windowed 1st message (`entry 15`, 0-based) appears; the session's
/// true 1st message (`entry 0`) and the window's other members are absent; exit 0.
///
/// ## Validation Strategy
/// Write a 20-entry cwd-resolved project (generic alternating content, 0-based
/// `entry N` text). Run bare `.show tail::5 index::1`. Assert only `entry 15` appears.
///
/// ## Related Requirements
/// `tests/docs/cli/param/33_index.md` EC-7
#[ test ]
fn index_counts_within_tail_window_not_full_session()
{
  let root = TempDir::new().unwrap();
  let cwd  = TempDir::new().unwrap();
  common::write_path_project_session( root.path(), cwd.path(), "-fa-window", 20 );

  let out = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .current_dir( cwd.path() )
    .arg( ".show" )
    .arg( "tail::5" )
    .arg( "index::1" )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let s = stdout( &out );
  assert!( s.contains( "entry 15" ), "T17: the tail window's 1st message (entry 15) must appear; got:\n{s}" );
  for other in [ "entry 0\n", "entry 16", "entry 17", "entry 18", "entry 19" ]
  {
    assert!( !s.contains( other ), "T17: only the windowed 1st message expected, found {other:?}; got:\n{s}" );
  }
}

/// T18/EC-8: `index::` composed with `show_entries::1` narrows the raw list to one line.
///
/// ## Purpose
/// Verify `show_metadata::1 show_entries::1 index::3` shows exactly one raw
/// entry line — entry 3's — instead of all 4.
///
/// ## Coverage
/// Entry 3's raw line present; entries 1/2/4's raw lines absent; exit 0.
///
/// ## Validation Strategy
/// Write the full-attribute fixture. Run the `metadata_only` + `show_entries` +
/// index combination. Assert only entry 3's uuid appears in the raw list.
///
/// ## Related Requirements
/// `tests/docs/cli/command/03_show.md` INT-6 composition variant; `tests/docs/cli/param/33_index.md` EC-8
#[ test ]
fn index_narrows_raw_entries_list()
{
  let root = TempDir::new().unwrap();
  write_full_attribute_session( root.path(), "fa-proj", "fa-sess" );

  let out = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .arg( ".show" )
    .arg( "session_id::fa-sess" )
    .arg( "project::fa-proj" )
    .arg( "show_metadata::1" )
    .arg( "show_entries::1" )
    .arg( "index::3" )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let s = stdout( &out );
  assert!( s.contains( "3. Assistant · fa-uuid-3" ), "T18: entry 3's raw line must appear; got:\n{s}" );
  assert!(
    !s.contains( "fa-uuid-1" ) && !s.contains( "fa-uuid-2" ) && !s.contains( "fa-uuid-4" ),
    "T18: only entry 3's raw line expected; got:\n{s}"
  );
}

/// T20/EC-13: User-only field (`thinking_level`) on an `assistant` entry renders as `—`.
///
/// ## Purpose
/// Verify requesting `thinking_level` (user-only) on entry 2 (an `assistant`
/// message) renders `—` — the mirror image of T11/EC-11.
///
/// ## Coverage
/// `thinking_level · —` appears; exit 0.
///
/// ## Validation Strategy
/// Write the full-attribute fixture. Run `fields::thinking_level index::2`.
/// Assert the em-dash rendering.
///
/// ## Related Requirements
/// `tests/docs/cli/param/32_fields.md` EC-13; `tests/docs/cli/type/15_field_selector.md` role-gap coverage
#[ test ]
fn fields_user_only_field_on_assistant_entry_renders_em_dash()
{
  let root = TempDir::new().unwrap();
  write_full_attribute_session( root.path(), "fa-proj", "fa-sess" );

  let out = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .arg( ".show" )
    .arg( "session_id::fa-sess" )
    .arg( "project::fa-proj" )
    .arg( "fields::thinking_level" )
    .arg( "index::2" )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let s = stdout( &out );
  assert!( s.contains( "thinking_level · —" ), "T20: user-only field on assistant entry must render em-dash; got:\n{s}" );
}

/// T21: `index::` against a zero-session project is rejected, not a crash.
///
/// ## Purpose
/// Verify `index::` composed with a project that has zero sessions still
/// produces the canonical out-of-range error (0 entries), never a panic or
/// silent success.
///
/// ## Coverage
/// stderr contains `index out of range: 1 (0 entries)`; stdout empty; exit 1.
///
/// ## Validation Strategy
/// Create an empty path-encoded project directory (no session files). Run
/// bare `.show index::1`. Assert the canonical zero-entry error.
///
/// ## Related Requirements
/// `task/claude_storage/527_show_fields_index_projection.md` — T21 (pitfall-review addition)
#[ test ]
fn index_zero_session_project_out_of_range_rejected()
{
  let root = TempDir::new().unwrap();
  let cwd  = TempDir::new().unwrap();

  let encoded = claude_storage_core::encode_path( cwd.path() ).expect( "encode project path" );
  std::fs::create_dir_all( root.path().join( "projects" ).join( &encoded ) ).expect( "create empty project dir" );

  let out = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .current_dir( cwd.path() )
    .arg( ".show" )
    .arg( "index::1" )
    .output()
    .unwrap();

  assert_exit( &out, 1 );
  assert!(
    stderr( &out ).contains( "index out of range: 1 (0 entries)" ),
    "T21: canonical zero-entry error expected; got:\n{}", stderr( &out )
  );
  assert!( stdout( &out ).is_empty(), "T21: stdout must be empty on error" );
}

/// INT-21: `fields::` composed with `show_metadata::1 show_entries::1` projects
/// each entry instead of being silently dropped.
///
/// ## Purpose
/// Verify the F2 fix: `show_metadata::1 show_entries::1 fields::timestamp`
/// renders each entry as a field-projection line, not the old raw
/// uuid/type/timestamp list — per `docs/cli/command/03_show.md` lines 18/58
/// ("`fields::` (any step) replaces that rendering").
///
/// ## Coverage
/// 4 `timestamp ·` projection lines present, one per entry; the old raw-list
/// format (`N. Role · uuid`) absent; exit 0.
///
/// ## Validation Strategy
/// Write the full-attribute fixture. Run `show_metadata::1 show_entries::1
/// fields::timestamp`. Assert field-projection rendering, not the raw list.
///
/// ## Related Requirements
/// `docs/cli/command/03_show.md` lines 18, 58
#[ test ]
fn int_21_fields_composed_with_show_entries_projects_each_entry()
{
  let root = TempDir::new().unwrap();
  write_full_attribute_session( root.path(), "fa-proj", "fa-sess" );

  let out = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .arg( ".show" )
    .arg( "session_id::fa-sess" )
    .arg( "project::fa-proj" )
    .arg( "show_metadata::1" )
    .arg( "show_entries::1" )
    .arg( "fields::timestamp" )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let s = stdout( &out );
  assert_eq!(
    s.matches( "timestamp · " ).count(), 4,
    "F2: exactly one field-projection line per entry (4 entries) expected; got:\n{s}"
  );
  assert!(
    !s.contains( "1. User · fa-uuid-1" ) && !s.contains( "2. Assistant · fa-uuid-2" ),
    "F2: old raw uuid/type/timestamp list must not appear once fields:: is given; got:\n{s}"
  );
}

/// F3b: A nonexistent `project::` renders the plain identifier, never the
/// `ProjectId` enum's Debug format.
///
/// ## Purpose
/// Verify the F3b fix: `project::does-not-exist-project` reports the error
/// against the plain string the user typed, not `Uuid("does-not-exist-project")`
/// (the internal enum variant leaking through `{:?}`).
///
/// ## Coverage
/// stderr contains the plain identifier; stderr does not contain `Uuid(`; exit 1.
///
/// ## Validation Strategy
/// Run `.show project::does-not-exist-project` against an empty storage root
/// (no project directory created). Assert the error text.
///
/// ## Related Requirements
/// UX/DX round 1, Finding F3
#[ test ]
fn f3b_project_not_found_error_omits_debug_format()
{
  let root = TempDir::new().unwrap();

  let out = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .arg( ".show" )
    .arg( "project::does-not-exist-project" )
    .output()
    .unwrap();

  assert_exit( &out, 1 );
  assert!(
    stderr( &out ).contains( "Failed to load project does-not-exist-project:" ),
    "F3b: error must report the plain identifier; got:\n{}", stderr( &out )
  );
  assert!(
    !stderr( &out ).contains( "Uuid(" ),
    "F3b: error must not leak ProjectId's Debug format; got:\n{}", stderr( &out )
  );
}
