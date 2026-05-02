//! Asset registry: enumerate available and installed artifacts.

use std::collections::BTreeSet;
use crate::artifact::{ ArtifactKind, ArtifactLayout };
use crate::error::AssetError;
use crate::paths::AssetPaths;

/// Whether a named artifact is currently installed (symlinked) in the target.
#[ derive( Debug, Clone, Copy, PartialEq, Eq ) ]
pub enum InstallStatus
{
  /// A symlink exists in the target `.claude/<kind>/` directory.
  Installed,
  /// The artifact is available in the source but not yet installed.
  Available,
}

/// List artifact names available in the source (`$PRO_CLAUDE/<kind>/`).
///
/// Returns an empty `Vec` — not an error — when the source subdirectory
/// does not exist yet (graceful degradation).
///
/// # Errors
///
/// Returns `AssetError` only if the directory exists but cannot be read.
#[ inline ]
pub fn list_available( paths : &AssetPaths, kind : ArtifactKind ) -> Result< Vec< String >, AssetError >
{
  let dir = paths.source_dir( kind );
  if !dir.exists()
  {
    return Ok( Vec::new() );
  }
  collect_names( &dir, kind )
}

/// List artifact names currently installed (symlinks) in `.claude/<kind>/`.
///
/// Returns an empty `Vec` — not an error — when the target subdirectory
/// does not exist yet.
///
/// # Errors
///
/// Returns `AssetError` only if the directory exists but cannot be read.
#[ inline ]
pub fn list_installed( paths : &AssetPaths, kind : ArtifactKind ) -> Result< Vec< String >, AssetError >
{
  let dir = paths.target_dir( kind );
  if !dir.exists()
  {
    return Ok( Vec::new() );
  }
  collect_symlink_names( &dir, kind )
}

/// Merge available and installed into a single status list.
///
/// Returns entries sorted alphabetically by name; each entry carries
/// `InstallStatus::Installed` or `InstallStatus::Available`.
///
/// # Errors
///
/// Propagates any `AssetError` from directory reads.
#[ inline ]
pub fn list_all( paths : &AssetPaths, kind : ArtifactKind ) -> Result< Vec< ( String, InstallStatus ) >, AssetError >
{
  let available  = list_available( paths, kind )?;
  let installed  = list_installed( paths, kind )?;
  let inst_set : BTreeSet< _ > = installed.iter().cloned().collect();

  let mut all : Vec< _ > = available.into_iter()
    .map( | name |
    {
      let status = if inst_set.contains( &name ) { InstallStatus::Installed } else { InstallStatus::Available };
      ( name, status )
    } )
    .collect();

  // Include installed entries that are not in the source (e.g., orphans).
  for name in inst_set
  {
    if !all.iter().any( | ( n, _ ) | *n == name )
    {
      all.push( ( name, InstallStatus::Installed ) );
    }
  }

  all.sort_by( | a, b | a.0.cmp( &b.0 ) );
  Ok( all )
}

// ── Private helpers ───────────────────────────────────────────────────────────

fn collect_names( dir : &std::path::Path, kind : ArtifactKind ) -> Result< Vec< String >, AssetError >
{
  let mut names = Vec::new();
  for entry in std::fs::read_dir( dir ).map_err( AssetError::Io )?
  {
    let entry = entry.map_err( AssetError::Io )?;
    let path  = entry.path();
    if let Some( name ) = artifact_name( &path, kind )
    {
      names.push( name );
    }
  }
  names.sort();
  Ok( names )
}

fn collect_symlink_names( dir : &std::path::Path, kind : ArtifactKind ) -> Result< Vec< String >, AssetError >
{
  let mut names = Vec::new();
  for entry in std::fs::read_dir( dir ).map_err( AssetError::Io )?
  {
    let entry    = entry.map_err( AssetError::Io )?;
    let path     = entry.path();
    let metadata = std::fs::symlink_metadata( &path ).map_err( AssetError::Io )?;
    if !metadata.file_type().is_symlink()
    {
      continue;
    }
    if let Some( name ) = artifact_name( &path, kind )
    {
      names.push( name );
    }
  }
  names.sort();
  Ok( names )
}

/// Extract the canonical artifact name from a filesystem path.
///
/// For `File` layout: strips the expected extension.
/// For `Directory` layout: returns the directory name as-is.
fn artifact_name( path : &std::path::Path, kind : ArtifactKind ) -> Option< String >
{
  match kind.layout()
  {
    ArtifactLayout::File =>
    {
      let ext  = kind.file_extension()?;
      let stem = path.file_stem()?.to_str()?.to_string();
      if path.extension().and_then( | e | e.to_str() ) == Some( ext )
      {
        Some( stem )
      }
      else
      {
        None
      }
    }
    ArtifactLayout::Directory =>
    {
      // Two callers with different invariants:
      //   collect_names()         (source dir) — entries are real dirs; is_dir() covers them.
      //   collect_symlink_names() (target dir) — entries are confirmed symlinks; the
      //     symlink_metadata() branch covers dangling symlinks whose is_dir() returns false.
      if path.is_dir() || std::fs::symlink_metadata( path ).is_ok_and( | m | m.file_type().is_symlink() )
      {
        path.file_name()?.to_str().map( str::to_string )
      }
      else
      {
        None
      }
    }
  }
}
