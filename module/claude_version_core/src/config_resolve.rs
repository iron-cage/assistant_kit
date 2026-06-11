//! 4-layer config resolution engine: env var → project config → user config → catalog default.
//!
//! See `docs/algorithm/002_config_resolution.md` for the algorithm specification
//! and `config_catalog` for the known settings catalog.

use std::path::{ Path, PathBuf };
use crate::config_catalog::SettingDef;
use crate::settings_io::read_all_settings;

/// The layer that supplied the resolved value for a settings key.
///
/// Displayed as source annotation in text and JSON output.
#[ derive( Debug, Clone, Copy, PartialEq, Eq ) ]
pub enum Layer
{
  /// Value came from an environment variable (highest priority).
  Env,
  /// Value came from a project `.claude/settings.json` ancestor.
  Project,
  /// Value came from the user's `~/.claude/settings.json`.
  User,
  /// Value came from the catalog default.
  Default,
  /// No layer supplied a value.
  Absent,
}

impl core::fmt::Display for Layer
{
  #[ inline ]
  fn fmt( &self, f : &mut core::fmt::Formatter< '_ > ) -> core::fmt::Result
  {
    f.write_str( match self
    {
      Self::Env     => "env",
      Self::Project => "project",
      Self::User    => "user",
      Self::Default => "default",
      Self::Absent  => "absent",
    } )
  }
}

/// The resolved effective value of a settings key.
#[ derive( Debug ) ]
pub struct ResolvedValue
{
  /// The effective value, or `None` when absent in all layers.
  pub value  : Option< String >,
  /// The layer that supplied the value.
  pub source : Layer,
}

/// Resolve the effective value for a single settings key.
///
/// Applies 4 steps in priority order per `docs/algorithm/002_config_resolution.md`:
/// 1. Env var check (catalog mapping only).
/// 2. Project config ancestor search (walks cwd upward, stops at git boundary or root).
/// 3. User config (`<home_dir>/.claude/settings.json`).
/// 4. Catalog default; returns `Absent` when no layer supplies a value.
///
/// # Arguments
///
/// * `key`      — Settings key to resolve.
/// * `home_dir` — User home directory (for user config).
/// * `cwd`      — Current working directory (for project config ancestor search).
/// * `catalog`  — Known settings catalog.
#[ inline ]
#[ must_use ]
pub fn resolve(
  key      : &str,
  home_dir : &Path,
  cwd      : &Path,
  catalog  : &[ SettingDef ],
) -> ResolvedValue
{
  // Step 1 — Environment variable check.
  if let Some( env_var ) = catalog.iter().find( | d | d.key == key ).and_then( | d | d.env_var )
  {
    if let Ok( val ) = std::env::var( env_var )
    {
      if !val.is_empty()
      {
        return ResolvedValue { value : Some( val ), source : Layer::Env };
      }
    }
  }

  // Step 2 — Project config ancestor search.
  if let Some( val ) = find_in_project_config( key, cwd )
  {
    return ResolvedValue { value : Some( val ), source : Layer::Project };
  }

  // Step 3 — User config check.
  let user_settings = home_dir.join( ".claude" ).join( "settings.json" );
  if let Ok( pairs ) = read_all_settings( &user_settings )
  {
    if let Some( ( _, v ) ) = pairs.into_iter().find( | ( k, _ ) | k == key )
    {
      return ResolvedValue { value : Some( v ), source : Layer::User };
    }
  }

  // Step 4 — Catalog default.
  if let Some( def ) = catalog.iter().find( | d | d.key == key ).and_then( | d | d.default )
  {
    return ResolvedValue { value : Some( def.to_string() ), source : Layer::Default };
  }

  ResolvedValue { value : None, source : Layer::Absent }
}

/// Resolve all settings across all layers, returning sorted key-value pairs.
///
/// Unions keys from: catalog, project config, and user config — then calls
/// [`resolve`] on each key in sorted order.
///
/// # Arguments
///
/// * `home_dir` — User home directory.
/// * `cwd`      — Current working directory (for project config ancestor search).
/// * `catalog`  — Known settings catalog.
#[ inline ]
#[ must_use ]
pub fn resolve_all(
  home_dir : &Path,
  cwd      : &Path,
  catalog  : &[ SettingDef ],
) -> Vec< ( String, ResolvedValue ) >
{
  let mut keys : Vec< String > = catalog.iter().map( | d | d.key.to_string() ).collect();

  // Add keys from project config (deduplicated).
  if let Some( project_file ) = find_project_config_file( cwd )
  {
    if let Ok( pairs ) = read_all_settings( &project_file )
    {
      for ( k, _ ) in pairs
      {
        if !keys.iter().any( | s | s == &k ) { keys.push( k ); }
      }
    }
  }

  // Add keys from user config (deduplicated).
  let user_settings = home_dir.join( ".claude" ).join( "settings.json" );
  if let Ok( pairs ) = read_all_settings( &user_settings )
  {
    for ( k, _ ) in pairs
    {
      if !keys.iter().any( | s | s == &k ) { keys.push( k ); }
    }
  }

  keys.sort();
  keys.into_iter()
  .map( | k |
  {
    let resolved = resolve( &k, home_dir, cwd, catalog );
    ( k, resolved )
  } )
  .collect()
}

// ── Private helpers ────────────────────────────────────────────────────────────

/// Look up `key` in the nearest project `.claude/settings.json` ancestor of `cwd`.
fn find_in_project_config( key : &str, cwd : &Path ) -> Option< String >
{
  let file  = find_project_config_file( cwd )?;
  let pairs = read_all_settings( &file ).ok()?;
  pairs.into_iter().find( | ( k, _ ) | k == key ).map( | ( _, v ) | v )
}

/// Locate the nearest `.claude/settings.json` ancestor of `cwd`.
///
/// Walks parent directories from `cwd` upward. Stops at:
/// - A directory containing `.git` (git repository boundary), OR
/// - Filesystem root.
///
/// Returns `None` if no project config is found before the boundary.
fn find_project_config_file( cwd : &Path ) -> Option< PathBuf >
{
  let mut dir = cwd.to_path_buf();
  loop
  {
    // Check for .claude/settings.json in this directory first.
    let candidate = dir.join( ".claude" ).join( "settings.json" );
    if candidate.exists()
    {
      return Some( candidate );
    }

    // Stop at git repository boundary.
    if dir.join( ".git" ).exists()
    {
      return None;
    }

    // Walk up one directory level.
    match dir.parent()
    {
      Some( parent ) => dir = parent.to_path_buf(),
      None           => return None,
    }
  }
}
