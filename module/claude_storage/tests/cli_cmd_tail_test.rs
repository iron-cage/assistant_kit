//! Integration tests for the `.tail` command.
//!
//! ## Source
//!
//! - Command spec: `tests/docs/cli/command/12_tail.md`
//! - Param spec: `tests/docs/cli/param/25_last.md`, `tests/docs/cli/param/42_full.md`,
//!   `tests/docs/cli/param/43_compact.md`
//!
//! ## Coverage
//!
//! Two families. INT-1..INT-11 cover **resolution** — which session `.tail` picks
//! and how much of it the window takes. INT-12..INT-23 cover **rendering** — how
//! records become turns and how those turns are drawn. The two use different
//! fixtures: resolution tests build sessions with `common::write_test_session`
//! (entry counts matter, shapes do not); rendering tests write raw JSONL via
//! `common::write_raw_session` (shapes are the whole point).
//!
//! - INT-1: No args prints last 4 entries of `default_topic` session (also covers EC-1)
//! - INT-2: `last::N` controls entry count (also covers EC-2)
//! - INT-3: `last::0` prints all entries (also covers EC-3)
//! - INT-4: `topic::` resolves a non-default session
//! - INT-5: `path::` resolves a different directory's project
//! - INT-6: Fewer entries than requested prints all available (also covers EC-6)
//! - INT-7: Exit code 2 when cwd has no project
//! - INT-8: Negative `last::` is rejected with exit code 1 (also covers EC-4)
//! - INT-9: No args falls back to the most recent session when no `-default_topic` session exists
//! - INT-10: No args picks the most recently modified session among multiple candidates
//! - INT-11: No args excludes agent sessions from the most-recent fallback
//! - INT-12: Consecutive records sharing one `message.id` collapse into one turn
//! - INT-13: Array-form user `message.content` is parsed, not silently dropped
//! - INT-14: A tool call renders its input summary and its result's line count
//! - INT-15: A turn holding only `tool_result` blocks never consumes a `last::` slot
//! - INT-16: Empty text and thinking blocks render nothing, not a bare label
//! - INT-17: Turns past 8 body lines fold; `full::1` unfolds them (also covers `42_full.md` EC-1)
//! - INT-18: `compact::1` prints one line per turn (also covers `43_compact.md` EC-1)
//! - INT-18b: `compact::1 full::1` — compact wins (also covers `42_full.md` and `43_compact.md` EC-2)
//! - INT-19: Session header reports project, session id, and turn span
//! - INT-20: Output ends with exactly one newline
//! - INT-21: An unmodelled block type is marked, not dropped along with its record
//! - INT-22: A failed tool call is annotated `↳ error`
//! - INT-23: Array-form `tool_result.content` flattens instead of rejecting the record
//! - INT-24: A tool with no path/command key still summarises (`status` outranks `taskId`)
//! - EC-5: Empty value rejected
//! - EC-7: Non-integer value rejected
//! - EC-8: `l::N` alias produces byte-identical output to `last::N`
// BUG-002 — real assertions replacing the "didn't hang" cheating tests
// BUG-488 — INT-9..INT-11 added for the -default_topic fallback fix

mod common;

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

/// INT-1: No args prints last 4 entries of `default_topic` session.
///
/// ## Purpose
/// Validates the zero-parameter default: current directory's project,
/// `-default_topic` session, last 4 entries.
///
/// ## Coverage
/// Exit 0; last 4 of 6 entries shown (entries 2-5), oldest-first; entries 0-1 absent.
/// Also covers EC-1 (`tests/docs/cli/param/25_last.md`) — identical scenario.
///
/// ## Related Requirements
/// `tests/docs/cli/command/12_tail.md` — INT-1
/// `tests/docs/cli/param/25_last.md` — EC-1
#[ test ]
fn int_1_no_args_shows_last_4_of_default_topic()
{
  let root = tempfile::TempDir::new().unwrap();
  let cwd  = tempfile::TempDir::new().unwrap();

  common::write_path_project_session( root.path(), cwd.path(), "-default_topic", 6 );

  let out = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .current_dir( cwd.path() )
    .arg( ".tail" )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let text = stdout( &out );
  for i in 2..6
  {
    assert!( text.contains( &format!( "entry {i}" ) ), "expected entry {i} in output: {text}" );
  }
  for i in 0..2
  {
    assert!( !text.contains( &format!( "entry {i}" ) ), "did not expect entry {i} in output: {text}" );
  }
}

/// INT-2: `last::N` controls entry count.
///
/// ## Purpose
/// Validates that `last::2` shows exactly the last 2 entries.
///
/// ## Coverage
/// Exit 0; last 2 of 6 entries shown (entries 4-5); entries 0-3 absent.
/// Also covers EC-2 (`tests/docs/cli/param/25_last.md`) — identical scenario.
///
/// ## Related Requirements
/// `tests/docs/cli/command/12_tail.md` — INT-2
/// `tests/docs/cli/param/25_last.md` — EC-2
#[ test ]
fn int_2_last_n_controls_entry_count()
{
  let root = tempfile::TempDir::new().unwrap();
  let cwd  = tempfile::TempDir::new().unwrap();

  common::write_path_project_session( root.path(), cwd.path(), "-default_topic", 6 );

  let out = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .current_dir( cwd.path() )
    .arg( ".tail" )
    .arg( "last::2" )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let text = stdout( &out );
  for i in 4..6
  {
    assert!( text.contains( &format!( "entry {i}" ) ), "expected entry {i} in output: {text}" );
  }
  for i in 0..4
  {
    assert!( !text.contains( &format!( "entry {i}" ) ), "did not expect entry {i} in output: {text}" );
  }
}

/// INT-3: `last::0` prints all entries.
///
/// ## Purpose
/// Validates that `last::0` disables the cap and shows every entry.
///
/// ## Coverage
/// Exit 0; all 6 entries shown, oldest-first.
/// Also covers EC-3 (`tests/docs/cli/param/25_last.md`) — identical scenario.
///
/// ## Related Requirements
/// `tests/docs/cli/command/12_tail.md` — INT-3
/// `tests/docs/cli/param/25_last.md` — EC-3
#[ test ]
fn int_3_last_zero_prints_all_entries()
{
  let root = tempfile::TempDir::new().unwrap();
  let cwd  = tempfile::TempDir::new().unwrap();

  common::write_path_project_session( root.path(), cwd.path(), "-default_topic", 6 );

  let out = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .current_dir( cwd.path() )
    .arg( ".tail" )
    .arg( "last::0" )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let text = stdout( &out );
  for i in 0..6
  {
    assert!( text.contains( &format!( "entry {i}" ) ), "expected entry {i} in output: {text}" );
  }
}

/// INT-4: `topic::` resolves a non-default session.
///
/// ## Purpose
/// Validates that `topic::work` reads the `-work` session instead of `-default_topic`.
///
/// ## Coverage
/// Exit 0; `-work` session's distinct content shown; `-default_topic` content absent.
///
/// ## Related Requirements
/// `tests/docs/cli/command/12_tail.md` — INT-4
#[ test ]
fn int_4_topic_resolves_non_default_session()
{
  let root = tempfile::TempDir::new().unwrap();
  let cwd  = tempfile::TempDir::new().unwrap();

  let encoded = claude_storage_core::encode_path( cwd.path() ).unwrap();
  common::write_test_session_with_last_message( root.path(), &encoded, "-default_topic", 1, "DEFAULTTOPICMARKER" );
  common::write_test_session_with_last_message( root.path(), &encoded, "-work", 1, "WORKTOPICMARKER" );

  let out = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .current_dir( cwd.path() )
    .arg( ".tail" )
    .arg( "topic::work" )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.contains( "WORKTOPICMARKER" ), "expected -work session marker in output: {text}" );
  assert!( !text.contains( "DEFAULTTOPICMARKER" ), "did not expect -default_topic marker in output: {text}" );
}

/// INT-5: `path::` resolves a different directory's project.
///
/// ## Purpose
/// Validates that `path::DIR` loads DIR's project instead of the process's cwd.
///
/// ## Coverage
/// Exit 0; last 4 of 6 entries from the `path::`-specified project's `-default_topic`
/// session are shown, even though the process cwd is a different, unrelated directory.
///
/// ## Related Requirements
/// `tests/docs/cli/command/12_tail.md` — INT-5
#[ test ]
fn int_5_path_resolves_different_directory_project()
{
  let root          = tempfile::TempDir::new().unwrap();
  let alpha_dir     = tempfile::TempDir::new().unwrap();
  let unrelated_cwd = tempfile::TempDir::new().unwrap();

  common::write_path_project_session( root.path(), alpha_dir.path(), "-default_topic", 6 );

  let out = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .current_dir( unrelated_cwd.path() )
    .arg( ".tail" )
    .arg( format!( "path::{}", alpha_dir.path().display() ) )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let text = stdout( &out );
  for i in 2..6
  {
    assert!( text.contains( &format!( "entry {i}" ) ), "expected entry {i} in output: {text}" );
  }
}

/// INT-6: Fewer entries than requested prints all available.
///
/// ## Purpose
/// Validates that requesting more entries than exist shows all available, no error.
///
/// ## Coverage
/// Exit 0; all 3 entries shown when `last::10` is requested against a 3-entry session.
/// Also covers EC-6 (`tests/docs/cli/param/25_last.md`) — same boundary condition.
///
/// ## Related Requirements
/// `tests/docs/cli/command/12_tail.md` — INT-6
/// `tests/docs/cli/param/25_last.md` — EC-6
#[ test ]
fn int_6_fewer_entries_than_requested_shows_all()
{
  let root = tempfile::TempDir::new().unwrap();
  let cwd  = tempfile::TempDir::new().unwrap();

  common::write_path_project_session( root.path(), cwd.path(), "-default_topic", 3 );

  let out = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .current_dir( cwd.path() )
    .arg( ".tail" )
    .arg( "last::10" )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let text = stdout( &out );
  for i in 0..3
  {
    assert!( text.contains( &format!( "entry {i}" ) ), "expected entry {i} in output: {text}" );
  }
}

/// INT-7: Exit code 2 when cwd has no project.
///
/// ## Purpose
/// Validates the "not found = usage error" convention: running from a directory
/// with no matching storage project exits 2, not the standard error exit 1.
///
/// ## Coverage
/// Exit 2; stderr non-empty.
///
/// ## Related Requirements
/// `tests/docs/cli/command/12_tail.md` — INT-7
#[ test ]
fn int_7_exit_2_when_no_project_for_cwd()
{
  let root = tempfile::TempDir::new().unwrap();
  let cwd  = tempfile::TempDir::new().unwrap();

  // No fixture written — CLAUDE_STORAGE_ROOT has no project for `cwd`.

  let out = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .current_dir( cwd.path() )
    .arg( ".tail" )
    .output()
    .unwrap();

  assert_exit( &out, 2 );
  assert!( !stderr( &out ).is_empty(), "INT-7: expected non-empty stderr for missing project" );
}

/// INT-8: Negative `last::` is rejected with exit code 1.
///
/// ## Purpose
/// Validates the exact stderr wording and exit code for negative `last::` counts.
///
/// ## Coverage
/// Exit 1; stderr exactly `"last must be non-negative"`. Rejection happens before
/// entries (or the project) are loaded — a valid project/session fixture is present
/// to prove the rejection is not a side effect of a missing project.
/// Also covers EC-4 (`tests/docs/cli/param/25_last.md`) — same scenario, stricter assertion.
///
/// ## Related Requirements
/// `tests/docs/cli/command/12_tail.md` — INT-8
/// `tests/docs/cli/param/25_last.md` — EC-4
#[ test ]
fn int_8_negative_last_rejected_exit_1()
{
  let root = tempfile::TempDir::new().unwrap();
  let cwd  = tempfile::TempDir::new().unwrap();

  common::write_path_project_session( root.path(), cwd.path(), "-default_topic", 6 );

  let out = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .current_dir( cwd.path() )
    .arg( ".tail" )
    .arg( "last::-1" )
    .output()
    .unwrap();

  assert_exit( &out, 1 );
  assert_eq!( stderr( &out ).trim_end(), "last must be non-negative" );
}

/// INT-9: No args falls back to the most recent session when no `-default_topic`
/// session exists.
///
/// ## Purpose
/// Validates that the zero-parameter default does not require a literal
/// `-default_topic` session to exist. Real Claude Code sessions are UUID-named,
/// never topic-tagged, so the default must resolve by recency instead of
/// requiring a fixed guessed ID to be present.
///
/// ## Coverage
/// Exit 0; last 4 of 6 entries shown from the UUID-named session (entries 2-5),
/// oldest-first; entries 0-1 absent. No `-default_topic` session is written.
///
/// ## Related Requirements
/// `tests/docs/cli/command/12_tail.md` — INT-9
#[ test ]
fn int_9_no_args_falls_back_to_most_recent_session_when_no_default_topic()
{
  let root = tempfile::TempDir::new().unwrap();
  let cwd  = tempfile::TempDir::new().unwrap();

  common::write_path_project_session( root.path(), cwd.path(), "7380351c-fde9-482a-afc7-ad738781488f", 6 );

  let out = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .current_dir( cwd.path() )
    .arg( ".tail" )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let text = stdout( &out );
  for i in 2..6
  {
    assert!( text.contains( &format!( "entry {i}" ) ), "expected entry {i} in output: {text}" );
  }
  for i in 0..2
  {
    assert!( !text.contains( &format!( "entry {i}" ) ), "did not expect entry {i} in output: {text}" );
  }
}

/// INT-10: No args picks the most recently modified session among multiple candidates.
///
/// ## Purpose
/// Validates that the recency fallback actually compares modification times
/// rather than picking an arbitrary/first-found session when more than one
/// UUID-named session exists in the project.
///
/// ## Coverage
/// Exit 0; output shows the newer session's marker text, not the older one's.
///
/// ## Related Requirements
/// `tests/docs/cli/command/12_tail.md` — INT-10
#[ test ]
fn int_10_no_args_picks_most_recently_modified_session_among_multiple()
{
  let root = tempfile::TempDir::new().unwrap();
  let cwd  = tempfile::TempDir::new().unwrap();

  common::write_path_project_session( root.path(), cwd.path(), "11111111-1111-1111-1111-111111111111", 2 );
  // Distinguishable mtimes across filesystems (matches continuation_tests.rs's own convention).
  std::thread::sleep( core::time::Duration::from_millis( 10 ) );
  common::write_path_project_session_with_last_message( root.path(), cwd.path(), "22222222-2222-2222-2222-222222222222", 1, "newer session marker" );

  let out = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .current_dir( cwd.path() )
    .arg( ".tail" )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.contains( "newer session marker" ), "expected the more recently modified session's content: {text}" );
}

/// INT-11: No args excludes agent sessions from the most-recent fallback.
///
/// ## Purpose
/// Validates that an `agent-*` sidecar session — even if more recently
/// modified than the main session — is never selected by the recency
/// fallback, matching `claude_storage_core::continuation`'s own established
/// agent-exclusion convention.
///
/// ## Coverage
/// Exit 0; output shows the main session's content, not the agent session's.
///
/// ## Related Requirements
/// `tests/docs/cli/command/12_tail.md` — INT-11
#[ test ]
fn int_11_no_args_excludes_agent_sessions_from_fallback()
{
  let root = tempfile::TempDir::new().unwrap();
  let cwd  = tempfile::TempDir::new().unwrap();

  common::write_path_project_session_with_last_message( root.path(), cwd.path(), "33333333-3333-3333-3333-333333333333", 1, "main session marker" );
  std::thread::sleep( core::time::Duration::from_millis( 10 ) );
  common::write_path_project_session_with_last_message( root.path(), cwd.path(), "agent-33333333", 1, "agent session marker" );

  let out = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .current_dir( cwd.path() )
    .arg( ".tail" )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.contains( "main session marker" ), "expected the main session's content: {text}" );
  assert!( !text.contains( "agent session marker" ), "did not expect the agent session's content: {text}" );
}

/// EC-5: Empty `last::` value is rejected.
///
/// ## Purpose
/// Validates that an empty value for the Integer-typed `last` parameter is
/// rejected by the framework's own type parsing, before the routine ever runs.
///
/// ## Coverage
/// Exit 1; stderr non-empty.
///
/// ## Related Requirements
/// `tests/docs/cli/param/25_last.md` — EC-5
#[ test ]
fn ec_5_empty_last_value_rejected()
{
  let out = common::clg_cmd()
    .arg( ".tail" )
    .arg( "last::" )
    .output()
    .unwrap();

  assert_exit( &out, 1 );
  assert!( !stderr( &out ).is_empty(), "EC-5: expected non-empty stderr for empty `last::` value" );
}

/// EC-7: Non-integer `last::` value is rejected.
///
/// ## Purpose
/// Validates that a non-integer value for the Integer-typed `last` parameter is
/// rejected by the framework's own type parsing, before the routine ever runs.
///
/// ## Coverage
/// Exit 1; stderr non-empty.
///
/// ## Related Requirements
/// `tests/docs/cli/param/25_last.md` — EC-7
#[ test ]
fn ec_7_non_integer_last_value_rejected()
{
  let out = common::clg_cmd()
    .arg( ".tail" )
    .arg( "last::four" )
    .output()
    .unwrap();

  assert_exit( &out, 1 );
  assert!( !stderr( &out ).is_empty(), "EC-7: expected non-empty stderr for non-integer `last::` value" );
}

/// EC-8: `l::N` alias produces byte-identical output to `last::N`.
///
/// ## Purpose
/// Validates that the `l` alias declared on `last` in `unilang.commands.yaml`
/// reaches the routine as the canonical `last` argument.
///
/// ## Coverage
/// Exit 0 for both spellings; stdout byte-identical between `l::2` and `last::2`
/// against the same fixture.
///
/// ## Validation Strategy
/// Byte-equality rather than a second copy of INT-2's entry assertions: unilang
/// binds an alias to its canonical argument name during semantic analysis
/// (`semantic/argument_binding.rs` inserts under `arg_def.name` regardless of
/// which spelling matched), so the routine is structurally incapable of telling
/// the two apart. Any observable difference means the alias never bound at all —
/// which byte-equality catches, while re-asserting "entries 4-5 present" would
/// also pass if `l::2` were silently ignored and the default 4 applied. A
/// 6-entry fixture makes that failure mode concrete: the default window (4) and
/// the requested window (2) select different entry sets.
///
/// ## Related Requirements
/// `tests/docs/cli/param/25_last.md` — EC-8
#[ test ]
fn ec_8_l_alias_matches_canonical_last()
{
  let root = tempfile::TempDir::new().unwrap();
  let cwd  = tempfile::TempDir::new().unwrap();

  common::write_path_project_session( root.path(), cwd.path(), "-default_topic", 6 );

  let run = | arg : &str |
  {
    common::clg_cmd()
      .env( "CLAUDE_STORAGE_ROOT", root.path() )
      .current_dir( cwd.path() )
      .arg( ".tail" )
      .arg( arg )
      .output()
      .unwrap()
  };

  let aliased   = run( "l::2" );
  let canonical = run( "last::2" );

  assert_exit( &aliased, 0 );
  assert_exit( &canonical, 0 );

  assert_eq!(
    stdout( &aliased ),
    stdout( &canonical ),
    "EC-8: `l::2` must produce byte-identical output to `last::2`"
  );

  // Guard against both spellings silently falling back to the default of 4:
  // with 6 entries, a 2-window excludes entries 0-3 and a 4-window does not.
  let text = stdout( &aliased );
  assert!( !text.contains( "entry 3" ), "EC-8: `l::2` must not fall back to the default 4-entry window; got:\n{text}" );
}

/// F7: An unresolved `topic::` reports the plain topic, never the internal
/// `-{topic}` session-ID form.
///
/// ## Purpose
/// Validates that `topic::does-not-exist` reports the error against what the
/// user typed, not `find_session_mut`'s internal `-{topic}` matching form —
/// a caller-facing message must not leak an internal naming convention.
///
/// ## Coverage
/// Exit 1; stderr contains `Session not found for topic: does-not-exist`;
/// stderr does not contain the leaked `-does-not-exist` form.
///
/// ## Related Requirements
/// UX/DX round 1, Finding F7
#[ test ]
fn f7_topic_not_found_error_omits_internal_hyphen_prefix()
{
  let root = tempfile::TempDir::new().unwrap();
  let cwd  = tempfile::TempDir::new().unwrap();

  common::write_path_project_session( root.path(), cwd.path(), "-default_topic", 3 );

  let out = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .current_dir( cwd.path() )
    .arg( ".tail" )
    .arg( "topic::does-not-exist" )
    .output()
    .unwrap();

  assert_exit( &out, 1 );
  assert!(
    stderr( &out ).contains( "Session not found for topic: does-not-exist" ),
    "F7: error must report the plain topic; got:\n{}", stderr( &out )
  );
  assert!(
    !stderr( &out ).contains( "-does-not-exist" ),
    "F7: error must not leak the internal '-' prefix form; got:\n{}", stderr( &out )
  );
}

// ── Rendering fixtures ──────────────────────────────────────────────────────
//
// The tests below exercise entry *shapes* rather than entry counts: records
// sharing a `message.id`, array-form user content, `tool_use`/`tool_result`
// pairs, unmodelled block types. `common::write_test_session` cannot express
// any of those, so these build raw JSONL lines directly.

/// Session ID used by the rendering fixtures — UUID-shaped, like real sessions.
const RENDER_SESSION : &str = "aaaaaaaa-bbbb-cccc-dddd-eeeeeeeeeeee";

/// Build a `user` JSONL record whose `message.content` is the given raw JSON.
fn user_entry( index : usize, content_json : &str ) -> String
{
  format!(
    r#"{{"type":"user","uuid":"u-{index:03}","parentUuid":null,"timestamp":"2025-01-01T00:{index:02}:00Z","cwd":"/tmp","sessionId":"{RENDER_SESSION}","version":"2.0.0","gitBranch":"master","userType":"human","isSidechain":false,"message":{{"role":"user","content":{content_json}}}}}"#
  )
}

/// Build an `assistant` JSONL record with an explicit `message.id` and content array.
fn assistant_entry( index : usize, message_id : &str, content_json : &str ) -> String
{
  format!(
    r#"{{"type":"assistant","uuid":"a-{index:03}","parentUuid":null,"timestamp":"2025-01-01T00:{index:02}:00Z","cwd":"/tmp","sessionId":"{RENDER_SESSION}","version":"2.0.0","gitBranch":"master","userType":"external","isSidechain":false,"requestId":"req-{index:03}","message":{{"role":"assistant","model":"claude-test","id":"{message_id}","content":{content_json},"stop_reason":"end_turn","stop_sequence":null,"usage":{{"input_tokens":10,"output_tokens":5,"cache_read_input_tokens":0,"cache_creation_input_tokens":0}}}}}}"#
  )
}

/// A storage root plus a cwd whose project holds one raw-JSONL session.
///
/// Held as a value rather than hidden inside a one-shot helper because two runs
/// that are compared byte-for-byte must share one fixture: the session header
/// carries the project label, which is derived from the temp directory's own
/// random name, so a second fixture would differ in the header alone.
struct Fixture
{
  root : tempfile::TempDir,
  cwd  : tempfile::TempDir,
}

impl Fixture
{
  /// Build a project whose sole session is exactly `lines`.
  fn new( lines : &[ String ] ) -> Self
  {
    let root = tempfile::TempDir::new().unwrap();
    let cwd  = tempfile::TempDir::new().unwrap();
    common::write_raw_session( root.path(), cwd.path(), RENDER_SESSION, lines );
    Self { root, cwd }
  }

  /// Run `.tail` against this fixture with the given extra args.
  fn tail( &self, args : &[ &str ] ) -> std::process::Output
  {
    let mut cmd = common::clg_cmd();
    cmd
      .env( "CLAUDE_STORAGE_ROOT", self.root.path() )
      .current_dir( self.cwd.path() )
      .arg( ".tail" );
    for arg in args
    {
      cmd.arg( arg );
    }
    cmd.output().unwrap()
  }
}

/// Run `.tail` once with the given extra args against a session built from `lines`.
fn tail_over( lines : &[ String ], args : &[ &str ] ) -> std::process::Output
{
  Fixture::new( lines ).tail( args )
}

/// Count turn-boundary rule lines in `.tail` output.
fn rule_lines( text : &str ) -> usize
{
  text.lines().filter( | line | line.starts_with( "── " ) ).count()
}

/// INT-12: Records sharing one `message.id` collapse into a single turn.
///
/// ## Purpose
/// Validates the turn-grouping rule: Claude Code writes one record per content
/// chunk, so a single API response spans several consecutive records carrying
/// the same `message.id`. `last::1` must yield the whole response, not its last
/// fragment.
///
/// ## Coverage
/// Exit 0; `last::1` shows both fragments of the shared-id response; exactly one
/// rule line is drawn; the earlier, separately-identified response is excluded.
///
/// ## Related Requirements
/// `tests/docs/cli/command/12_tail.md` — INT-12
#[ test ]
fn int_12_records_sharing_message_id_form_one_turn()
{
  let lines = vec!
  [
    assistant_entry( 1, "msg_earlier", r#"[{"type":"text","text":"EARLIERTURN"}]"# ),
    assistant_entry( 2, "msg_shared", r#"[{"type":"text","text":"FRAGMENTONE"}]"# ),
    assistant_entry( 3, "msg_shared", r#"[{"type":"text","text":"FRAGMENTTWO"}]"# ),
  ];

  let out = tail_over( &lines, &[ "last::1" ] );
  let text = stdout( &out );

  assert_exit( &out, 0 );
  assert!( text.contains( "FRAGMENTONE" ), "INT-12: first fragment missing:\n{text}" );
  assert!( text.contains( "FRAGMENTTWO" ), "INT-12: second fragment missing:\n{text}" );
  assert!( !text.contains( "EARLIERTURN" ), "INT-12: last::1 must not reach the previous turn:\n{text}" );
  assert_eq!( rule_lines( &text ), 1, "INT-12: shared message.id must draw one rule line:\n{text}" );
}

/// INT-13: Array-form `message.content` on a user record is parsed, not dropped.
///
/// ## Purpose
/// Validates the parser accepts both shapes Claude Code writes for user records:
/// a plain string (typed prompt) and an array of blocks (tool-result turn).
/// The array form is the overwhelming majority of real user records; rejecting
/// it made `load_entries` silently discard them.
///
/// ## Coverage
/// Exit 0; text carried in an array-form user record reaches the output.
///
/// ## Related Requirements
/// `tests/docs/cli/command/12_tail.md` — INT-13
#[ test ]
fn int_13_array_form_user_content_is_parsed()
{
  let lines = vec!
  [
    user_entry( 1, r#""STRINGFORMMARKER""# ),
    user_entry( 2, r#"[{"type":"text","text":"ARRAYFORMMARKER"}]"# ),
  ];

  let out = tail_over( &lines, &[] );
  let text = stdout( &out );

  assert_exit( &out, 0 );
  assert!( text.contains( "STRINGFORMMARKER" ), "INT-13: string-form content missing:\n{text}" );
  assert!( text.contains( "ARRAYFORMMARKER" ), "INT-13: array-form content missing:\n{text}" );
}

/// INT-14: A tool call renders its input summary and its result size on one line.
///
/// ## Purpose
/// Validates that `tool_use` shows what was actually run rather than a bare
/// `Using tool · Bash`, and that the answering `tool_result` is folded onto the
/// same line as a `↳ N lines` annotation instead of occupying a turn of its own.
///
/// ## Coverage
/// Exit 0; output contains `⚙ Bash · git status --short` and `↳ 3 lines`; the
/// raw result body does not appear.
///
/// ## Related Requirements
/// `tests/docs/cli/command/12_tail.md` — INT-14
#[ test ]
fn int_14_tool_call_shows_input_summary_and_result_size()
{
  let lines = vec!
  [
    assistant_entry( 1, "msg_tool", r#"[{"type":"tool_use","id":"toolu_01","name":"Bash","input":{"command":"git status --short"}}]"# ),
    user_entry( 2, r#"[{"type":"tool_result","tool_use_id":"toolu_01","content":"RESULTLINEA\nRESULTLINEB\nRESULTLINEC","is_error":false}]"# ),
  ];

  let out = tail_over( &lines, &[] );
  let text = stdout( &out );

  assert_exit( &out, 0 );
  assert!( text.contains( "⚙ Bash · git status --short" ), "INT-14: tool input summary missing:\n{text}" );
  assert!( text.contains( "↳ 3 lines" ), "INT-14: result size annotation missing:\n{text}" );
  assert!( !text.contains( "RESULTLINEB" ), "INT-14: successful result body must not be printed:\n{text}" );
}

/// INT-15: A pure tool-result turn never consumes a `last::` slot.
///
/// ## Purpose
/// Validates that turns rendering nothing are dropped before the window is
/// taken. A tool-result record's content is folded onto the `⚙` line that
/// invoked it, so counting it as a turn would silently shrink the window.
///
/// ## Coverage
/// Exit 0; a 4-record session with one tool-result record yields 3 turns; the
/// header reports `turns 1-3 of 3`; all three speak.
///
/// ## Related Requirements
/// `tests/docs/cli/command/12_tail.md` — INT-15
#[ test ]
fn int_15_tool_result_only_turn_is_absorbed()
{
  let lines = vec!
  [
    user_entry( 1, r#""QUESTIONMARKER""# ),
    assistant_entry( 2, "msg_tool", r#"[{"type":"tool_use","id":"toolu_01","name":"Bash","input":{"command":"ls"}}]"# ),
    user_entry( 3, r#"[{"type":"tool_result","tool_use_id":"toolu_01","content":"ok","is_error":false}]"# ),
    assistant_entry( 4, "msg_answer", r#"[{"type":"text","text":"ANSWERMARKER"}]"# ),
  ];

  let out = tail_over( &lines, &[] );
  let text = stdout( &out );

  assert_exit( &out, 0 );
  assert_eq!( rule_lines( &text ), 3, "INT-15: tool-result record must not form its own turn:\n{text}" );
  assert!( text.contains( "turns 1-3 of 3" ), "INT-15: header must count 3 displayable turns:\n{text}" );
  assert!( text.contains( "QUESTIONMARKER" ), "INT-15: first turn missing:\n{text}" );
  assert!( text.contains( "ANSWERMARKER" ), "INT-15: last turn missing:\n{text}" );
}

/// INT-16: An empty text or thinking block renders nothing, not a bare label.
///
/// ## Purpose
/// Validates that a block carrying no text contributes no output. Rendering a
/// bare `Thinking ·` header above nothing wastes a slot on a turn that says
/// nothing.
///
/// ## Coverage
/// Exit 0; a turn whose only block is empty thinking is dropped entirely (no
/// `Thinking ·` anywhere); a turn mixing an empty text block with a real one
/// shows only the real one.
///
/// ## Related Requirements
/// `tests/docs/cli/command/12_tail.md` — INT-16
#[ test ]
fn int_16_empty_blocks_render_nothing()
{
  let lines = vec!
  [
    assistant_entry( 1, "msg_empty", r#"[{"type":"thinking","thinking":"","signature":"sig"}]"# ),
    assistant_entry( 2, "msg_mixed", r#"[{"type":"text","text":""},{"type":"text","text":"REALCONTENT"}]"# ),
  ];

  let out = tail_over( &lines, &[] );
  let text = stdout( &out );

  assert_exit( &out, 0 );
  assert!( !text.contains( "Thinking ·" ), "INT-16: empty thinking block must not print a label:\n{text}" );
  assert!( text.contains( "REALCONTENT" ), "INT-16: non-empty sibling block missing:\n{text}" );
  assert_eq!( rule_lines( &text ), 1, "INT-16: the empty-only turn must be dropped:\n{text}" );
}

/// INT-17: Long turns fold by default and unfold under `full::1`.
///
/// ## Purpose
/// Validates the per-turn body cap: one long answer must not push the rest of
/// the window off screen, and the fold must be reversible.
///
/// ## Coverage
/// Exit 0 both ways; default output shows the first 8 body lines plus a
/// `⋯ 12 more lines` hint naming a `.show` invocation; `full::1` shows all 20
/// lines and no hint.
///
/// ## Related Requirements
/// `tests/docs/cli/command/12_tail.md` — INT-17
/// `tests/docs/cli/param/42_full.md` — EC-1
#[ test ]
fn int_17_long_turn_folds_by_default_and_unfolds_with_full()
{
  let body = ( 1..=20 ).map( | n | format!( "BODYLINE{n:02}" ) ).collect::< Vec< _ > >().join( "\\n" );
  let lines = vec![ assistant_entry( 1, "msg_long", &format!( r#"[{{"type":"text","text":"{body}"}}]"# ) ) ];

  let folded = tail_over( &lines, &[] );
  let folded_text = stdout( &folded );

  assert_exit( &folded, 0 );
  assert!( folded_text.contains( "BODYLINE08" ), "INT-17: eighth line must survive folding:\n{folded_text}" );
  assert!( !folded_text.contains( "BODYLINE09" ), "INT-17: ninth line must be folded away:\n{folded_text}" );
  assert!( folded_text.contains( "⋯ 12 more lines" ), "INT-17: fold hint missing:\n{folded_text}" );
  assert!( folded_text.contains( ".show session_id::aaaaaaaa index::1" ), "INT-17: fold hint must name a working .show call:\n{folded_text}" );

  let unfolded = tail_over( &lines, &[ "full::1" ] );
  let unfolded_text = stdout( &unfolded );

  assert_exit( &unfolded, 0 );
  assert!( unfolded_text.contains( "BODYLINE20" ), "INT-17: full::1 must print every line:\n{unfolded_text}" );
  assert!( !unfolded_text.contains( '⋯' ), "INT-17: full::1 must not fold:\n{unfolded_text}" );
}

/// INT-18: `compact::1` prints exactly one line per turn.
///
/// ## Purpose
/// Validates the scan-oriented layout: one row per turn, carrying the turn
/// ordinal, so a long stretch of history fits on one screen.
///
/// ## Coverage
/// Exit 0; three turns produce three body rows and zero rule lines; each row
/// carries its turn ordinal and speaker.
///
/// ## Related Requirements
/// `tests/docs/cli/command/12_tail.md` — INT-18
/// `tests/docs/cli/param/43_compact.md` — EC-1
#[ test ]
fn int_18_compact_prints_one_line_per_turn()
{
  let lines = vec!
  [
    user_entry( 1, r#""FIRSTMARKER""# ),
    assistant_entry( 2, "msg_a", r#"[{"type":"text","text":"SECONDMARKER"}]"# ),
    user_entry( 3, r#""THIRDMARKER""# ),
  ];

  let out = tail_over( &lines, &[ "compact::1" ] );
  let text = stdout( &out );

  assert_exit( &out, 0 );
  assert_eq!( rule_lines( &text ), 0, "INT-18: compact mode must not draw rule lines:\n{text}" );

  let rows : Vec< &str > = text.lines().filter( | line | line.contains( "MARKER" ) ).collect();
  assert_eq!( rows.len(), 3, "INT-18: expected one row per turn:\n{text}" );
  assert!( rows[ 0 ].contains( "You" ) && rows[ 0 ].contains( '1' ), "INT-18: first row must carry ordinal and speaker: {}", rows[ 0 ] );
  assert!( rows[ 1 ].contains( "Claude" ), "INT-18: second row must name the assistant: {}", rows[ 1 ] );
}

/// INT-18b: `compact::1 full::1` — compact wins.
///
/// ## Purpose
/// Pins the documented precedence between the two layout switches. Both param
/// docs assert "compact wins", but the rule is only observable when the two are
/// combined: `full::` lifts the per-turn body cap, and compact mode never
/// prints bodies at all, so an implementation that checked `full::` first would
/// silently produce the folded layout instead.
///
/// ## Coverage
/// Exit 0; `compact::1 full::1` renders the same rows as `compact::1` alone,
/// over a fixture whose long turn would fold without `full::` and expand with
/// it — so the two would differ if `full::` were consulted at all. Both runs go
/// through one `Fixture`, which is what lets the whole output be compared,
/// header included.
///
/// ## Related Requirements
/// `tests/docs/cli/param/43_compact.md` — EC-2
/// `tests/docs/cli/param/42_full.md` — EC-2
#[ test ]
fn int_18b_compact_wins_over_full()
{
  // A 20-line body is the discriminator: under the default layout it folds at 8
  // lines, and `full::1` prints all 20. If compact mode did not take precedence,
  // adding `full::1` would change the output.
  let body = ( 1..=20 ).map( | n | format!( "BODYLINE{n:02}" ) ).collect::< Vec< _ > >().join( "\\n" );
  let lines = vec!
  [
    user_entry( 1, r#""FIRSTMARKER""# ),
    assistant_entry( 2, "msg_long", &format!( r#"[{{"type":"text","text":"{body}"}}]"# ) ),
  ];

  let fixture = Fixture::new( &lines );
  let compact_only = fixture.tail( &[ "compact::1" ] );
  let compact_full = fixture.tail( &[ "compact::1", "full::1" ] );

  assert_exit( &compact_only, 0 );
  assert_exit( &compact_full, 0 );

  assert_eq!(
    stdout( &compact_full ),
    stdout( &compact_only ),
    "INT-18b: `full::1` must be inert alongside `compact::1`"
  );

  let text = stdout( &compact_full );
  assert!( !text.contains( "BODYLINE20" ), "INT-18b: compact rows must not carry unfolded bodies:\n{text}" );
  assert_eq!( rule_lines( &text ), 0, "INT-18b: compact mode must not draw rule lines:\n{text}" );
}

/// INT-19: The session header names the project, session, and turn span.
///
/// ## Purpose
/// Validates that the output states where it came from and how much of the
/// session is on screen — without it, a tail of unknown provenance and unknown
/// remaining depth is guesswork.
///
/// ## Coverage
/// Exit 0; header carries the 8-character session prefix and the displayed span
/// out of the session total.
///
/// ## Related Requirements
/// `tests/docs/cli/command/12_tail.md` — INT-19
#[ test ]
fn int_19_session_header_reports_provenance_and_span()
{
  let lines : Vec< String > = ( 1..=5 )
    .map( | n | assistant_entry( n, &format!( "msg_{n}" ), &format!( r#"[{{"type":"text","text":"LINE{n}"}}]"# ) ) )
    .collect();

  let out = tail_over( &lines, &[ "last::2" ] );
  let text = stdout( &out );
  let header = text.lines().next().unwrap_or_default();

  assert_exit( &out, 0 );
  assert!( header.contains( "aaaaaaaa" ), "INT-19: header must carry the short session id: {header}" );
  assert!( header.contains( "turns 4-5 of 5" ), "INT-19: header must report the displayed span: {header}" );
}

/// INT-20: Output ends with exactly one newline.
///
/// ## Purpose
/// Validates that turns are joined rather than each pushing its own trailing
/// separator — the old per-entry push left two stray blank lines at the end of
/// every invocation.
///
/// ## Coverage
/// Exit 0; stdout ends with a single `\n` and no blank line before it.
///
/// ## Related Requirements
/// `tests/docs/cli/command/12_tail.md` — INT-20
#[ test ]
fn int_20_output_has_no_trailing_blank_lines()
{
  let lines = vec![ assistant_entry( 1, "msg_a", r#"[{"type":"text","text":"ONLYLINE"}]"# ) ];

  let out = tail_over( &lines, &[] );
  let text = stdout( &out );

  assert_exit( &out, 0 );
  assert!( text.ends_with( "ONLYLINE\n" ), "INT-20: expected exactly one trailing newline; got {:?}", &text[ text.len().saturating_sub( 20 ).. ] );
}

/// INT-21: An unmodelled block type is retained, not dropped with its record.
///
/// ## Purpose
/// Validates graceful degradation against schema drift: a block whose `type`
/// this tool does not model must not take the whole record down with it, since
/// a rejected record is invisible — `load_entries` skips unparseable lines.
///
/// ## Coverage
/// Exit 0; a user record mixing an `image` block with text renders both the
/// `⧉ image` marker and the text.
///
/// ## Related Requirements
/// `tests/docs/cli/command/12_tail.md` — INT-21
#[ test ]
fn int_21_unmodelled_block_type_does_not_drop_the_record()
{
  let lines = vec![ user_entry( 1, r#"[{"type":"image","source":{}},{"type":"text","text":"CAPTIONMARKER"}]"# ) ];

  let out = tail_over( &lines, &[] );
  let text = stdout( &out );

  assert_exit( &out, 0 );
  assert!( text.contains( "CAPTIONMARKER" ), "INT-21: record must survive the unmodelled block:\n{text}" );
  assert!( text.contains( "⧉ image" ), "INT-21: unmodelled block must be marked:\n{text}" );
}

/// INT-22: A failed tool call is annotated `↳ error`.
///
/// ## Purpose
/// Validates that failure is visible at the call site. A tool error folded to
/// `↳ 2 lines` would read as an ordinary result.
///
/// ## Coverage
/// Exit 0; output contains `↳ error` on the `⚙` line.
///
/// ## Related Requirements
/// `tests/docs/cli/command/12_tail.md` — INT-22
#[ test ]
fn int_22_failed_tool_call_is_marked_as_an_error()
{
  let lines = vec!
  [
    assistant_entry( 1, "msg_tool", r#"[{"type":"tool_use","id":"toolu_09","name":"Bash","input":{"command":"false"}}]"# ),
    user_entry( 2, r#"[{"type":"tool_result","tool_use_id":"toolu_09","content":"boom","is_error":true}]"# ),
  ];

  let out = tail_over( &lines, &[] );
  let text = stdout( &out );

  assert_exit( &out, 0 );
  assert!( text.contains( "↳ error" ), "INT-22: error annotation missing:\n{text}" );
}

/// INT-23: A `tool_result` whose content is a nested block array flattens to text.
///
/// ## Purpose
/// Validates the second `tool_result.content` shape Claude Code writes — an
/// array of nested blocks rather than a string. Requiring a string rejected the
/// whole record.
///
/// ## Coverage
/// Exit 0; the call is annotated with the flattened nested text's line count
/// rather than the record being dropped.
///
/// ## Related Requirements
/// `tests/docs/cli/command/12_tail.md` — INT-23
#[ test ]
fn int_23_nested_tool_result_content_is_flattened()
{
  let lines = vec!
  [
    assistant_entry( 1, "msg_tool", r#"[{"type":"tool_use","id":"toolu_11","name":"Read","input":{"file_path":"/tmp/x.rs"}}]"# ),
    user_entry( 2, r#"[{"type":"tool_result","tool_use_id":"toolu_11","content":[{"type":"text","text":"alpha"},{"type":"text","text":"beta"}]}]"# ),
  ];

  let out = tail_over( &lines, &[] );
  let text = stdout( &out );

  assert_exit( &out, 0 );
  assert!( text.contains( "⚙ Read · /tmp/x.rs" ), "INT-23: tool line missing:\n{text}" );
  assert!( text.contains( "↳ 2 lines" ), "INT-23: nested content must flatten to 2 lines:\n{text}" );
}

/// INT-24: A tool whose input carries no path or command still summarises.
///
/// ## Purpose
/// `TaskUpdate` is the single most common tool in the local store after the
/// file and shell tools, and none of its inputs carry `command`, `file_path`,
/// or any other originally-listed key — every one of its calls rendered as a
/// bare `⚙ TaskUpdate`. `status` is what makes the line worth reading.
///
/// ## Coverage
/// Exit 0; output contains `⚙ TaskUpdate · completed`; `status` outranks
/// `taskId`, which would render an opaque number.
///
/// ## Related Requirements
/// `tests/docs/cli/command/12_tail.md` — INT-24
#[ test ]
fn int_24_task_tool_summarises_by_status_not_id()
{
  let lines = vec!
  [
    assistant_entry( 1, "msg_task", r#"[{"type":"tool_use","id":"toolu_21","name":"TaskUpdate","input":{"taskId":"42","status":"completed"}}]"# ),
  ];

  let out = tail_over( &lines, &[] );
  let text = stdout( &out );

  assert_exit( &out, 0 );
  assert!( text.contains( "⚙ TaskUpdate · completed" ), "INT-24: status must be the summary:\n{text}" );
  assert!( !text.contains( "· 42" ), "INT-24: the opaque id must not win over status:\n{text}" );
}
