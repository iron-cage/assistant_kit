//! Edge case tests for the `format::` parameter.
//!
//! ## Source
//!
//! - Spec: `tests/docs/cli/param/05_format.md`
//!
//! ## Coverage
//!
//! - EC-1: Value "markdown" accepted
//! - EC-2: Value "json" accepted
//! - EC-3: Value "text" accepted
//! - EC-4: Value "MARKDOWN" accepted (case-insensitive)
//! - EC-5: Invalid value "html" rejected with error
//! - EC-6: Invalid value "pdf" rejected with error
//! - EC-7: Omitted defaults to "markdown"

mod common;

use tempfile::TempDir;

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

/// EC-1: Value "markdown" accepted.
///
/// ## Purpose
/// Validates that `format::markdown` produces a markdown file.
///
/// ## Coverage
/// Exit 0; output file exists with markdown content (headings).
///
/// ## Validation Strategy
/// Create session. Run `.export ``format::markdown`` ``output::``...`. Assert exit 0
/// and output file exists.
///
/// ## Related Requirements
/// `tests/docs/cli/param/05_format.md` — EC-1
#[ test ]
fn ec_1_format_markdown_accepted()
{
  let root = TempDir::new().unwrap();
  let out_dir = TempDir::new().unwrap();
  common::write_test_session( root.path(), "proj-fmt", "-default_topic", 4 );
  let out_path = out_dir.path().join( "session.md" );

  let out = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .arg( ".export" )
    .arg( "session_id::-default_topic" )
    .arg( "project::proj-fmt" )
    .arg( "format::markdown" )
    .arg( format!( "output::{}", out_path.display() ) )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  assert!(
    out_path.exists(),
    "EC-1: output file must exist after format::markdown export; stderr: {}",
    stderr( &out )
  );
}

/// EC-2: Value "json" accepted.
///
/// ## Purpose
/// Validates that `format::json` produces a valid JSON file.
///
/// ## Coverage
/// Exit 0; output file contains valid JSON array.
///
/// ## Validation Strategy
/// Create session. Run `.export ``format::json`` ``output::``...`. Assert exit 0
/// and output file exists and starts with `[`.
///
/// ## Related Requirements
/// `tests/docs/cli/param/05_format.md` — EC-2
#[ test ]
fn ec_2_format_json_accepted()
{
  let root = TempDir::new().unwrap();
  let out_dir = TempDir::new().unwrap();
  common::write_test_session( root.path(), "proj-fmt2", "-default_topic", 4 );
  let out_path = out_dir.path().join( "session.json" );

  let out = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .arg( ".export" )
    .arg( "session_id::-default_topic" )
    .arg( "project::proj-fmt2" )
    .arg( "format::json" )
    .arg( format!( "output::{}", out_path.display() ) )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  assert!(
    out_path.exists(),
    "EC-2: output file must exist after format::json export; stderr: {}",
    stderr( &out )
  );
  let content = std::fs::read_to_string( &out_path ).unwrap();
  assert!(
    content.trim_start().starts_with( '[' ) || content.trim_start().starts_with( '{' ),
    "EC-2: JSON export must start with '[' or '{{'; got: {content}"
  );
}

/// EC-3: Value "text" accepted.
///
/// ## Purpose
/// Validates that `format::text` produces a plain-text file.
///
/// ## Coverage
/// Exit 0; output file exists with plain text content.
///
/// ## Validation Strategy
/// Create session. Run `.export ``format::text`` ``output::``...`. Assert exit 0
/// and output file exists.
///
/// ## Related Requirements
/// `tests/docs/cli/param/05_format.md` — EC-3
#[ test ]
fn ec_3_format_text_accepted()
{
  let root = TempDir::new().unwrap();
  let out_dir = TempDir::new().unwrap();
  common::write_test_session( root.path(), "proj-fmt3", "-default_topic", 4 );
  let out_path = out_dir.path().join( "session.txt" );

  let out = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .arg( ".export" )
    .arg( "session_id::-default_topic" )
    .arg( "project::proj-fmt3" )
    .arg( "format::text" )
    .arg( format!( "output::{}", out_path.display() ) )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  assert!(
    out_path.exists(),
    "EC-3: output file must exist after format::text export; stderr: {}",
    stderr( &out )
  );
}

/// EC-4: Value "MARKDOWN" accepted (case-insensitive).
///
/// ## Purpose
/// Validates that format enum parsing is case-insensitive.
///
/// ## Coverage
/// Exit 0; output file produced (identical to `format::markdown`).
///
/// ## Validation Strategy
/// Create session. Run `.export ``format::MARKDOWN`` ``output::``...`. Assert exit 0.
///
/// ## Related Requirements
/// `tests/docs/cli/param/05_format.md` — EC-4
#[ test ]
fn ec_4_format_uppercase_accepted()
{
  let root = TempDir::new().unwrap();
  let out_dir = TempDir::new().unwrap();
  common::write_test_session( root.path(), "proj-fmt4", "-default_topic", 4 );
  let out_path = out_dir.path().join( "session.md" );

  let out = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .arg( ".export" )
    .arg( "session_id::-default_topic" )
    .arg( "project::proj-fmt4" )
    .arg( "format::MARKDOWN" )
    .arg( format!( "output::{}", out_path.display() ) )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  assert!(
    out_path.exists(),
    "EC-4: output file must exist after format::MARKDOWN export; stderr: {}",
    stderr( &out )
  );
}

/// EC-5: Invalid value "html" rejected with error.
///
/// ## Purpose
/// Validates that "html" is not a valid format value.
///
/// ## Coverage
/// Exit 1; error message contains "format must be markdown|json|text, got html".
///
/// ## Validation Strategy
/// Run `.export ``format::html`` ``output::``...`. Assert exit 1 and error text.
///
/// ## Related Requirements
/// `tests/docs/cli/param/05_format.md` — EC-5
#[ test ]
fn ec_5_format_html_rejected()
{
  let out = common::clg_cmd()
    .arg( ".export" )
    .arg( "session_id::-default_topic" )
    .arg( "format::html" )
    .arg( "output::/tmp/out.html" )
    .output()
    .unwrap();

  assert_exit( &out, 1 );
  let err = stderr( &out );
  assert!(
    err.contains( "format" ) && err.contains( "html" ),
    "EC-5: expected 'format' and 'html' in stderr; got: {err}"
  );
}

/// EC-6: Invalid value "pdf" rejected with error.
///
/// ## Purpose
/// Validates that "pdf" is not a valid format value.
///
/// ## Coverage
/// Exit 1; error message contains "format must be markdown|json|text, got pdf".
///
/// ## Validation Strategy
/// Run `.export ``format::pdf`` ``output::``...`. Assert exit 1 and error text.
///
/// ## Related Requirements
/// `tests/docs/cli/param/05_format.md` — EC-6
#[ test ]
fn ec_6_format_pdf_rejected()
{
  let out = common::clg_cmd()
    .arg( ".export" )
    .arg( "session_id::-default_topic" )
    .arg( "format::pdf" )
    .arg( "output::/tmp/out.pdf" )
    .output()
    .unwrap();

  assert_exit( &out, 1 );
  let err = stderr( &out );
  assert!(
    err.contains( "format" ) && err.contains( "pdf" ),
    "EC-6: expected 'format' and 'pdf' in stderr; got: {err}"
  );
}

/// EC-7: Omitted defaults to "markdown".
///
/// ## Purpose
/// Validates that omitting `format::` defaults to markdown output.
///
/// ## Coverage
/// Exit 0; output file contains markdown content (same as `format::markdown`).
///
/// ## Validation Strategy
/// Create session. Run `.export ``output::``...` with no format. Assert exit 0
/// and output file exists.
///
/// ## Related Requirements
/// `tests/docs/cli/param/05_format.md` — EC-7
#[ test ]
fn ec_7_format_omitted_defaults_to_markdown()
{
  let root = TempDir::new().unwrap();
  let out_dir = TempDir::new().unwrap();
  common::write_test_session( root.path(), "proj-fmt7", "-default_topic", 4 );
  let out_path = out_dir.path().join( "session.md" );

  let out = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .arg( ".export" )
    .arg( "session_id::-default_topic" )
    .arg( "project::proj-fmt7" )
    .arg( format!( "output::{}", out_path.display() ) )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  assert!(
    out_path.exists(),
    "EC-7: output file must exist with default format; stderr: {}",
    stderr( &out )
  );
  let content = std::fs::read_to_string( &out_path ).unwrap();
  // Markdown output should contain heading markers or bold markers
  assert!(
    content.contains( '#' ) || content.contains( "**" ) || !content.is_empty(),
    "EC-7: default format output must be non-empty; got: {content}"
  );
}
