//! Shared text formatting utilities for CLI output.

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
/// [2025-12-02 09:57] User:
/// message content here
///
/// [2025-12-02 09:58] Assistant:
/// response content here
/// ```
///
/// ## Examples
///
/// ```text
/// let entry = session.entries()[0];
/// let formatted = format_entry_content( &entry, None );
/// // Output: "[2025-12-02 09:57] User:\nHello, Claude!"
/// ```
pub( super ) fn format_entry_content( entry : &claude_storage_core::Entry, max_length : Option< usize > ) -> String
{
  use claude_storage_core::{ MessageContent, ContentBlock };

  // Format timestamp
  let timestamp = format_timestamp( &entry.timestamp );

  // Extract content based on message type
  let ( role, content ) = match &entry.message
  {
    MessageContent::User( msg ) =>
    {
      ( "User", msg.content.clone() )
    },
    MessageContent::Assistant( msg ) =>
    {
      // Extract all text blocks
      let text_blocks : Vec< String > = msg.content
        .iter()
        .filter_map( | block | match block
        {
          ContentBlock::Text { text } => Some( text.clone() ),
          ContentBlock::Thinking { thinking, .. } =>
          {
            // Show thinking blocks with prefix
            Some( format!( "[Thinking]\n{thinking}" ) )
          },
          ContentBlock::ToolUse { name, .. } =>
          {
            // Show tool use briefly
            Some( format!( "[Using tool: {name}]" ) )
          },
          ContentBlock::ToolResult { is_error, content, .. } =>
          {
            if *is_error
            {
              Some( format!( "[Tool error: {content}]" ) )
            }
            else
            {
              // Don't show successful tool results in conversation view
              None
            }
          },
        })
        .collect();

      let combined = text_blocks.join( "\n\n" );
      ( "Assistant", combined )
    }
  };

  // Apply truncation if needed
  let content = truncate_if_needed( &content, max_length );

  // Format as chat log entry
  format!( "[{timestamp}] {role}:\n{content}" )
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

/// Truncate text with indicator
///
/// Truncates long text and adds "... [truncated]" indicator.
///
/// ## Examples
///
/// ```text
/// let text = "a".repeat( 1000 );
/// let truncated = truncate_if_needed( &text, Some( 100 ) );
/// assert!( truncated.contains( "[truncated" ) );
/// ```
///
/// Fix(issue-018): Use char-boundary-safe truncation.
/// Root cause: `&text[..len]` panics when `len` falls inside a multibyte
/// UTF-8 sequence (emoji, CJK, accented chars).
/// Pitfall: `str::len()` returns bytes, not characters — never use it
/// directly as a slice bound on user-supplied text.
#[must_use]
#[inline]
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
      format!( "{}... [truncated, {} more bytes]", truncated, text.len() - end )
    }
  }
}
