//! Line-oriented IPC framing with a hard size cap.
//!
//! # Why capped
//!
//! `claude_runner/src/cli/query.rs` reads its socket with a bare
//! `BufRead::read_line`, which grows its buffer until a newline arrives. A peer
//! that never sends one — broken, wedged, or hostile — drives the daemon to
//! allocate without bound. Since one daemon now hosts every session, that failure
//! is no longer isolated to a single session's helper process.
//!
//! [`read_capped_line`] refuses at [`MAX_IPC_LINE_BYTES`] instead.

use std::io::BufRead;

use crate::error::{ Error, Result };

/// Largest single protocol line accepted, in bytes.
///
/// A `Send` request carrying a large paste is the biggest legitimate message;
/// 1 MiB is far above that and far below anything that threatens the process.
pub const MAX_IPC_LINE_BYTES : usize = 1024 * 1024;

/// Read one newline-terminated line, refusing anything over
/// [`MAX_IPC_LINE_BYTES`].
///
/// Returns `Ok( None )` at end of stream. The trailing newline is not included
/// in the returned string.
///
/// # Errors
///
/// - [`Error::LineTooLong`] — the cap was reached before a newline arrived.
/// - [`Error::Io`] — the underlying read failed.
/// - [`Error::NonUtf8Line`] — the bytes read were not valid UTF-8.
#[ inline ]
pub fn read_capped_line< R : BufRead >( reader : &mut R ) -> Result< Option< String > >
{
  let mut buf : Vec< u8 > = Vec::new();
  loop
  {
    let available = reader.fill_buf().map_err( Error::Io )?;
    if available.is_empty()
    {
      // Clean EOF with nothing buffered; a partial line at EOF is still returned.
      return if buf.is_empty()
      {
        Ok( None )
      }
      else
      {
        finish( buf ).map( Some )
      };
    }

    // Both cases accumulate a prefix of what is buffered and consume it; a found
    // newline differs only in that it also ends the line. Checking the cap
    // against `take` before extending is what bounds the buffer — a peer that
    // never sends a newline is refused here rather than allocated for.
    let newline = available.iter().position( | b | *b == b'\n' );
    let take = newline.unwrap_or( available.len() );
    if buf.len() + take > MAX_IPC_LINE_BYTES
    {
      return Err( Error::LineTooLong );
    }
    buf.extend_from_slice( &available[ ..take ] );
    // The newline is a delimiter, not content: consumed from the reader, never
    // pushed into `buf`.
    reader.consume( take + usize::from( newline.is_some() ) );
    if newline.is_some()
    {
      return finish( buf ).map( Some );
    }
  }
}

/// Convert an accumulated line buffer to `String`, trimming a trailing `\r`.
fn finish( mut buf : Vec< u8 > ) -> Result< String >
{
  if buf.last() == Some( &b'\r' )
  {
    buf.pop();
  }
  String::from_utf8( buf ).map_err( | _ | Error::NonUtf8Line )
}
