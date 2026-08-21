//! Shared text formatting utilities for CLI output.

use super::color;
use claude_storage_core::{ ContentBlock, JsonValue };

/// Tool-input keys surfaced beside a tool name, most-specific first.
///
/// Chosen from the actual key distribution of the local store: `command` covers
/// `Bash`, `file_path` covers `Read`/`Edit`/`Write`, `pattern` covers
/// `Grep`/`Glob`, and the tail covers the task/agent/web tools. The first key
/// present on a block wins, so `pattern` precedes `path` to keep `Grep` showing
/// what it searched for rather than where.
const TOOL_SUMMARY_KEYS : [ &str; 11 ] =
[
  "command", "file_path", "notebook_path", "pattern", "path",
  "query", "url", "skill", "subject", "description", "prompt",
];

/// Longest tool-input summary rendered beside a tool name, in characters.
const TOOL_SUMMARY_MAX : usize = 64;

/// Nominal rendering width for `.tail`'s rule lines and right-aligned annotations.
///
/// Fixed rather than probed from the terminal: this crate has no terminal-size
/// dependency, and a fixed width keeps piped output byte-identical to what a
/// terminal shows, which is what the integration tests assert against.
pub( super ) const RULE_WIDTH : usize = 76;

/// What a tool call returned, folded onto the `⚙` line that invoked it.
pub( super ) struct ToolResultSummary
{
  /// Number of lines in the result body
  pub lines : usize,
  /// Whether the tool reported failure
  pub is_error : bool,
}

impl ToolResultSummary
{
  /// The `↳ …` annotation shown at the right edge of a tool-use line.
  pub( super ) fn label( &self ) -> String
  {
    if self.is_error
    {
      return "↳ error".to_string();
    }

    match self.lines
    {
      0 => "↳ empty".to_string(),
      1 => "↳ 1 line".to_string(),
      n => format!( "↳ {n} lines" ),
    }
  }
}

/// Tool results keyed by the `tool_use` id they answer.
pub( super ) type ToolResults = std::collections::HashMap< String, ToolResultSummary >;

/// Format entry content for display
///
/// ## Behavior
///
/// - Extracts actual message content from Entry
/// - Formats as readable chat log entry
/// - Supports text, thinking, tool use blocks
/// - Optional truncation for long messages
///
/// ## Format
///
/// ```text
/// 2025-12-02 09:57 · User:
/// message content here
///
/// 2025-12-02 09:58 · Assistant:
/// response content here
/// ```
///
/// ## Examples
///
/// ```text
/// let entry = session.entries()[0];
/// let formatted = format_entry_content( &entry, None );
/// // Output: "2025-12-02 09:57 · User:\nHello, Claude!"
/// ```
pub( super ) fn format_entry_content( entry : &claude_storage_core::Entry, max_length : Option< usize > ) -> String
{
  // Format timestamp
  let timestamp = format_timestamp( &entry.timestamp );

  let role = match entry.entry_type
  {
    claude_storage_core::EntryType::User => "User",
    claude_storage_core::EntryType::Assistant => "Assistant",
  };

  let content = render_blocks( entry.content_blocks() ).join( "\n\n" );

  // Apply truncation if needed
  let content = truncate_if_needed( &content, max_length );

  // Format as chat log entry
  let role_label = color::role( &format!( "{role}:" ) );
  format!( "{timestamp} · {role_label}\n{content}" )
}

/// Render content blocks into displayable pieces, one per meaningful block.
///
/// Blocks that carry nothing to read are dropped rather than rendered as a bare
/// label: an empty `text` or `thinking` block contributes no piece at all, and a
/// successful `tool_result` is suppressed because the conversation view shows
/// what was asked, not what came back. A `tool_use` renders its most telling
/// input value beside the tool name, so `Bash` reads as the command it ran.
pub( super ) fn render_blocks( blocks : &[ ContentBlock ] ) -> Vec< String >
{
  render_blocks_impl( blocks, None )
}

/// Render content blocks with each tool call annotated by what it returned.
///
/// Same as [`render_blocks`] except `tool_result` blocks are never rendered on
/// their own — they are folded onto the `⚙` line of the `tool_use` they answer,
/// as a right-aligned `↳ 3 lines` / `↳ error`. This is what collapses a
/// call-and-response pair into the single line a reader actually wants.
pub( super ) fn render_blocks_annotated( blocks : &[ ContentBlock ], results : &ToolResults ) -> Vec< String >
{
  render_blocks_impl( blocks, Some( results ) )
}

fn render_blocks_impl( blocks : &[ ContentBlock ], results : Option< &ToolResults > ) -> Vec< String >
{
  blocks
    .iter()
    .filter_map( | block | match block
    {
      ContentBlock::Text { text } => non_blank( text ),
      ContentBlock::Thinking { thinking, .. } =>
        non_blank( thinking ).map( | body | format!( "Thinking ·\n{body}" ) ),
      ContentBlock::ToolUse { id, name, input } =>
      {
        let Some( summary ) = results.and_then( | r | r.get( id ) )
        else
        {
          return Some( color::tool( &tool_use_line( name, input, RULE_WIDTH ) ) );
        };

        // Budget the headline around the annotation rather than let the two run
        // together — a tool name plus a long path exceeds the width on its own.
        let label = summary.label();
        let headline = tool_use_line( name, input, RULE_WIDTH.saturating_sub( label.chars().count() + 2 ) );
        let gap = " ".repeat( right_gap( &headline, &label ) );
        Some( format!( "{}{gap}{}", color::tool( &headline ), color::muted( &label ) ) )
      }
      ContentBlock::ToolResult { is_error, content, .. } =>
      {
        if results.is_some()
        {
          // Folded onto the ⚙ line that invoked it
          return None;
        }
        if *is_error
        {
          Some( color::error_marker( &format!( "Tool error · {content}" ) ) )
        }
        else
        {
          // Don't show successful tool results in conversation view
          None
        }
      }
      ContentBlock::Other { kind } => Some( format!( "⧉ {kind}" ) ),
    })
    .collect()
}

/// Join rendered blocks into a turn body.
///
/// A blank line separates two pieces only when at least one of them spans
/// multiple lines. Consecutive one-liners — a run of tool calls, typically —
/// stay packed, so a turn that made four calls reads as a list of four rather
/// than four paragraphs.
pub( super ) fn join_pieces( pieces : &[ String ] ) -> String
{
  let mut out = String::new();

  for ( position, piece ) in pieces.iter().enumerate()
  {
    if position > 0
    {
      let previous_spans = pieces[ position - 1 ].contains( '\n' );
      out.push_str( if previous_spans || piece.contains( '\n' ) { "\n\n" } else { "\n" } );
    }
    out.push_str( piece );
  }

  out
}

/// Spaces needed to push `right` against the [`RULE_WIDTH`] edge after `left`.
///
/// Never returns less than 2, so an overlong left side degrades to a visible
/// separator instead of running the two fields together.
pub( super ) fn right_gap( left : &str, right : &str ) -> usize
{
  let used = left.chars().count() + right.chars().count();
  RULE_WIDTH.saturating_sub( used ).max( 2 )
}

/// Render a `tool_use` block's one-line headline: `⚙ Bash · git status --short`.
///
/// The whole line is held to `budget` characters, with the summary — never the
/// tool name — absorbing the loss.
pub( super ) fn tool_use_line( name : &str, input : &JsonValue, budget : usize ) -> String
{
  let prefix = format!( "⚙ {name} · " );
  let room = budget.saturating_sub( prefix.chars().count() ).min( TOOL_SUMMARY_MAX );

  match tool_summary( input, room )
  {
    Some( summary ) => format!( "{prefix}{summary}" ),
    None => format!( "⚙ {name}" ),
  }
}

/// Pick the most telling string value out of a tool's input object.
///
/// Returns `None` when the input carries no string under any known key —
/// parameterless tools (`TaskList`, `AskUserQuestion`) legitimately have none.
///
/// Filesystem paths elide from the front (`…/src/cli/tail.rs`) because the
/// filename is the identifying part; everything else elides from the back,
/// where a command's or query's opening words carry the meaning.
fn tool_summary( input : &JsonValue, room : usize ) -> Option< String >
{
  const PATH_KEYS : [ &str; 3 ] = [ "file_path", "notebook_path", "path" ];

  let obj = input.as_object()?;

  let ( key, raw ) = TOOL_SUMMARY_KEYS
    .iter()
    .find_map( | key | obj.get( *key ).and_then( | v | v.as_str() ).map( | value | ( *key, value ) ) )?;

  let flattened = raw.split_whitespace().collect::< Vec< _ > >().join( " " );
  if flattened.is_empty()
  {
    return None;
  }

  // Below this there is no room for anything recognisable — drop the summary
  // rather than emit a lone ellipsis.
  if room < 8
  {
    return None;
  }

  Some
  (
    if PATH_KEYS.contains( &key )
    {
      ellipsize_start( &flattened, room )
    }
    else
    {
      ellipsize( &flattened, room )
    }
  )
}

/// Trim trailing whitespace, discarding the piece entirely when nothing is left.
fn non_blank( text : &str ) -> Option< String >
{
  let trimmed = text.trim_end();
  if trimmed.trim().is_empty() { None } else { Some( trimmed.to_string() ) }
}

/// Shorten to `max_chars` characters, appending `…` when anything was cut.
///
/// Counts characters, not bytes — the input is arbitrary user text, so a byte
/// bound would both mis-measure and risk splitting a multibyte sequence.
pub( super ) fn ellipsize( text : &str, max_chars : usize ) -> String
{
  if text.chars().count() <= max_chars
  {
    return text.to_string();
  }

  let kept : String = text.chars().take( max_chars.saturating_sub( 1 ) ).collect();
  format!( "{kept}…" )
}

/// Shorten to `max_chars` characters by dropping the *front*, prefixing `…`.
///
/// For a path, the last segments identify the file; the leading directories are
/// the part a reader can afford to lose.
pub( super ) fn ellipsize_start( text : &str, max_chars : usize ) -> String
{
  let total = text.chars().count();
  if total <= max_chars
  {
    return text.to_string();
  }

  let kept : String = text.chars().skip( total - max_chars.saturating_sub( 1 ) ).collect();
  format!( "…{kept}" )
}

/// Format timestamp for display
///
/// Converts ISO 8601 timestamp to readable format:
/// "2025-12-02T09:57:02.237Z" → "2025-12-02 09:57"
///
/// ## Examples
///
/// ```text
/// let ts = "2025-12-02T09:57:02.237Z";
/// assert_eq!( format_timestamp( ts ), "2025-12-02 09:57" );
/// ```
pub( super ) fn format_timestamp( timestamp : &str ) -> String
{
  // Try to parse ISO 8601
  if let Some( datetime_part ) = timestamp.split( '.' ).next()
  {
    if let Some( ( date, time ) ) = datetime_part.split_once( 'T' )
    {
      // Extract HH:MM from time
      let time_short = time.split( ':' ).take( 2 ).collect::< Vec< _ > >().join( ":" );
      return format!( "{date} {time_short}" );
    }
  }

  // Fallback: use raw timestamp
  timestamp.to_string()
}

/// Extract just the wall clock from an ISO 8601 timestamp: `…T09:57:02.237Z` → `09:57`
///
/// Falls back to the whole timestamp when it does not parse, so a malformed
/// value stays visible rather than silently rendering blank.
pub( super ) fn format_clock( timestamp : &str ) -> String
{
  timestamp
    .split_once( 'T' )
    .map_or_else(
      || timestamp.to_string(),
      | ( _, time ) | time.split( ':' ).take( 2 ).collect::< Vec< _ > >().join( ":" ),
    )
}

/// Current wall clock as whole seconds since the Unix epoch.
pub( super ) fn now_epoch_seconds() -> i64
{
  std::time::SystemTime::now()
    .duration_since( std::time::UNIX_EPOCH )
    .map_or( 0, | d | i64::try_from( d.as_secs() ).unwrap_or( i64::MAX ) )
}

/// Describe how long ago `timestamp` was, relative to `now_epoch`.
///
/// Rendered at one significant unit (`4m ago`, `17h ago`, `3d ago`) because the
/// point is recency at a glance; the absolute clock is printed alongside it for
/// anyone who needs precision. Unparseable or future timestamps render `now`.
pub( super ) fn relative_time( timestamp : &str, now_epoch : i64 ) -> String
{
  let Some( then ) = epoch_seconds( timestamp ) else { return "now".to_string() };

  match now_epoch - then
  {
    d if d < 60 => "now".to_string(),
    d if d < 3_600 => format!( "{}m ago", d / 60 ),
    d if d < 86_400 => format!( "{}h ago", d / 3_600 ),
    d if d < 86_400 * 60 => format!( "{}d ago", d / 86_400 ),
    d => format!( "{}mo ago", d / ( 86_400 * 30 ) ),
  }
}

/// Convert an ISO 8601 UTC timestamp to seconds since the Unix epoch.
///
/// Hand-rolled rather than pulling in a date crate — this crate deliberately
/// carries no such dependency (see `src/cli/show.rs`'s `format_json_value` for
/// the same reasoning applied to JSON rendering). Claude Code always writes UTC
/// with a `Z` suffix, so no zone-offset handling is needed.
fn epoch_seconds( timestamp : &str ) -> Option< i64 >
{
  let ( date, rest ) = timestamp.split_once( 'T' )?;

  let mut date_parts = date.split( '-' );
  let year : i64 = date_parts.next()?.parse().ok()?;
  let month : i64 = date_parts.next()?.parse().ok()?;
  let day : i64 = date_parts.next()?.parse().ok()?;

  let time = rest.split( [ '.', 'Z', '+' ] ).next()?;
  let mut time_parts = time.split( ':' );
  let hour : i64 = time_parts.next()?.parse().ok()?;
  let minute : i64 = time_parts.next()?.parse().ok()?;
  let second : i64 = time_parts.next().unwrap_or( "0" ).parse().ok()?;

  Some( days_from_civil( year, month, day ) * 86_400 + hour * 3_600 + minute * 60 + second )
}

/// Days between 1970-01-01 and the given proleptic-Gregorian date.
///
/// Howard Hinnant's `days_from_civil` — exact for the whole `i64` range, with
/// no lookup tables and no leap-year special cases beyond the era arithmetic.
fn days_from_civil( year : i64, month : i64, day : i64 ) -> i64
{
  let shifted_year = if month <= 2 { year - 1 } else { year };
  let era = if shifted_year >= 0 { shifted_year } else { shifted_year - 399 } / 400;
  let year_of_era = shifted_year - era * 400;
  let shifted_month = ( month + 9 ) % 12;
  let day_of_year = ( 153 * shifted_month + 2 ) / 5 + day - 1;
  let day_of_era = year_of_era * 365 + year_of_era / 4 - year_of_era / 100 + day_of_year;

  era * 146_097 + day_of_era - 719_468
}

/// Truncate text with indicator
///
/// Truncates long text and adds "... truncated" indicator.
///
/// ## Examples
///
/// ```text
/// let text = "a".repeat( 1000 );
/// let truncated = truncate_if_needed( &text, Some( 100 ) );
/// assert!( truncated.contains( "truncated ·" ) );
/// ```
///
/// Fix(issue-018): Use char-boundary-safe truncation.
/// Root cause: `&text[..len]` panics when `len` falls inside a multibyte
/// UTF-8 sequence (emoji, CJK, accented chars).
/// Pitfall: `str::len()` returns bytes, not characters — never use it
/// directly as a slice bound on user-supplied text.
#[ must_use ]
#[ inline ]
pub fn truncate_if_needed( text : &str, max_length : Option< usize > ) -> String
{
  match max_length
  {
    None => text.to_string(),
    Some( len ) if text.len() <= len => text.to_string(),
    Some( len ) =>
    {
      // Find the nearest valid char boundary at or before `len`
      let mut end = len;
      while end > 0 && !text.is_char_boundary( end )
      {
        end -= 1;
      }
      let truncated = &text[ ..end ];
      format!( "{truncated}... truncated · {} more bytes", text.len() - end )
    }
  }
}
