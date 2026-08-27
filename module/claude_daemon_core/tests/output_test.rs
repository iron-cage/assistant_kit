//! Output-buffer tests — cursors, eviction, and character boundaries.
//!
//! These are pure: no threads, no pty, no timing. The buffer is where the
//! arithmetic lives, so it is the one part of the output path that can be pinned
//! down exactly rather than observed. The pump that feeds it is exercised
//! end-to-end against a real child in `table_test.rs`.
//!
//! The UTF-8 cases are the reason this file exists. A terminal emits bytes, not
//! characters, and both a chunk boundary and an eviction can land in the middle
//! of one — with opposite correct answers. Getting them backwards produces
//! mojibake that appears only under load, which is the worst way to find out.
//!
//! ## Specification References
//!
//! - `docs/feature/004_session_output.md` — buffering, cursors, and eviction
//!
//! ## Coverage
//!
//! | TC | Scenario | Expected |
//! |----|----------|----------|
//! | out01 | A fresh buffer | Empty, cursor zero, not ended |
//! | out02 | Push, then read from zero | The text, cursor at its end |
//! | out03 | Read again from the returned cursor | Empty, cursor unmoved |
//! | out04 | Push past capacity | Oldest bytes evicted, `dropped` grows |
//! | out05 | A cursor ahead of the newest byte | Clamped to the end, no text |
//! | out06 | A character split across two pushes | Withheld, then delivered whole |
//! | out07 | Eviction cutting a character in half | Replacement character, cursor advances |
//! | out08 | `mark_ended` | Reported on every subsequent read |
//! | out09 | A single push larger than capacity | Only the tail is retained |
//! | out10 | Reading after eviction | `missed` counts exactly what was lost |

use claude_daemon_core::OutputBuffer;

/// `U+2713 CHECK MARK` — three bytes, so it can be cut two different ways.
const CHECK : &[ u8 ] = &[ 0xE2, 0x9C, 0x93 ];

/// out01: a fresh buffer reports nothing, from any cursor.
#[ test ]
fn out01_fresh_buffer_is_empty()
{
  let mut buffer = OutputBuffer::with_capacity( 64 );

  assert_eq!( buffer.end(), 0 );
  assert_eq!( buffer.dropped(), 0 );
  assert_eq!( buffer.capacity(), 64 );
  assert!( !buffer.has_ended() );

  let slice = buffer.read_from( 0 );
  assert_eq!( slice.text, "" );
  assert_eq!( slice.cursor, 0 );
  assert_eq!( slice.missed, 0 );
  assert!( !slice.ended );
}

/// out02: what was pushed comes back, and the cursor lands past it.
#[ test ]
fn out02_push_then_read_from_zero()
{
  let mut buffer = OutputBuffer::with_capacity( 64 );
  buffer.push( b"hello" );

  let slice = buffer.read_from( 0 );

  assert_eq!( slice.text, "hello" );
  assert_eq!( slice.cursor, 5 );
  assert_eq!( slice.missed, 0 );
  assert_eq!( buffer.end(), 5 );
}

/// out03: a read is non-destructive but not repeating.
///
/// Re-reading from the *same* cursor returns the same bytes — the buffer keeps
/// them. Reading from the *returned* cursor returns nothing, which is what stops
/// a polling client printing its whole scrollback on every tick.
#[ test ]
fn out03_reading_from_the_returned_cursor_is_empty()
{
  let mut buffer = OutputBuffer::with_capacity( 64 );
  buffer.push( b"hello" );

  let first = buffer.read_from( 0 );
  let replay = buffer.read_from( 0 );
  let next = buffer.read_from( first.cursor );

  assert_eq!( replay.text, "hello", "re-reading the same cursor lost the bytes" );
  assert_eq!( next.text, "", "reading past the newest byte replayed old output" );
  assert_eq!( next.cursor, first.cursor, "an empty read moved the cursor" );
}

/// out04: pushing past capacity discards from the front.
#[ test ]
fn out04_push_past_capacity_evicts_oldest()
{
  let mut buffer = OutputBuffer::with_capacity( 4 );
  buffer.push( b"abc" );
  buffer.push( b"de" );

  assert_eq!( buffer.dropped(), 1, "one byte should have been evicted" );
  assert_eq!( buffer.end(), 5, "the absolute cursor must count evicted bytes too" );
  assert_eq!( buffer.read_from( 1 ).text, "bcde" );
}

/// out05: a cursor past the end is walked back rather than trusted.
///
/// It arrives from a client that saved a cursor across a daemon restart, or that
/// mixed up two sessions. Returning the current end tells it where it actually
/// is; treating a large number as an offset would panic or return garbage.
#[ test ]
fn out05_cursor_ahead_of_the_end_is_clamped()
{
  let mut buffer = OutputBuffer::with_capacity( 64 );
  buffer.push( b"hi" );

  let slice = buffer.read_from( 9_999 );

  assert_eq!( slice.text, "" );
  assert_eq!( slice.cursor, 2, "an out-of-range cursor was not corrected to the end" );
  assert_eq!( slice.missed, 0 );
}

/// out06: a character split across two pushes is delivered whole.
///
/// The first two bytes of a three-byte character are a valid *prefix*, not an
/// error — the rest simply has not been read yet. Emitting a replacement here
/// would corrupt text that was never damaged, and it is the common case: any
/// non-ASCII output crossing an 8 KiB read boundary hits it.
#[ test ]
fn out06_character_split_across_pushes_is_withheld_then_delivered()
{
  let mut buffer = OutputBuffer::with_capacity( 64 );
  buffer.push( &CHECK[ ..2 ] );

  let partial = buffer.read_from( 0 );
  assert_eq!( partial.text, "", "an unfinished character was decoded early" );
  assert_eq!( partial.cursor, 0, "the cursor moved past bytes that were not decoded" );

  buffer.push( &CHECK[ 2.. ] );
  let whole = buffer.read_from( partial.cursor );

  assert_eq!( whole.text, "✓", "the character did not survive being split" );
  assert_eq!( whole.cursor, 3 );
}

/// out07: eviction cutting a character in half resynchronises.
///
/// The opposite case to out06, and it needs the opposite answer. These bytes are
/// not an unfinished character — their start is gone for good, so no later read
/// can complete them. Withholding them would stall the cursor on them forever,
/// so they are replaced and stepped over.
#[ test ]
fn out07_character_cut_by_eviction_becomes_a_replacement()
{
  let mut buffer = OutputBuffer::with_capacity( 2 );
  buffer.push( CHECK );

  assert_eq!( buffer.dropped(), 1, "test premise broken: the character was not cut" );

  let slice = buffer.read_from( 0 );

  assert_eq!( slice.text, "\u{FFFD}", "a severed character was not replaced" );
  assert_eq!( slice.missed, 1 );
  assert!( slice.cursor > 1, "the cursor stalled on bytes that can never decode" );
}

/// out08: an ended stream says so on every read.
#[ test ]
fn out08_mark_ended_is_reported()
{
  let mut buffer = OutputBuffer::with_capacity( 64 );
  buffer.push( b"bye" );

  assert!( !buffer.read_from( 0 ).ended );

  buffer.mark_ended();

  assert!( buffer.has_ended() );
  assert!( buffer.read_from( 0 ).ended, "an ended buffer reported itself live" );
  assert!( buffer.read_from( 3 ).ended, "an ended buffer reported itself live at the end cursor" );
}

/// out09: a single push larger than capacity keeps only its tail.
#[ test ]
fn out09_oversized_push_keeps_the_tail()
{
  let mut buffer = OutputBuffer::with_capacity( 4 );
  buffer.push( b"abcdefgh" );

  let slice = buffer.read_from( 0 );

  assert_eq!( slice.text, "efgh", "the retained window is not the newest bytes" );
  assert_eq!( slice.missed, 4 );
  assert_eq!( slice.cursor, 8 );
  assert_eq!( buffer.dropped(), 4 );
}

/// out10: `missed` is the exact count of what a late reader lost.
///
/// A client prints "N bytes of output were dropped"; an approximate N is worse
/// than none, because it reads as precise.
#[ test ]
fn out10_missed_counts_exactly_what_was_evicted()
{
  let mut buffer = OutputBuffer::with_capacity( 4 );
  buffer.push( b"0123" );

  // A reader that keeps up misses nothing.
  let current = buffer.read_from( 0 );
  assert_eq!( current.missed, 0 );

  buffer.push( b"456789" );

  // Six more bytes into a four-byte window: bytes 0..6 are gone.
  assert_eq!( buffer.dropped(), 6 );
  assert_eq!( buffer.read_from( 0 ).missed, 6, "a reader starting at zero lost six bytes" );
  assert_eq!( buffer.read_from( 4 ).missed, 2, "a reader at cursor four lost two" );
  assert_eq!( buffer.read_from( 6 ).missed, 0, "a reader at the window's edge lost nothing" );
  assert_eq!( buffer.read_from( 6 ).text, "6789" );
}
