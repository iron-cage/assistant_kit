//! Render tests — escape removal, in-line cursor motion, and what is left over.
//!
//! Pure: a string in, a string out. That is the whole point of the renderer
//! being a function of the accumulated stream rather than something stateful
//! bolted onto a read — every case here is a literal, and every expectation is
//! exact rather than "contains".
//!
//! The cases worth having are the ones where a naive implementation is *nearly*
//! right: stripping escapes but not honouring the `\r` they accompany (rnd04),
//! honouring `\r` but not the erase that follows it (rnd05), or trimming so
//! eagerly that a blank line the session actually printed disappears (rnd12).
//!
//! ## Specification References
//!
//! - `docs/feature/007_readable_output.md` — what is modelled and what is not
//!
//! ## Coverage
//!
//! | TC | Scenario | Expected |
//! |----|----------|----------|
//! | rnd01 | Plain text, nothing to do | Returned unchanged |
//! | rnd02 | Empty input | Empty output |
//! | rnd03 | SGR colour around a word | The word, unstyled |
//! | rnd04 | `\r` then shorter text, no erase | Overwritten in place, tail survives |
//! | rnd05 | `\r` then `ESC [ K` then shorter text | The tail is erased |
//! | rnd06 | `ESC [ 2K` mid-line | The line is cleared, cursor unmoved |
//! | rnd07 | Backspace | Cursor steps back and overwrites |
//! | rnd08 | Tab | Advances to the next multiple of eight |
//! | rnd09 | OSC title, both terminators | Removed entirely |
//! | rnd10 | Cursor addressing and erase-in-display | Removed, never obeyed |
//! | rnd11 | Leading and trailing blank lines | Trimmed away |
//! | rnd12 | A blank line between two lines of content | Kept |
//! | rnd13 | Trailing spaces on a line | Trimmed |
//! | rnd14 | Parameters running past the cap | Sequence abandoned, not swallowed forever |
//! | rnd15 | A truncated sequence at the very end | Dropped, no panic |
//! | rnd16 | Multi-byte characters around an escape | Preserved whole |

use claude_daemon_core::render::{ to_plain_text, MAX_ESCAPE_PARAM_CHARS };

/// rnd01: text with nothing in it to render comes back as it went in.
#[ test ]
fn rnd01_plain_text_is_unchanged()
{
  assert_eq!( to_plain_text( "hello\nworld" ), "hello\nworld" );
}

/// rnd02: nothing in, nothing out.
#[ test ]
fn rnd02_empty_input_renders_empty()
{
  assert_eq!( to_plain_text( "" ), "" );
}

/// rnd03: colour is presentation, and the text under it survives.
#[ test ]
fn rnd03_sgr_colour_is_dropped()
{
  assert_eq!( to_plain_text( "\u{1b}[1;31mred\u{1b}[0m text" ), "red text" );
}

/// rnd04: `\r` rewrites in place — and leaves behind whatever it did not cover.
///
/// This is the honest behaviour of a real terminal, not a shortcoming: a program
/// that rewrites a long line with a short one and does not erase really does
/// leave the tail on screen. rnd05 is the same case done properly.
#[ test ]
fn rnd04_carriage_return_overwrites_in_place()
{
  // "done" covers the first four columns of "working..."; "ing..." is what was
  // already there past column four, and a terminal would still be showing it.
  assert_eq!( to_plain_text( "working...\rdone" ), "doneing..." );
}

/// rnd05: `\r` with the erase that belongs with it leaves only the new text.
#[ test ]
fn rnd05_erase_in_line_removes_the_tail()
{
  assert_eq!( to_plain_text( "working...\r\u{1b}[Kdone" ), "done" );
}

/// rnd06: erase-the-whole-line blanks it without moving the cursor.
///
/// The cursor staying put is the part worth pinning: text written afterwards
/// lands at the column it was already at, padded from the start of the line.
#[ test ]
fn rnd06_erase_whole_line_keeps_the_cursor()
{
  assert_eq!( to_plain_text( "abcdef\u{1b}[2KX" ), "      X" );
}

/// rnd07: backspace steps back one column, and the next character overwrites.
#[ test ]
fn rnd07_backspace_overwrites()
{
  assert_eq!( to_plain_text( "cat\u{8}b" ), "cab" );
}

/// rnd08: a tab advances to the next multiple of eight, not by a fixed amount.
#[ test ]
fn rnd08_tab_advances_to_the_next_stop()
{
  assert_eq!( to_plain_text( "ab\tc" ), "ab      c" );
  assert_eq!( to_plain_text( "abcdefgh\tc" ), "abcdefgh        c" );
}

/// rnd09: an OSC string vanishes, whichever way it is terminated.
#[ test ]
fn rnd09_osc_strings_are_dropped()
{
  assert_eq!( to_plain_text( "\u{1b}]0;a title\u{7}body" ), "body" );
  assert_eq!( to_plain_text( "\u{1b}]0;a title\u{1b}\\body" ), "body" );
}

/// rnd10: sequences this renderer does not model are removed, not obeyed.
///
/// Cursor addressing and erase-in-display are exactly the cases where a
/// half-implemented emulator would be worse than none: obeying `ESC [ H` without
/// modelling a screen would silently discard everything printed before it.
#[ test ]
fn rnd10_screen_addressing_is_removed_not_obeyed()
{
  assert_eq!( to_plain_text( "first\u{1b}[2J\u{1b}[3;5Hsecond" ), "firstsecond" );
}

/// rnd11: blank lines at either end are the screen's padding, not the text's.
#[ test ]
fn rnd11_leading_and_trailing_blank_lines_go()
{
  assert_eq!( to_plain_text( "\n\n  \nhello\n\n  \n\n" ), "hello" );
}

/// rnd12: a blank line between two lines of content belongs to the content.
#[ test ]
fn rnd12_interior_blank_lines_stay()
{
  assert_eq!( to_plain_text( "one\n\ntwo" ), "one\n\ntwo" );
}

/// rnd13: trailing spaces are padding, and every line loses them.
#[ test ]
fn rnd13_trailing_spaces_are_trimmed()
{
  assert_eq!( to_plain_text( "one   \ntwo\t\n" ), "one\ntwo" );
}

/// rnd14: a sequence whose parameters run past the cap is abandoned.
///
/// The property under test is that the renderer stops swallowing input — a
/// scanner with no cap treats the entire rest of the stream as parameters and
/// returns nothing at all.
#[ test ]
fn rnd14_overlong_parameters_abandon_the_sequence()
{
  let overlong = ";".repeat( MAX_ESCAPE_PARAM_CHARS + 10 );
  let rendered = to_plain_text( &format!( "before\u{1b}[{overlong}mafter" ) );

  assert!( rendered.starts_with( "before" ), "input before the sequence was lost: {rendered:?}" );
  assert!( rendered.ends_with( "after" ), "input after the sequence was lost: {rendered:?}" );
}

/// rnd15: a stream that ends mid-sequence renders what it had, without panicking.
#[ test ]
fn rnd15_truncated_sequence_at_the_end()
{
  assert_eq!( to_plain_text( "text\u{1b}[3" ), "text" );
  assert_eq!( to_plain_text( "text\u{1b}" ), "text" );
  assert_eq!( to_plain_text( "text\u{1b}]0;unterminated" ), "text" );
}

/// rnd16: multi-byte characters survive intact across an escape boundary.
///
/// The renderer works in `char`s rather than bytes precisely so a sequence
/// cannot cut one in half; this is the assertion that says so.
#[ test ]
fn rnd16_multibyte_characters_survive()
{
  assert_eq!( to_plain_text( "héllo \u{1b}[32m→\u{1b}[0m wörld" ), "héllo → wörld" );
}
