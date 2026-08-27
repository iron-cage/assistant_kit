//! Turning a terminal's raw output into something readable.
//!
//! A program running on a real terminal does not emit text — it emits a byte
//! stream addressed to a screen, carrying escape sequences, carriage-return
//! rewrites and padding. Handed to a reader verbatim it is unreadable; handed
//! through here it reads like a transcript.
//!
//! # What this is, and is not
//!
//! This is a **line renderer**, not a terminal emulator. It models exactly one
//! thing: a cursor moving within the current line. That covers the idioms a
//! command-line program actually uses to rewrite what it has already printed —
//! `\r` to return to column zero, `ESC [ K` to erase what the shorter new text
//! did not cover, `\b` to back up one column — and it covers them exactly.
//!
//! It does not model a screen. Cursor addressing (`ESC [ 3;5H`), scroll regions,
//! alternate screens and erase-in-display are recognised well enough to be
//! *removed*, never to be obeyed. A full-screen program that repaints by moving
//! the cursor around will therefore render as every repaint concatenated, in the
//! order it was emitted — legible, but not what a screen would have shown.
//!
//! That boundary is deliberate. A real emulator is a large amount of state to
//! carry, and the alternative is not "slightly worse output" but a dependency on
//! getting scroll semantics right to print a sentence. Callers that need the
//! bytes exactly as they arrived should ask for them and do their own work; the
//! raw stream is always still there.
//!
//! # Why a function and not a type
//!
//! Rendering needs the whole stream: a `\r` at the boundary between two reads
//! rewrites text that arrived in the previous one. Rather than carry cursor state
//! across calls and make every caller thread it correctly, a caller accumulates
//! the raw text it read and renders once. Cheap, and impossible to hold wrong.

/// Longest parameter run accepted inside one escape sequence.
///
/// A sequence whose parameters run past this is not a sequence any more — it is
/// a stream that lost sync, and the only useful thing to do is stop swallowing
/// text into it. Same reasoning as `claude_daemon_core`'s `MAX_IPC_LINE_BYTES`
/// one layer up: a cap bounds the damage a malformed stream can do to the reader
/// of it.
pub const MAX_ESCAPE_PARAM_CHARS : usize = 64;

/// Columns a tab advances to, as a multiple of.
const TAB_STOP : usize = 8;

/// Render `raw` as plain text, obeying in-line cursor motion and dropping escape
/// sequences.
///
/// Trailing whitespace is removed from every line, and blank lines are removed
/// from the start and end of the result — both are padding a screen needs and a
/// transcript does not. Blank lines *between* content are kept, because there
/// they are the author's, not the terminal's.
///
/// ```
/// use claude_terminal_core::to_plain_text;
///
/// // A spinner rewriting its own line leaves only what it settled on.
/// assert_eq!( to_plain_text( "working... \r\u{1b}[Kdone" ), "done" );
///
/// // Colour is presentation; the text under it survives.
/// assert_eq!( to_plain_text( "\u{1b}[31mred\u{1b}[0m" ), "red" );
/// ```
#[ inline ]
#[ must_use ]
// One scanner over one stream. Splitting the match arms into helpers would move
// the cursor state out of reach of the branches that exist to move it, which is
// the opposite of clearer.
#[ allow( clippy::too_many_lines ) ]
pub fn to_plain_text( raw : &str ) -> String
{
  /// Where the scanner is in an escape sequence.
  enum State
  {
    /// Ordinary text.
    Text,
    /// Just saw `ESC`.
    Escape,
    /// Inside `ESC [ … final`.
    Csi,
    /// Inside `ESC ] … BEL` or `ESC ] … ESC \`.
    Osc,
    /// Saw `ESC` while inside an OSC — the string terminator, or not.
    OscEscape,
    /// Skip exactly one more character (charset selection and friends).
    SkipOne,
  }

  let mut lines : Vec< String > = Vec::new();
  let mut line : Vec< char > = Vec::new();
  let mut column : usize = 0;
  let mut state = State::Text;
  let mut params = String::new();

  for ch in raw.chars()
  {
    match state
    {
      State::Text => match ch
      {
        '\u{1b}' => state = State::Escape,
        '\n' =>
        {
          lines.push( line.iter().collect() );
          line.clear();
          column = 0;
        },
        // Return to column zero without ending the line: everything written from
        // here overwrites what is already there.
        '\r' => column = 0,
        '\t' =>
        {
          let stop = ( column / TAB_STOP + 1 ) * TAB_STOP;
          while column < stop
          {
            put( &mut line, &mut column, ' ' );
          }
        },
        '\u{8}' => column = column.saturating_sub( 1 ),
        // Remaining C0 controls and DEL address a device, not a reader.
        c if c.is_control() => {},
        c => put( &mut line, &mut column, c ),
      },
      State::Escape =>
      {
        params.clear();
        state = match ch
        {
          '[' => State::Csi,
          ']' => State::Osc,
          // Charset selection, character-size and the like: one parameter byte
          // follows and neither it nor the introducer is text.
          '(' | ')' | '*' | '+' | '-' | '.' | '/' | '#' | ' ' => State::SkipOne,
          // Everything else two bytes long, including a stray string terminator.
          _ => State::Text,
        };
      },
      State::Csi =>
      {
        if ( '\u{20}'..='\u{3f}' ).contains( &ch ) && params.len() < MAX_ESCAPE_PARAM_CHARS
        {
          params.push( ch );
        }
        else
        {
          // A final byte ends the sequence; anything else means the stream lost
          // sync, and the cap has run out. Either way, stop swallowing text.
          if ( '\u{40}'..='\u{7e}' ).contains( &ch )
          {
            apply_csi( ch, &params, &mut line, column );
          }
          state = State::Text;
        }
      },
      State::Osc => match ch
      {
        '\u{7}' => state = State::Text,
        '\u{1b}' => state = State::OscEscape,
        _ => {},
      },
      // `ESC \` terminates the string; an `ESC` followed by anything else was not
      // a terminator, and the string is still running.
      State::OscEscape => state = if ch == '\\' { State::Text } else { State::Osc },
      State::SkipOne => state = State::Text,
    }
  }

  lines.push( line.iter().collect() );
  tidy( lines )
}

/// Write `ch` at `column`, padding with spaces if the cursor ran ahead.
fn put( line : &mut Vec< char >, column : &mut usize, ch : char )
{
  while line.len() < *column
  {
    line.push( ' ' );
  }
  if *column < line.len()
  {
    line[ *column ] = ch;
  }
  else
  {
    line.push( ch );
  }
  *column += 1;
}

/// Obey the one CSI sequence that changes text rather than presentation.
///
/// `ESC [ K` — erase in line — is the other half of the `\r` rewrite idiom:
/// without it, rewriting a long line with a shorter one leaves the tail of the
/// old one behind. Every other final byte is colour, cursor motion beyond this
/// line, or a mode change, none of which this renderer models.
fn apply_csi( final_byte : char, params : &str, line : &mut Vec< char >, column : usize )
{
  if final_byte != 'K'
  {
    return;
  }

  let mode = params
    .trim_start_matches( | c : char | !c.is_ascii_digit() && c != ';' )
    .split( ';' )
    .next()
    .unwrap_or( "" )
    .parse::< u8 >()
    .unwrap_or( 0 );

  match mode
  {
    // Cursor to end of line. `truncate` past the end is already a no-op.
    0 => line.truncate( column ),
    // Start of line to cursor, inclusive — blanked, not removed, because the
    // cursor does not move and what follows it still occupies its columns.
    1 =>
    {
      for cell in line.iter_mut().take( column + 1 )
      {
        *cell = ' ';
      }
    },
    // The whole line, cursor unmoved.
    2 => line.clear(),
    _ => {},
  }
}

/// Drop the whitespace a screen needed and a transcript does not.
fn tidy( lines : Vec< String > ) -> String
{
  let mut trimmed : Vec< &str > = lines.iter().map( | line | line.trim_end() ).collect();

  while trimmed.first().is_some_and( | line | line.is_empty() )
  {
    trimmed.remove( 0 );
  }
  while trimmed.last().is_some_and( | line | line.is_empty() )
  {
    trimmed.pop();
  }

  trimmed.join( "\n" )
}
