//! Symlink-based artifact installation and removal.
//!
//! ## Invariant
//!
//! All install operations use [`std::os::unix::fs::symlink`] — never
//! [`std::fs::copy`]. Symlinks preserve the single source of truth in
//! `$PRO_CLAUDE`; edits there propagate to every project instantly.
//!
//! ## Uninstall guard
//!
//! [`uninstall`] calls [`std::fs::symlink_metadata`] (not `metadata`) to detect
//! whether the target is a symlink before removing it. Regular files are refused
//! with [`AssetError::NotASymlink`] to prevent accidental data loss.

use std::path::PathBuf;
use crate::artifact::{ ArtifactKind, ArtifactLayout };
use crate::error::AssetError;
use crate::paths::AssetPaths;

/// Outcome of a successful install or uninstall operation.
#[ derive( Debug ) ]
pub struct InstallReport
{
  /// Artifact kind (e.g., `ArtifactKind::Rule`).
  pub kind   : ArtifactKind,
  /// Artifact name (e.g., `"rust"`).
  pub name   : String,
  /// Which action was taken.
  pub action : InstallAction,
}

/// The specific action recorded in an [`InstallReport`].
#[ derive( Debug, Clone, Copy, PartialEq, Eq ) ]
pub enum InstallAction
{
  /// A new symlink was created.
  Installed,
  /// A symlink was already present and re-linked (idempotent update).
  Reinstalled,
  /// The symlink was removed.
  Uninstalled,
  /// No installed symlink was found; nothing was changed.
  NotInstalled,
}

/// Install `name` of `kind` from the source root into the target `.claude/<kind>/`.
///
/// - Creates the target subdirectory if it does not exist.
/// - If a symlink already exists at the target path, removes it and re-links
///   (idempotent: re-linking updates to any source change).
/// - Never replaces a regular file (returns `AssetError::NotASymlink`).
///
/// # Errors
///
/// - `AssetError::SourceNotFound` — source artifact absent from `$PRO_CLAUDE/<kind>/`
/// - `AssetError::NotASymlink` — target path exists as a regular file
/// - `AssetError::Io` — filesystem error
#[ inline ]
pub fn install( paths : &AssetPaths, kind : ArtifactKind, name : &str ) -> Result< InstallReport, AssetError >
{
  use std::os::unix::fs::symlink;

  let src_path = source_path( paths, kind, name );
  if !src_path.exists()
  {
    return Err( AssetError::SourceNotFound
    {
      kind : kind.as_str().to_string(),
      name : name.to_string(),
    } );
  }

  let tgt_dir  = paths.target_dir( kind );
  let tgt_path = target_path( paths, kind, name );

  // Ensure target subdirectory exists.
  std::fs::create_dir_all( &tgt_dir ).map_err( AssetError::Io )?;

  // Determine action — re-link or fresh link.
  let action = if tgt_path.exists() || std::fs::symlink_metadata( &tgt_path ).is_ok()
  {
    let meta = std::fs::symlink_metadata( &tgt_path ).map_err( AssetError::Io )?;
    if !meta.file_type().is_symlink()
    {
      return Err( AssetError::NotASymlink
      {
        kind : kind.as_str().to_string(),
        name : name.to_string(),
      } );
    }
    std::fs::remove_file( &tgt_path ).map_err( AssetError::Io )?;
    InstallAction::Reinstalled
  }
  else
  {
    InstallAction::Installed
  };

  symlink( &src_path, &tgt_path ).map_err( AssetError::Io )?;
  Ok( InstallReport { kind, name : name.to_string(), action } )
}

/// Uninstall `name` of `kind` by removing the symlink from `.claude/<kind>/`.
///
/// - Returns `InstallReport { action: NotInstalled }` (not an error) when no
///   symlink exists at the target path.
/// - Refuses to remove a regular file: returns `AssetError::NotASymlink`.
///
/// # Errors
///
/// - `AssetError::NotASymlink` — target path is a regular file (data-loss guard)
/// - `AssetError::Io` — filesystem error during metadata read or removal
#[ inline ]
pub fn uninstall( paths : &AssetPaths, kind : ArtifactKind, name : &str ) -> Result< InstallReport, AssetError >
{
  let tgt_path = target_path( paths, kind, name );

  // symlink_metadata does not follow symlinks — correctly detects dangling symlinks.
  let meta = match std::fs::symlink_metadata( &tgt_path )
  {
    Ok( m )                                    => m,
    Err( e ) if e.kind() == std::io::ErrorKind::NotFound =>
    {
      return Ok( InstallReport
      {
        kind,
        name   : name.to_string(),
        action : InstallAction::NotInstalled,
      } );
    }
    Err( e ) => return Err( AssetError::Io( e ) ),
  };

  if !meta.file_type().is_symlink()
  {
    return Err( AssetError::NotASymlink
    {
      kind : kind.as_str().to_string(),
      name : name.to_string(),
    } );
  }

  std::fs::remove_file( &tgt_path ).map_err( AssetError::Io )?;
  Ok( InstallReport { kind, name : name.to_string(), action : InstallAction::Uninstalled } )
}

// ── Private helpers ───────────────────────────────────────────────────────────

fn source_path( paths : &AssetPaths, kind : ArtifactKind, name : &str ) -> PathBuf
{
  let dir = paths.source_dir( kind );
  match kind.layout()
  {
    ArtifactLayout::File =>
    {
      let ext = kind.file_extension().unwrap_or( "" );
      dir.join( format!( "{name}.{ext}" ) )
    }
    ArtifactLayout::Directory => dir.join( name ),
  }
}

fn target_path( paths : &AssetPaths, kind : ArtifactKind, name : &str ) -> PathBuf
{
  let dir = paths.target_dir( kind );
  match kind.layout()
  {
    ArtifactLayout::File =>
    {
      let ext = kind.file_extension().unwrap_or( "" );
      dir.join( format!( "{name}.{ext}" ) )
    }
    ArtifactLayout::Directory => dir.join( name ),
  }
}
