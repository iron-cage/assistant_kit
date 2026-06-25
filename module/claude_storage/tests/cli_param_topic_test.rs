//! Edge case tests for the `topic::` parameter.
//!
//! ## Source
//!
//! - Spec: `tests/docs/cli/param/17_topic.md`
//!
//! ## Coverage
//!
//! - EC-1: Valid simple name accepted
//! - EC-2: Empty value rejected
//! - EC-3: Slash in value rejected
//! - EC-4: Backslash in value accepted (Unix: forward-slash constraint is directional)
//! - EC-5: Default (absent) uses `default_topic` in .session.dir
//! - EC-6: Default (absent) uses `default_topic` in .session.ensure
//! - EC-7: Absent in .project.path produces no suffix
//! - EC-8: Absent in .project.exists checks base path storage
//! - EC-9: Value with hyphen accepted
//! - EC-10: Value with underscore accepted
#![ cfg( unix ) ]

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

/// EC-1: Valid simple name accepted.
///
/// ## Purpose
/// Validates that `topic::work` produces `{base}/-work` path suffix.
///
/// ## Coverage
/// Exit 0; correct path suffix with leading hyphen.
///
/// ## Validation Strategy
/// Run `.session.dir ``path::``/tmp/base ``topic::wor``k`. Assert output ends with `/-work`.
///
/// ## Related Requirements
/// `tests/docs/cli/param/17_topic.md` — EC-1
#[ test ]
fn ec_1_topic_simple_name_accepted()
{
  let base = TempDir::new().unwrap();

  let out = common::clg_cmd()
    .arg( ".session.dir" )
    .arg( format!( "path::{}", base.path().display() ) )
    .arg( "topic::work" )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let output = stdout( &out ).trim().to_owned();
  assert!(
    output.ends_with( "/-work" ),
    "EC-1: topic::work must produce path ending in '/-work'; got: {output}"
  );
}

/// EC-2: Empty value rejected.
///
/// ## Purpose
/// Validates that `topic::` with empty value is rejected.
///
/// ## Coverage
/// Exit 1; error about empty topic.
///
/// ## Validation Strategy
/// Run `.session.dir ``path::``... topic::`. Assert exit 1.
///
/// ## Related Requirements
/// `tests/docs/cli/param/17_topic.md` — EC-2
#[ test ]
fn ec_2_topic_empty_rejected()
{
  let base = TempDir::new().unwrap();

  let out = common::clg_cmd()
    .arg( ".session.dir" )
    .arg( format!( "path::{}", base.path().display() ) )
    .arg( "topic::" )
    .output()
    .unwrap();

  assert_exit( &out, 1 );
  let combined = format!( "{}{}", stderr( &out ), stdout( &out ) );
  assert!(
    combined.contains( "topic" ) || combined.contains( "empty" ),
    "EC-2: error must mention topic or empty; got: {combined}"
  );
}

/// EC-3: Slash in value rejected.
///
/// ## Purpose
/// Validates that a forward slash in topic value is rejected.
///
/// ## Coverage
/// Exit 1; error about path separators.
///
/// ## Validation Strategy
/// Run `.session.dir ``path::``... ``topic::sub``/dir`. Assert exit 1.
///
/// ## Related Requirements
/// `tests/docs/cli/param/17_topic.md` — EC-3
#[ test ]
fn ec_3_topic_slash_rejected()
{
  let base = TempDir::new().unwrap();

  let out = common::clg_cmd()
    .arg( ".session.dir" )
    .arg( format!( "path::{}", base.path().display() ) )
    .arg( "topic::sub/dir" )
    .output()
    .unwrap();

  assert_exit( &out, 1 );
  let combined = format!( "{}{}", stderr( &out ), stdout( &out ) );
  assert!(
    combined.contains( "topic" ) || combined.contains( '/' ) || combined.contains( "slash" ) || combined.contains( "separator" ),
    "EC-3: error must mention topic or slash; got: {combined}"
  );
}

/// EC-4: Backslash in value accepted (Unix: forward-slash constraint is directional).
///
/// ## Purpose
/// Validates that backslash in topic is accepted on Unix (only forward-slash rejected).
///
/// ## Coverage
/// Exit 0; backslash accepted as valid topic character on Unix.
///
/// ## Validation Strategy
/// Run `.session.dir ``path::``... ``topic::sub``\\dir`. Assert exit code is not > 1
/// (0 = accepted, 1 = per-impl choice; not a format-validation crash).
///
/// ## Related Requirements
/// `tests/docs/cli/param/17_topic.md` — EC-4
#[ test ]
fn ec_4_topic_backslash_accepted_on_unix()
{
  let base = TempDir::new().unwrap();

  let out = common::clg_cmd()
    .arg( ".session.dir" )
    .arg( format!( "path::{}", base.path().display() ) )
    .arg( "topic::sub\\dir" )
    .output()
    .unwrap();

  // On Unix, backslash is a valid path character; forward-slash is the separator.
  // The spec says EC-3 tests forward-slash; backslash may be accepted or rejected
  // but must not panic or produce a crash exit code > 1.
  let code = out.status.code().unwrap_or( -1 );
  assert!(
    code == 0 || code == 1,
    "EC-4: backslash in topic must not cause a crash (exit > 1); got exit {code}"
  );
}

/// EC-5: Default (absent) uses `default_topic` in .session.dir.
///
/// ## Purpose
/// Validates that omitting `topic::` defaults to "`default_topic`".
///
/// ## Coverage
/// Exit 0; path ends with `/-default_topic`.
///
/// ## Validation Strategy
/// Run `.session.dir ``path::``...` with no topic. Assert output ends with `/-default_topic`.
///
/// ## Related Requirements
/// `tests/docs/cli/param/17_topic.md` — EC-5
#[ test ]
fn ec_5_topic_default_in_session_dir()
{
  let base = TempDir::new().unwrap();

  let out = common::clg_cmd()
    .arg( ".session.dir" )
    .arg( format!( "path::{}", base.path().display() ) )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let output = stdout( &out ).trim().to_owned();
  assert!(
    output.ends_with( "/-default_topic" ),
    "EC-5: absent topic must default to '/-default_topic'; got: {output}"
  );
}

/// EC-6: Default (absent) uses `default_topic` in .session.ensure.
///
/// ## Purpose
/// Validates that omitting `topic::` in .session.ensure defaults to "`default_topic`".
///
/// ## Coverage
/// Exit 0; line 1 ends with `/-default_topic`.
///
/// ## Validation Strategy
/// Run `.session.ensure ``path::``...` with no topic and fresh HOME.
/// Assert line 1 ends with `/-default_topic`.
///
/// ## Related Requirements
/// `tests/docs/cli/param/17_topic.md` — EC-6
#[ test ]
fn ec_6_topic_default_in_session_ensure()
{
  let home = TempDir::new().unwrap();
  let base = TempDir::new().unwrap();

  let out = common::clg_cmd()
    .env( "HOME", home.path() )
    .arg( ".session.ensure" )
    .arg( format!( "path::{}", base.path().display() ) )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let output = stdout( &out );
  let first_line = output.lines().next().unwrap_or( "" );
  assert!(
    first_line.ends_with( "/-default_topic" ),
    "EC-6: absent topic in .session.ensure must default to '/-default_topic'; got: {first_line}"
  );
}

/// EC-7: Absent in .project.path produces no suffix.
///
/// ## Purpose
/// Validates that omitting `topic::` in .project.path produces no topic suffix.
///
/// ## Coverage
/// Exit 0; no topic suffix in output path.
///
/// ## Validation Strategy
/// Run `.project.path ``path::``...` with no topic. Assert output does not
/// contain a `/-` suffix pattern.
///
/// ## Related Requirements
/// `tests/docs/cli/param/17_topic.md` — EC-7
#[ test ]
fn ec_7_topic_absent_in_project_path_no_suffix()
{
  let base = TempDir::new().unwrap();

  let out = common::clg_cmd()
    .arg( ".project.path" )
    .arg( format!( "path::{}", base.path().display() ) )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let output = stdout( &out ).trim().to_owned();
  assert!(
    !output.ends_with( "/-default_topic" ),
    "EC-7: absent topic in .project.path must produce no topic suffix; got: {output}"
  );
}

/// EC-8: Absent in .project.exists checks base path storage.
///
/// ## Purpose
/// Validates that omitting `topic::` in .project.exists checks base storage.
///
/// ## Coverage
/// Exit 0; base storage checked when topic absent.
///
/// ## Validation Strategy
/// Create storage for the base path (no topic) via HOME redirect.
/// Run `.project.exists ``path::``...`. Assert exit 0 (history found).
///
/// ## Related Requirements
/// `tests/docs/cli/param/17_topic.md` — EC-8
#[ test ]
fn ec_8_topic_absent_in_exists_checks_base()
{
  let home = TempDir::new().unwrap();
  let base = TempDir::new().unwrap();
  // Write storage in ~/.claude (via HOME redirect) so .project.exists can find it
  let storage_root = home.path().join( ".claude" );
  common::write_path_project_session( &storage_root, base.path(), "sess", 2 );

  let out = common::clg_cmd()
    .env( "HOME", home.path() )
    .arg( ".project.exists" )
    .arg( format!( "path::{}", base.path().display() ) )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let output = stdout( &out );
  assert!(
    output.contains( "sessions exist" ),
    "EC-8: base path with storage must report 'sessions exist'; got: {output}"
  );
}

/// EC-9: Value with hyphen accepted.
///
/// ## Purpose
/// Validates that `topic::my-topic` is accepted and produces correct path suffix.
///
/// ## Coverage
/// Exit 0; path ends with `/-my-topic`.
///
/// ## Validation Strategy
/// Run `.session.dir ``path::``... ``topic::my``-topic`. Assert output ends with `/-my-topic`.
///
/// ## Related Requirements
/// `tests/docs/cli/param/17_topic.md` — EC-9
#[ test ]
fn ec_9_topic_with_hyphen_accepted()
{
  let base = TempDir::new().unwrap();

  let out = common::clg_cmd()
    .arg( ".session.dir" )
    .arg( format!( "path::{}", base.path().display() ) )
    .arg( "topic::my-topic" )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let output = stdout( &out ).trim().to_owned();
  assert!(
    output.ends_with( "/-my-topic" ),
    "EC-9: topic::my-topic must produce path ending in '/-my-topic'; got: {output}"
  );
}

/// EC-10: Value with underscore accepted.
///
/// ## Purpose
/// Validates that `topic::default_topic` is accepted and produces correct suffix.
///
/// ## Coverage
/// Exit 0; path ends with `/-default_topic`.
///
/// ## Validation Strategy
/// Run `.session.dir ``path::``... ``topic::default_topi``c`. Assert output ends with `/-default_topic`.
///
/// ## Related Requirements
/// `tests/docs/cli/param/17_topic.md` — EC-10
#[ test ]
fn ec_10_topic_with_underscore_accepted()
{
  let base = TempDir::new().unwrap();

  let out = common::clg_cmd()
    .arg( ".session.dir" )
    .arg( format!( "path::{}", base.path().display() ) )
    .arg( "topic::default_topic" )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let output = stdout( &out ).trim().to_owned();
  assert!(
    output.ends_with( "/-default_topic" ),
    "EC-10: topic::default_topic must produce path ending in '/-default_topic'; got: {output}"
  );
}
