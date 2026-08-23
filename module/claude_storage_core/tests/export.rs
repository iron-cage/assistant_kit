//! Export functionality integration tests
//!
//! Tests for exporting sessions to markdown, JSON, and text formats.
//!
//! Every test builds its own deterministic storage tree in a temp directory via
//! `storage_fixture` and asserts against the exact bytes each format produces. No
//! test reads the developer's real `~/.claude/` directory, so none of them can be
//! skipped by an empty-storage guard.
#![ cfg( unix ) ]
// `core` has no `io` module — `Cursor`'s std::io::{Read,Write} impls require std; no core equivalent exists.
#![ allow( clippy::std_instead_of_core ) ]

mod storage_fixture;

use claude_storage_core::{ ExportFormat, export_session, export_session_to_file };
use std::io::Cursor;
use std::path::PathBuf;
use tempfile::TempDir;

/// Session id every fixture in this file writes.
const SESSION : &str = "aaaaaaaa-bbbb-cccc-dddd-eeeeeeeeeeee";

/// Build a one-project, one-session storage tree holding `lines`.
///
/// Returns the temp root — which must stay alive for the whole test — and the
/// session file's path, so expectations can name the exact path export embeds.
fn fixture( lines : &[ String ] ) -> ( TempDir, PathBuf )
{
  let temp = storage_fixture::storage_root();
  let project = storage_fixture::project_dir( temp.path(), "-home-user-alpha" );
  let path = storage_fixture::write_session( &project, SESSION, lines );
  ( temp, path )
}

/// Test markdown export format
///
/// ## Purpose
///
/// Verifies markdown export emits the exact document shape `.export format::md`
/// depends on: a session header carrying path, entry count and both timestamps,
/// then one `## Entry N - Role` section per conversation entry, each closed by a
/// horizontal rule.
///
/// ## Coverage
///
/// A two-entry conversation — one user entry, one assistant entry — exported to an
/// in-memory writer.
///
/// ## Validation Strategy
///
/// Builds a temp storage tree holding exactly one project and one session with
/// known text and known timestamps, then asserts byte-for-byte equality against the
/// full expected markdown document: every header field, both entry sections, and
/// every separator. Nothing is conditional — a missing header field, a reordered
/// section or a dropped entry all fail the test.
///
/// ## Related Requirements
///
/// `docs/feature/003_export_formats.md` § Design — markdown format contract
#[ test ]
fn export_markdown_basic()
{
  let session_id = SESSION;
  let lines =
  [
    storage_fixture::user_line( session_id, 1, "Hello from the user" ),
    storage_fixture::assistant_line( session_id, 2, "Hello from the assistant" ),
  ];
  let ( temp, path ) = fixture( &lines );
  let mut session = storage_fixture::single_session( temp.path() );

  let mut output = Cursor::new( Vec::new() );
  export_session( &mut session, ExportFormat::Markdown, &mut output )
    .expect( "markdown export must succeed" );
  let result = String::from_utf8( output.into_inner() ).expect( "export output must be UTF-8" );

  let expected = format!
  (
    "# Session: {session_id}\n\n\
     **Path**: `{path}`\n\
     **Entries**: 2\n\
     **Created**: 2026-01-01T00:00:01Z\n\
     **Last Updated**: 2026-01-01T00:00:02Z\n\
     \n---\n\n\
     ## Entry 1 - User\n\
     *2026-01-01T00:00:01Z*\n\n\
     Hello from the user\n\n\
     ---\n\n\
     ## Entry 2 - Assistant\n\
     *2026-01-01T00:00:02Z*\n\n\
     Hello from the assistant\n\n\
     ---\n\n",
    path = path.display(),
  );

  assert_eq!( result, expected, "markdown export must match the full expected document" );
}

/// Test JSON export format
///
/// ## Purpose
///
/// Verifies JSON export wraps the session's raw JSONL lines verbatim in a single
/// object carrying `session_id`, `storage_path` and an `entries` array.
///
/// ## Coverage
///
/// A two-entry conversation exported to an in-memory writer.
///
/// ## Validation Strategy
///
/// Builds a temp storage tree with known JSONL lines and asserts the export equals
/// the exact expected object — the two source lines joined by a comma inside
/// `entries`, the fixture's session id, and the real on-disk session path. Because
/// the fixture owns the input lines, the expectation is derived from them rather
/// than restated, so a mangled or re-encoded entry fails the comparison.
///
/// ## Related Requirements
///
/// `docs/feature/003_export_formats.md` § Design — JSON format contract
#[ test ]
fn export_json_basic()
{
  let session_id = SESSION;
  let lines =
  [
    storage_fixture::user_line( session_id, 1, "Hello from the user" ),
    storage_fixture::assistant_line( session_id, 2, "Hello from the assistant" ),
  ];
  let ( temp, path ) = fixture( &lines );
  let mut session = storage_fixture::single_session( temp.path() );

  let mut output = Cursor::new( Vec::new() );
  export_session( &mut session, ExportFormat::Json, &mut output )
    .expect( "JSON export must succeed" );
  let result = String::from_utf8( output.into_inner() ).expect( "export output must be UTF-8" );

  let expected = format!
  (
    "{{\"session_id\":\"{session_id}\",\"storage_path\":\"{path}\",\"entries\":[{entries}]}}\n",
    path = path.display(),
    entries = lines.join( "," ),
  );

  assert_eq!( result, expected, "JSON export must wrap the raw JSONL lines verbatim" );
  assert!( result.starts_with( '{' ), "JSON export must start with an open brace" );
  assert!( result.trim_end().ends_with( '}' ), "JSON export must end with a close brace" );
}

/// Test text export format
///
/// ## Purpose
///
/// Verifies plain-text export emits the exact document `.export format::txt`
/// depends on: an unadorned header, then one `[Role] timestamp` block per
/// conversation entry separated by horizontal rules.
///
/// ## Coverage
///
/// A two-entry conversation — one user entry, one assistant entry — exported to an
/// in-memory writer.
///
/// ## Validation Strategy
///
/// Builds a temp storage tree with known text and known timestamps, then asserts
/// byte-for-byte equality against the full expected text document, including the
/// absence of any markdown decoration. Nothing is conditional.
///
/// ## Related Requirements
///
/// `docs/feature/003_export_formats.md` § Design — text format contract
#[ test ]
fn export_text_basic()
{
  let session_id = SESSION;
  let lines =
  [
    storage_fixture::user_line( session_id, 1, "Hello from the user" ),
    storage_fixture::assistant_line( session_id, 2, "Hello from the assistant" ),
  ];
  let ( temp, path ) = fixture( &lines );
  let mut session = storage_fixture::single_session( temp.path() );

  let mut output = Cursor::new( Vec::new() );
  export_session( &mut session, ExportFormat::Text, &mut output )
    .expect( "text export must succeed" );
  let result = String::from_utf8( output.into_inner() ).expect( "export output must be UTF-8" );

  let expected = format!
  (
    "Session: {session_id}\n\
     Path: {path}\n\
     Entries: 2\n\
     \n---\n\n\
     [User] 2026-01-01T00:00:01Z\n\
     Hello from the user\n\n\
     ---\n\n\
     [Assistant] 2026-01-01T00:00:02Z\n\
     Hello from the assistant\n\n\
     ---\n\n",
    path = path.display(),
  );

  assert_eq!( result, expected, "text export must match the full expected document" );
  assert!( !result.contains( "# Session:" ), "text export must not emit markdown headings" );
  assert!( !result.contains( "**Path**" ), "text export must not emit markdown emphasis" );
}

/// Test `ExportFormat::from_str`
#[ test ]
fn export_format_from_str()
{
  // Test valid formats
  assert_eq!
  (
    ExportFormat::from_str( "markdown" ).unwrap(),
    ExportFormat::Markdown
  );

  assert_eq!
  (
    ExportFormat::from_str( "md" ).unwrap(),
    ExportFormat::Markdown
  );

  assert_eq!
  (
    ExportFormat::from_str( "json" ).unwrap(),
    ExportFormat::Json
  );

  assert_eq!
  (
    ExportFormat::from_str( "text" ).unwrap(),
    ExportFormat::Text
  );

  assert_eq!
  (
    ExportFormat::from_str( "txt" ).unwrap(),
    ExportFormat::Text
  );

  // Test case insensitivity
  assert_eq!
  (
    ExportFormat::from_str( "MARKDOWN" ).unwrap(),
    ExportFormat::Markdown
  );

  // Test invalid format
  assert!( ExportFormat::from_str( "invalid" ).is_err() );
}

/// Test `ExportFormat::extension`
#[ test ]
fn export_format_extension()
{
  assert_eq!( ExportFormat::Markdown.extension(), "md" );
  assert_eq!( ExportFormat::Json.extension(), "json" );
  assert_eq!( ExportFormat::Text.extension(), "txt" );
}

/// Test export to file
///
/// ## Purpose
///
/// Verifies `export_session_to_file` writes to disk exactly what
/// `export_session` writes to a writer — the file path must be the only
/// difference between the two entry points.
///
/// ## Coverage
///
/// The same two-entry conversation exported twice: once to an in-memory writer,
/// once to a file named with `ExportFormat::extension()`.
///
/// ## Validation Strategy
///
/// Builds a temp storage tree, exports the session both ways, and asserts the
/// file's contents equal the writer's bytes character for character, then spot
/// checks the header and both entries so a mutually-empty result cannot pass.
///
/// ## Related Requirements
///
/// `docs/feature/003_export_formats.md` § Design — file output and extension
#[ test ]
fn export_to_file()
{
  let session_id = SESSION;
  let lines =
  [
    storage_fixture::user_line( session_id, 1, "Hello from the user" ),
    storage_fixture::assistant_line( session_id, 2, "Hello from the assistant" ),
  ];
  let ( temp, _path ) = fixture( &lines );
  let mut session = storage_fixture::single_session( temp.path() );

  let mut buffer = Cursor::new( Vec::new() );
  export_session( &mut session, ExportFormat::Markdown, &mut buffer )
    .expect( "writer export must succeed" );
  let via_writer = String::from_utf8( buffer.into_inner() ).expect( "export output must be UTF-8" );

  let output_path = temp.path().join( format!( "export.{}", ExportFormat::Markdown.extension() ) );
  export_session_to_file( &mut session, ExportFormat::Markdown, &output_path )
    .expect( "file export must succeed" );

  assert!( output_path.exists(), "file export must create the output file" );

  let written = std::fs::read_to_string( &output_path ).expect( "exported file must be readable" );

  assert_eq!( written, via_writer, "file export must produce the same bytes as the writer export" );
  assert!( written.contains( &format!( "# Session: {session_id}" ) ), "exported file must carry the session header" );
  assert!( written.contains( "## Entry 1 - User" ), "exported file must carry the user entry" );
  assert!( written.contains( "Hello from the assistant" ), "exported file must carry the assistant text" );
}

/// Test markdown with thinking blocks
///
/// ## Purpose
///
/// Verifies an assistant entry carrying a thinking block renders as a collapsible
/// `<details>` section whose summary reports the thinking block's token count, with
/// the assistant's visible text following outside the collapsed region.
///
/// ## Coverage
///
/// A two-entry conversation whose assistant entry holds a thinking block followed
/// by a text block.
///
/// ## Validation Strategy
///
/// Builds a temp storage tree containing a thinking block of known length — four
/// whitespace-separated words, so the reported token count is deterministic — and
/// asserts byte-for-byte equality against the full expected markdown document,
/// including the exact `<summary>Thinking (4 tokens)</summary>` line and the
/// `</details>` close. A count of one `<details>` open proves the block is not
/// duplicated across the two entries.
///
/// ## Related Requirements
///
/// `docs/feature/003_export_formats.md` § Design — thinking block rendering
#[ test ]
fn export_markdown_with_thinking()
{
  let session_id = SESSION;
  let lines =
  [
    storage_fixture::user_line( session_id, 1, "Please think about it" ),
    storage_fixture::assistant_thinking_line( session_id, 2, "Let me reason carefully", "Here is the answer" ),
  ];
  let ( temp, path ) = fixture( &lines );
  let mut session = storage_fixture::single_session( temp.path() );

  let mut output = Cursor::new( Vec::new() );
  export_session( &mut session, ExportFormat::Markdown, &mut output )
    .expect( "markdown export must succeed" );
  let result = String::from_utf8( output.into_inner() ).expect( "export output must be UTF-8" );

  let expected = format!
  (
    "# Session: {session_id}\n\n\
     **Path**: `{path}`\n\
     **Entries**: 2\n\
     **Created**: 2026-01-01T00:00:01Z\n\
     **Last Updated**: 2026-01-01T00:00:02Z\n\
     \n---\n\n\
     ## Entry 1 - User\n\
     *2026-01-01T00:00:01Z*\n\n\
     Please think about it\n\n\
     ---\n\n\
     ## Entry 2 - Assistant\n\
     *2026-01-01T00:00:02Z*\n\n\
     <details>\n\
     <summary>Thinking (4 tokens)</summary>\n\n\
     Let me reason carefully\n\
     </details>\n\n\
     Here is the answer\n\n\
     ---\n\n",
    path = path.display(),
  );

  assert_eq!( result, expected, "markdown export must render the thinking block as a details section" );
  assert_eq!( result.matches( "<details>" ).count(), 1, "exactly one collapsible thinking block" );
  assert_eq!( result.matches( "</details>" ).count(), 1, "the thinking block must be closed exactly once" );
}

/// Test export of sessions containing non-conversation metadata entries
///
/// ## Purpose
///
/// Real Claude Code sessions interleave non-conversation metadata entries —
/// `queue-operation`, `summary`, `file-history-snapshot` — with the conversation.
/// Export must skip them silently rather than fail, and must not leak their
/// contents into the rendered document.
///
/// ## Coverage
///
/// A five-line session: three metadata entries surrounding one user and one
/// assistant conversation entry.
///
/// ## Validation Strategy
///
/// Builds a temp storage tree with the metadata entries placed first, in the
/// middle, and last, then asserts byte-for-byte equality against the expected
/// markdown — which contains exactly the two conversation entries and reports
/// `**Entries**: 2`. Explicit negative assertions prove no metadata type name or
/// payload reached the output.
///
/// ## Related Requirements
///
/// `docs/feature/003_export_formats.md` § Design — metadata entries are skipped
#[ test ]
fn export_with_metadata_entries()
{
  let session_id = SESSION;
  let lines =
  [
    storage_fixture::metadata_line( "queue-operation", 1 ),
    storage_fixture::user_line( session_id, 2, "Hello from the user" ),
    storage_fixture::metadata_line( "summary", 3 ),
    storage_fixture::assistant_line( session_id, 4, "Hello from the assistant" ),
    storage_fixture::metadata_line( "file-history-snapshot", 5 ),
  ];
  let ( temp, path ) = fixture( &lines );
  let mut session = storage_fixture::single_session( temp.path() );

  let mut output = Cursor::new( Vec::new() );
  export_session( &mut session, ExportFormat::Markdown, &mut output )
    .expect( "export must succeed despite metadata entries" );
  let result = String::from_utf8( output.into_inner() ).expect( "export output must be UTF-8" );

  let expected = format!
  (
    "# Session: {session_id}\n\n\
     **Path**: `{path}`\n\
     **Entries**: 2\n\
     **Created**: 2026-01-01T00:00:02Z\n\
     **Last Updated**: 2026-01-01T00:00:04Z\n\
     \n---\n\n\
     ## Entry 1 - User\n\
     *2026-01-01T00:00:02Z*\n\n\
     Hello from the user\n\n\
     ---\n\n\
     ## Entry 2 - Assistant\n\
     *2026-01-01T00:00:04Z*\n\n\
     Hello from the assistant\n\n\
     ---\n\n",
    path = path.display(),
  );

  assert_eq!( result, expected, "only the two conversation entries may be rendered" );
  assert_eq!( result.matches( "## Entry " ).count(), 2, "metadata entries must not become entry sections" );
  assert!( !result.contains( "queue-operation" ), "queue-operation entry must not leak into the export" );
  assert!( !result.contains( "file-history-snapshot" ), "file-history-snapshot entry must not leak into the export" );
  assert!( !result.contains( "non-conversation metadata" ), "metadata payload must not leak into the export" );
}

/// Bug Reproducer (issue-019): `ExportFormat::from_str()` returned `std::io::Error`, producing
/// "I/O error during unknown operation: Unknown export format: xml" for invalid format strings.
///
/// ## Root Cause
///
/// `from_str()` used `std::io::Error::new(InvalidInput, "...").into()` for the validation
/// failure. The blanket `From<io::Error> for Error` impl sets context to "unknown operation",
/// so the error displayed as "I/O error during unknown operation: Unknown export format: xml"
/// — misleading because no I/O operation was attempted.
///
/// ## Why Not Caught
///
/// Tests only tested valid format strings (markdown, json, text). No test verified the
/// error message content for invalid formats, only that `from_str()` returned `Err`.
///
/// ## Fix Applied
///
/// Changed the `_` match arm in `from_str()` to return `Error::WriteFailed` with a clear
/// message listing valid format options, instead of wrapping in `std::io::Error`.
///
/// ## Prevention
///
/// Never use `std::io::Error` for non-I/O validation failures. The blanket `From<io::Error>`
/// impl always produces "unknown operation" context, which obscures the actual error.
///
/// ## Pitfall
///
/// The `.into()` shorthand on a `std::io::Error` silently goes through `From<io::Error> for
/// crate::Error`, setting a generic "unknown operation" context. Always use crate-level
/// error constructors for semantic errors — reserve `std::io::Error` for actual file I/O.
// test_kind: bug_reproducer(issue-019)
#[ test ]
fn export_format_invalid_string_returns_clear_error()
{
  // Before fix: "I/O error during unknown operation: Unknown export format: xml"
  // After fix: clear message listing valid options, no "I/O error" prefix

  let result = ExportFormat::from_str( "xml" );
  assert!( result.is_err(), "Unknown format 'xml' must return Err" );

  let err_msg = result.unwrap_err().to_string();

  // Must NOT contain the confusing "I/O error" prefix
  assert!(
    !err_msg.contains( "I/O error" ),
    "Error for invalid format should not say 'I/O error'. Got: {err_msg}"
  );

  // Must contain the invalid format name so user knows what they typed
  assert!(
    err_msg.contains( "xml" ),
    "Error must reference the invalid format value. Got: {err_msg}"
  );

  // Verify all valid format aliases still parse correctly
  assert!( ExportFormat::from_str( "markdown" ).is_ok() );
  assert!( ExportFormat::from_str( "md" ).is_ok() );
  assert!( ExportFormat::from_str( "json" ).is_ok() );
  assert!( ExportFormat::from_str( "text" ).is_ok() );
  assert!( ExportFormat::from_str( "txt" ).is_ok() );

  // Case insensitive
  assert!( ExportFormat::from_str( "Markdown" ).is_ok() );
  assert!( ExportFormat::from_str( "JSON" ).is_ok() );
}
