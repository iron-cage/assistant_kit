//! Line-framing tests, including the cap that bounds the read buffer.
//!
//! ## Specification References
//!
//! - `docs/feature/002_wire_protocol.md` — the framing contract
//! - `docs/invariant/001_capped_line_reads.md` — why the cap exists
//!
//! ## Coverage
//!
//! | TC | Scenario | Expected |
//! |----|----------|----------|
//! | ipc01 | One newline-terminated line | The line, newline stripped |
//! | ipc02 | Several lines | Returned one at a time, in order |
//! | ipc03 | Clean end of stream | `Ok( None )` |
//! | ipc04 | Trailing partial line at EOF | Returned, then `Ok( None )` |
//! | ipc05 | CRLF line ending | `\r` stripped along with `\n` |
//! | ipc06 | Empty line | `Ok( Some( "" ) )` — not EOF |
//! | ipc07 | Non-UTF-8 bytes | `Err( NonUtf8Line )` |
//! | ipc08 | Exactly `MAX_IPC_LINE_BYTES` | Accepted |
//! | ipc09 | One byte over the cap | `Err( LineTooLong )` |
//! | ipc10 | No newline ever, across many small reads | `Err( LineTooLong )`, bounded memory |
//! | ipc11 | A line that only *contains* the cap's worth across chunks | Accepted |

use std::io::{ BufReader, Cursor };

use claude_daemon_core::{ read_capped_line, Error, MAX_IPC_LINE_BYTES };

/// A reader that hands out at most `chunk` bytes per `fill_buf`.
///
/// The cap must hold when a line arrives in many small pieces, which is how it
/// actually arrives over a socket — a single-shot `Cursor` would exercise only
/// the one-pass path and never the accumulation loop the cap protects.
fn chunked( bytes : Vec< u8 >, chunk : usize ) -> BufReader< Cursor< Vec< u8 > > >
{
  BufReader::with_capacity( chunk, Cursor::new( bytes ) )
}

/// Read one line from `text` through a single-shot cursor.
fn read_one( text : &str ) -> claude_daemon_core::Result< Option< String > >
{
  read_capped_line( &mut Cursor::new( text.as_bytes().to_vec() ) )
}

/// ipc01: a terminated line comes back without its terminator.
#[ test ]
fn ipc01_reads_one_line_without_the_newline()
{
  let line = read_one( "{\"method\":\"ping\"}\n" ).expect( "read failed" );

  assert_eq!( line.as_deref(), Some( "{\"method\":\"ping\"}" ) );
}

/// ipc02: consecutive lines are framed one at a time, in order.
#[ test ]
fn ipc02_reads_lines_in_order()
{
  let mut reader = Cursor::new( b"first\nsecond\nthird\n".to_vec() );

  let mut seen = Vec::new();
  while let Some( line ) = read_capped_line( &mut reader ).expect( "read failed" )
  {
    seen.push( line );
  }

  assert_eq!( seen, vec![ "first", "second", "third" ] );
}

/// ipc03: end of stream is `None`, not an empty line.
///
/// A client that closes its socket must be distinguishable from one that sent a
/// blank line, or the daemon would answer a peer that is no longer there.
#[ test ]
fn ipc03_end_of_stream_is_none()
{
  assert_eq!( read_one( "" ).expect( "read failed" ), None );
}

/// ipc04: a final line without a terminator is still delivered.
#[ test ]
fn ipc04_unterminated_trailing_line_is_returned()
{
  let mut reader = Cursor::new( b"complete\npartial".to_vec() );

  assert_eq!(
    read_capped_line( &mut reader ).expect( "read failed" ).as_deref(),
    Some( "complete" ),
  );
  assert_eq!(
    read_capped_line( &mut reader ).expect( "read failed" ).as_deref(),
    Some( "partial" ),
  );
  assert_eq!( read_capped_line( &mut reader ).expect( "read failed" ), None );
}

/// ipc05: a `\r\n` terminator loses both bytes.
///
/// A stray `\r` left on the end would land inside the JSON handed to the parser.
#[ test ]
fn ipc05_crlf_is_stripped()
{
  let line = read_one( "windows\r\n" ).expect( "read failed" );

  assert_eq!( line.as_deref(), Some( "windows" ) );
}

/// ipc06: an empty line is a line.
#[ test ]
fn ipc06_empty_line_is_not_end_of_stream()
{
  let mut reader = Cursor::new( b"\nafter\n".to_vec() );

  assert_eq!( read_capped_line( &mut reader ).expect( "read failed" ).as_deref(), Some( "" ) );
  assert_eq!( read_capped_line( &mut reader ).expect( "read failed" ).as_deref(), Some( "after" ) );
}

/// ipc07: invalid UTF-8 is refused rather than lossily converted.
///
/// Lossy conversion would replace the offending bytes with `U+FFFD` and hand the
/// parser a line that no client sent.
#[ test ]
fn ipc07_non_utf8_line_is_refused()
{
  let mut reader = Cursor::new( vec![ b'{', 0xFF, 0xFE, b'}', b'\n' ] );

  match read_capped_line( &mut reader )
  {
    Err( Error::NonUtf8Line ) => {}
    other => panic!( "expected NonUtf8Line, got {other:?}" ),
  }
}

/// ipc08: a line of exactly the cap is accepted.
///
/// The boundary belongs to the accepted side: the cap is the largest permitted
/// line, not the smallest rejected one.
#[ test ]
fn ipc08_line_of_exactly_the_cap_is_accepted()
{
  let mut body = vec![ b'x'; MAX_IPC_LINE_BYTES ];
  body.push( b'\n' );

  let line = read_capped_line( &mut chunked( body, 8192 ) ).expect( "read of a cap-sized line failed" );

  assert_eq!( line.map( | l | l.len() ), Some( MAX_IPC_LINE_BYTES ) );
}

/// ipc09: one byte past the cap is refused.
#[ test ]
fn ipc09_line_one_byte_over_the_cap_is_refused()
{
  let mut body = vec![ b'x'; MAX_IPC_LINE_BYTES + 1 ];
  body.push( b'\n' );

  match read_capped_line( &mut chunked( body, 8192 ) )
  {
    Err( Error::LineTooLong ) => {}
    other => panic!( "expected LineTooLong, got {other:?}" ),
  }
}

/// ipc10: a peer that never sends a newline is refused, not accommodated.
///
/// This is the failure the cap exists for. `BufRead::read_line` grows its buffer
/// until a newline arrives, so a broken or hostile peer drives the daemon to
/// allocate without bound — and with one daemon hosting every session, that is no
/// longer one session's problem. The small chunk size forces the accumulation
/// loop rather than a single oversized `fill_buf`.
#[ test ]
fn ipc10_stream_without_a_newline_is_refused_at_the_cap()
{
  let body = vec![ b'x'; MAX_IPC_LINE_BYTES * 2 ];

  match read_capped_line( &mut chunked( body, 4096 ) )
  {
    Err( Error::LineTooLong ) => {}
    other => panic!( "expected LineTooLong, got {other:?}" ),
  }
}

/// ipc11: a legitimate large line still assembles across chunk boundaries.
///
/// The cap must reject unbounded growth without also rejecting the biggest
/// message the protocol genuinely carries — a `Send` request holding a large
/// paste.
#[ test ]
fn ipc11_large_line_assembles_across_chunks()
{
  let payload = "p".repeat( 300_000 );
  let mut body = payload.clone().into_bytes();
  body.push( b'\n' );
  body.extend_from_slice( b"next\n" );

  let mut reader = chunked( body, 1024 );

  assert_eq!(
    read_capped_line( &mut reader ).expect( "read failed" ).as_deref(),
    Some( payload.as_str() ),
  );
  assert_eq!(
    read_capped_line( &mut reader ).expect( "read failed" ).as_deref(),
    Some( "next" ),
  );
}
