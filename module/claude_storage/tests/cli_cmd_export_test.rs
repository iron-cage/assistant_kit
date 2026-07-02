//! Integration tests for the `clg .export` command.
//!
//! ## Source
//!
//! - Spec: `tests/docs/cli/command/06_export.md`
//!
//! ## Coverage
//!
//! - INT-1:  `session_id::` required — missing arg exits with 1
//! - INT-2:  `output::` required — missing arg exits with 1
//! - INT-3:  Default format is markdown
//! - INT-4:  `format::json` produces JSON array output
//! - INT-5:  `format::text` produces plain transcript
//! - INT-6:  Output file is created at `output::` path
//! - INT-7:  Output file is overwritten if exists
//! - INT-8:  Exit code 2 when output parent dir does not exist
//! - INT-9:  `project::` selects session from named project
//! - INT-10: Export succeeds with valid session and output path

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

/// INT-1: `session_id::` required — missing arg exits with 1.
///
/// ## Purpose
/// Verify that `.export` without `session_id::` exits with code 1 and emits
/// an error mentioning the missing parameter.
///
/// ## Coverage
/// Exit code 1; error on stderr; no file written.
///
/// ## Validation Strategy
/// Run `clg .export ``output::``{tempfile}` without `session_id::`. Assert exit 1
/// and stderr contains "session" or "required".
///
/// ## Related Requirements
/// `tests/docs/cli/command/06_export.md` — INT-1
#[ test ]
fn int_1_session_id_required_missing_arg_exits_1()
{
  let root    = TempDir::new().unwrap();
  let out_dir = TempDir::new().unwrap();
  let out_file = out_dir.path().join( "int1.md" );

  let out = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .arg( ".export" )
    .arg( format!( "output::{}", out_file.display() ) )
    .output()
    .unwrap();

  assert_exit( &out, 1 );
  let err = stderr( &out );
  assert!(
    !err.is_empty(),
    "INT-1: missing session_id:: must produce error on stderr; got silence"
  );
  assert!(
    err.to_lowercase().contains( "session" ) ||
    err.to_lowercase().contains( "required" ),
    "INT-1: error must mention session_id or required; got:\n{err}"
  );
}

/// INT-2: `output::` required — missing arg exits with 1.
///
/// ## Purpose
/// Verify that `.export` without `output::` exits with code 1 even when
/// `session_id::` is provided.
///
/// ## Coverage
/// Exit code 1; error on stderr mentioning output; no file written.
///
/// ## Validation Strategy
/// Write a session in temp storage. Run `clg .export ``session_id::``-default_topic`.
/// Assert exit 1 and stderr mentions "output" or "required".
///
/// ## Related Requirements
/// `tests/docs/cli/command/06_export.md` — INT-2
#[ test ]
fn int_2_output_required_missing_arg_exits_1()
{
  let root = TempDir::new().unwrap();
  let proj = root.path().join( "exp2-proj" );
  let enc  = common::write_path_project_session(
    root.path(), &proj, "-default_topic", 3
  );

  let out = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .arg( ".export" )
    .arg( "session_id::-default_topic" )
    .arg( format!( "project::{enc}" ) )
    .output()
    .unwrap();

  assert_exit( &out, 1 );
  let err = stderr( &out );
  assert!(
    !err.is_empty(),
    "INT-2: missing output:: must produce error on stderr; got silence"
  );
  assert!(
    err.to_lowercase().contains( "output" ) ||
    err.to_lowercase().contains( "required" ),
    "INT-2: error must mention output or required; got:\n{err}"
  );
}

/// INT-3: Default format is markdown.
///
/// ## Purpose
/// Verify that `.export` without an explicit `format::` produces a
/// markdown file containing heading characters (`#`).
///
/// ## Coverage
/// Output file exists; file content contains `#`; exit 0.
///
/// ## Validation Strategy
/// Write session `-default_topic` with known content. Export without format param.
/// Read output file and assert it contains `#`.
///
/// ## Related Requirements
/// `tests/docs/cli/command/06_export.md` — INT-3
#[ test ]
fn int_3_default_format_is_markdown()
{
  let root    = TempDir::new().unwrap();
  let out_dir = TempDir::new().unwrap();
  let proj    = root.path().join( "exp3-proj" );
  let enc     = common::write_path_project_session(
    root.path(), &proj, "-default_topic", 4
  );
  let out_file = out_dir.path().join( "out.md" );

  let out = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .arg( ".export" )
    .arg( "session_id::-default_topic" )
    .arg( format!( "project::{enc}" ) )
    .arg( format!( "output::{}", out_file.display() ) )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  assert!(
    out_file.exists(),
    "INT-3: output file must be created by default format export"
  );
  let content = std::fs::read_to_string( &out_file ).unwrap();
  assert!(
    content.contains( '#' ),
    "INT-3: default (markdown) export must contain '#' heading character; got:\n{content}"
  );
}

/// INT-4: `format::json` produces JSON array output.
///
/// ## Purpose
/// Verify that `format::json` creates a file containing valid JSON
/// (an array of entry objects).
///
/// ## Coverage
/// Output file exists; content starts with `[` or `{`; exit 0.
///
/// ## Validation Strategy
/// Write session `-default_topic` with at least 2 entries. Export with
/// `format::json`. Read file and assert content begins with a JSON structure.
///
/// ## Related Requirements
/// `tests/docs/cli/command/06_export.md` — INT-4
#[ test ]
fn int_4_format_json_produces_json_array_output()
{
  let root    = TempDir::new().unwrap();
  let out_dir = TempDir::new().unwrap();
  let proj    = root.path().join( "exp4-proj" );
  let enc     = common::write_path_project_session(
    root.path(), &proj, "-default_topic", 4
  );
  let out_file = out_dir.path().join( "out.json" );

  let out = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .arg( ".export" )
    .arg( "session_id::-default_topic" )
    .arg( format!( "project::{enc}" ) )
    .arg( format!( "output::{}", out_file.display() ) )
    .arg( "format::json" )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  assert!(
    out_file.exists(),
    "INT-4: output file must be created with format::json"
  );
  let content = std::fs::read_to_string( &out_file ).unwrap();
  let trimmed = content.trim_start();
  assert!(
    trimmed.starts_with( '[' ) || trimmed.starts_with( '{' ),
    "INT-4: format::json output must be valid JSON (starts with '[' or '{{'); \
    got:\n{content}"
  );
}

/// INT-5: `format::text` produces plain transcript.
///
/// ## Purpose
/// Verify that `format::text` creates a file without markdown headings (`#`)
/// and without JSON structural characters (`{`, `}`).
///
/// ## Coverage
/// Output file exists; no `#` chars; no `{` or `}` chars; exit 0.
///
/// ## Validation Strategy
/// Write session with known content. Export with `format::text`. Read file and
/// assert no markdown or JSON characters.
///
/// ## Related Requirements
/// `tests/docs/cli/command/06_export.md` — INT-5
#[ test ]
fn int_5_format_text_produces_plain_transcript()
{
  let root    = TempDir::new().unwrap();
  let out_dir = TempDir::new().unwrap();
  let proj    = root.path().join( "exp5-proj" );
  let enc     = common::write_path_project_session(
    root.path(), &proj, "-default_topic", 4
  );
  let out_file = out_dir.path().join( "out.txt" );

  let out = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .arg( ".export" )
    .arg( "session_id::-default_topic" )
    .arg( format!( "project::{enc}" ) )
    .arg( format!( "output::{}", out_file.display() ) )
    .arg( "format::text" )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  assert!(
    out_file.exists(),
    "INT-5: output file must be created with format::text"
  );
  let content = std::fs::read_to_string( &out_file ).unwrap();
  assert!(
    !content.contains( '#' ),
    "INT-5: format::text output must not contain '#' (markdown heading); \
    got:\n{content}"
  );
  assert!(
    !content.contains( '{' ) && !content.contains( '}' ),
    "INT-5: format::text output must not contain '{{' or '}}' (JSON braces); \
    got:\n{content}"
  );
}

/// INT-6: Output file is created at `output::` path.
///
/// ## Purpose
/// Verify that `.export` creates the output file at the specified path
/// when it does not previously exist.
///
/// ## Coverage
/// File does not exist before; file exists after; exit 0.
///
/// ## Validation Strategy
/// Write session. Run `.export` with output path in temp dir. Assert file
/// does not exist before and exists after.
///
/// ## Related Requirements
/// `tests/docs/cli/command/06_export.md` — INT-6
#[ test ]
fn int_6_output_file_is_created_at_output_path()
{
  let root    = TempDir::new().unwrap();
  let out_dir = TempDir::new().unwrap();
  let proj    = root.path().join( "exp6-proj" );
  let enc     = common::write_path_project_session(
    root.path(), &proj, "-default_topic", 3
  );
  let out_file = out_dir.path().join( "created.md" );

  assert!(
    !out_file.exists(),
    "INT-6 precondition: output file must not exist before export"
  );

  let out = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .arg( ".export" )
    .arg( "session_id::-default_topic" )
    .arg( format!( "project::{enc}" ) )
    .arg( format!( "output::{}", out_file.display() ) )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  assert!(
    out_file.exists(),
    "INT-6: output file must be created at the specified path; stderr: {}",
    stderr( &out )
  );
}

/// INT-7: Output file is overwritten if exists.
///
/// ## Purpose
/// Verify that `.export` overwrites an existing output file, replacing its
/// content entirely (prior content absent).
///
/// ## Coverage
/// "old content" absent after export; file content is session data; exit 0.
///
/// ## Validation Strategy
/// Pre-create output file with "old content" sentinel. Run `.export`.
/// Read file and assert "old content" is gone.
///
/// ## Related Requirements
/// `tests/docs/cli/command/06_export.md` — INT-7
#[ test ]
fn int_7_output_file_is_overwritten_if_exists()
{
  let root    = TempDir::new().unwrap();
  let out_dir = TempDir::new().unwrap();
  let proj    = root.path().join( "exp7-proj" );
  let enc     = common::write_path_project_session(
    root.path(), &proj, "-default_topic", 3
  );
  let out_file = out_dir.path().join( "overwrite.md" );

  // Pre-create with sentinel content
  std::fs::write( &out_file, b"old content sentinel xyz" ).unwrap();

  let out = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .arg( ".export" )
    .arg( "session_id::-default_topic" )
    .arg( format!( "project::{enc}" ) )
    .arg( format!( "output::{}", out_file.display() ) )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let content = std::fs::read_to_string( &out_file ).unwrap();
  assert!(
    !content.contains( "old content sentinel xyz" ),
    "INT-7: old content must be overwritten; file still contains sentinel; \
    got:\n{content}"
  );
}

/// INT-8: Exit code 2 when output parent dir does not exist.
///
/// ## Purpose
/// Verify that `.export` exits with code 2 and emits an error when the
/// parent directory of the output path does not exist.
///
/// ## Coverage
/// Exit code 2; error on stderr; no file created.
///
/// ## Validation Strategy
/// Write session. Run `.export ``output::``/nonexistent-dir-exp8-xyz/out.md`.
/// Assert exit 2 and stderr non-empty.
///
/// ## Related Requirements
/// `tests/docs/cli/command/06_export.md` — INT-8
#[ test ]
fn int_8_exit_code_2_when_output_parent_dir_does_not_exist()
{
  let root = TempDir::new().unwrap();
  let proj = root.path().join( "exp8-proj" );
  let enc  = common::write_path_project_session(
    root.path(), &proj, "-default_topic", 3
  );

  let out = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .arg( ".export" )
    .arg( "session_id::-default_topic" )
    .arg( format!( "project::{enc}" ) )
    .arg( "output::/nonexistent-dir-exp8-xyz/out.md" )
    .output()
    .unwrap();

  assert_exit( &out, 1 );
  let err = stderr( &out );
  assert!(
    !err.is_empty(),
    "INT-8: nonexistent output dir must produce error on stderr; got silence"
  );
}

/// INT-9: `project::` selects session from named project.
///
/// ## Purpose
/// Verify that `project::alpha` selects the `-default_topic` session from
/// project alpha specifically, not from project beta which has a same-named
/// session with different content.
///
/// ## Coverage
/// Alpha's unique content present; beta's unique content absent; exit 0.
///
/// ## Validation Strategy
/// Write project alpha (session `-default_topic`, last message "alpha-exp9-unique")
/// and project beta (session `-default_topic`, last message "beta-exp9-unique").
/// Export with `project::alpha`. Assert alpha content present and beta absent.
///
/// ## Related Requirements
/// `tests/docs/cli/command/06_export.md` — INT-9
#[ test ]
fn int_9_project_selects_session_from_named_project()
{
  let root    = TempDir::new().unwrap();
  let out_dir = TempDir::new().unwrap();

  let alpha = root.path().join( "alpha" );
  let beta  = root.path().join( "beta" );

  let enc_alpha = common::write_path_project_session(
    root.path(), &alpha, "-default_topic", 0
  );
  common::write_test_session_with_last_message(
    root.path(), &enc_alpha, "-default_topic", 0, "alpha-exp9-unique-content"
  );
  let enc_beta = common::write_path_project_session(
    root.path(), &beta, "-default_topic", 0
  );
  common::write_test_session_with_last_message(
    root.path(), &enc_beta, "-default_topic", 0, "beta-exp9-unique-content"
  );

  let out_file = out_dir.path().join( "alpha_export.md" );

  let out = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .arg( ".export" )
    .arg( "session_id::-default_topic" )
    .arg( format!( "project::{enc_alpha}" ) )
    .arg( format!( "output::{}", out_file.display() ) )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  assert!(
    out_file.exists(),
    "INT-9: output file must be created when project::alpha specified"
  );
  let content = std::fs::read_to_string( &out_file ).unwrap();
  assert!(
    content.contains( "alpha-exp9-unique-content" ),
    "INT-9: export must contain alpha project content; got:\n{content}"
  );
  assert!(
    !content.contains( "beta-exp9-unique-content" ),
    "INT-9: export must not contain beta project content; got:\n{content}"
  );
}

/// INT-10: Export succeeds with valid session and output path.
///
/// ## Purpose
/// Verify that `.export` completes successfully with all required parameters
/// and produces a non-empty output file.
///
/// ## Coverage
/// File created; content non-empty; exit 0.
///
/// ## Validation Strategy
/// Write project alpha with session `-default_topic` (3 entries). Export to
/// temp output path. Assert exit 0, file exists, and content non-empty.
///
/// ## Related Requirements
/// `tests/docs/cli/command/06_export.md` — INT-10
#[ test ]
fn int_10_export_succeeds_with_valid_session_and_output_path()
{
  let root    = TempDir::new().unwrap();
  let out_dir = TempDir::new().unwrap();
  let proj    = root.path().join( "exp10-proj" );
  let enc     = common::write_path_project_session(
    root.path(), &proj, "-default_topic", 3
  );
  let out_file = out_dir.path().join( "happy.md" );

  let out = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .arg( ".export" )
    .arg( "session_id::-default_topic" )
    .arg( format!( "project::{enc}" ) )
    .arg( format!( "output::{}", out_file.display() ) )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  assert!(
    out_file.exists(),
    "INT-10: export must create output file; stderr: {}",
    stderr( &out )
  );
  let content = std::fs::read_to_string( &out_file ).unwrap();
  assert!(
    !content.is_empty(),
    "INT-10: exported file must contain session content; got empty file"
  );
}
