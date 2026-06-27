//! Append-only journal writer — open-write-close per event for crash safety.

use std::{ fs, io, io::Write, path::{ Path, PathBuf } };
use crate::event::EventRecord;

/// Append-only writer that records structured events to daily JSONL files.
///
/// Each [`JournalWriter::append`] call opens the current UTC day's file (creating
/// it and its parent directory if absent), serializes the [`EventRecord`] to a
/// single JSON line, writes it with a trailing `\n`, and closes the file handle.
/// The open-write-close pattern makes each append crash-safe — no state is held
/// between calls.
///
/// `JournalWriter` is `Send + Sync` because it contains only a `PathBuf`.
/// Concurrent `append()` calls from different threads are safe: the OS serializes
/// writes at the file level when `O_APPEND` is set.
#[ derive( Debug ) ]
pub struct JournalWriter
{
  dir : PathBuf,
}

impl JournalWriter
{
  /// Create a writer targeting `dir`.
  ///
  /// Does not create the directory until the first [`JournalWriter::append`] call.
  #[ inline ]
  #[ must_use ]
  pub fn new( dir : PathBuf ) -> Self
  {
    Self { dir }
  }

  /// Append one event to today's UTC-dated journal file (`YYYY-MM-DD.jsonl`).
  ///
  /// Creates `dir` (and any missing ancestors) and the daily file if absent.
  /// Opens in append mode — existing content is never modified or truncated.
  /// Serializes `event` to a single JSON line terminated by `\n`.
  ///
  /// # Errors
  ///
  /// Returns `Err` on I/O failure (permission denied, disk full, etc.).
  #[ inline ]
  pub fn append( &self, event : &EventRecord ) -> io::Result< () >
  {
    fs::create_dir_all( &self.dir )?;
    let path = self.dir.join( crate::rotation::today_filename() );
    let mut line = serde_json::to_string( event )
      .map_err( | e | io::Error::new( io::ErrorKind::InvalidData, e ) )?;
    line.push( '\n' );
    let mut file = fs::OpenOptions::new()
      .create( true )
      .append( true )
      .open( &path )?;
    file.write_all( line.as_bytes() )
  }

  /// Return the configured journal directory.
  #[ inline ]
  #[ must_use ]
  pub fn dir( &self ) -> &Path
  {
    &self.dir
  }
}
