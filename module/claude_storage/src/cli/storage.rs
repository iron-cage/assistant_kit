//! Shared storage utilities: factory, validation, addressing helpers.
//!
//! Collected here because every command routine depends on at least one of
//! these functions, so a single import site per module (`use super::storage::*`)
//! is cleaner than scattering them across individual command files.

use unilang::{ ErrorData, ErrorCode };
use claude_storage_core::Storage;

// ─── constants ─────────────────────────────────────────────────────────────

/// Maximum accepted verbosity level (inclusive).
pub( super ) const VERBOSITY_MAX : i64 = 5;

// ─── storage factory ───────────────────────────────────────────────────────

/// Create a `Storage` instance, respecting `CLAUDE_STORAGE_ROOT` env var.
///
/// Precedence: `CLAUDE_STORAGE_ROOT` env var > `~/.claude/` default.
/// An empty env var falls back to the default. Does not override an explicit
/// `path::` command parameter — callers handling `path::` must use
/// `Storage::with_root()` directly (see `status_routine`).
///
/// # Errors
///
/// Returns an `ErrorData` if storage creation fails (e.g., path does not exist).
pub( super ) fn create_storage() -> core::result::Result< Storage, ErrorData >
{
  match std::env::var( "CLAUDE_STORAGE_ROOT" )
  {
    Ok( root ) if !root.is_empty() =>
      Ok( Storage::with_root( std::path::Path::new( &root ) ) ),
    _ =>
      Storage::new()
        .map_err( | e | ErrorData::new( ErrorCode::InternalError, format!( "Failed to create storage: {e}" ) ) ),
  }
}

// ─── parameter validation ──────────────────────────────────────────────────

/// Validate that `verbosity` is within `0..=VERBOSITY_MAX`.
///
/// # Errors
///
/// Returns `ErrorData` when `verbosity` is outside the valid range.
pub( super ) fn validate_verbosity( verbosity : i64 ) -> core::result::Result< (), ErrorData >
{
  if ( 0..=VERBOSITY_MAX ).contains( &verbosity )
  {
    Ok( () )
  }
  else
  {
    Err( ErrorData::new(
      ErrorCode::InternalError,
      format!( "Invalid verbosity: {verbosity}. Valid range: 0-{VERBOSITY_MAX}" ),
    ) )
  }
}

// ─── path resolution ───────────────────────────────────────────────────────

/// Resolve path parameter with smart detection
///
/// ## Behavior
///
/// - `.` → Current working directory (absolute)
/// - `..` → Parent directory (absolute)
/// - `~` → Home directory (absolute)
/// - `~/path` → Home + path (absolute)
/// - `/abs/path` → Use as-is (already absolute)
/// - `rel/path` → Resolve to absolute
/// - `pattern` → Use as-is (no path separators = pattern)
///
/// ## Examples
///
/// ```text
/// // Current dir: /home/user/project
/// resolve_path_parameter(".") → "/home/user/project"
/// resolve_path_parameter("..") → "/home/user"
/// resolve_path_parameter("~") → "/home/user"
/// resolve_path_parameter("myproject") → "myproject" (unchanged)
/// ```
pub( super ) fn resolve_path_parameter( param : &str ) -> core::result::Result< String, String >
{
  use std::path::Path;

  match param
  {
    // Current directory
    "." =>
    {
      std::env::current_dir()
        .map( | p | p.to_string_lossy().to_string() )
        .map_err( | e | format!( "Failed to get current directory: {e}" ) )
    },

    // Parent directory
    ".." =>
    {
      let current = std::env::current_dir()
        .map_err( | e | format!( "Failed to get current directory: {e}" ) )?;

      let parent = current.parent()
        .ok_or_else( || "Current directory has no parent".to_string() )?;

      Ok( parent.to_string_lossy().to_string() )
    },

    // Home directory or home + relative path
    s if s.starts_with( '~' ) =>
    {
      let home = std::env::var( "HOME" )
        .map_err( | e | format!( "Failed to get HOME directory: {e}" ) )?;

      if s.len() == 1
      {
        // Just "~"
        Ok( home )
      }
      else if let Some( stripped ) = s.strip_prefix( "~/" )
      {
        // "~/path"
        let path = Path::new( &home ).join( stripped );
        Ok( path.to_string_lossy().to_string() )
      }
      else
      {
        // "~user" not supported, use as-is
        Ok( s.to_string() )
      }
    },

    // Absolute path - use as-is
    s if s.starts_with( '/' ) =>
    {
      Ok( s.to_string() )
    },

    // Relative path with separators - resolve to absolute
    s if s.contains( '/' ) =>
    {
      let current = std::env::current_dir()
        .map_err( | e | format!( "Failed to get current directory: {e}" ) )?;

      let resolved = current.join( s );
      Ok( resolved.to_string_lossy().to_string() )
    },

    // Pattern (no path separators) - use as-is
    s =>
    {
      Ok( s.to_string() )
    },
  }
}

// ─── project addressing ────────────────────────────────────────────────────

/// Parse project parameter into `ProjectId`
///
/// Supports multiple formats:
/// - Absolute path: `/home/user/project` → `ProjectId::Path`
/// - Path-encoded: `-home-user-project` → `ProjectId::Path` (decoded)
/// - UUID: `abc-123-def` → `ProjectId::Uuid`
/// - Debug format: `Path("/home/user/project")` → `ProjectId::Path`
///
/// # Fix
///
/// Fix(issue-project-param-2025-11-30): Project parameter always treated as UUID
///
/// # Root Cause
///
/// Code at line 239 hardcoded `ProjectId::uuid(proj_id)` without checking
/// if the parameter was a path. This caused all path projects to fail with
/// "Project not found" error.
///
/// # Pitfall
///
/// When accepting string parameters that could have multiple formats, always
/// implement smart detection logic. Hardcoding assumptions about parameter
/// format leads to silent failures for valid inputs.
///
/// # Errors
///
/// Returns error if a path-encoded input cannot be decoded, or if the current
/// directory cannot be resolved for relative path inputs.
#[inline]
pub fn parse_project_parameter( input : &str )
  -> core::result::Result< claude_storage_core::ProjectId, String >
{
  use claude_storage_core::{ ProjectId, decode_path };
  use std::path::PathBuf;

  // [1] Check for Debug format from .list output
  if let Some( path_str ) = input.strip_prefix( "Path(\"" ).and_then( | s | s.strip_suffix( "\")" ) )
  {
    return Ok( ProjectId::path( path_str ) );
  }

  // [2] Check for absolute path (cross-platform)
  let path = PathBuf::from( input );
  if path.is_absolute()
  {
    return Ok( ProjectId::path( input ) );
  }

  // [3] Check for path-encoded
  if input.starts_with( '-' )
  {
    match decode_path( input )
    {
      Ok( decoded ) => return Ok( ProjectId::path( decoded ) ),
      Err( e ) => return Err( format!( "Failed to decode path: {e}" ) ),
    }
  }

  // Fix(issue-013): Handle relative paths (".", "..", "~", "./foo", "../bar")
  //
  // Root cause: Only checked for absolute paths and path-encoded, missing relative
  // path conventions that users commonly use for directory references.
  //
  // Pitfall: Assuming only absolute paths need path treatment; users commonly use
  // "." for CWD, ".." for parent, "~" for home in shell contexts.

  // [4] Check for home directory expansion (~)
  if input == "~" || input.starts_with( "~/" )
  {
    let home = std::env::var( "HOME" )
      .map_err( | _ | "HOME environment variable not set".to_string() )?;
    let expanded = if input == "~"
    {
      home
    }
    else
    {
      format!( "{}{}", home, &input[ 1.. ] )
    };
    return Ok( ProjectId::path( expanded ) );
  }

  // [5] Check for relative path markers (".", "..", "./", "../")
  if input == "." || input == ".." ||
     input.starts_with( "./" ) || input.starts_with( "../" )
  {
    // Get current working directory and resolve relative path
    let cwd = std::env::current_dir()
      .map_err( | e | format!( "Failed to get current directory: {e}" ) )?;
    let path = cwd.join( input );

    // For "." and "..", try to canonicalize (they should exist)
    // For "./foo" patterns, normalize without requiring existence
    if input == "." || input == ".."
    {
      match path.canonicalize()
      {
        Ok( abs_path ) => return Ok( ProjectId::path( abs_path.to_string_lossy().to_string() ) ),
        Err( e ) => return Err( format!( "Failed to resolve path '{input}': {e}" ) ),
      }
    }

    // For "./foo" or "../bar" - normalize path components without canonicalize
    // This handles non-existent paths correctly
    use std::path::Component;
    let mut normalized = PathBuf::new();
    for component in path.components()
    {
      match component
      {
        Component::ParentDir =>
        {
          normalized.pop();
        }
        Component::CurDir =>
        {
          // Skip "." components
        }
        _ => normalized.push( component ),
      }
    }
    return Ok( ProjectId::path( normalized.to_string_lossy().to_string() ) );
  }

  // [6] Default: UUID
  Ok( ProjectId::uuid( input ) )
}

/// Parse a raw project ID string and load the corresponding project.
///
/// Combines `parse_project_parameter` + `storage.load_project` into a single
/// call, eliminating the repeated two-step pattern at every command site that
/// accepts a `project::` parameter.
///
/// # Errors
///
/// Returns `ErrorData` when parsing fails or the project cannot be loaded.
pub( super ) fn load_project_for_param(
  storage  : &Storage,
  proj_id  : &str,
) -> core::result::Result< claude_storage_core::Project, ErrorData >
{
  let id = parse_project_parameter( proj_id )
    .map_err( | e | ErrorData::new( ErrorCode::InternalError, e ) )?;
  storage.load_project( &id )
    .map_err( | e | ErrorData::new( ErrorCode::InternalError, format!( "Failed to load project: {e}" ) ) )
}

/// Find a mutable session reference by exact ID or UUID prefix.
///
/// Checks `s.id() == session_id || s.id().starts_with(session_id)` so both
/// full UUIDs and 8-character prefixes resolve correctly.
///
/// # Errors
///
/// Returns `ErrorData` when no session matches.
pub( super ) fn find_session_mut<'a>(
  sessions   : &'a mut [ claude_storage_core::Session ],
  session_id : &str,
) -> core::result::Result< &'a mut claude_storage_core::Session, ErrorData >
{
  sessions.iter_mut()
    .find( | s | s.id() == session_id || s.id().starts_with( session_id ) )
    .ok_or_else( || ErrorData::new( ErrorCode::InternalError, format!( "Session not found: {session_id}" ) ) )
}
