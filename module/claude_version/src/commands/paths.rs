//! `.paths` — report filesystem paths clv reads from or writes to.
//!
//! Complements `.runtime_files` (unlabeled, pipeline-only, version-history-cache
//! path only) by adding labels, descriptions, and the settings, project-settings,
//! versions-directory, and binary-symlink paths that `.runtime_files` does not report.
//!
//! # Mode dispatch
//!
//! | `key::` | Mode |
//! |---------|------|
//! | absent  | show-all (all 5 known paths) |
//! | present | single resolved path for that key |
//!
//! ## Exit Codes
//!
//! | Code | Meaning |
//! |------|---------|
//! | 0 | Success |
//! | 1 | Invalid or empty `key::` value |
//! | 2 | `HOME` unset or empty |

use core::fmt::Write;
use std::path::Path;

use unilang::data::{ ErrorCode, ErrorData, OutputData };
use unilang::interpreter::ExecutionContext;
use unilang::semantic::VerifiedCommand;
use unilang::types::Value;

use crate::output::{ OutputFormat, OutputOptions, json_escape };
use claude_version_core::paths::ClaudeVersionPaths;

/// The 5 path keys `.paths` can report.
///
/// Not part of the crate's public API — reachable only within `commands::paths`,
/// since the declaring `mod paths;` in `commands/mod.rs` is private.
#[ derive( Debug, Clone, Copy, PartialEq, Eq ) ]
enum PathKey
{
  /// `~/.claude/settings.json`.
  Settings,
  /// Nearest ancestor project `.claude/settings.json`; `None` if not found.
  ProjectSettings,
  /// `~/.local/share/claude/versions`.
  VersionsDir,
  /// `~/.local/bin/claude`.
  BinarySymlink,
  /// `~/.claude/.transient/version_history_cache.json`.
  VersionHistoryCache,
}

impl PathKey
{
  const ALL : [ PathKey; 5 ] =
    [ Self::Settings, Self::ProjectSettings, Self::VersionsDir, Self::BinarySymlink, Self::VersionHistoryCache ];

  /// The `key::` string form of this variant.
  fn label( self ) -> &'static str
  {
    match self
    {
      Self::Settings => "settings",
      Self::ProjectSettings => "project_settings",
      Self::VersionsDir => "versions_dir",
      Self::BinarySymlink => "binary_symlink",
      Self::VersionHistoryCache => "version_history_cache",
    }
  }

  /// Parse a `key::` value; case-sensitive exact match against the 5 labels.
  fn parse( s : &str ) -> Result< Self, String >
  {
    match s
    {
      "settings" => Ok( Self::Settings ),
      "project_settings" => Ok( Self::ProjectSettings ),
      "versions_dir" => Ok( Self::VersionsDir ),
      "binary_symlink" => Ok( Self::BinarySymlink ),
      "version_history_cache" => Ok( Self::VersionHistoryCache ),
      other => Err( format!(
        "unknown key '{other}': expected one of settings, project_settings, versions_dir, binary_symlink, version_history_cache"
      ) ),
    }
  }
}

/// `.paths` — show-all or single-key filesystem path lookup.
///
/// Mode is determined by `key::`: absent shows all 5 known paths (labeled per
/// `v::`); present shows the single resolved path for that key.
///
/// # Errors
///
/// Returns `Err(ArgumentTypeMismatch)` for an unrecognised or empty `key::` value (exit 1).
/// Returns `Err(InternalError)` when `HOME` is unset or empty (exit 2).
#[ allow( clippy::missing_inline_in_public_items ) ]
pub fn paths_routine( cmd : VerifiedCommand, _ctx : ExecutionContext ) -> Result< OutputData, ErrorData >
{
  let key_opt = match cmd.arguments.get( "key" )
  {
    Some( Value::String( s ) ) if !s.is_empty() =>
      Some( PathKey::parse( s ).map_err( | e | ErrorData::new( ErrorCode::ArgumentTypeMismatch, e ) )? ),
    Some( Value::String( _ ) ) =>
      return Err( ErrorData::new( ErrorCode::ArgumentTypeMismatch, "key:: value cannot be empty".to_string() ) ),
    _ => None,
  };
  let opts  = OutputOptions::from_cmd( &cmd )?;
  let paths = ClaudeVersionPaths::new().ok_or_else( || ErrorData::new(
    ErrorCode::InternalError, "HOME environment variable is required".to_string() ) )?;
  let cwd   = std::env::current_dir().unwrap_or_default();

  let content = match key_opt
  {
    Some( k ) => render_single( &paths, k, &cwd, &opts ),
    None      => render_show_all( &paths, &cwd, &opts ),
  };
  Ok( OutputData::new( content, "text" ) )
}

// ── Path resolution ───────────────────────────────────────────────────────────

/// Resolve one key to its path string. `None` only when unresolvable
/// (only `ProjectSettings` can be — the other 4 always resolve).
fn resolve_key( paths : &ClaudeVersionPaths, key : PathKey, cwd : &Path ) -> Option< String >
{
  match key
  {
    PathKey::Settings            => Some( paths.settings_file().display().to_string() ),
    PathKey::ProjectSettings     => paths.project_settings_file( cwd ).map( | p | p.display().to_string() ),
    PathKey::VersionsDir         => Some( paths.versions_dir().display().to_string() ),
    PathKey::BinarySymlink       => Some( paths.binary_symlink().display().to_string() ),
    PathKey::VersionHistoryCache => Some( paths.version_history_cache_file().display().to_string() ),
  }
}

/// One-line description shown at `v::2`.
fn description( key : PathKey ) -> &'static str
{
  match key
  {
    PathKey::Settings            => "User-level settings; read directly and written by .settings.set/.config",
    PathKey::ProjectSettings     => "Nearest ancestor project settings.json; overrides user settings when present",
    PathKey::VersionsDir         => "Directory holding all installed Claude Code version binaries",
    PathKey::BinarySymlink       => "Hot-swap target; retargeted by .version.install to activate a version",
    PathKey::VersionHistoryCache => "Cached GitHub release history for .version.history",
  }
}

// ── Show-all rendering ────────────────────────────────────────────────────────

fn render_show_all( paths : &ClaudeVersionPaths, cwd : &Path, opts : &OutputOptions ) -> String
{
  match opts.format
  {
    OutputFormat::Json => render_show_all_json( paths, cwd ),
    OutputFormat::Text => render_show_all_text( paths, cwd, opts.verbosity ),
  }
}

fn render_show_all_text( paths : &ClaudeVersionPaths, cwd : &Path, verbosity : u8 ) -> String
{
  let mut out = String::new();
  for key in PathKey::ALL
  {
    let resolved = resolve_key( paths, key, cwd );
    if verbosity == 0
    {
      // Unresolved keys are omitted entirely at v::0 — no label to attach a placeholder to.
      if let Some( path ) = resolved
      {
        let _ = writeln!( out, "{path}" );
      }
    }
    else
    {
      let label   = format!( "{}:", key.label() );
      let display = resolved.unwrap_or_else( || "(none found)".to_string() );
      let _ = writeln!( out, "{label:<22}  {display}" );
      if verbosity >= 2
      {
        let _ = writeln!( out, "  {}", description( key ) );
      }
    }
  }
  out
}

fn render_show_all_json( paths : &ClaudeVersionPaths, cwd : &Path ) -> String
{
  let mut entries : Vec< String > = Vec::with_capacity( PathKey::ALL.len() );
  for key in PathKey::ALL
  {
    let val_json = match resolve_key( paths, key, cwd )
    {
      Some( p ) => format!( "\"{}\"", json_escape( &p ) ),
      None      => "null".to_string(),
    };
    entries.push( format!( "  \"{}\": {val_json}", key.label() ) );
  }
  format!( "{{\n{}\n}}\n", entries.join( ",\n" ) )
}

// ── Single-key rendering ──────────────────────────────────────────────────────

fn render_single( paths : &ClaudeVersionPaths, key : PathKey, cwd : &Path, opts : &OutputOptions ) -> String
{
  match opts.format
  {
    OutputFormat::Json => render_single_json( paths, key, cwd ),
    OutputFormat::Text => render_single_text( paths, key, cwd, opts.verbosity ),
  }
}

fn render_single_text( paths : &ClaudeVersionPaths, key : PathKey, cwd : &Path, verbosity : u8 ) -> String
{
  let display = resolve_key( paths, key, cwd ).unwrap_or_else( || "(none found)".to_string() );

  if verbosity >= 2
  {
    let mut out = String::new();
    let _ = writeln!( out, "{}:  {display}", key.label() );
    let _ = writeln!( out, "  {}", description( key ) );
    out
  }
  else
  {
    format!( "{display}\n" )
  }
}

fn render_single_json( paths : &ClaudeVersionPaths, key : PathKey, cwd : &Path ) -> String
{
  let val_json = match resolve_key( paths, key, cwd )
  {
    Some( p ) => format!( "\"{}\"", json_escape( &p ) ),
    None      => "null".to_string(),
  };
  format!( "{{\"{}\": {val_json}}}\n", key.label() )
}

// ── Tests ──────────────────────────────────────────────────────────────────────

#[ cfg( test ) ]
mod path_key_tests
{
  use super::PathKey;

  /// Guards against drift between `label()` and `parse()` — the two functions
  /// independently encode the same 5-pair mapping in opposite directions, and
  /// exhaustiveness checking only protects `label()`'s match (`parse()`'s catch-all
  /// `other =>` arm compiles even if a new variant's string mapping is never added).
  #[ test ]
  fn label_parse_round_trip()
  {
    for key in PathKey::ALL
    {
      assert_eq!( PathKey::parse( key.label() ), Ok( key ) );
    }
  }
}
