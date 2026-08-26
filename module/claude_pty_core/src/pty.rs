//! Pseudo-terminal master/slave pair allocation.

use std::fs::OpenOptions;
use std::os::unix::io::{ AsRawFd, OwnedFd };

use crate::error::Result;
use crate::ffi;

/// Terminal dimensions in character cells.
///
/// Pixel dimensions are deliberately absent — nothing in this workspace renders
/// in pixel units, and the kernel accepts zero for both.
#[ derive( Debug, Clone, Copy, PartialEq, Eq ) ]
pub struct WinSize
{
  /// Height in character cells.
  pub rows : u16,
  /// Width in character cells.
  pub cols : u16,
}

impl WinSize
{
  /// Construct a window size.
  #[ inline ]
  #[ must_use ]
  pub const fn new( rows : u16, cols : u16 ) -> Self
  {
    Self { rows, cols }
  }
}

impl Default for WinSize
{
  /// 24×80 — the historical default, and what a child assumes when never told.
  #[ inline ]
  fn default() -> Self
  {
    Self { rows : 24, cols : 80 }
  }
}

/// An allocated pty master plus the path of the slave device paired with it.
///
/// The master fd is owned: dropping `Pty` closes it, which delivers `EOF`/
/// `SIGHUP` to whatever is attached to the slave.
#[ derive( Debug ) ]
pub struct Pty
{
  master : OwnedFd,
  slave_path : String,
}

impl Pty
{
  /// Allocate a new pty pair.
  ///
  /// # Errors
  ///
  /// Returns [`crate::Error::Os`] naming the POSIX call that failed, or
  /// [`crate::Error::NonUtf8SlavePath`] if the kernel reports a non-UTF-8 slave
  /// device path.
  #[ inline ]
  pub fn open() -> Result< Self >
  {
    let master = ffi::open_master()?;
    let slave_path = ffi::slave_path( master.as_raw_fd() )?;
    Ok( Self { master, slave_path } )
  }

  /// Borrow the master file descriptor.
  #[ inline ]
  #[ must_use ]
  pub const fn master( &self ) -> &OwnedFd
  {
    &self.master
  }

  /// Path of the slave device, e.g. `/dev/pts/7`.
  #[ inline ]
  #[ must_use ]
  pub fn slave_path( &self ) -> &str
  {
    &self.slave_path
  }

  /// Open a fresh read/write handle to the slave device.
  ///
  /// Each call returns an independent descriptor. A child needs three (stdin,
  /// stdout, stderr), and the parent must drop every copy it holds after
  /// spawning — while any slave descriptor remains open in the parent, reads
  /// from the master never see `EOF`, so an exited child looks like a live one.
  ///
  /// # Errors
  ///
  /// Returns [`crate::Error::Os`] if the device cannot be opened.
  #[ inline ]
  pub fn open_slave( &self ) -> Result< std::fs::File >
  {
    OpenOptions::new()
      .read( true )
      .write( true )
      .open( &self.slave_path )
      .map_err( | source | crate::Error::Os { op : "open(slave)", source } )
  }

  /// Set the terminal window size.
  ///
  /// Applied to the master, which the kernel propagates to the slave and
  /// announces to the foreground process group as `SIGWINCH`.
  ///
  /// # Errors
  ///
  /// Returns [`crate::Error::Os`] if the `TIOCSWINSZ` request fails.
  #[ inline ]
  pub fn resize( &self, size : WinSize ) -> Result< () >
  {
    ffi::set_win_size( self.master.as_raw_fd(), size.rows, size.cols )
  }
}
