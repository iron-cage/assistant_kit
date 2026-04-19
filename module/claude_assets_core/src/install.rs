//! Symlink-based artifact installation and removal.
//!
//! ## Invariant
//!
//! All install operations create a filesystem symlink — never [`std::fs::copy`].
//! Symlinks preserve the single source of truth in `$PRO_CLAUDE`; edits there
//! propagate to every project instantly. On Unix one call covers both files and
//! directories; on Windows `symlink_file` / `symlink_dir` are dispatched on
//! [`ArtifactLayout`] because Windows requires different APIs per target type.
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
pub struct InstallReport< A >
{
  /// Artifact kind (e.g., `ArtifactKind::Rule`).
  pub kind   : ArtifactKind,
  /// Artifact name (e.g., `"rust"`).
  pub name   : String,
  /// Which action was taken.
  pub action : A,
}

/// Outcome of a successful [`install`] call.
#[ derive( Debug, Clone, Copy, PartialEq, Eq ) ]
pub enum InstallOutcome
{
  /// A new symlink was created.
  Installed,
  /// A symlink was already present and re-linked (idempotent update).
  Reinstalled,
}

/// Outcome of a successful [`uninstall`] call.
#[ derive( Debug, Clone, Copy, PartialEq, Eq ) ]
pub enum UninstallOutcome
{
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
pub fn install( paths : &AssetPaths, kind : ArtifactKind, name : &str ) -> Result< InstallReport< InstallOutcome >, AssetError >
{
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
  let action = if std::fs::symlink_metadata( &tgt_path ).is_ok()
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
    InstallOutcome::Reinstalled
  }
  else
  {
    InstallOutcome::Installed
  };

  create_symlink( &src_path, &tgt_path, kind.layout() ).map_err( AssetError::Io )?;
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
pub fn uninstall( paths : &AssetPaths, kind : ArtifactKind, name : &str ) -> Result< InstallReport< UninstallOutcome >, AssetError >
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
        action : UninstallOutcome::NotInstalled,
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
  Ok( InstallReport { kind, name : name.to_string(), action : UninstallOutcome::Uninstalled } )
}

// ── Private helpers ───────────────────────────────────────────────────────────

// Fix(issue-108): cross-platform symlink dispatch.
// Root cause: std::os::unix::fs::symlink is cfg(unix)-gated and absent on Windows;
//   Windows needs symlink_file vs symlink_dir based on whether the target is a
//   file or a directory — a distinction Unix's single symlink() doesn't require.
// Pitfall: on Windows, calling symlink_file on a directory path (or vice versa)
//   produces a broken symlink without returning an error.
#[ cfg( unix ) ]
#[ inline ]
fn create_symlink( src : &std::path::Path, dst : &std::path::Path, _layout : ArtifactLayout ) -> std::io::Result<()>
{
  std::os::unix::fs::symlink( src, dst )
}

#[ cfg( windows ) ]
#[ inline ]
fn create_symlink( src : &std::path::Path, dst : &std::path::Path, layout : ArtifactLayout ) -> std::io::Result<()>
{
  match layout
  {
    ArtifactLayout::File      => std::os::windows::fs::symlink_file( src, dst ),
    ArtifactLayout::Directory => std::os::windows::fs::symlink_dir( src, dst ),
  }
}

fn artifact_path( base_dir : &std::path::Path, kind : ArtifactKind, name : &str ) -> PathBuf
{
  match kind.layout()
  {
    ArtifactLayout::File =>
    {
      let ext = kind.file_extension().unwrap_or( "" );
      base_dir.join( format!( "{name}.{ext}" ) )
    }
    ArtifactLayout::Directory => base_dir.join( name ),
  }
}

fn source_path( paths : &AssetPaths, kind : ArtifactKind, name : &str ) -> PathBuf
{
  artifact_path( &paths.source_dir( kind ), kind, name )
}

fn target_path( paths : &AssetPaths, kind : ArtifactKind, name : &str ) -> PathBuf
{
  artifact_path( &paths.target_dir( kind ), kind, name )
}
