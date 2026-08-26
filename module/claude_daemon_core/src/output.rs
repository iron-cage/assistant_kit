//! A session's terminal output: bounded, cursor-addressed, and kept drained.
//!
//! # Why the daemon buffers at all
//!
//! A pty master must be read continuously. The kernel's buffer for it is small
//! and fixed; once it fills, the child blocks on its next write, and a blocked
//! child looks exactly like a thinking one. So the daemon cannot wait for a
//! client to ask for output before reading it — [`OutputPump`] drains the master
//! from the moment the session starts, whether anyone is listening or not.
//!
//! That makes the daemon the owner of output nobody has read yet, which raises
//! the two questions this module answers.
//!
//! # Bounded, and honest about it
//!
//! A session left running for a week produces more output than the daemon should
//! hold. [`OutputBuffer`] keeps the newest [`OutputBuffer::capacity`] bytes and
//! discards from the front — the same reasoning as
//! [`MAX_IPC_LINE_BYTES`](crate::ipc::MAX_IPC_LINE_BYTES): one session must not be
//! able to exhaust a process that hosts every session.
//!
//! Eviction is *reported*, not hidden. [`OutputSlice::missed`] counts the bytes a
//! reader arrived too late for, so a client can print "output truncated" instead
//! of presenting a gap as continuous text.
//!
//! # Cursors, not draining
//!
//! Reads do not consume. A reader passes the cursor it last received and gets
//! everything since, so disconnecting and reconnecting resumes rather than loses,
//! and two clients can watch one session without stealing each other's output.
//! Cursors are absolute byte counts over the session's whole lifetime and never
//! reset — a cursor that has fallen behind the retained window is clamped forward
//! and the gap is reported, never silently reinterpreted as position zero.

use core::fmt;
use std::collections::VecDeque;
use std::fs::File;
use std::io::Read;
use std::sync::{ Arc, Mutex, MutexGuard, PoisonError };
use std::thread::JoinHandle;

use serde::{ Deserialize, Serialize };

/// Bytes of output retained per session by default.
///
/// A print-mode turn is far smaller than this; the headroom is for an
/// interactive session left scrolling between reads.
pub const DEFAULT_OUTPUT_CAP : usize = 256 * 1024;

/// Bytes read from the master per iteration of the pump loop.
const READ_CHUNK_BYTES : usize = 8192;

/// What a read of a session's output returns.
///
/// Serialized directly as the payload of a `read` response.
#[ derive( Debug, Clone, PartialEq, Eq, Serialize, Deserialize ) ]
pub struct OutputSlice
{
  /// Output decoded since the requested cursor, ending on a character boundary.
  pub text : String,
  /// Cursor to pass to the next read.
  pub cursor : u64,
  /// Bytes evicted from the buffer before this read could reach them.
  ///
  /// Non-zero means output is missing between the caller's previous read and
  /// this one. Report it; do not present the result as continuous.
  pub missed : u64,
  /// Whether the session's output stream has ended.
  ///
  /// Once true, no further output will ever arrive — the child's terminal is
  /// closed. A client polling for more should stop.
  pub ended : bool,
}

/// A session's retained output, addressed by an absolute cursor.
pub struct OutputBuffer
{
  /// The retained window. Not contiguous in memory; see [`OutputBuffer::read_from`].
  held : VecDeque< u8 >,
  /// Absolute index of `held`'s first byte — equivalently, how many bytes have
  /// been evicted over the buffer's lifetime.
  dropped : u64,
  /// Retention limit in bytes.
  cap : usize,
  /// Whether the producing stream has ended.
  ended : bool,
}

// Hand-written: the derived form would print every retained byte, so a routine
// `{:?}` of a session would dump a quarter-megabyte of terminal escapes into the
// log it was meant to help read.
impl fmt::Debug for OutputBuffer
{
  #[ inline ]
  fn fmt( &self, f : &mut fmt::Formatter< '_ > ) -> fmt::Result
  {
    f.debug_struct( "OutputBuffer" )
      .field( "held", &self.held.len() )
      .field( "dropped", &self.dropped )
      .field( "cap", &self.cap )
      .field( "ended", &self.ended )
      .finish()
  }
}

impl OutputBuffer
{
  /// An empty buffer retaining at most `cap` bytes.
  #[ inline ]
  #[ must_use ]
  pub fn with_capacity( cap : usize ) -> Self
  {
    Self { held : VecDeque::new(), dropped : 0, cap, ended : false }
  }

  /// Retention limit in bytes.
  #[ inline ]
  #[ must_use ]
  pub const fn capacity( &self ) -> usize
  {
    self.cap
  }

  /// Absolute cursor one past the newest byte written.
  #[ inline ]
  #[ must_use ]
  pub fn end( &self ) -> u64
  {
    self.dropped + self.held.len() as u64
  }

  /// Total bytes evicted over this buffer's lifetime.
  #[ inline ]
  #[ must_use ]
  pub const fn dropped( &self ) -> u64
  {
    self.dropped
  }

  /// Whether the producing stream has ended.
  #[ inline ]
  #[ must_use ]
  pub const fn has_ended( &self ) -> bool
  {
    self.ended
  }

  /// Record that no further output will arrive.
  #[ inline ]
  pub fn mark_ended( &mut self )
  {
    self.ended = true;
  }

  /// Append `bytes`, evicting from the front to stay within capacity.
  #[ inline ]
  pub fn push( &mut self, bytes : &[ u8 ] )
  {
    self.held.extend( bytes.iter().copied() );
    let overflow = self.held.len().saturating_sub( self.cap );
    if overflow > 0
    {
      self.held.drain( ..overflow );
      self.dropped += overflow as u64;
    }
  }

  /// Read everything retained since `cursor`.
  ///
  /// Takes `&mut self` because the retained window is a ring and decoding needs
  /// it contiguous. Nothing is consumed — the same `cursor` returns the same
  /// bytes again, as long as they have not been evicted since.
  #[ inline ]
  pub fn read_from( &mut self, cursor : u64 ) -> OutputSlice
  {
    let missed = self.dropped.saturating_sub( cursor );
    let start = cursor.max( self.dropped );
    let end = self.end();
    let empty = OutputSlice { text : String::new(), cursor : end, missed, ended : self.ended };

    if start >= end
    {
      // Caller is current, or ahead of us — a cursor from a longer-lived buffer,
      // or one saved across a restart. Reporting `end` walks it back to reality.
      return empty;
    }
    let Ok( offset ) = usize::try_from( start - self.dropped )
    else
    {
      return empty;
    };

    let held = self.held.make_contiguous();
    let ( text, consumed ) = decode_prefix( &held[ offset.. ] );
    OutputSlice { text, cursor : start + consumed as u64, missed, ended : self.ended }
  }
}

/// Decode the longest valid UTF-8 prefix of `bytes`, reporting the bytes it used.
///
/// A terminal emits a byte stream and a read can land mid-character, so the two
/// ways UTF-8 decoding fails need opposite handling:
///
/// - **Unfinished** (`error_len() == None`): the sequence is a valid prefix that
///   simply has not arrived in full. Leave it — the next read picks it up with
///   its remaining bytes and the character arrives whole.
/// - **Invalid** (`error_len() == Some( n )`): the bytes cannot start a valid
///   sequence, usually because eviction cut a character in half. Emit a
///   replacement and step past them. Waiting would stall the cursor forever on
///   bytes that can never become valid.
fn decode_prefix( bytes : &[ u8 ] ) -> ( String, usize )
{
  match core::str::from_utf8( bytes )
  {
    Ok( text ) => ( text.to_owned(), bytes.len() ),
    Err( error ) =>
    {
      let valid = error.valid_up_to();
      let mut text = core::str::from_utf8( &bytes[ ..valid ] ).unwrap_or_default().to_owned();
      match error.error_len()
      {
        Some( bad ) =>
        {
          text.push( char::REPLACEMENT_CHARACTER );
          ( text, valid + bad )
        },
        None => ( text, valid ),
      }
    },
  }
}

/// Lock `buffer`, recovering from a poisoned mutex.
///
/// A panic in the pump thread cannot leave an [`OutputBuffer`] logically broken:
/// the worst it can interrupt is a partial `push`, and a buffer holding fewer
/// bytes than intended is still a valid buffer. Propagating the poison instead
/// would make one thread's panic permanently un-readable output for a session
/// that is otherwise fine.
fn lock( buffer : &Mutex< OutputBuffer > ) -> MutexGuard< '_, OutputBuffer >
{
  buffer.lock().unwrap_or_else( PoisonError::into_inner )
}

/// A thread that keeps a session's pty master drained into an [`OutputBuffer`].
///
/// # The descriptor this holds
///
/// The reader handed to [`OutputPump::spawn`] is a *clone of the pty master*, and
/// while this thread lives it is a master descriptor
/// [`PtySession::shutdown`](claude_pty_core::PtySession::shutdown) cannot reach.
/// A child blocked reading its terminal therefore never sees the hangup, and
/// `shutdown` waits for a process that is waiting for it.
///
/// So the pump ends only when its read ends — which happens when the child's own
/// descriptors close, i.e. when the child exits. Teardown must make the child
/// exit *first*, then [`OutputPump::join`], and only then shut the session down.
/// [`HostedSession::shutdown`](crate::table::HostedSession::shutdown) is that
/// sequence; dropping an `OutputPump` without it leaks a thread and wedges the
/// session it belonged to.
#[ derive( Debug ) ]
pub struct OutputPump
{
  buffer : Arc< Mutex< OutputBuffer > >,
  thread : Option< JoinHandle< () > >,
}

impl OutputPump
{
  /// Start draining `reader` into a fresh buffer retaining `cap` bytes.
  #[ inline ]
  #[ must_use ]
  pub fn spawn( mut reader : File, cap : usize ) -> Self
  {
    let buffer = Arc::new( Mutex::new( OutputBuffer::with_capacity( cap ) ) );
    let sink = Arc::clone( &buffer );

    let thread = std::thread::spawn( move ||
    {
      let mut chunk = [ 0_u8; READ_CHUNK_BYTES ];
      loop
      {
        match reader.read( &mut chunk )
        {
          Ok( 0 ) => break,
          Ok( read ) => lock( &sink ).push( &chunk[ ..read ] ),
          // A signal landing mid-read is not the stream ending, so this arm goes
          // round again. Treating it as an ending would silently stop pumping a
          // live session — the arm below is what that would fall into.
          Err( error ) if error.kind() == std::io::ErrorKind::Interrupted => {},
          // Anything else means the terminal is gone. On Linux a master whose
          // slave has closed reports `EIO` rather than end-of-file, so this arm
          // is the normal way a session ends, not an exceptional one.
          Err( _ ) => break,
        }
      }
      lock( &sink ).mark_ended();
      // `reader` drops here — this thread's master descriptor closes with it.
    } );

    Self { buffer, thread : Some( thread ) }
  }

  /// Read everything buffered since `cursor`.
  #[ inline ]
  #[ must_use ]
  pub fn read_from( &self, cursor : u64 ) -> OutputSlice
  {
    lock( &self.buffer ).read_from( cursor )
  }

  /// Absolute cursor one past the newest byte pumped so far.
  #[ inline ]
  #[ must_use ]
  pub fn end( &self ) -> u64
  {
    lock( &self.buffer ).end()
  }

  /// Whether the pump has seen its stream end.
  #[ inline ]
  #[ must_use ]
  pub fn has_ended( &self ) -> bool
  {
    lock( &self.buffer ).has_ended()
  }

  /// Wait for the pump thread to finish and release its master descriptor.
  ///
  /// Blocks until the child's terminal closes. Call it only once the child is
  /// known to be exiting — see the type-level warning above.
  ///
  /// A pump thread that panicked is joined the same as one that returned: the
  /// descriptor is released either way, which is what the caller is waiting for.
  #[ inline ]
  pub fn join( &mut self )
  {
    if let Some( thread ) = self.thread.take()
    {
      drop( thread.join() );
    }
  }
}
