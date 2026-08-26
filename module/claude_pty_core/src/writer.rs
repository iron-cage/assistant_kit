//! Bounded-queue writer thread for a pty master.
//!
//! # Why a thread at all
//!
//! Writing to a pty master blocks once the kernel's input buffer fills, and that
//! buffer fills whenever the child stops draining its stdin — busy, stopped, or
//! hung. A synchronous `write_all` from a caller's event loop would therefore
//! stall the entire loop on an unresponsive child: no reads, no timers, no
//! shutdown. Moving writes to a dedicated thread means only that thread blocks.
//!
//! # Why the queue is bounded
//!
//! An unbounded queue converts a stalled child into unbounded memory growth,
//! which is the same outage arriving later and harder to diagnose. At capacity,
//! [`WriterHandle::send`] fails with [`crate::Error::WriterFull`] and the caller
//! decides — drop the input, surface backpressure, or kill the session.

use std::io::Write;
use std::sync::mpsc::{ sync_channel, SyncSender, TrySendError };
use std::thread::JoinHandle;

use crate::error::{ Error, Result };

/// Default queue depth, in messages.
///
/// Sized for interactive typing and paste bursts, not bulk transfer: a child
/// that has not drained 256 queued writes is not slow, it is stuck.
pub const DEFAULT_QUEUE_CAPACITY : usize = 256;

/// Handle to a running writer thread.
///
/// Dropping the handle closes the queue and detaches the thread; call
/// [`WriterHandle::shutdown`] instead to close the queue and join it.
#[ derive( Debug ) ]
pub struct WriterHandle
{
  tx : Option< SyncSender< Vec< u8 > > >,
  thread : Option< JoinHandle< () > >,
}

impl WriterHandle
{
  /// Start a writer thread that drains queued writes into `sink`.
  ///
  /// `sink` is typically a duplicate of the pty master; the writer owns it for
  /// the thread's lifetime.
  #[ inline ]
  #[ must_use ]
  pub fn spawn< W >( mut sink : W, capacity : usize ) -> Self
  where
    W : Write + Send + 'static,
  {
    let ( tx, rx ) = sync_channel::< Vec< u8 > >( capacity );
    let thread = std::thread::spawn( move ||
    {
      // A write error means the far end is gone (child exited, master closed).
      // There is no one left to report it to and nothing useful to retry, so the
      // thread simply stops draining — `send` then reports `WriterGone`.
      while let Ok( chunk ) = rx.recv()
      {
        if sink.write_all( &chunk ).is_err() || sink.flush().is_err()
        {
          break;
        }
      }
    });
    Self { tx : Some( tx ), thread : Some( thread ) }
  }

  /// Queue `bytes` for writing.
  ///
  /// Never blocks. Returns immediately whether the queue had room or not.
  ///
  /// # Errors
  ///
  /// - [`Error::WriterFull`] — the queue is at capacity; `bytes` was not queued.
  /// - [`Error::WriterGone`] — the writer thread has exited.
  #[ inline ]
  pub fn send( &self, bytes : &[ u8 ] ) -> Result< () >
  {
    let Some( tx ) = self.tx.as_ref() else { return Err( Error::WriterGone ) };
    match tx.try_send( bytes.to_vec() )
    {
      Ok( () ) => Ok( () ),
      Err( TrySendError::Full( _ ) ) => Err( Error::WriterFull ),
      Err( TrySendError::Disconnected( _ ) ) => Err( Error::WriterGone ),
    }
  }

  /// Close the queue and wait for the writer thread to drain and exit.
  ///
  /// Idempotent — a second call is a no-op.
  #[ inline ]
  pub fn shutdown( &mut self )
  {
    // Dropping the sender is what ends the thread's `recv` loop; joining before
    // dropping it would deadlock.
    self.tx = None;
    if let Some( thread ) = self.thread.take()
    {
      let _ = thread.join();
    }
  }
}

impl Drop for WriterHandle
{
  #[ inline ]
  fn drop( &mut self )
  {
    self.shutdown();
  }
}
