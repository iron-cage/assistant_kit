//! Session management - represents a conversation session (JSONL file)

use std::
{
  fs,
  path::{ Path, PathBuf },
};

use crate::
{
  Entry,
  EntryType,
  Error,
  Result,
  SessionId,
  stats::SessionStats,
};

/// A conversation session (one JSONL file)
#[derive( Debug )]
pub struct Session
{
  /// Session UUID (filename without .jsonl extension)
  id : SessionId,

  /// Path to the JSONL file
  storage_path : PathBuf,

  /// Loaded entries (lazily loaded)
  entries : Option< Vec< Entry > >,
}

impl Session
{
  /// Create a session reference (doesn't load entries yet)
  #[must_use]
  #[inline]
  pub fn new( id : impl Into< SessionId >, storage_path : PathBuf ) -> Self
  {
    Self
    {
      id : id.into(),
      storage_path,
      entries : None,
    }
  }

  /// Load session from a JSONL file
  ///
  /// # Errors
  ///
  /// Returns error if the path does not exist, if the filename is missing or
  /// contains invalid UTF-8, or if the file does not have a `.jsonl` extension.
  #[inline]
  pub fn load( path : &Path ) -> Result< Self >
  {
    if !path.exists()
    {
      return Err( Error::session_not_found
      (
        path.to_string_lossy().to_string()
      ));
    }

    // Extract session ID from filename
    let filename = path
      .file_name()
      .ok_or_else( || Error::invalid_structure
      (
        path.to_path_buf(),
        "missing filename"
      ))?
      .to_str()
      .ok_or_else( || Error::invalid_structure
      (
        path.to_path_buf(),
        "filename contains invalid UTF-8"
      ))?;

    let id = filename
      .strip_suffix( ".jsonl" )
      .ok_or_else( || Error::invalid_structure
      (
        path.to_path_buf(),
        "not a .jsonl file"
      ))?;

    Ok( Self
    {
      id : SessionId::new( id ),
      storage_path : path.to_path_buf(),
      entries : None,
    })
  }

  /// Get session ID
  #[must_use]
  #[inline]
  pub fn id( &self ) -> &str
  {
    self.id.as_str()
  }

  /// Get storage path
  #[must_use] 
  #[inline]
  pub fn storage_path( &self ) -> &Path
  {
    &self.storage_path
  }

  /// Get entries (loads if not already loaded)
  ///
  /// # Errors
  ///
  /// Returns error if the JSONL file cannot be read from disk.
  ///
  /// # Panics
  ///
  /// Panics if the entries cache is unexpectedly absent after a successful load,
  /// which should never occur under normal operation.
  #[inline]
  pub fn entries( &mut self ) -> Result< &Vec< Entry > >
  {
    if self.entries.is_none()
    {
      self.load_entries()?;
    }

    Ok( self.entries.as_ref().unwrap() )
  }

  /// Force reload entries from disk
  ///
  /// # Errors
  ///
  /// Returns error if the JSONL file cannot be read from disk.
  #[inline]
  pub fn reload( &mut self ) -> Result< ()>
  {
    self.entries = None;
    self.load_entries()
  }

  /// Load entries from JSONL file
  ///
  /// Gracefully skips non-conversation entries (queue-operation, summary, etc.)
  /// that don't have the standard conversation entry structure.
  ///
  /// ## Graceful Degradation Design
  ///
  /// Real Claude Code sessions contain multiple entry types:
  /// - **Conversation entries** (`type:"user"`, `type:"assistant"`) - Have `uuid` field, parse successfully
  /// - **Metadata entries** (`type:"queue-operation"`, `type:"summary"`) - Lack `uuid` field, skipped
  ///
  /// Production data shows ~3-4% of entries are metadata that should be skipped.
  /// Example: 4901-line session had 4710 conversation entries + 191 metadata entries.
  ///
  /// This is intentional behavior, not a bug. We only need conversation entries for:
  /// - Statistics (entry counts, token usage)
  /// - Export (markdown/text output)
  /// - Search (finding user/assistant messages)
  ///
  /// Metadata entries serve Claude Code's internal operation (command queueing, summaries)
  /// but aren't part of the conversation history we expose.
  fn load_entries( &mut self ) -> Result< ()>
  {
    use std::io::{ BufRead, BufReader };

    let file = fs::File::open( &self.storage_path )
      .map_err( | e | Error::io
      (
        e,
        format!( "reading session file: {}", self.storage_path.display() )
      ))?;
    let reader = BufReader::new( file );

    let mut entries = Vec::new();

    // Fix(BUG-508)
    // Root cause: `fs::read_to_string()` validated UTF-8 across the WHOLE file in one pass, so a
    //   single invalid-UTF-8 byte anywhere failed this function before any line was processed —
    //   inconsistent with `search()`'s own per-line `BufReader::lines()` skip (BUG-503).
    // Pitfall: `fs::read_to_string()` and `BufReader::lines()` both look like "read this JSONL
    //   file," but only the latter validates UTF-8 per-line rather than across the whole buffer.
    for line in reader.lines()
    {
      let Ok( line ) = line else { continue; };

      if line.trim().is_empty()
      {
        continue; // Skip empty lines
      }

      // Try to parse as conversation entry; silently skip metadata entries
      // (queue-operation, summary, etc.) - graceful degradation, we only need conversation entries
      if let Ok( entry ) = Entry::from_json_line( &line )
      {
        entries.push( entry );
      }
    }

    self.entries = Some( entries );
    Ok( () )
  }

  /// Append an entry to the session (append-only operation)
  ///
  /// # Errors
  ///
  /// Returns error if the entry cannot be serialized to JSON, if the session
  /// file cannot be opened for appending, or if writing to the file fails.
  #[inline]
  pub fn append_entry( &mut self, entry : &Entry ) -> Result< ()>
  {
    let json_line = entry.to_json_line()?;

    // Append to file
    use std::io::Write;
    let mut file = fs::OpenOptions::new()
      .create( true )
      .append( true )
      .open( &self.storage_path )
      .map_err( | e | Error::io
      (
        e,
        format!( "opening session file for append: {}", self.storage_path.display() )
      ))?;

    writeln!( file, "{json_line}" )
      .map_err( | e | Error::io
      (
        e,
        format!( "appending to session file: {}", self.storage_path.display() )
      ))?;

    // Invalidate cache to force reload
    self.entries = None;

    Ok( () )
  }

  /// Count conversation entries (user + assistant) without loading them all into memory.
  ///
  /// Counts only entries with `"type":"user"` or `"type":"assistant"`. Metadata lines
  /// (queue-operation, summary, system, etc.) are excluded, matching the count shown by
  /// `stats().total_entries` and the "Total Entries" display in `.show`.
  ///
  /// # Implementation Note: String Search vs Full JSON Parse
  ///
  /// This function uses a fast byte-level substring search instead of full JSON parsing.
  /// JSON string-escape rules make this safe: a top-level field `"type":"user"` appears
  /// literally in the raw JSONL bytes, while the same text inside a content string is
  /// escaped to `\"type\":\"user\"`. The literal pattern therefore matches ONLY top-level
  /// type fields, never nested content values — no false positives or negatives are possible
  /// for well-formed Claude Code JSONL.
  ///
  /// # Fix(issue-016): `count_entries()` counted ALL JSONL lines, not just conversation entries
  ///
  /// Root cause: Original implementation used `content.lines().count()` — counted every
  /// non-empty JSONL line including internal metadata (`queue-operation`, `system`, `tool_use`).
  /// This produced counts 100+ higher than `stats().total_entries`, creating inconsistency
  /// between `.count target::entries` and "Total Entries" shown in `.show`.
  ///
  /// # Fix(issue-018): Full JSON parse in `count_entries()` made `.list min_entries::N` hang
  ///
  /// Root cause: Issue-016 fix replaced `lines().count()` with `parse_json()` per line.
  /// `matches_filter()` calls `count_entries()` for every session in every project.
  /// With 1903 projects / 2429 sessions / ~7 GB of JSONL, full JSON parsing per line
  /// caused `.list min_entries::N` to take > 2 minutes and SIGTERM in nextest.
  ///
  /// The string-search approach restores O(bytes)-but-cheap performance while keeping
  /// correctness: JSON escaping guarantees the literal `"type":"user"` pattern is unique
  /// to top-level type fields in well-formed JSONL.
  ///
  /// Pitfall: Never use a full JSON parse inside a function called O(session count) times.
  /// If you need to read one field from a JSONL line, prefer a targeted string search that
  /// exploits JSON escaping invariants rather than allocating and walking a full value tree.
  ///
  /// # Errors
  ///
  /// Returns error if the JSONL file cannot be read from disk.
  #[inline]
  pub fn count_entries( &self ) -> Result< usize >
  {
    use std::io::{ BufRead, BufReader };

    let file = fs::File::open( &self.storage_path )
      .map_err( | e | Error::io
      (
        e,
        format!( "reading session file: {}", self.storage_path.display() )
      ))?;
    let reader = BufReader::new( file );

    let mut count = 0usize;
    // Fix(BUG-508)
    // Root cause: `fs::read_to_string()` validated UTF-8 across the WHOLE file in one pass, so a
    //   single invalid-UTF-8 byte anywhere failed this function before any line was processed —
    //   inconsistent with `search()`'s own per-line `BufReader::lines()` skip (BUG-503).
    // Pitfall: `fs::read_to_string()` and `BufReader::lines()` both look like "read this JSONL
    //   file," but only the latter validates UTF-8 per-line rather than across the whole buffer.
    for line in reader.lines()
    {
      let Ok( line ) = line else { continue; };
      let t = line.trim();
      if t.is_empty() { continue; }

      // Fast path: JSON escaping makes `"type":"user"` unique to top-level type fields.
      // Content inside string values is always escaped (`\"type\":\"user\"`), so
      // literal matching is safe and avoids full JSON parse overhead.
      if t.contains( r#""type":"user""# )
        || t.contains( r#""type":"assistant""# )
        || t.contains( r#""type": "user""# )
        || t.contains( r#""type": "assistant""# )
      {
        count += 1;
      }
    }

    Ok( count )
  }

  /// Filter entries by type (user or assistant)
  ///
  /// Returns only entries matching the specified type. Requires entries to be loaded.
  ///
  /// # Errors
  ///
  /// Returns error if the session file cannot be read when entries are not yet loaded.
  #[inline]
  pub fn entries_by_type( &mut self, entry_type : EntryType ) -> Result< Vec< &Entry > >
  {
    let entries = self.entries()?;
    Ok( entries.iter().filter( | e | e.entry_type == entry_type ).collect() )
  }

  /// Filter entries after a specific timestamp
  ///
  /// Returns entries with timestamps after the specified ISO 8601 timestamp string.
  /// Requires entries to be loaded.
  ///
  /// # Errors
  ///
  /// Returns error if the session file cannot be read when entries are not yet loaded.
  #[inline]
  pub fn entries_after( &mut self, timestamp : &str ) -> Result< Vec< &Entry > >
  {
    let entries = self.entries()?;
    Ok( entries.iter().filter( | e | e.timestamp.as_str() > timestamp ).collect() )
  }

  /// Fix(BUG-491): Corrected doc comment — removed the false claim that this
  /// function also checks `isSidechain` on the session's entries.
  ///
  /// Root cause: the doc comment (present since the initial commit) described
  /// an aspirational second signal that was never implemented and doesn't
  /// match the canonical algorithm (`docs/algorithm/001_agent_session_tracking.md`),
  /// which deliberately keeps filename-based session classification
  /// (`is_agent_session`) separate from entry-level sidechain tagging
  /// (`is_agent_entry` — itself never implemented, since it depends on an
  /// `Entry::agent_id` field that was deliberately never added).
  ///
  /// Pitfall: a function's own doc comment can silently drift from the
  /// authoritative design doc, since nothing forces them to agree. When a doc
  /// comment and a canonical algorithm/design doc disagree, check which one
  /// matches actual behavior and existing test expectations before assuming
  /// the code is what's wrong.
  ///
  /// Detect if this is an agent/sidechain session
  ///
  /// Checks the filename convention only: `agent-{id}.jsonl` (`id` = filename
  /// without the `.jsonl` extension). Entry-level `isSidechain`/`agentId`
  /// tagging is a deliberately separate, per-entry concept — see
  /// `docs/algorithm/001_agent_session_tracking.md`'s `is_agent_entry` — not
  /// combined here because doing so would require loading entries (I/O) into
  /// what is otherwise a cheap, synchronous, filename-only check. In practice
  /// this doesn't lose detections: Claude Code always writes sidechain entries
  /// inside a file that also matches the `agent-*.jsonl` naming convention.
  #[must_use]
  #[inline]
  pub fn is_agent_session( &self ) -> bool
  {
    self.id.as_str().starts_with( "agent-" )
  }

  /// Compute session statistics
  ///
  /// Calculates comprehensive statistics including entry counts, token usage,
  /// and timestamps without parsing all entry content (only parses usage metadata).
  ///
  /// # Errors
  ///
  /// Returns error if the session file cannot be read, or if a line contains
  /// invalid JSON that cannot be parsed.
  ///
  /// Fix(issue-session-stats-entry-counting): Entry counting returned 0 for all sessions
  /// Root cause: Code checked for top-level `"role"` field instead of `"type"` field. Claude Code
  ///   v2.0+ format has `"type"` at top level ("user"|"assistant") and `"role"` nested inside
  ///   the `"message"` object, so checking `role` at top level always returned None.
  /// Pitfall: When parsing Claude Code storage, ALWAYS use the top-level `"type"` field to
  ///   determine entry type, not `"role"`. The role field exists but is nested inside message.
  #[inline]
  pub fn stats( &mut self ) -> Result< SessionStats >
  {
    use crate::json::parse_json;
    use std::io::{ BufRead, BufReader };

    let mut stats = SessionStats::new( self.id.as_str().to_string() );
    stats.is_agent_session = self.is_agent_session();

    // Read file content
    let file = fs::File::open( &self.storage_path )
      .map_err( | e | Error::io
      (
        e,
        format!( "reading session file: {}", self.storage_path.display() )
      ))?;
    let reader = BufReader::new( file );

    // Process each line
    // Fix(BUG-508)
    // Root cause: `fs::read_to_string()` validated UTF-8 across the WHOLE file in one pass, so a
    //   single invalid-UTF-8 byte anywhere failed this function before any line was processed —
    //   BUG-489's fix below already skips a per-line JSON *parse* failure, but that never covered
    //   this earlier read/decode layer.
    // Pitfall: fixing one failure layer of a per-line loop (parse) does not imply an earlier layer
    //   (read/decode) is also fixed — each must be swapped to a per-line-fallible strategy on its own.
    for line in reader.lines()
    {
      let Ok( line ) = line else { continue; };

      if line.trim().is_empty()
      {
        continue;
      }

      // Fix(BUG-489)
      // Root cause: `parse_json(line)?` hard-propagated any per-line JSON syntax error, unlike
      //   `load_entries()`'s `if let Ok(entry) = Entry::from_json_line(line) { entries.push(entry); }`,
      //   which silently skips any line it can't parse — a single malformed/truncated line anywhere
      //   in an otherwise-valid session file made `stats()` fail entirely.
      // Pitfall: sibling functions reading the same JSONL data must handle malformed input the same
      //   way — one graceful, one hard-failing creates command-dependent brittleness where `.tail`
      //   (`load_entries()`) succeeds on a file that `.show`/`.export` (`stats()`) reject outright.
      let Ok( json ) = parse_json( &line ) else { continue; };

      // Extract type - only count conversation entries, skip metadata entries
      // In Claude Code v2.0+, the top-level "type" field indicates entry type ("user" or "assistant")
      // Role is also available nested inside message.role, but we use type for consistency
      if let Some( entry_type ) = json.get_str( "type" )
      {
        match entry_type
        {
          "user" =>
          {
            stats.user_entries += 1;
            stats.total_entries += 1;
          }
          "assistant" =>
          {
            stats.assistant_entries += 1;
            stats.total_entries += 1;
          }
          _ =>
          {
            // Skip non-conversation entries (queue-operation, summary, etc.)
            continue;
          }
        }
      }

      // Extract timestamp
      if let Some( timestamp ) = json.get_str( "timestamp" )
      {
        if stats.first_timestamp.is_none()
        {
          stats.first_timestamp = Some( timestamp.to_string() );
        }
        stats.last_timestamp = Some( timestamp.to_string() );
      }

      // Extract token usage from assistant messages
      if let Some( "assistant" ) = json.get_str( "type" )
      {
        if let Some( message ) = json.get( "message" )
        {
          if let Some( usage ) = message.get( "usage" )
          {
            // Token counts from JSON are always non-negative integers stored as f64.
            // The cast to u64 is safe: values are positive and well within u64 range.
            #[ allow( clippy::cast_possible_truncation, clippy::cast_sign_loss ) ]
            {
              if let Some( input_tokens ) = usage.get_number( "input_tokens" )
              {
                stats.total_input_tokens += input_tokens as u64;
              }

              if let Some( output_tokens ) = usage.get_number( "output_tokens" )
              {
                stats.total_output_tokens += output_tokens as u64;
              }

              if let Some( cache_read ) = usage.get_number( "cache_read_input_tokens" )
              {
                stats.total_cache_read_tokens += cache_read as u64;
              }

              if let Some( cache_creation ) = usage.get_number( "cache_creation_input_tokens" )
              {
                stats.total_cache_creation_tokens += cache_creation as u64;
              }
            }
          }
        }
      }
    }

    Ok( stats )
  }

  /// Check if session matches filter
  ///
  /// ## Filtering Logic (AND composition)
  ///
  /// All filter conditions must match:
  /// - `agent_only`: If set, checks if session is agent session
  /// - `min_entries`: If set, checks if session has at least this many entries
  /// - `session_id_substring`: If set, checks if session ID contains substring (case-insensitive)
  ///
  /// ## Examples
  ///
  /// ```rust,no_run
  /// use claude_storage_core::{ Session, SessionFilter };
  /// use std::path::PathBuf;
  ///
  /// let mut session = Session::new( "agent-abc123", PathBuf::from( "/tmp/test.jsonl" ) );
  ///
  /// let filter = SessionFilter
  /// {
  ///   agent_only : Some( true ),
  ///   min_entries : None,
  ///   session_id_substring : None,
  /// };
  ///
  /// assert!( session.matches_filter( &filter ).unwrap() );
  /// ```
  /// # Errors
  ///
  /// Returns error if reading the session file fails when checking entry count.
  #[inline]
  pub fn matches_filter( &mut self, filter : &crate::SessionFilter ) -> Result< bool >
  {
    // Agent filter
    if let Some( agent_only ) = filter.agent_only
    {
      if self.is_agent_session() != agent_only
      {
        return Ok( false );
      }
    }

    // Minimum entries filter
    if let Some( min_entries ) = filter.min_entries
    {
      let count = self.count_entries()?;
      if count < min_entries
      {
        return Ok( false );
      }
    }

    // Session ID substring filter
    if let Some( ref substring ) = filter.session_id_substring
    {
      let matcher = crate::StringMatcher::new( substring );
      if !matcher.matches( self.id.as_str() )
      {
        return Ok( false );
      }
    }

    Ok( true )
  }

  /// Search session content for query
  ///
  /// Streams through JSONL file line-by-line for memory efficiency.
  /// Returns all matches with context.
  ///
  /// ## Examples
  ///
  /// ```
  /// use claude_storage_core::{ Session, SearchFilter };
  /// use std::path::PathBuf;
  ///
  /// # fn example() -> claude_storage_core::Result< () > {
  /// let mut session = Session::new( "test-id", PathBuf::from( "/tmp/test.jsonl" ) );
  /// let filter = SearchFilter::new( "error_tools" );
  /// let matches = session.search( &filter )?;
  ///
  /// for m in matches
  /// {
  ///   println!( "Entry #{}: {}", m.entry_index(), m.excerpt() );
  /// }
  /// # Ok( () )
  /// # }
  /// ```
  ///
  /// # Errors
  ///
  /// Returns error if the session file cannot be opened. A line that cannot be read (e.g.
  /// invalid UTF-8 from a crash-truncated write) is silently skipped, not surfaced as an error —
  /// matching `load_entries()`'s and `stats()`'s per-line graceful degradation.
  #[inline]
  pub fn search( &mut self, filter : &crate::SearchFilter ) -> Result< Vec< crate::SearchMatch > >
  {
    use std::io::{ BufRead, BufReader };
    use std::fs::File;

    let file = File::open( &self.storage_path )?;
    let reader = BufReader::new( file );

    let mut matches = Vec::new();
    let mut entry_index = 0;

    for line in reader.lines()
    {
      // Fix(BUG-503)
      // Root cause: `let line = line?;` hard-propagated a single unreadable (e.g. invalid-UTF-8)
      //   line's error through `?`, discarding every match already collected for this session —
      //   inconsistent with this same loop's own `Entry::from_json_line` skip a few lines below.
      // Pitfall: `BufReader::lines()` yields one `io::Result<String>` per line; treating a single
      //   line's `Err` as fatal for the whole stream (instead of skipping just that line) silently
      //   discards all results gathered so far, not just the unreadable line's own (non-)match.
      let Ok( line ) = line else
      {
        entry_index += 1;
        continue;
      };

      // Try to extract role and content from JSON without full parsing
      // This is a lightweight check to see if we should parse the full entry
      if !filter.matches_text( &line )
      {
        entry_index += 1;
        continue; // Skip this entry, doesn't contain query
      }

      // Parse entry to get role and content
      match Entry::from_json_line( &line )
      {
        Ok( entry ) =>
        {
          // Check entry type filter
          if let Some( target_type ) = filter.match_entry_type
          {
            if entry.entry_type() != target_type
            {
              entry_index += 1;
              continue;
            }
          }

          // Search in content blocks
          let content_text = entry.content_text();

          // Split into lines and search
          for ( line_num, content_line ) in content_text.lines().enumerate()
          {
            if filter.matches_text( content_line )
            {
              let search_match = crate::SearchMatch::new
              (
                entry_index,
                entry.entry_type(),
                line_num,
                content_line.to_string(),
              );
              matches.push( search_match );
            }
          }
        }
        Err( _e ) =>
        {
          // Skip malformed entries (graceful degradation)
        }
      }

      entry_index += 1;
    }

    Ok( matches )
  }
}

#[cfg( test )]
mod tests
{
  use super::*;
  use std::io::Write;
  use tempfile::NamedTempFile;

  #[test]
  fn test_session_new()
  {
    let session = Session::new
    (
      "test-uuid".to_string(),
      PathBuf::from( "test.jsonl" )
    );

    assert_eq!( session.id(), "test-uuid" );
  }

  #[test]
  fn test_session_load_missing_file()
  {
    let result = Session::load( Path::new( "/nonexistent/session.jsonl" ) );
    assert!( result.is_err() );
  }

  #[test]
  fn test_session_load_valid_file()
  {
    // Create temp JSONL file
    let mut temp_file = NamedTempFile::new().unwrap();
    writeln!( temp_file, r#"{{"uuid": "test"}}"# ).unwrap();

    // Rename to have .jsonl extension
    let jsonl_path = temp_file.path().with_extension( "jsonl" );
    fs::copy( temp_file.path(), &jsonl_path ).unwrap();

    let result = Session::load( &jsonl_path );

    // Cleanup
    let _ = fs::remove_file( jsonl_path );

    assert!( result.is_ok() );
  }
}
