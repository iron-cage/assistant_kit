//! A child process running on a pty, with a non-blocking write path.

use std::ffi::OsString;
use std::fs::File;
use std::path::PathBuf;
use std::process::{ Child, Command, Stdio };

use crate::env_scrub::{ self, CHILD_TERM };
use crate::error::{ Error, Result };
use crate::ffi;
use crate::pty::{ Pty, WinSize };
use crate::writer::{ WriterHandle, DEFAULT_QUEUE_CAPACITY };

/// How to spawn a child on a fresh pty.
#[ derive( Debug, Clone ) ]
pub struct SessionConfig
{
  program : OsString,
  args : Vec< OsString >,
  envs : Vec< ( OsString, OsString ) >,
  cwd : Option< PathBuf >,
  win_size : WinSize,
  queue_capacity : usize,
}

impl SessionConfig
{
  /// Start a configuration for `program`.
  #[ inline ]
  #[ must_use ]
  pub fn new( program : impl Into< OsString > ) -> Self
  {
    Self
    {
      program : program.into(),
      args : Vec::new(),
      envs : Vec::new(),
      cwd : None,
      win_size : WinSize::default(),
      queue_capacity : DEFAULT_QUEUE_CAPACITY,
    }
  }

  /// Append one argument.
  #[ inline ]
  #[ must_use ]
  pub fn arg( mut self, arg : impl Into< OsString > ) -> Self
  {
    self.args.push( arg.into() );
    self
  }

  /// Set an environment variable on the child.
  ///
  /// Applied *after* scrubbing, so this is also how a caller deliberately
  /// restores a variable the scrub list would otherwise remove.
  #[ inline ]
  #[ must_use ]
  pub fn env( mut self, key : impl Into< OsString >, value : impl Into< OsString > ) -> Self
  {
    self.envs.push( ( key.into(), value.into() ) );
    self
  }

  /// Set the child's working directory.
  #[ inline ]
  #[ must_use ]
  pub fn cwd( mut self, dir : impl Into< PathBuf > ) -> Self
  {
    self.cwd = Some( dir.into() );
    self
  }

  /// Set the initial terminal size.
  #[ inline ]
  #[ must_use ]
  pub const fn win_size( mut self, size : WinSize ) -> Self
  {
    self.win_size = size;
    self
  }

  /// Set the writer queue depth.
  #[ inline ]
  #[ must_use ]
  pub const fn queue_capacity( mut self, capacity : usize ) -> Self
  {
    self.queue_capacity = capacity;
    self
  }
}

/// A live child process attached to a pty.
///
/// Writes go through a bounded queue drained by a dedicated thread, so
/// [`PtySession::write`] never blocks on an unresponsive child.
#[ derive( Debug ) ]
pub struct PtySession
{
  /// `None` after [`PtySession::shutdown`] — dropping the `Pty` is what closes
  /// the master, and closing the master is what hangs the child up.
  pty : Option< Pty >,
  /// Cached from `pty`, so the slave path stays reportable after shutdown.
  slave_path : String,
  child : Child,
  writer : WriterHandle,
  reader : Option< File >,
}

impl PtySession
{
  /// Allocate a pty and spawn `config`'s program attached to it.
  ///
  /// The child leads a new session with the pty slave as its controlling
  /// terminal, and its environment is scrubbed per
  /// [`crate::env_scrub`] before `config`'s own overrides are applied.
  ///
  /// # Errors
  ///
  /// Returns [`Error::Os`] if pty allocation fails, or [`Error::Spawn`] if the
  /// child cannot be started.
  #[ inline ]
  pub fn spawn( config : &SessionConfig ) -> Result< Self >
  {
    let pty = Pty::open()?;
    pty.resize( config.win_size )?;

    // Three independent descriptors: the child dup2's each onto one of its
    // standard streams, and a single shared descriptor would make closing one
    // close all three.
    let slave_in = pty.open_slave()?;
    let slave_out = pty.open_slave()?;
    let slave_err = pty.open_slave()?;

    let mut cmd = Command::new( &config.program );
    cmd.args( &config.args );
    if let Some( dir ) = config.cwd.as_ref()
    {
      cmd.current_dir( dir );
    }

    for name in env_scrub::scrub_list( std::env::vars().map( | ( k, _ ) | k ).collect::< Vec< _ > >().iter().map( String::as_str ) )
    {
      cmd.env_remove( name );
    }
    cmd.env( "TERM", CHILD_TERM );
    for ( key, value ) in &config.envs
    {
      cmd.env( key, value );
    }

    cmd.stdin( Stdio::from( slave_in ) );
    cmd.stdout( Stdio::from( slave_out ) );
    cmd.stderr( Stdio::from( slave_err ) );
    ffi::attach_controlling_terminal( &mut cmd );

    let child = cmd.spawn().map_err( Error::Spawn )?;

    // Every slave descriptor was moved into `Stdio` above and is closed in the
    // parent by `spawn`. That matters: while the parent holds any slave open,
    // reads from the master never reach EOF, so an exited child is
    // indistinguishable from a live one.

    let write_end = File::from(
      pty.master().try_clone().map_err( | source | Error::Os { op : "dup(master)", source } )?
    );
    let read_end = File::from(
      pty.master().try_clone().map_err( | source | Error::Os { op : "dup(master)", source } )?
    );

    Ok( Self
    {
      slave_path : pty.slave_path().to_string(),
      pty : Some( pty ),
      child,
      writer : WriterHandle::spawn( write_end, config.queue_capacity ),
      reader : Some( read_end ),
    })
  }

  /// Queue `bytes` for delivery to the child's stdin.
  ///
  /// Never blocks — see [`crate::writer`] for why.
  ///
  /// # Errors
  ///
  /// - [`Error::WriterFull`] — the child has stopped reading stdin.
  /// - [`Error::WriterGone`] — the writer thread has exited.
  #[ inline ]
  pub fn write( &self, bytes : &[ u8 ] ) -> Result< () >
  {
    self.writer.send( bytes )
  }

  /// Change the terminal size, raising `SIGWINCH` in the child.
  ///
  /// # Errors
  ///
  /// - [`Error::SessionClosed`] — [`PtySession::shutdown`] has already run.
  /// - [`Error::Os`] — the `TIOCSWINSZ` request failed.
  #[ inline ]
  pub fn resize( &self, size : WinSize ) -> Result< () >
  {
    self.pty.as_ref().ok_or( Error::SessionClosed )?.resize( size )
  }

  /// Take the read side of the pty master.
  ///
  /// Returns `None` on any call after the first — the reader is owned by
  /// whoever took it, so ownership cannot be handed out twice.
  #[ inline ]
  pub fn take_reader( &mut self ) -> Option< File >
  {
    self.reader.take()
  }

  /// The child's process id.
  #[ inline ]
  #[ must_use ]
  pub fn pid( &self ) -> u32
  {
    self.child.id()
  }

  /// Path of the slave device, e.g. `/dev/pts/7`.
  ///
  /// Remains readable after [`PtySession::shutdown`]: the path is what a log line
  /// or an error message names, and losing it exactly when the session ends would
  /// make the shutdown itself the hardest event to attribute.
  #[ inline ]
  #[ must_use ]
  pub fn slave_path( &self ) -> &str
  {
    &self.slave_path
  }

  /// Check whether the child has exited, without blocking.
  ///
  /// # Errors
  ///
  /// Returns [`Error::Os`] if the underlying `waitpid` fails.
  #[ inline ]
  pub fn try_wait( &mut self ) -> Result< Option< std::process::ExitStatus > >
  {
    self.child.try_wait().map_err( | source | Error::Os { op : "waitpid", source } )
  }

  /// Close every master descriptor and reap the child.
  ///
  /// Does not signal the child. Closing the last master descriptor is what ends
  /// the session: the child's reads from the slave see `EOF` and the kernel
  /// delivers `SIGHUP` to its process group. Killing outright is the caller's
  /// decision, not this crate's.
  ///
  /// All three master descriptors must go, in order — the writer thread's clone,
  /// this session's read clone, and the `Pty`'s own. Leaving any one open leaves
  /// the child's stdin apparently connected, so a child blocked on a read never
  /// returns and this call waits forever.
  ///
  /// A reader handed out by [`PtySession::take_reader`] is the caller's to drop:
  /// while it is alive it is a fourth master descriptor, and this call blocks.
  ///
  /// Idempotent — a second call returns the exit status recorded by the first.
  ///
  /// # Errors
  ///
  /// Returns [`Error::Os`] if reaping the child fails.
  #[ inline ]
  pub fn shutdown( &mut self ) -> Result< std::process::ExitStatus >
  {
    self.writer.shutdown();
    self.reader = None;
    self.pty = None;
    self.child.wait().map_err( | source | Error::Os { op : "waitpid", source } )
  }
}
