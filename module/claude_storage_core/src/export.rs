//! Export functionality for Claude Code storage sessions
//!
//! Supports exporting sessions to multiple formats:
//! - Markdown: Human-readable conversation format
//! - JSON: Machine-readable structured format
//! - Text: Simple conversation transcript

use crate::{ Session, Entry, ContentBlock, MessageContent, EntryType, Result, Error };
use std::io::Write;
use std::path::Path;
use std::fs::File;

/// Export format specification
#[derive( Debug, Clone, Copy, PartialEq, Eq )]
pub enum ExportFormat
{
  /// Markdown format (.md)
  Markdown,

  /// JSON format (.json)
  Json,

  /// Plain text format (.txt)
  Text,
}

impl ExportFormat
{
  /// Get file extension for format
  #[must_use]
  #[inline]
  pub fn extension( &self ) -> &'static str
  {
    match self
    {
      ExportFormat::Markdown => "md",
      ExportFormat::Json => "json",
      ExportFormat::Text => "txt",
    }
  }

  /// Parse format from string
  ///
  /// Note: This is not the standard `FromStr` trait to avoid confusion with
  /// Result type mismatch. CLI code uses this method directly.
  /// # Errors
  ///
  /// Returns error if the string does not match a known export format name.
  #[ allow( clippy::should_implement_trait ) ]
  #[inline]
  pub fn from_str( s : &str ) -> Result< Self >
  {
    match s.to_lowercase().as_str()
    {
      "markdown" | "md" => Ok( ExportFormat::Markdown ),
      "json" => Ok( ExportFormat::Json ),
      "text" | "txt" => Ok( ExportFormat::Text ),
      // Fix(issue-019): Use crate Error type instead of std::io::Error for format validation.
      //
      // Root cause: `std::io::Error::new(InvalidInput, ...).into()` converts to
      // `Error::Io { context: "unknown operation", ... }` via the blanket From impl.
      // Display produces the misleading "I/O error during unknown operation: Unknown export
      // format: xml" instead of a clear validation message.
      //
      // Pitfall: Never use `std::io::Error` for non-I/O validation failures. The blanket
      // `From<io::Error> for Error` impl always sets context to "unknown operation", which
      // is confusing to users. Use the crate's semantic error types directly.
      _ =>
      {
        Err
        (
          Error::WriteFailed
          {
            target : "export format".into(),
            reason : format!
            (
              "unknown format '{s}'; valid values: markdown (or md), json, text (or txt)"
            ),
          }
        )
      }
    }
  }
}

/// Export a session to a writer
///
/// Streams session content to the provided writer in the specified format.
/// Memory-efficient for large sessions (doesn't load entire session into memory).
///
/// # Errors
///
/// Returns error if loading session entries fails or if writing to the writer fails.
#[inline]
pub fn export_session< W : Write >
(
  session : &mut Session,
  format : ExportFormat,
  writer : &mut W,
) -> Result< () >
{
  match format
  {
    ExportFormat::Markdown => export_markdown( session, writer ),
    ExportFormat::Json => export_json( session, writer ),
    ExportFormat::Text => export_text( session, writer ),
  }
}

/// Export a session to a file
///
/// Convenience function that creates a file and exports to it.
///
/// # Errors
///
/// Returns error if the output file cannot be created, if exporting fails,
/// or if flushing the file to disk fails.
#[inline]
pub fn export_session_to_file
(
  session : &mut Session,
  format : ExportFormat,
  output_path : &Path,
) -> Result< () >
{
  // Fix(issue-026): Use Error::io() with context instead of bare `?` on File::create.
  //
  // Root cause: The blanket `From<io::Error> for Error` sets context to "unknown operation",
  // producing "I/O error during unknown operation: No such file or directory". This gives no
  // indication that the file creation failed or which path was involved.
  //
  // Pitfall: Always use `.map_err(|e| Error::io(e, context))` when converting IO errors
  // that benefit from path context. The `?` operator silently strips path information.
  let mut file = File::create( output_path )
    .map_err( | e | Error::io( e, format!( "create output file '{}'", output_path.display() ) ) )?;
  export_session( session, format, &mut file )?;
  file.sync_all()
    .map_err( | e | Error::io( e, format!( "flush output file '{}'", output_path.display() ) ) )?;
  Ok( () )
}

/// Export session as markdown
fn export_markdown< W : Write >
(
  session : &mut Session,
  writer : &mut W,
) -> Result< () >
{
  // Get session metadata before loading entries (to avoid borrow issues)
  let session_id = session.id().to_string();
  let storage_path = session.storage_path().to_path_buf();

  // Get stats first (before entries to avoid double borrow)
  let stats = session.stats()?;
  let first_timestamp = stats.first_timestamp.clone();
  let last_timestamp = stats.last_timestamp.clone();
  let total_entries = stats.total_entries;

  // Load entries
  let entries = session.entries()?;

  // Write header
  writeln!( writer, "# Session: {session_id}\n" )?;
  writeln!( writer, "**Path**: `{}`", storage_path.display() )?;
  writeln!( writer, "**Entries**: {total_entries}" )?;

  if let Some( first ) = first_timestamp
  {
    writeln!( writer, "**Created**: {first}" )?;
  }

  if let Some( last ) = last_timestamp
  {
    writeln!( writer, "**Last Updated**: {last}" )?;
  }

  writeln!( writer, "\n---\n" )?;

  // Write entries
  for ( idx, entry ) in entries.iter().enumerate()
  {
    write_markdown_entry( writer, entry, idx + 1 )?;
  }

  Ok( () )
}

/// Write a single entry in markdown format
fn write_markdown_entry< W : Write >
(
  writer : &mut W,
  entry : &Entry,
  entry_num : usize,
) -> Result< () >
{
  let role_name = match entry.entry_type
  {
    EntryType::User => "User",
    EntryType::Assistant => "Assistant",
  };

  writeln!( writer, "## Entry {entry_num} - {role_name}" )?;
  writeln!( writer, "*{}*\n", entry.timestamp )?;

  match &entry.message
  {
    MessageContent::User( user_msg ) =>
    {
      writeln!( writer, "{}\n", user_msg.content )?;
    }
    MessageContent::Assistant( assistant_msg ) =>
    {
      // Process content blocks
      for block in &assistant_msg.content
      {
        match block
        {
          ContentBlock::Thinking { thinking, .. } =>
          {
            // Collapsible thinking block
            let token_count = thinking.split_whitespace().count();
            writeln!( writer, "<details>" )?;
            writeln!( writer, "<summary>Thinking ({token_count} tokens)</summary>\n" )?;
            writeln!( writer, "{thinking}" )?;
            writeln!( writer, "</details>\n" )?;
          }
          ContentBlock::Text { text } =>
          {
            writeln!( writer, "{text}\n" )?;
          }
          ContentBlock::ToolUse { name, input, .. } =>
          {
            writeln!( writer, "**Tool Use**: `{name}`" )?;
            writeln!( writer, "```json" )?;
            writeln!( writer, "{input:#?}" )?;
            writeln!( writer, "```\n" )?;
          }
          ContentBlock::ToolResult { content, .. } =>
          {
            writeln!( writer, "**Tool Result**:" )?;
            writeln!( writer, "```" )?;
            writeln!( writer, "{content}" )?;
            writeln!( writer, "```\n" )?;
          }
        }
      }
    }
  }

  writeln!( writer, "---\n" )?;

  Ok( () )
}

/// Export session as JSON
///
/// Writes JSONL format: one JSON object per line, preserving the original
/// JSONL structure from the session file. Each line is parseable independently.
fn export_json< W : Write >
(
  session : &mut Session,
  writer : &mut W,
) -> Result< () >
{
  use std::io::{ BufRead, BufReader };
  use std::fs::File as StdFile;

  let storage_path = session.storage_path().to_path_buf();

  // Open session file and stream each JSONL line directly — preserves original format
  // and produces valid JSONL (one JSON object per line) for programmatic processing.
  let file = StdFile::open( &storage_path )?;
  let reader = BufReader::new( file );

  for line in reader.lines()
  {
    let line = line?;
    if !line.trim().is_empty()
    {
      writeln!( writer, "{line}" )?;
    }
  }

  Ok( () )
}

/// Export session as plain text
fn export_text< W : Write >
(
  session : &mut Session,
  writer : &mut W,
) -> Result< () >
{
  // Get session metadata before loading entries (to avoid borrow issues)
  let session_id = session.id().to_string();
  let storage_path = session.storage_path().to_path_buf();

  // Get stats first (before entries to avoid double borrow)
  let stats = session.stats()?;
  let total_entries = stats.total_entries;

  // Load entries
  let entries = session.entries()?;

  // Write header
  writeln!( writer, "Session: {session_id}" )?;
  writeln!( writer, "Path: {}", storage_path.display() )?;
  writeln!( writer, "Entries: {total_entries}" )?;
  writeln!( writer, "\n---\n" )?;

  // Write entries
  for entry in entries
  {
    write_text_entry( writer, entry )?;
  }

  Ok( () )
}

/// Write a single entry in text format
fn write_text_entry< W : Write >
(
  writer : &mut W,
  entry : &Entry,
) -> Result< () >
{
  let role_name = match entry.entry_type
  {
    EntryType::User => "User",
    EntryType::Assistant => "Assistant",
  };

  writeln!( writer, "[{}] {}", role_name, entry.timestamp )?;

  match &entry.message
  {
    MessageContent::User( user_msg ) =>
    {
      writeln!( writer, "{}\n", user_msg.content )?;
    }
    MessageContent::Assistant( assistant_msg ) =>
    {
      // Extract text content only (skip thinking blocks and tool use)
      for block in &assistant_msg.content
      {
        if let ContentBlock::Text { text } = block
        {
          writeln!( writer, "{text}" )?;
        }
        // Skip thinking, tool use, tool results in text format
      }
      writeln!( writer )?;
    }
  }

  writeln!( writer, "---\n" )?;

  Ok( () )
}
