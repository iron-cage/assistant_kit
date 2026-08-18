//! Feature tests (Task 510): `SessionStats.cwd` populated by `Session::stats()`
//!
//! ## Source
//!
//! - Task: `task/claude_storage_core/510_session_stats_cwd_field.md`
//! - Consumer being unblocked: `.usage` CLI command (`docs/cli/command/13_usage.md`)
//!
//! ## Coverage
//!
//! - T01: Every line carries the same `cwd` → `Some(that value)`
//! - T02: First line's `cwd` differs from later lines' → FIRST line's value wins
//! - T03: Empty session (zero parseable lines) → `None`
//! - T04: Malformed first line skipped (BUG-489 behavior intact); `cwd` from first valid line

use std::fs;
use tempfile::TempDir;

/// Helper: create a project directory in `projects_dir`
fn create_project( projects_dir : &std::path::Path, name : &str ) -> std::path::PathBuf
{
  let p = projects_dir.join( name );
  fs::create_dir_all( &p ).expect( "create project dir" );
  p
}

/// Helper: write `content` as a session file and load it.
fn load_session( p_dir : &std::path::Path, file_name : &str, content : &str ) -> claude_storage_core::Session
{
  let session_path = p_dir.join( file_name );
  fs::write( &session_path, content ).expect( "write session file" );
  claude_storage_core::Session::load( &session_path ).expect( "load session" )
}

/// Test `stats()` populates `cwd` when every line carries the same value (T01)
///
/// ## Purpose
/// Validates the baseline: a session whose entries all share one working
/// directory yields that directory in `SessionStats.cwd`.
///
/// ## Coverage
/// Two valid entries, both `"cwd":"/home/alice/proj-a"` → `Some("/home/alice/proj-a")`.
///
/// ## Validation Strategy
/// Build a synthetic session, call `stats()`, assert `cwd` equality and that
/// entry counting is unaffected.
///
/// ## Related Requirements
/// Task 510 Test Matrix T01
#[test]
fn stats_cwd_uniform_lines()
{
  let temp = TempDir::new().expect( "temp dir" );
  let p_dir = create_project( &temp.path().join( "projects" ), "-cwd-uniform" );

  let content = concat!(
    r#"{"type":"user","cwd":"/home/alice/proj-a","message":{"role":"user","content":"hello"},"timestamp":"2026-01-01T00:00:00Z"}"#, "\n",
    r#"{"type":"assistant","cwd":"/home/alice/proj-a","message":{"role":"assistant","content":"hi"},"timestamp":"2026-01-01T00:00:01Z"}"#, "\n",
  );
  let mut session = load_session( &p_dir, "aaaaaaaa-bbbb-cccc-dddd-eeeeeeeeeeee.jsonl", content );

  let stats = session.stats().expect( "stats() must succeed" );

  assert_eq!( stats.cwd.as_deref(), Some( "/home/alice/proj-a" ), "cwd must hold the shared value" );
  assert_eq!( stats.total_entries, 2, "cwd extraction must not disturb entry counting" );
}

/// Test `stats()` keeps the FIRST line's `cwd` when later lines differ (T02)
///
/// ## Purpose
/// Validates first-entry-wins — the same semantics `first_timestamp` uses.
/// A last-entry-wins bug would return the final line's value instead.
///
/// ## Coverage
/// First line `"cwd":"/home/alice/first"`, later lines `"cwd":"/home/alice/second"`
/// and `"cwd":"/home/alice/third"` (genuinely varying, per Task 510 AF2) →
/// `Some("/home/alice/first")`.
///
/// ## Validation Strategy
/// Assert `cwd` equals the first line's value, which any overwrite-per-line
/// implementation would fail.
///
/// ## Related Requirements
/// Task 510 Test Matrix T02, Anti-faking check AF2
#[test]
fn stats_cwd_first_entry_wins()
{
  let temp = TempDir::new().expect( "temp dir" );
  let p_dir = create_project( &temp.path().join( "projects" ), "-cwd-first-wins" );

  let content = concat!(
    r#"{"type":"user","cwd":"/home/alice/first","message":{"role":"user","content":"one"},"timestamp":"2026-01-01T00:00:00Z"}"#, "\n",
    r#"{"type":"assistant","cwd":"/home/alice/second","message":{"role":"assistant","content":"two"},"timestamp":"2026-01-01T00:00:01Z"}"#, "\n",
    r#"{"type":"user","cwd":"/home/alice/third","message":{"role":"user","content":"three"},"timestamp":"2026-01-01T00:00:02Z"}"#, "\n",
  );
  let mut session = load_session( &p_dir, "bbbbbbbb-cccc-dddd-eeee-ffffffffffff.jsonl", content );

  let stats = session.stats().expect( "stats() must succeed" );

  assert_eq!(
    stats.cwd.as_deref(),
    Some( "/home/alice/first" ),
    "cwd must be the FIRST line's value (first-entry-wins), not a later line's"
  );
}

/// Test `stats()` leaves `cwd` as `None` for an empty session (T03)
///
/// ## Purpose
/// Validates the no-data case: with zero parseable lines there is no `cwd`
/// source, and the field must stay `None` rather than defaulting to anything.
///
/// ## Coverage
/// Zero-byte session file → `cwd == None`, zero entries.
///
/// ## Validation Strategy
/// Assert `cwd.is_none()` and `total_entries == 0`.
///
/// ## Related Requirements
/// Task 510 Test Matrix T03
#[test]
fn stats_cwd_empty_session_none()
{
  let temp = TempDir::new().expect( "temp dir" );
  let p_dir = create_project( &temp.path().join( "projects" ), "-cwd-empty" );

  let mut session = load_session( &p_dir, "cccccccc-dddd-eeee-ffff-000000000000.jsonl", "" );

  let stats = session.stats().expect( "stats() must succeed on an empty session" );

  assert!( stats.cwd.is_none(), "empty session must yield cwd == None" );
  assert_eq!( stats.total_entries, 0 );
}

/// Test `stats()` populates `cwd` from the first VALID line after a malformed one (T04)
///
/// ## Purpose
/// Validates that the BUG-489/BUG-508 skip-and-continue behavior composes with
/// `cwd` extraction: a malformed opening line is skipped exactly as before, and
/// `cwd` comes from the first line that actually parses.
///
/// ## Coverage
/// Line 1 malformed (`{`), lines 2-3 valid with differing `cwd` values →
/// `stats()` succeeds, counts 2 entries, `cwd` from line 2.
///
/// ## Validation Strategy
/// Assert `Ok`, entry counts excluding the malformed line, and `cwd` equal to
/// the first valid line's value.
///
/// ## Related Requirements
/// Task 510 Test Matrix T04, Anti-faking check AF1
#[test]
fn stats_cwd_survives_malformed_first_line()
{
  let temp = TempDir::new().expect( "temp dir" );
  let p_dir = create_project( &temp.path().join( "projects" ), "-cwd-malformed" );

  let content = concat!(
    "{", "\n",
    r#"{"type":"user","cwd":"/home/alice/valid","message":{"role":"user","content":"hello"},"timestamp":"2026-01-01T00:00:00Z"}"#, "\n",
    r#"{"type":"assistant","cwd":"/home/alice/later","message":{"role":"assistant","content":"hi"},"timestamp":"2026-01-01T00:00:01Z"}"#, "\n",
  );
  let mut session = load_session( &p_dir, "dddddddd-eeee-ffff-0000-111111111111.jsonl", content );

  let stats = session.stats().expect( "stats() must skip the malformed line, not hard-fail" );

  assert_eq!( stats.total_entries, 2, "malformed line skipped; both valid entries counted" );
  assert_eq!(
    stats.cwd.as_deref(),
    Some( "/home/alice/valid" ),
    "cwd must come from the first VALID line, unaffected by the skipped malformed line"
  );
}
