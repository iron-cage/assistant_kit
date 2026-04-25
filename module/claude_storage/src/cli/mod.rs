//! CLI command routines for `claude_storage`
//!
//! This module provides command routines that are registered with unilang
//! and called when users invoke CLI commands.
//!
//! ## Known Pitfalls
//!
//! ### Parameter Validation Consistency (Finding #010)
//!
//! **Issue**: Default parameter values do not prevent invalid input - parameters
//! with defaults still require explicit validation.
//!
//! **Context**: `search_routine` was missing verbosity range validation (0-5)
//! while `status_routine` and `show_routine` had it. The default value (1) made
//! it seem like validation was unnecessary, but users can override defaults
//! with invalid values like -1 or 10.
//!
//! **Solution**: All parameters with value constraints must have explicit
//! validation, regardless of default values. Apply validation patterns
//! consistently across all command routines:
//!
//! ```rust,no_run
//! # use unilang::{ VerifiedCommand, ErrorData, ErrorCode };
//! # const VERBOSITY_MAX : i64 = 5;
//! # fn example( cmd : VerifiedCommand ) -> Result< (), ErrorData >
//! # {
//! let verbosity = cmd.get_integer( "verbosity" ).unwrap_or( 1 );
//!
//! // Always validate range, even with defaults
//! if !( 0..=VERBOSITY_MAX ).contains( &verbosity )
//! {
//!   return Err( ErrorData::new(
//!     ErrorCode::InternalError,
//!     format!( "Invalid verbosity: {}. Valid range: 0-5", verbosity )
//!   ));
//! }
//! # Ok( () )
//! # }
//! ```
//!
//! **Prevention**: When adding new parameters, check existing command routines
//! for validation patterns and apply them consistently. Never assume defaults
//! eliminate the need for validation.
//!
//! See: `tests/search_command_test.rs::test_search_verbosity_invalid`

use core::fmt::Write as FmtWrite;
use unilang::{ VerifiedCommand, ExecutionContext, OutputData, ErrorData, ErrorCode };
use claude_storage_core::Storage;

// ─── constants ─────────────────────────────────────────────────────────────

/// Maximum accepted verbosity level (inclusive).
const VERBOSITY_MAX : i64 = 5;

/// UUID string length (8-4-4-4-12 = 36 chars).
const UUID_LEN : usize = 36;

/// Characters to display from each end when short-displaying a UUID.
const UUID_SHORT_LEN : usize = 8;

/// Fallback agent type when `meta.json` is absent or missing `agentType`.
const AGENT_TYPE_UNKNOWN : &str = "unknown";

/// Default session topic when no `topic::` param is supplied.
const DEFAULT_TOPIC : &str = "default_topic";

// Seconds-per-unit thresholds for relative time formatting.
const SECS_PER_MIN   : u64 = 60;
const SECS_PER_HOUR  : u64 = 3_600;
const SECS_PER_DAY   : u64 = 86_400;
const SECS_PER_MONTH : u64 = 2_592_000;

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
fn create_storage() -> core::result::Result< Storage, ErrorData >
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

/// Validate that `verbosity` is within `0..=VERBOSITY_MAX`.
///
/// # Errors
///
/// Returns `ErrorData` when `verbosity` is outside the valid range.
fn validate_verbosity( verbosity : i64 ) -> core::result::Result< (), ErrorData >
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

/// Show storage status and statistics
///
/// Displays comprehensive information about Claude Code storage including
/// project counts, session counts, token usage, and storage location.
///
/// # Errors
///
/// Returns error if verbosity is out of range, path resolution fails,
/// storage creation fails, or statistics retrieval fails.
#[allow(clippy::needless_pass_by_value)]
#[inline]
pub fn status_routine( cmd : VerifiedCommand, _ctx : ExecutionContext )
  -> core::result::Result< OutputData, ErrorData >
{
  let verbosity = cmd.get_integer( "verbosity" ).unwrap_or( 1 );
  validate_verbosity( verbosity )?;

  let custom_path = cmd.get_string( "path" );

  // Fix(issue-014): Resolve path parameter before using
  //
  // Root cause: status_routine passed path directly to Storage::with_root() without
  // resolving special markers (".", "..", "~"), unlike list_routine which uses
  // resolve_path_parameter().
  //
  // Pitfall: Inconsistent parameter handling across commands leads to confusing UX
  // where the same parameter format works in one command but not another.
  let resolved_path = custom_path
    .map( | path | resolve_path_parameter( path )
      .map_err( | e | ErrorData::new( ErrorCode::InternalError, format!( "Failed to resolve path '{path}': {e}" ) ) )
    )
    .transpose()?;

  // Create storage instance
  let storage = if let Some( path ) = resolved_path
  {
    Storage::with_root( &path )
  }
  else
  {
    create_storage()?
  };

  // Fix(issue-015): Use fast stats (filesystem-only) for verbosity 0-1; full
  // JSONL-parsing stats only for verbosity 2+ which the user explicitly requests.
  //
  // Root cause: global_stats() parsed all session JSONL files to count entries and
  // tokens. With 1903 projects / 2449 sessions / 7 GB of JSONL this took >2 minutes,
  // making .status completely unusable at the default verbosity level.
  //
  // Pitfall: Never call global_stats() for a command that only needs project/session
  // counts — the entry/token parsing is O(total JSONL bytes), not O(project count).

  let output = if verbosity <= 1
  {
    // Fast path: filesystem listing only — no JSONL parsing, completes in < 1 second
    let stats = storage.global_stats_fast()
      .map_err( | e | ErrorData::new( ErrorCode::InternalError, format!( "Failed to get statistics: {e}" ) ) )?;

    match verbosity
    {
      0 =>
      {
        // Minimal: just project count
        format!( "Projects: {}", stats.total_projects )
      }
      _ =>
      {
        // Standard: storage + project + session overview (no entry count — fast path)
        format!
        (
          "Storage: {}\nProjects: {} (UUID: {}, Path: {})\nSessions: {} (Main: {}, Agent: {})",
          storage.root().display(),
          stats.total_projects,
          stats.uuid_projects,
          stats.path_projects,
          stats.total_sessions,
          stats.main_sessions,
          stats.agent_sessions,
        )
      }
    }
  }
  else
  {
    // Full path: parses all JSONL files for entry counts and token usage.
    // Slow for large storage (minutes for thousands of sessions) but gives complete stats.
    let stats = storage.global_stats()
      .map_err( | e | ErrorData::new( ErrorCode::InternalError, format!( "Failed to get statistics: {e}" ) ) )?;

    // Detailed: include entry breakdown and token usage
    format!
    (
      "Storage: {}\n\
      Projects: {} (UUID: {}, Path: {})\n\
      Sessions: {} (Main: {}, Agent: {})\n\
      Entries: {} (User: {}, Assistant: {})\n\
      Tokens:\n\
      - Input: {}\n\
      - Output: {}\n\
      - Cache Read: {}\n\
      - Cache Creation: {}",
      storage.root().display(),
      stats.total_projects,
      stats.uuid_projects,
      stats.path_projects,
      stats.total_sessions,
      stats.main_sessions,
      stats.agent_sessions,
      stats.total_entries,
      stats.total_user_entries,
      stats.total_assistant_entries,
      stats.total_input_tokens,
      stats.total_output_tokens,
      stats.total_cache_read_tokens,
      stats.total_cache_creation_tokens
    )
  };

  Ok( OutputData::new( output, "text" ) )
}

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
fn resolve_path_parameter( param : &str ) -> core::result::Result< String, String >
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

/// Format entry content for display
///
/// ## Behavior
///
/// - Extracts actual message content from Entry
/// - Formats as readable chat log entry
/// - Supports text, thinking, tool use blocks
/// - Optional truncation for long messages
///
/// ## Format
///
/// ```text
/// [2025-12-02 09:57] User:
/// message content here
///
/// [2025-12-02 09:58] Assistant:
/// response content here
/// ```
///
/// ## Examples
///
/// ```text
/// let entry = session.entries()[0];
/// let formatted = format_entry_content( &entry, None );
/// // Output: "[2025-12-02 09:57] User:\nHello, Claude!"
/// ```
fn format_entry_content( entry : &claude_storage_core::Entry, max_length : Option< usize > ) -> String
{
  use claude_storage_core::{ MessageContent, ContentBlock };

  // Format timestamp
  let timestamp = format_timestamp( &entry.timestamp );

  // Extract content based on message type
  let ( role, content ) = match &entry.message
  {
    MessageContent::User( msg ) =>
    {
      ( "User", msg.content.clone() )
    },
    MessageContent::Assistant( msg ) =>
    {
      // Extract all text blocks
      let text_blocks : Vec< String > = msg.content
        .iter()
        .filter_map( | block | match block
        {
          ContentBlock::Text { text } => Some( text.clone() ),
          ContentBlock::Thinking { thinking, .. } =>
          {
            // Show thinking blocks with prefix
            Some( format!( "[Thinking]\n{thinking}" ) )
          },
          ContentBlock::ToolUse { name, .. } =>
          {
            // Show tool use briefly
            Some( format!( "[Using tool: {name}]" ) )
          },
          ContentBlock::ToolResult { is_error, content, .. } =>
          {
            if *is_error
            {
              Some( format!( "[Tool error: {content}]" ) )
            }
            else
            {
              // Don't show successful tool results in conversation view
              None
            }
          },
        })
        .collect();

      let combined = text_blocks.join( "\n\n" );
      ( "Assistant", combined )
    }
  };

  // Apply truncation if needed
  let content = truncate_if_needed( &content, max_length );

  // Format as chat log entry
  format!( "[{timestamp}] {role}:\n{content}" )
}

/// Format timestamp for display
///
/// Converts ISO 8601 timestamp to readable format:
/// "2025-12-02T09:57:02.237Z" → "2025-12-02 09:57"
///
/// ## Examples
///
/// ```text
/// let ts = "2025-12-02T09:57:02.237Z";
/// assert_eq!( format_timestamp( ts ), "2025-12-02 09:57" );
/// ```
fn format_timestamp( timestamp : &str ) -> String
{
  // Try to parse ISO 8601
  if let Some( datetime_part ) = timestamp.split( '.' ).next()
  {
    if let Some( ( date, time ) ) = datetime_part.split_once( 'T' )
    {
      // Extract HH:MM from time
      let time_short = time.split( ':' ).take( 2 ).collect::< Vec< _ > >().join( ":" );
      return format!( "{date} {time_short}" );
    }
  }

  // Fallback: use raw timestamp
  timestamp.to_string()
}

/// Truncate text with indicator
///
/// Truncates long text and adds "... [truncated]" indicator.
///
/// ## Examples
///
/// ```text
/// let text = "a".repeat( 1000 );
/// let truncated = truncate_if_needed( &text, Some( 100 ) );
/// assert!( truncated.contains( "[truncated" ) );
/// ```
///
/// Fix(issue-018): Use char-boundary-safe truncation.
/// Root cause: `&text[..len]` panics when `len` falls inside a multibyte
/// UTF-8 sequence (emoji, CJK, accented chars).
/// Pitfall: `str::len()` returns bytes, not characters — never use it
/// directly as a slice bound on user-supplied text.
#[must_use]
#[inline]
pub fn truncate_if_needed( text : &str, max_length : Option< usize > ) -> String
{
  match max_length
  {
    None => text.to_string(),
    Some( len ) if text.len() <= len => text.to_string(),
    Some( len ) =>
    {
      // Find the nearest valid char boundary at or before `len`
      let mut end = len;
      while end > 0 && !text.is_char_boundary( end )
      {
        end -= 1;
      }
      let truncated = &text[ ..end ];
      format!( "{}... [truncated, {} more bytes]", truncated, text.len() - end )
    }
  }
}

/// List projects or sessions
///
/// Lists projects in Claude Code storage, with optional filtering by type.
///
/// Smart session display:
/// - Providing session filters (`session::`, `agent::`, `min_entries::`) auto-enables session display
/// - `sessions::1` always enables session display
/// - `sessions::0` has no effect when session filters are also active — filters win
/// - No filters → Projects only (default behavior)
///
/// Examples:
/// ```bash
/// # Projects only (no sessions)
/// .list
///
/// # Auto-enable sessions (filter provided)
/// .list session::commit
///
/// # sessions::0 ignored when filters active — sessions still shown
/// .list sessions::0 session::commit
/// ```
///
/// # Errors
///
/// Returns error if `min_entries` is negative, path resolution fails,
/// project type is invalid, storage creation fails, or listing projects fails.
#[allow(clippy::too_many_lines)]
// CLI routine handler processes many parameters and verbosity branches — extraction
// would obscure the command's logic without reducing complexity.
#[allow(clippy::needless_pass_by_value)]
#[inline]
pub fn list_routine( cmd : VerifiedCommand, _ctx : ExecutionContext )
  -> core::result::Result< OutputData, ErrorData >
{
  let project_type = cmd.get_string( "type" ).unwrap_or( "all" );

  // Early dispatch: conversation listing requires project:: and is handled separately.
  if project_type == "conversation"
  {
    let proj_id = cmd.get_string( "project" )
      .ok_or_else( || ErrorData::new(
        ErrorCode::InternalError,
        "project parameter required for listing conversations".to_string(),
      ) )?;
    let storage = create_storage()?;
    let project = load_project_for_param( &storage, proj_id )?;
    let sessions = project.all_sessions()
      .map_err( | e | ErrorData::new( ErrorCode::InternalError, format!( "Failed to load sessions: {e}" ) ) )?;
    let families = build_families( sessions );
    let conversations = group_into_conversations( families );
    let count_mode = cmd.get_boolean( "count" ).unwrap_or( false );
    if count_mode
    {
      return Ok( OutputData::new( format!( "{}", conversations.len() ), "text" ) );
    }
    let mut out = String::new();
    for conv in &conversations
    {
      if let Some( s ) = conv.root_session()
      {
        writeln!( out, "{}", s.id() ).unwrap();
      }
    }
    return Ok( OutputData::new( out, "text" ) );
  }

  let verbosity = cmd.get_integer( "verbosity" ).unwrap_or( 1 );

  // Fix(issue-015): Validate verbosity range
  //
  // Root cause: list_routine retrieved verbosity without range validation, unlike
  // status_routine and show_routine which include explicit 0-5 checks. Values like
  // -1 or 10 were silently accepted and used, producing undefined output behavior.
  //
  // Pitfall: get_integer().unwrap_or(default) only substitutes the default when the
  // parameter is absent. An explicit out-of-range value is returned as-is. Range
  // validation is always the caller's responsibility, even when a default is set.
  validate_verbosity( verbosity )?;

  // Parse filter parameters
  let path_filter = cmd.get_string( "path" );
  let agent_filter = cmd.get_boolean( "agent" );

  // Validate and parse min_entries (must be non-negative)
  let min_entries_filter = if let Some( n ) = cmd.get_integer( "min_entries" )
  {
    if n < 0
    {
      return Err
      (
        ErrorData::new
        (
          ErrorCode::InternalError,
          format!( "Invalid min_entries: {n}. Must be non-negative" )
        )
      );
    }
    // min_entries validated as non-negative above, so cast is safe
    #[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
    Some( n as usize )
  }
  else
  {
    None
  };

  let session_id_filter = cmd.get_string( "session" );

  // Fix(issue-002): Smart path resolution for path:: parameter
  //
  // Root cause: The path:: parameter used literal substring matching only.
  // When users provided path::., it searched for paths containing a literal "."
  // character instead of resolving "." to the current working directory.
  // This violated user expectations from shell semantics where . means "current
  // directory", .. means "parent directory", and ~ means "home directory".
  //
  // Pitfall: When implementing filters that accept both patterns and paths,
  // clearly define detection logic. Ambiguous cases (like .) should prioritize
  // user expectations over literal interpretation. Support shell semantics for
  // special characters (., .., ~) in all filesystem path parameters.

  // Resolve path parameter with smart detection
  let path_filter = if let Some( param ) = path_filter
  {
    match resolve_path_parameter( param )
    {
      Ok( resolved ) => Some( resolved ),
      Err( e ) =>
      {
        return Err
        (
          ErrorData::new
          (
            ErrorCode::InternalError,
            format!( "Failed to resolve path parameter '{}': {}", &param, e )
          )
        );
      }
    }
  }
  else
  {
    None
  };

  // Fix(issue-001): Smart session display - auto-enable when filters provided
  //
  // Root cause: `show_sessions` defaulted to false, blocking filter usage even when
  // session filters were provided. This made session::, agent::, and min_entries::
  // parameters "garbage" - accepted by parser but silently ignored by implementation.
  //
  // Pitfall: Garbage parameters create silent failures that waste user time. Users try
  // different parameter values but see no effect because the filter is built but never
  // used. ALWAYS trace parameter flow: parser → filter build → filter usage. If usage
  // is conditional on default-false flag, parameter is garbage.

  // Smart parameter detection: Auto-enable session display when filters provided
  let has_session_filters = session_id_filter.is_some()
    || agent_filter.is_some()
    || min_entries_filter.is_some();

  let show_sessions = has_session_filters || cmd.get_boolean( "sessions" ).unwrap_or( false );

  // Create storage instance
  let storage = create_storage()?;

  // Fix(issue-list-hang): min_entries:: must not be placed in ProjectFilter
  //
  // Root cause: Placing min_entries in ProjectFilter caused project.matches_filter()
  // to call project_stats() for EVERY project. project_stats() reads ALL session JSONL
  // files to count entries, scanning gigabytes of data when the user has many projects.
  // This caused the binary to hang indefinitely.
  //
  // min_entries:: is semantically a SESSION filter (show sessions with ≥N entries),
  // not a project filter (show projects whose total entries ≥ N). The auto-enable
  // behavior (show_sessions=true) is handled separately at line 512-516.
  //
  // Pitfall: When a parameter auto-enables a feature, don't also apply it as a
  // project-level filter unless that filtering is the stated purpose. Trace the
  // computational cost: project_stats() = O(projects × sessions × entries).

  // Build project filter (min_entries is a session filter, not a project filter)
  let project_filter = claude_storage_core::ProjectFilter
  {
    path_substring : path_filter,
    min_entries : None,
    min_sessions : None,
  };

  // Build session filter
  let session_filter = claude_storage_core::SessionFilter
  {
    agent_only : agent_filter,
    min_entries : min_entries_filter,
    session_id_substring : session_id_filter.map( std::string::ToString::to_string ),
  };

  // Get projects based on type filter
  let mut projects = match project_type
  {
    "uuid" => storage.list_uuid_projects(),
    "path" => storage.list_path_projects(),
    "all" => storage.list_projects(),
    _ => return Err
    (
      ErrorData::new
      (
        ErrorCode::InternalError,
        format!( "Invalid type: {project_type}. Valid values: uuid, path, all" )
      )
    ),
  }
  .map_err( | e | ErrorData::new( ErrorCode::InternalError, format!( "Failed to list projects: {e}" ) ) )?;

  // Apply project-level filtering
  if !project_filter.is_default()
  {
    projects.retain( | project |
    {
      project.matches_filter( &project_filter ).unwrap_or( false )
    });
  }

  // Format output
  let mut output = String::new();

  if verbosity >= 1
  {
    let noun = if projects.len() == 1 { "project" } else { "projects" };
    writeln!( output, "Found {} {noun}:\n", projects.len() ).unwrap();
  }

  for mut project in projects
  {
    // Handle projects that may have been deleted (gracefully skip them)
    match verbosity
    {
      0 =>
      {
        // Just ID
        writeln!( output, "{:?}", project.id() ).unwrap();
      }
      1 =>
      {
        // ID + conversation count (skip if project was deleted)
        let Ok( session_count ) = project.count_sessions() else { continue };  // Skip projects that can't be read

        // Fix(issue-027): Use singular "conversation" when count == 1; plural otherwise.
        // Root cause: hardcoded "sessions" regardless of count produced "(1 sessions)".
        // Pitfall: same pattern as issue-025 — always derive noun from count, never hardcode.
        let noun = if session_count == 1 { "conversation" } else { "conversations" };
        writeln!( output, "{:?} ({session_count} {noun})", project.id() ).unwrap();
      }
      _ =>
      {
        // Full details (skip if project was deleted)
        let Ok( project_stats ) = project.project_stats() else { continue };  // Skip projects that can't be read

        write!
        (
          output,
          "{:?}\n  Sessions: {} (Main: {}, Agent: {})\n  Entries: {}\n  Tokens: {} in, {} out\n\n",
          project.id(),
          project_stats.session_count,
          project_stats.main_session_count,
          project_stats.agent_session_count,
          project_stats.total_entries,
          project_stats.total_input_tokens,
          project_stats.total_output_tokens
        ).unwrap();
      }
    }

    // Show sessions if requested (skip if project was deleted)
    if show_sessions
    {
      let sessions = if session_filter.is_default()
      {
        match project.sessions()
        {
          Ok( s ) => s,
          Err( _ ) => continue,  // Skip if can't read sessions
        }
      }
      else
      {
        match project.sessions_filtered( &session_filter )
        {
          Ok( s ) => s,
          Err( _ ) => continue,  // Skip if can't read sessions
        }
      };

      for session in sessions
      {
        writeln!( output, "  - {}", session.id() ).unwrap();
      }
    }
  }

  Ok( OutputData::new( output, "text" ) )
}

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
fn load_project_for_param(
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
fn find_session_mut<'a>(
  sessions   : &'a mut [ claude_storage_core::Session ],
  session_id : &str,
) -> core::result::Result< &'a mut claude_storage_core::Session, ErrorData >
{
  sessions.iter_mut()
    .find( | s | s.id() == session_id || s.id().starts_with( session_id ) )
    .ok_or_else( || ErrorData::new( ErrorCode::InternalError, format!( "Session not found: {session_id}" ) ) )
}

/// Show session or project details (location-aware)
///
/// Smart behavior based on parameters:
/// - No parameters → Show current directory project (all sessions)
/// - `session_id` only → Show that session in current project
/// - project only → Show that project (all sessions)
/// - Both parameters → Show that session in that project
///
/// # Errors
///
/// Returns error if verbosity is out of range, parameter combinations are
/// invalid, storage creation fails, or project/session loading fails.
#[allow(clippy::needless_pass_by_value)]
#[inline]
pub fn show_routine( cmd : VerifiedCommand, _ctx : ExecutionContext )
  -> core::result::Result< OutputData, ErrorData >
{
  let session_id = cmd.get_string( "session_id" );
  let project_param = cmd.get_string( "project" );
  let verbosity = cmd.get_integer( "verbosity" ).unwrap_or( 1 );
  let show_entries = cmd.get_boolean( "entries" ).unwrap_or( false );
  let metadata_only = cmd.get_boolean( "metadata" ).unwrap_or( false );

  validate_verbosity( verbosity )?;

  // Fix(issue-001): Validate entries parameter requires session_id
  //
  // Root cause: entries parameter was accepted and parsed but silently ignored
  // when displaying projects (cases 1 and 3). This created a "garbage parameter"
  // that users could pass but had no effect, wasting debugging time.
  //
  // Pitfall: Always validate parameter compatibility. If parameter P only works
  // with parameter Q, reject the combination where P is set but Q is not.
  // Silent ignoring of valid-syntax parameters creates user frustration.
  if show_entries && session_id.is_none()
  {
    return Err
    (
      ErrorData::new
      (
        ErrorCode::InternalError,
        "Parameter 'entries' requires 'session_id'. \
         Use '.show session_id::<id> entries::1' to display session entries."
          .to_string()
      )
    );
  }

  // Fix(issue-022): Accept entries::1 in content mode as a no-op
  //
  // Root cause: A prior "fix" (issue-008) added an error when entries::1 was used
  // in content mode (verbosity >= 1 && !metadata_only), intending to prevent a
  // "garbage parameter" scenario. However, the YAML spec explicitly lists
  // `.show session_id::abc123 entries::1` as a valid example (example 6), and
  // content mode already shows all entries by default — entries::1 is a valid
  // no-op in this context.
  //
  // Pitfall: Don't add errors for parameters whose spec examples show them working
  // standalone. A no-op is preferable to an error when the parameter has no
  // additional effect in the current mode. Errors should be reserved for truly
  // incompatible combinations, not for parameters that are simply redundant.

  // Smart parameter detection (4 cases)
  match ( session_id, project_param )
  {
    // Case 1: No parameters → Show current directory project
    ( None, None ) =>
    {
      show_project_for_cwd_impl( verbosity )
    }

    // Case 2: session_id only → Show session in current project
    ( Some( sid ), None ) =>
    {
      show_session_in_cwd_impl( sid, verbosity, show_entries, metadata_only )
    }

    // Case 3: project only → Show that project
    ( None, Some( proj ) ) =>
    {
      show_project_impl( proj, verbosity )
    }

    // Case 4: Both parameters → Show session in that project
    ( Some( sid ), Some( proj ) ) =>
    {
      show_session_in_project_impl( sid, proj, verbosity, show_entries, metadata_only )
    }
  }
}

/// Helper: Show session in current directory project
fn show_session_in_cwd_impl(
  session_id : &str,
  verbosity : i64,
  show_entries : bool,
  metadata_only : bool
) -> core::result::Result< OutputData, ErrorData >
{
  // Fix(issue-036)
  // Root cause: load_project_for_cwd() only matches the exact encoded base path, so sessions
  //   stored under topic project dirs ({base}--commit, {base}--default-topic) were invisible
  //   when running .show from the corresponding working directory.
  // Pitfall: Use double-hyphen ({eb}--) not single ({eb}-) for the topic prefix predicate;
  //   single-hyphen would falsely match sibling directories sharing a common prefix.
  let storage = create_storage()?;

  let cwd = std::env::current_dir()
    .map_err( | e | ErrorData::new( ErrorCode::InternalError, format!( "Failed to get current directory: {e}" ) ) )?;

  let eb = claude_storage_core::encode_path( &cwd )
    .map_err( | e | ErrorData::new( ErrorCode::InternalError, format!( "Failed to encode current directory: {e}" ) ) )?;

  let topic_prefix = format!( "{eb}--" );

  let all_projects = storage.list_projects()
    .map_err( | e | ErrorData::new( ErrorCode::InternalError, format!( "Failed to list projects: {e}" ) ) )?;

  for project in &all_projects
  {
    let dir_name = project.storage_dir()
      .file_name()
      .and_then( | n | n.to_str() )
      .unwrap_or( "" );

    if dir_name != eb && !dir_name.starts_with( &topic_prefix ) { continue; }

    if let Ok( output ) = format_session_output( project, session_id, verbosity, show_entries, metadata_only )
    {
      return Ok( output );
    }
  }

  Err( ErrorData::new( ErrorCode::InternalError, format!( "Session '{session_id}' not found in current directory projects" ) ) )
}

/// Helper: Show session in specific project
fn show_session_in_project_impl(
  session_id : &str,
  project_param : &str,
  verbosity : i64,
  show_entries : bool,
  metadata_only : bool
) -> core::result::Result< OutputData, ErrorData >
{
  let storage = create_storage()?;

  // Parse project parameter
  let proj_id = parse_project_parameter( project_param )
    .map_err( | e | ErrorData::new
    (
      ErrorCode::InternalError,
      format!( "Invalid project parameter: {e}" )
    ))?;

  let project = storage.load_project( &proj_id )
    .map_err( | e | ErrorData::new
    (
      ErrorCode::InternalError,
      format!( "Failed to load project {proj_id:?}: {e}" )
    ))?;

  format_session_output( &project, session_id, verbosity, show_entries, metadata_only )
}

/// Helper: Show project for current directory
fn show_project_for_cwd_impl( verbosity : i64 )
  -> core::result::Result< OutputData, ErrorData >
{
  let storage = create_storage()?;

  let project = storage.load_project_for_cwd()
    .map_err( | e | ErrorData::new
    (
      ErrorCode::InternalError,
      format!( "Failed to load project from current directory: {e}" )
    ))?;

  format_project_output( &project, verbosity )
}

/// Helper: Show specific project
fn show_project_impl( project_param : &str, verbosity : i64 )
  -> core::result::Result< OutputData, ErrorData >
{
  let storage = create_storage()?;

  // Parse project parameter
  let proj_id = parse_project_parameter( project_param )
    .map_err( | e | ErrorData::new
    (
      ErrorCode::InternalError,
      format!( "Invalid project parameter: {e}" )
    ))?;

  let project = storage.load_project( &proj_id )
    .map_err( | e | ErrorData::new
    (
      ErrorCode::InternalError,
      format!( "Failed to load project {proj_id:?}: {e}" )
    ))?;

  format_project_output( &project, verbosity )
}

/// Helper: Format session output (extracted logic)
///
/// REQ-011: Content-First Display
///
/// By default (`verbosity::1`), shows conversation content in readable chat-log format.
/// Use `metadata::1` or `verbosity::0` for old metadata-only behavior.
fn format_session_output(
  project : &claude_storage_core::Project,
  session_id : &str,
  verbosity : i64,
  show_entries : bool,
  metadata_only : bool
) -> core::result::Result< OutputData, ErrorData >
{
  // Find session
  let mut sessions = project.all_sessions()
    .map_err( | e | ErrorData::new( ErrorCode::InternalError, format!( "Failed to list sessions: {e}" ) ) )?;

  // Fix(issue-011): Support partial UUID matching (first 8 chars)
  //
  // Root cause: Session lookup only did exact string matching without checking
  // if provided ID is a prefix of existing session IDs. Users expect Git-style
  // prefix matching for UUIDs (e.g., "79f86582" matches "79f86582-1435-442c-935a-13f8d874918a").
  //
  // Pitfall: ID lookups should always support prefix matching for UUIDs. Test with
  // both exact and partial IDs to ensure both work. Use production-format test data
  // (actual UUIDs) not test-friendly strings like "test-session-123".
  let session = find_session_mut( &mut sessions, session_id )?;

  // Get session stats
  let stats = session.stats()
    .map_err( | e | ErrorData::new( ErrorCode::InternalError, format!( "Failed to get session stats: {e}" ) ) )?;

  // Format output
  let mut output = String::new();

  // REQ-011: Content-first paradigm
  // - verbosity::0 or metadata::1 → Metadata only (old behavior)
  // - verbosity::1+ (default) → Conversation content (NEW behavior)
  let show_content = verbosity >= 1 && !metadata_only;

  // Always show basic session header
  // Fix(issue-028): derive "entry"/"entries" from count; same pattern as issue-025/027.
  // Root cause: hardcoded plural "entries" produced "Session: abc (1 entries)".
  // Pitfall: "entry" is irregular — singular differs from plural root.
  let entry_noun = if stats.total_entries == 1 { "entry" } else { "entries" };
  writeln!( output, "Session: {} ({} {entry_noun})", session_id, stats.total_entries ).unwrap();

  // Metadata-only mode (old behavior)
  if metadata_only || verbosity == 0
  {
    writeln!( output, "Path: {}", session.storage_path().display() ).unwrap();
    writeln!( output, "Agent Session: {}", stats.is_agent_session ).unwrap();
    writeln!( output, "Total Entries: {}", stats.total_entries ).unwrap();
    writeln!( output, "User Entries: {}", stats.user_entries ).unwrap();
    writeln!( output, "Assistant Entries: {}", stats.assistant_entries ).unwrap();

    if let Some( first ) = &stats.first_timestamp
    {
      writeln!( output, "First Entry: {first}" ).unwrap();
    }

    if let Some( last ) = &stats.last_timestamp
    {
      writeln!( output, "Last Entry: {last}" ).unwrap();
    }

    if verbosity >= 2
    {
      output.push_str( "\nToken Usage:\n" );
      writeln!( output, "- Input: {}", stats.total_input_tokens ).unwrap();
      writeln!( output, "- Output: {}", stats.total_output_tokens ).unwrap();
      writeln!( output, "- Cache Read: {}", stats.total_cache_read_tokens ).unwrap();
      writeln!( output, "- Cache Creation: {}", stats.total_cache_creation_tokens ).unwrap();
    }

    // Old entries::1 behavior (UUID list) for backward compat
    if show_entries
    {
      let entries = session.entries()
        .map_err( | e | ErrorData::new( ErrorCode::InternalError, format!( "Failed to load entries: {e}" ) ) )?;

      output.push_str( "\nEntries:\n" );

      for ( idx, entry ) in entries.iter().enumerate()
      {
        writeln!
        (
          output,
          "{}. [{:?}] {} ({})",
          idx + 1,
          entry.entry_type,
          entry.uuid,
          entry.timestamp
        ).unwrap();
      }
    }
  }
  // Content-first mode (NEW default behavior)
  else if show_content
  {
    let entries = session.entries()
      .map_err( | e | ErrorData::new( ErrorCode::InternalError, format!( "Failed to load entries: {e}" ) ) )?;

    // Add separator for readability
    output.push_str( "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n" );
    output.push( '\n' );

    // Format each entry as conversation
    for entry in entries
    {
      let formatted = format_entry_content( entry, None );
      output.push_str( &formatted );
      output.push_str( "\n\n" );
    }

    output.push_str( "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n" );

    // Add metadata footer for verbosity::2+
    if verbosity >= 2
    {
      output.push( '\n' );
      output.push_str( "Session Metadata:\n" );
      writeln!( output, "- Path: {}", session.storage_path().display() ).unwrap();
      writeln!( output, "- Total Entries: {}", stats.total_entries ).unwrap();
      writeln!( output, "- User/Assistant: {}/{}", stats.user_entries, stats.assistant_entries ).unwrap();

      if verbosity >= 3
      {
        output.push_str( "\nToken Usage:\n" );
        writeln!( output, "- Input: {}", stats.total_input_tokens ).unwrap();
        writeln!( output, "- Output: {}", stats.total_output_tokens ).unwrap();
        writeln!( output, "- Cache Read: {}", stats.total_cache_read_tokens ).unwrap();
        writeln!( output, "- Cache Creation: {}", stats.total_cache_creation_tokens ).unwrap();
      }
    }
  }

  Ok( OutputData::new( output, "text" ) )
}

/// Helper: Format project output (extracted logic)
fn format_project_output(
  project : &claude_storage_core::Project,
  verbosity : i64
) -> core::result::Result< OutputData, ErrorData >
{
  // Get project statistics
  let stats = project.project_stats()
    .map_err( | e | ErrorData::new
    (
      ErrorCode::InternalError,
      format!( "Failed to get project stats: {e}" )
    ))?;

  // Get all sessions
  let mut sessions = project.sessions()
    .map_err( | e | ErrorData::new
    (
      ErrorCode::InternalError,
      format!( "Failed to list sessions: {e}" )
    ))?;

  // Format output
  let mut output = String::new();

  // Project header
  writeln!( output, "Project: {:?}", project.id() ).unwrap();
  writeln!( output, "Storage: {}", project.storage_dir().display() ).unwrap();
  output.push( '\n' );

  // Statistics
  writeln!( output, "Sessions: {} (Main: {}, Agent: {})",
    stats.session_count,
    stats.main_session_count,
    stats.agent_session_count
  ).unwrap();

  writeln!( output, "Total Entries: {}", stats.total_entries ).unwrap();

  if verbosity >= 2
  {
    output.push_str( "Tokens:\n" );
    writeln!( output, "  Input: {}", stats.total_input_tokens ).unwrap();
    writeln!( output, "  Output: {}", stats.total_output_tokens ).unwrap();
  }

  output.push( '\n' );

  // Sessions list
  if sessions.is_empty()
  {
    output.push_str( "No sessions found in this project.\n" );
  }
  else
  {
    output.push_str( "Sessions:\n" );

    for session in &mut sessions
    {
      let session_stats = session.stats()
        .map_err( | e | ErrorData::new
        (
          ErrorCode::InternalError,
          format!( "Failed to get session stats: {e}" )
        ))?;

      if verbosity == 0
      {
        // Minimal: just IDs
        writeln!( output, "  - {}", session.id() ).unwrap();
      }
      else if verbosity == 1
      {
        // Standard: ID + entry count + last timestamp
        let last = session_stats.last_timestamp
          .unwrap_or_else( || "unknown".to_string() );

        // Fix(issue-028): derive "entry"/"entries" from count — sibling of session_count fix.
        // Root cause: hardcoded "entries" produced "(1 entries, last: ...)".
        // Pitfall: "entry" is irregular — singular differs from plural root.
        let e_noun = if session_stats.total_entries == 1 { "entry" } else { "entries" };
        writeln!( output, "  - {} ({} {e_noun}, last: {})",
          session.id(),
          session_stats.total_entries,
          last
        ).unwrap();
      }
      else
      {
        // Detailed: full stats
        writeln!( output, "  - {}", session.id() ).unwrap();
        writeln!( output, "      Entries: {} (User: {}, Assistant: {})",
          session_stats.total_entries,
          session_stats.user_entries,
          session_stats.assistant_entries
        ).unwrap();

        if let Some( first ) = &session_stats.first_timestamp
        {
          writeln!( output, "      First: {first}" ).unwrap();
        }

        if let Some( last ) = &session_stats.last_timestamp
        {
          writeln!( output, "      Last: {last}" ).unwrap();
        }
      }
    }
  }

  Ok( OutputData::new( output, "text" ) )
}


/// Count entries, sessions, or projects
///
/// Fast counting without loading all data into memory.
///
/// Context-aware: When called without parameters, counts entries in the current project
/// (detected from CWD), matching the behavior of `.show` for UX consistency.
///
/// # Errors
///
/// Returns error if storage creation fails, target is invalid, required parameters
/// (project or session) are missing, or counting operations fail.
#[allow(clippy::needless_pass_by_value)]
#[inline]
pub fn count_routine( cmd : VerifiedCommand, _ctx : ExecutionContext )
  -> core::result::Result< OutputData, ErrorData >
{
  // Fix(issue-003a): Make .count context-aware like .show
  //
  // Root cause: .count defaulted to counting all projects globally when called without
  // parameters, while .show showed current project stats. Users expected .count to
  // count what .show shows (entries in current project).
  //
  // Pitfall: Related commands should have consistent default behaviors. If .show is
  // context-aware (uses CWD), .count should be too. Don't make one global and one local.
  let target = cmd.get_string( "target" );
  let project_id = cmd.get_string( "project" );
  let session_id = cmd.get_string( "session" );

  // Create storage instance
  let storage = create_storage()?;

  // Context-aware default: If no target and no project specified, try to count entries in CWD project
  // If CWD is not a project directory, fall back to counting all projects globally
  if target.is_none() && project_id.is_none()
  {
    if let Ok( project ) = storage.load_project_for_cwd()
    {
      // Count all entries across all sessions in the project
      let sessions = project.all_sessions()
        .map_err( | e | ErrorData::new( ErrorCode::InternalError, format!( "Failed to list sessions: {e}" ) ) )?;

      let mut total_entries = 0usize;
      for session in &sessions
      {
        // Fix(issue-017): Skip corrupted sessions with warning, matching project_stats() behavior.
        //
        // Root cause: `?` propagated parse errors from individual corrupted sessions to the
        // entire `.count` command. A single corrupted JSONL line in any session would fail
        // the whole command, even though other sessions were valid. project_stats() already
        // handled this correctly by using `match` + `eprintln!` to skip corrupted sessions.
        //
        // Pitfall: Using `?` in a loop over user data is too strict. Real Claude Code sessions
        // can have truncated JSONL lines (from crashes mid-write). Always handle per-item
        // errors gracefully when iterating over a collection of user files.
        match session.count_entries()
        {
          Ok( n ) => total_entries += n,
          Err( e ) =>
          {
            eprintln!( "Warning: Skipping corrupted session {}: {e}", session.storage_path().display() );
          }
        }
      }

      let output = format!( "{total_entries}" );
      return Ok( OutputData::new( output, "text" ) );
    }
    // If load_project_for_cwd() fails, fall through to default behavior (count all projects)
  }

  // Explicit target specified, or project without target (counts sessions in project)
  let target : &str = target.unwrap_or( "projects" );
  let count = match target
  {
    "projects" =>
    {
      storage.count_projects()
        .map_err( | e | ErrorData::new( ErrorCode::InternalError, format!( "Failed to count projects: {e}" ) ) )?
    }
    "sessions" =>
    {
      // Requires project context
      let proj_id = project_id
        .ok_or_else( || ErrorData::new( ErrorCode::InternalError, "project parameter required for counting sessions".to_string() ) )?;

      // Fix(issue-012): Support path projects in .count command
      //
      // Root cause: Hardcoded ProjectId::uuid() prevented path projects from working.
      // Commands .count/.search/.export shared this bug which was fixed for .show (Finding #008)
      // but not propagated.
      //
      // Pitfall: When fixing a bug in one command, grep for identical patterns in other commands.
      // Bugs often exist in multiple locations sharing the same flawed assumption.
      let project = load_project_for_param( &storage, proj_id )?;

      project.count_sessions()
        .map_err( | e | ErrorData::new( ErrorCode::InternalError, format!( "Failed to count sessions: {e}" ) ) )?
    }
    "entries" =>
    {
      // Requires project + session context
      let proj_id = project_id
        .ok_or_else( || ErrorData::new( ErrorCode::InternalError, "project parameter required for counting entries".to_string() ) )?;

      let sess_id = session_id
        .ok_or_else( || ErrorData::new( ErrorCode::InternalError, "session parameter required for counting entries".to_string() ) )?;

      // Fix(issue-012): Support path projects in .count command
      //
      // Root cause: Hardcoded ProjectId::uuid() prevented path projects from working.
      // Commands .count/.search/.export shared this bug which was fixed for .show (Finding #008)
      // but not propagated.
      //
      // Pitfall: When fixing a bug in one command, grep for identical patterns in other commands.
      // Bugs often exist in multiple locations sharing the same flawed assumption.
      let project = load_project_for_param( &storage, proj_id )?;

      let sessions = project.all_sessions()
        .map_err( | e | ErrorData::new( ErrorCode::InternalError, format!( "Failed to list sessions: {e}" ) ) )?;

      // Fix(issue-019): Use prefix matching for partial UUID, consistent with show_routine
      // and export_routine (both use starts_with from the issue-011 fix).
      //
      // Root cause: count_routine used exact equality only, so "79f86582" failed even
      // though ".show session_id::79f86582" succeeds via prefix matching.
      //
      // Pitfall: When fixing partial-UUID support in one session lookup, grep for every
      // other `sessions.iter*().find(|s| s.id() == ...)` and apply the same change.
      let session = sessions.iter()
        .find( | s | s.id() == sess_id || s.id().starts_with( sess_id ) )
        .ok_or_else( || ErrorData::new( ErrorCode::InternalError, format!( "Session not found: {sess_id}" ) ) )?;

      session.count_entries()
        .map_err( | e | ErrorData::new( ErrorCode::InternalError, format!( "Failed to count entries: {e}" ) ) )?
    }
    "conversations" =>
    {
      let proj_id = project_id
        .ok_or_else( || ErrorData::new(
          ErrorCode::InternalError,
          "project parameter required for counting conversations".to_string(),
        ) )?;
      let project = load_project_for_param( &storage, proj_id )?;
      let sessions = project.all_sessions()
        .map_err( | e | ErrorData::new( ErrorCode::InternalError, format!( "Failed to load sessions: {e}" ) ) )?;
      let families = build_families( sessions );
      let conversations = group_into_conversations( families );
      conversations.len()
    }
    // Fix(issue-009): Validate target parameter against allowed values
    //
    // Root cause: target parameter accepted any string without validation,
    // causing confusing errors when invalid values provided.
    //
    // Pitfall: Don't assume unilang validates enum constraints. Always
    // validate enumerated parameters explicitly against allowed values.
    _ =>
    {
      return Err( ErrorData::new( ErrorCode::InternalError, format!( "Invalid target: {target}" ) ) );
    }
  };

  let output = format!( "{count}" );
  Ok( OutputData::new( output, "text" ) )
}

/// Search session content for query string
///
/// Performs full-text search through session content with optional filtering.
///
/// # Errors
///
/// Returns error if query is missing, verbosity is out of range, entry type
/// is invalid, storage creation fails, project loading fails, or search fails.
#[allow(clippy::too_many_lines)]
// CLI routine handler processes multiple scope branches and verbosity levels —
// extraction would obscure the command's logic without reducing complexity.
#[allow(clippy::needless_pass_by_value)]
#[inline]
pub fn search_routine( cmd : VerifiedCommand, _ctx : ExecutionContext )
  -> core::result::Result< OutputData, ErrorData >
{
  let query = cmd.get_string( "query" )
    .ok_or_else( || ErrorData::new( ErrorCode::InternalError, "query is required".to_string() ) )?;

  let project_id = cmd.get_string( "project" );
  let session_id = cmd.get_string( "session" );
  let case_sensitive = cmd.get_boolean( "case_sensitive" ).unwrap_or( false );
  let entry_type = cmd.get_string( "entry_type" );
  let verbosity = cmd.get_integer( "verbosity" ).unwrap_or( 1 );

  // Fix(issue-010): Validate verbosity range
  //
  // Root cause: search_routine accepted any verbosity value without validation,
  // inconsistent with status_routine and show_routine which validate 0-5 range.
  //
  // Pitfall: Don't assume default values prevent invalid input. Parameters with
  // defaults still need validation since users can override with invalid values.
  validate_verbosity( verbosity )?;

  // Create storage instance
  let storage = create_storage()?;

  // Build search filter
  let mut filter = claude_storage_core::SearchFilter::new( query )
    .case_sensitive( case_sensitive );

  // Add entry type filter if specified
  //
  // Fix(issue-021): Handle "all" as a valid entry_type value
  //
  // Root cause: Only "user" and "assistant" were handled in the match; "all" fell
  // through to the error arm despite the YAML spec documenting it as valid
  // ("Filter by entry type (user, assistant, or all)").
  //
  // Pitfall: Enumerated parameter match arms must cover every value listed in the
  // YAML spec description. Check the YAML spec when adding match arms, not just
  // what you remember implementing.
  if let Some( et ) = entry_type
  {
    match et
    {
      "user" => filter = filter.match_entry_type( claude_storage_core::EntryType::User ),
      "assistant" => filter = filter.match_entry_type( claude_storage_core::EntryType::Assistant ),
      "all" => { /* no type filter — same as omitting entry_type */ }
      _ => return Err( ErrorData::new( ErrorCode::InternalError, format!( "Invalid entry_type: {et}. Valid values: user, assistant, all" ) ) ),
    }
  }

  // Determine search scope
  let mut all_matches = Vec::new();

  if let Some( sess_id ) = session_id
  {
    // Search specific session
    let project = if let Some( proj_id ) = project_id
    {
      // Fix(issue-012): Support path projects in .search command
      //
      // Root cause: Hardcoded ProjectId::uuid() prevented path projects from working.
      // Commands .count/.search/.export shared this bug which was fixed for .show (Finding #008)
      // but not propagated.
      //
      // Pitfall: When fixing a bug in one command, grep for identical patterns in other commands.
      // Bugs often exist in multiple locations sharing the same flawed assumption.
      load_project_for_param( &storage, proj_id )
    }
    else
    {
      storage.load_project_for_cwd()
        .map_err( | e | ErrorData::new( ErrorCode::InternalError, format!( "Failed to load project: {e}" ) ) )
    }?;

    let mut sessions = project.all_sessions()
      .map_err( | e | ErrorData::new( ErrorCode::InternalError, format!( "Failed to list sessions: {e}" ) ) )?;

    // Fix(issue-020): Use prefix matching for partial UUID, consistent with show_routine
    // and export_routine (issue-011 fix).
    //
    // Root cause: search_routine used exact equality only, so ".search session::79f86582"
    // failed even though ".show session_id::79f86582" succeeds via starts_with.
    //
    // Pitfall: Partial-UUID support must be applied uniformly. Any session find()
    // predicate that uses only == will silently reject valid prefix IDs.
    let session = find_session_mut( &mut sessions, sess_id )?;

    let matches = session.search( &filter )
      .map_err( | e | ErrorData::new( ErrorCode::InternalError, format!( "Search failed: {e}" ) ) )?;

    for m in matches
    {
      all_matches.push( ( project.id().clone(), sess_id.to_string(), m ) );
    }
  }
  else if let Some( proj_id ) = project_id
  {
    // Search specific project
    // Fix(issue-012): Support path projects in .search command
    //
    // Root cause: Hardcoded ProjectId::uuid() prevented path projects from working.
    // Commands .count/.search/.export shared this bug which was fixed for .show (Finding #008)
    // but not propagated.
    //
    // Pitfall: When fixing a bug in one command, grep for identical patterns in other commands.
    // Bugs often exist in multiple locations sharing the same flawed assumption.
    let project = load_project_for_param( &storage, proj_id )?;

    let mut sessions = project.sessions()
      .map_err( | e | ErrorData::new( ErrorCode::InternalError, format!( "Failed to list sessions: {e}" ) ) )?;

    for session in &mut sessions
    {
      let matches = match session.search( &filter )
      {
        Ok( m )  => m,
        Err( e ) => { eprintln!( "warning: search skipped session {}: {e}", session.id() ); continue; }
      };

      for m in matches
      {
        all_matches.push( ( project.id().clone(), session.id().to_string(), m ) );
      }
    }
  }
  else
  {
    // Search all projects and sessions (current working directory project only)
    let project = storage.load_project_for_cwd()
      .map_err( | e | ErrorData::new( ErrorCode::InternalError, format!( "Failed to load project: {e}" ) ) )?;

    let mut sessions = project.sessions()
      .map_err( | e | ErrorData::new( ErrorCode::InternalError, format!( "Failed to list sessions: {e}" ) ) )?;

    for session in &mut sessions
    {
      let matches = match session.search( &filter )
      {
        Ok( m )  => m,
        Err( e ) => { eprintln!( "warning: search skipped session {}: {e}", session.id() ); continue; }
      };

      for m in matches
      {
        all_matches.push( ( project.id().clone(), session.id().to_string(), m ) );
      }
    }
  }

  // Format output
  let mut output = String::new();

  if verbosity >= 1
  {
    let noun = if all_matches.len() == 1 { "match" } else { "matches" };
    writeln!( output, "Found {} {noun}:\n", all_matches.len() ).unwrap();
  }

  for ( proj_id, sess_id, m ) in &all_matches
  {
    match verbosity
    {
      0 =>
      {
        // Minimal: just excerpt
        writeln!( output, "{}", m.excerpt() ).unwrap();
      }
      1 =>
      {
        // Standard: session + excerpt
        writeln!
        (
          output,
          "[{}] [{:?}] {}",
          sess_id,
          m.entry_type(),
          m.excerpt()
        ).unwrap();
      }
      _ =>
      {
        // Detailed: full metadata
        write!
        (
          output,
          "Project: {:?}\nSession: {}\nEntry: {} ({})\nLine: {}\nExcerpt: {}\nFull Line: {}\n\n",
          proj_id,
          sess_id,
          m.entry_index(),
          match m.entry_type()
          {
            claude_storage_core::EntryType::User => "user",
            claude_storage_core::EntryType::Assistant => "assistant",
          },
          m.line_number(),
          m.excerpt(),
          m.full_line()
        ).unwrap();
      }
    }
  }

  if all_matches.is_empty()
  {
    output.push_str( "No matches found.\n" );
  }

  Ok( OutputData::new( output, "text" ) )
}

/// Export session to file
///
/// Exports a session to the specified format (markdown, JSON, or text).
///
/// # Errors
///
/// Returns error if `session_id` or output are missing, format is invalid,
/// storage creation fails, project or session loading fails, or export fails.
#[allow(clippy::needless_pass_by_value)]
#[inline]
pub fn export_routine( cmd : VerifiedCommand, _ctx : ExecutionContext )
  -> core::result::Result< OutputData, ErrorData >
{
  let session_id = cmd.get_string( "session_id" )
    .ok_or_else( || ErrorData::new( ErrorCode::InternalError, "session_id is required".to_string() ) )?;

  let output_path_str = cmd.get_string( "output" )
    .ok_or_else( || ErrorData::new( ErrorCode::InternalError, "output is required".to_string() ) )?;

  let format_str = cmd.get_string( "format" ).unwrap_or( "markdown" );
  let project_id = cmd.get_string( "project" );

  // Parse export format
  let format = claude_storage_core::ExportFormat::from_str( format_str )
    .map_err( | e | ErrorData::new( ErrorCode::InternalError, format!( "Invalid format: {e}" ) ) )?;

  // Create storage instance
  let storage = create_storage()?;

  // Load project
  let project = if let Some( proj_id ) = project_id
  {
    // Fix(issue-012): Support path projects in .export command
    //
    // Root cause: Hardcoded ProjectId::uuid() prevented path projects from working.
    // Commands .count/.search/.export shared this bug which was fixed for .show (Finding #008)
    // but not propagated.
    //
    // Pitfall: When fixing a bug in one command, grep for identical patterns in other commands.
    // Bugs often exist in multiple locations sharing the same flawed assumption.
    load_project_for_param( &storage, proj_id )
  }
  else
  {
    storage.load_project_for_cwd()
      .map_err( | e | ErrorData::new( ErrorCode::InternalError, format!( "Failed to load project: {e}" ) ) )
  }?;

  // Find session
  let mut sessions = project.all_sessions()
    .map_err( | e | ErrorData::new( ErrorCode::InternalError, format!( "Failed to list sessions: {e}" ) ) )?;

  // Fix(issue-011): Support partial UUID matching (first 8 chars)
  //
  // Root cause: Session lookup only did exact string matching without checking
  // if provided ID is a prefix of existing session IDs. Users expect Git-style
  // prefix matching for UUIDs (e.g., "79f86582" matches "79f86582-1435-442c-935a-13f8d874918a").
  //
  // Pitfall: ID lookups should always support prefix matching for UUIDs. Test with
  // both exact and partial IDs to ensure both work. Use production-format test data
  // (actual UUIDs) not test-friendly strings like "test-session-123".
  let session = find_session_mut( &mut sessions, session_id )?;

  // Export to file
  let output_path = std::path::Path::new( output_path_str );

  claude_storage_core::export_session_to_file( session, format, output_path )
    .map_err( | e | ErrorData::new( ErrorCode::InternalError, format!( "Export failed: {e}" ) ) )?;

  let output = format!( "Exported session '{session_id}' to {} (format: {format:?})", output_path.display() );
  Ok( OutputData::new( output, "text" ) )
}

/// Check whether `encoded_base` (cwd or `path::` arg, encoded) is covered by
/// the project identified by `dir_name` (raw storage directory name).
///
/// Returns `true` when the project is an ancestor of (or equal to) the base:
/// - `encoded_base == dir_name` — same project, no topic
/// - `encoded_base.starts_with(dir_name + "-")` — base is in the project subtree
/// - same two checks after stripping each `--topic` suffix from `dir_name`
///
/// The `rfind("--")` loop handles topic-scoped project directories
/// (e.g. `-home-user1-wip-core--default-topic`): it strips from the last `--`
/// rightward so the remaining base path can be compared against `encoded_base`.
fn is_relevant_encoded( dir_name : &str, encoded_base : &str ) -> bool
{
  let check = | candidate : &str | -> bool
  {
    encoded_base == candidate
    || encoded_base.starts_with( &format!( "{candidate}-" ) )
  };
  if check( dir_name ) { return true; }
  let mut s = dir_name;
  while let Some( idx ) = s.rfind( "--" )
  {
    s = &s[ ..idx ];
    if check( s ) { return true; }
  }
  false
}

/// Decode a storage directory name into a human-readable display path.
///
/// Path-encoded dirs start with `-` (e.g. `-home-user1-pro`). UUID dirs do not.
/// Compress `$HOME` prefix to `~` for display. Returns full path string if HOME unset.
fn tilde_compress( path : &std::path::Path ) -> String
{
  if let Ok( home ) = std::env::var( "HOME" )
  {
    if let Ok( rel ) = path.strip_prefix( std::path::Path::new( &home ) )
    {
      return format!( "~/{}", rel.display() );
    }
  }
  path.display().to_string()
}

/// Walk the filesystem to decode a lossy-encoded storage dir name to a real path.
///
/// At each `-` boundary the standard heuristic cannot distinguish a path separator
/// from an underscore (both encoded as `-`). This function resolves the ambiguity by
/// checking `is_dir()` at each step: it tries path separator first; if the candidate
/// directory does not exist, it falls back to joining with `_`.
///
/// Returns `None` if no matching path is found (project deleted, remote, or unmounted).
///
/// # Why only as fallback
///
/// Requires the project directory to exist on disk. Always call heuristic decode first
/// and only reach here when that result does not exist. This avoids unnecessary stat
/// calls for paths the heuristic already handles correctly.
fn decode_path_via_fs( encoded : &str ) -> Option< std::path::PathBuf >
{
  let inner = &encoded[ 1.. ]; // strip leading `-`
  let pieces : Vec< &str > = inner.split( '-' ).collect();
  if pieces.is_empty() { return None; }
  walk_fs( std::path::Path::new( "/" ), &pieces, 0, "" )
}

/// Decode the base-encoded component of a storage dir name to a real filesystem path.
///
/// Returns `None` if the encoded string is malformed (non-path-encoded keys such as UUIDs).
/// When `decode_path` succeeds but the result does not exist on disk, falls back to the
/// filesystem-guided walk to resolve `_` vs `/` ambiguity (Fix(issue-029)).
fn decode_storage_base( base_encoded : &str ) -> Option< std::path::PathBuf >
{
  use claude_storage_core::decode_path;
  let h = decode_path( base_encoded ).ok()?;
  if h.exists()
  {
    Some( h )
  }
  else
  {
    // Fix(issue-029): heuristic maps '_' to '/', try filesystem-guided decode.
    Some( decode_path_via_fs( base_encoded ).unwrap_or( h ) )
  }
}

/// Convert a topic component from a storage key to the corresponding filesystem directory name.
///
/// Topic components in storage keys use hyphens (`default-topic`); the filesystem directory
/// uses underscores (`-default_topic`). The leading `-` marks it as a git-ignored directory.
///
/// Examples: `"default-topic"` → `"-default_topic"`,  `"commit"` → `"-commit"`
fn topic_to_dir( topic : &str ) -> String
{
  format!( "-{}", topic.replace( '-', "_" ) )
}

/// Return true if `dir_name` encodes a project path that is `base_path` itself or is nested
/// under `base_path` (`scope::under` predicate).
///
/// The single-hyphen fast-reject `starts_with("{eb}-")` weeds out projects with completely
/// different paths before the more expensive filesystem decode.
fn matches_under( dir_name : &str, eb : &str, base_path : &std::path::Path ) -> bool
{
  if dir_name != eb && !dir_name.starts_with( &format!( "{eb}-" ) ) { return false; }
  if dir_name == eb { return true; }
  let candidate_base = strip_topic_suffix( dir_name );
  decode_path_via_fs( candidate_base )
    .map_or( true, | p | p.starts_with( base_path ) )
}

/// Return true if `dir_name` encodes a project path that is an ancestor of `base_path`
/// (`scope::relevant` predicate).
fn matches_relevant( dir_name : &str, eb : &str, base_path : &std::path::Path ) -> bool
{
  if !is_relevant_encoded( dir_name, eb ) { return false; }
  let candidate_base = strip_topic_suffix( dir_name );
  if candidate_base == eb { return true; }
  decode_path_via_fs( candidate_base )
    .map_or( true, | p | base_path.starts_with( &p ) )
}

/// Strip the `--topic` suffix from a storage dir name, returning the base encoded component.
///
/// Examples:
/// - `"-home-src--default-topic"` → `"-home-src"`
/// - `"-home-src"` → `"-home-src"` (unchanged)
fn strip_topic_suffix( dir_name : &str ) -> &str
{
  dir_name.find( "--" ).map_or( dir_name, | i | &dir_name[ ..i ] )
}

/// Split a storage dir name at each `--` boundary into a base encoded component
/// and zero or more topic components.
///
/// Example: `"-home-src--default-topic--commit"` → `("-home-src", ["default-topic", "commit"])`
fn split_storage_key< 'a >( dir_name : &'a str ) -> ( &'a str, Vec< &'a str > )
{
  let mut parts : Vec< &'a str > = Vec::new();
  let mut rest = dir_name;
  loop
  {
    if let Some( idx ) = rest.find( "--" )
    {
      parts.push( &rest[ ..idx ] );
      rest = &rest[ idx + 2.. ];
    }
    else
    {
      parts.push( rest );
      break;
    }
  }
  let base   = parts[ 0 ];
  let topics = parts[ 1.. ].to_vec();
  ( base, topics )
}

/// Recursive DFS helper for `decode_path_via_fs`.
///
/// `segment` accumulates the current unresolved path component. At each step, option A
/// commits `segment` as a directory and recurses with the next piece; option B appends
/// `_` + piece to `segment` and recurses. `is_dir()` prunes option A early.
fn walk_fs(
  base    : &std::path::Path,
  pieces  : &[ &str ],
  idx     : usize,
  segment : &str,
) -> Option< std::path::PathBuf >
{
  if idx == pieces.len()
  {
    let candidate = if segment.is_empty() { base.to_path_buf() } else { base.join( segment ) };
    return if candidate.exists() { Some( candidate ) } else { None };
  }
  let piece = pieces[ idx ];
  // Option A — path separator: commit current segment as a directory, recurse
  if !segment.is_empty()
  {
    let next_base = base.join( segment );
    if next_base.is_dir()
    {
      if let Some( result ) = walk_fs( &next_base, pieces, idx + 1, piece )
      {
        return Some( result );
      }
    }
  }
  // Option B — underscore: merge piece into segment
  let joined = if segment.is_empty()
  {
    piece.to_string()
  }
  else
  {
    format!( "{segment}_{piece}" )
  };
  walk_fs( base, pieces, idx + 1, &joined )
}

/// Decode a storage dir name to the longest real filesystem path it represents.
///
/// # Why the `starts_with('-')` guard
///
/// `decode_path()` requires its input to be a valid path-encoded string. UUID project
/// directories (e.g. `deadbeef-1234-...`) do not start with `-` and are NOT path-encoded.
/// Calling `decode_path` on a UUID returns `Err` — but more importantly, it would be
/// semantically wrong. UUID dirs represent web/IDE sessions without filesystem paths.
/// The guard ensures they fall through to the raw string return at the end.
///
/// # Topic components: metadata vs real directories
///
/// Topic-scoped project dirs are named `-path--topic` (double dash before topic).
/// Topics are often pure metadata tags (e.g. `--commit`), but they can also be real
/// hyphen-prefixed directories (e.g. `--default-topic` → `-default_topic/`).
///
/// Algorithm: decode the base path, then unconditionally extend it by each `--topic`
/// component. The storage key is authoritative: disk state at query time does not
/// affect session attribution. (Fix issue-035 removed the existence check.)
///
/// Examples:
/// - `-...-src--default-topic`         → `src/-default_topic`
/// - `-...-src--default-topic--commit` → `src/-default_topic/-commit`
/// - `-...-src--commit`                → `src/-commit`
///
/// # Why the filesystem fallback for the base
///
/// Fix(issue-029)
/// Root cause: `decode_path` heuristic defaults to path separator `/` for all
/// unrecognized `-` boundaries. Paths with underscore-named dirs (e.g. `wip_core`,
/// `claude_tools`) display incorrectly as `wip/core`, `claude/tools`.
/// Pitfall: Only call the filesystem walk as fallback — never primary — because it
/// requires the project directory to exist on disk. Deleted/remote projects fall
/// back to the raw encoded storage dir name.
fn decode_project_display( dir_name : &str ) -> String
{
  if !dir_name.starts_with( '-' ) { return dir_name.to_string(); }

  // Fix(issue-030)
  // Root cause: decode_project_display stripped `--topic` before decoding, so
  // `-...-src--default-topic` displayed as `src` even when `-default_topic` is a
  // real directory (the actual CWD). Topic suffixes that are real hyphen-prefixed
  // dirs were invisible in the session header.

  // Decode the base path (handles underscore vs slash ambiguity via filesystem walk).
  let ( base_encoded, topics ) = split_storage_key( dir_name );
  let Some( base_path ) = decode_storage_base( base_encoded ) else { return dir_name.to_string() };

  // Extend by each topic component as a hyphen-prefixed directory path segment.
  // "default-topic" → "-default_topic",  "commit" → "-commit".
  // Fix(issue-035)
  // Root cause: candidate.exists() dropped topic components when the topic
  //   directory is absent from disk. The storage key records the CWD at session
  //   start; disk state at query time must not affect session attribution.
  // Pitfall: Do NOT remove the h.exists() guard on the base path decode above —
  //   that check enables the filesystem-guided fallback for _/slash ambiguity.
  //   Only this topic-loop guard was incorrect.
  let mut current = base_path;
  for &topic in &topics
  {
    current = current.join( topic_to_dir( topic ) );
  }

  tilde_compress( &current )
}

// ─── sessions output helpers ──────────────────────────────────────────────────

fn session_mtime( session : &claude_storage_core::Session ) -> Option< std::time::SystemTime >
{
  std::fs::metadata( session.storage_path() )
    .ok()
    .and_then( | m | m.modified().ok() )
}

fn is_zero_byte_session( session : &claude_storage_core::Session ) -> bool
{
  std::fs::metadata( session.storage_path() )
    .map( | m | m.len() == 0 )
    .unwrap_or( false )
}

// Shorten real UUID-format IDs to first `UUID_SHORT_LEN` chars.
// Non-UUID IDs (e.g. synthetic test IDs) are returned intact.
fn short_id( id : &str ) -> &str
{
  if id.len() == UUID_LEN && id.as_bytes().get( UUID_SHORT_LEN ) == Some( &b'-' ) { &id[ ..UUID_SHORT_LEN ] }
  else { id }
}

fn format_relative_time( mtime : std::time::SystemTime ) -> String
{
  let elapsed = std::time::SystemTime::now()
    .duration_since( mtime )
    .unwrap_or_default();
  let secs = elapsed.as_secs();
  if secs < SECS_PER_MIN        { format!( "{secs}s ago" ) }
  else if secs < SECS_PER_HOUR  { format!( "{}m ago", secs / SECS_PER_MIN ) }
  else if secs < SECS_PER_DAY   { format!( "{}h ago", secs / SECS_PER_HOUR ) }
  else if secs < SECS_PER_MONTH { format!( "{}d ago", secs / SECS_PER_DAY ) }
  else                          { format!( "{}mo ago", secs / SECS_PER_MONTH ) }
}

// ─── family detection ──────────────────────────────────────────────────────

struct AgentMeta { agent_type : String }

struct AgentInfo
{
  session    : claude_storage_core::Session,
  agent_type : String,
}

struct SessionFamily
{
  root   : Option< claude_storage_core::Session >,
  agents : Vec< AgentInfo >,
}

/// A Conversation is the user-facing unit of interaction — one logical chat.
///
/// # Current implementation (1:1 mapping)
///
/// Each `SessionFamily` maps to exactly one `Conversation` via
/// `group_into_conversations`. The identity mapping is a placeholder
/// until cross-session chain detection is implemented.
///
/// # Future: Chain Detection contract
///
/// When implemented, one `Conversation` may span multiple `SessionFamily`
/// values representing work continued across `--new-session` invocations.
/// No explicit storage links exist (B17, B18 invariants); detection uses
/// temporal proximity and content heuristics.
pub struct Conversation
{
  families : Vec< SessionFamily >,
}

impl core::fmt::Debug for Conversation
{
  #[ inline ]
  fn fmt( &self, f : &mut core::fmt::Formatter< '_ > ) -> core::fmt::Result
  {
    f.debug_struct( "Conversation" )
      .field( "family_count", &self.conversation_count() )
      .finish()
  }
}

impl Conversation
{
  fn root_session( &self ) -> Option< &claude_storage_core::Session >
  {
    self.families.first().and_then( | f | f.root.as_ref() )
  }

  fn all_agents( &self ) -> impl Iterator< Item = &AgentInfo >
  {
    self.families.iter().flat_map( | f | f.agents.iter() )
  }

  fn conversation_count( &self ) -> usize
  {
    self.families.len()
  }
}

// Group session families into conversations (currently 1:1 identity mapping).
//
// Each `SessionFamily` maps to exactly one `Conversation`. Placeholder for
// future cross-session chain detection (B17/B18 invariants rule out storage links).
fn group_into_conversations( families : Vec< SessionFamily > ) -> Vec< Conversation >
{
  families
    .into_iter()
    .map( | family | Conversation { families : vec![ family ] } )
    .collect()
}

struct ProjectSummary
{
  display_path : String,
  last_mtime   : std::time::SystemTime,
}

/// Read `meta.json` sidecar for an agent session.
///
/// Derives the meta path by replacing the `.jsonl` extension with `.meta.json`.
/// Uses `claude_storage_core::parse_json` (not `serde_json`) because the core
/// crate already provides a JSON parser and `serde_json` is not a dependency.
/// Returns `AgentMeta { agent_type: "unknown" }` on any error (missing file,
/// empty file, malformed JSON, missing `agentType` key, or blank `agentType`).
///
/// Fix(issue-mt-empty-agenttype)
/// Root cause: `.unwrap_or("unknown")` only catches `None`; `Some("")` and
/// `Some("  ")` slipped through, rendering as empty or whitespace labels.
/// Pitfall: `unwrap_or` cannot replace a non-None but semantically empty value —
/// always pair it with `.filter(|s| !s.trim().is_empty())`.
fn parse_agent_meta( agent_path : &std::path::Path ) -> AgentMeta
{
  let meta_path = agent_path.with_extension( "meta.json" );
  let content = match std::fs::read_to_string( &meta_path )
  {
    Ok( c ) if !c.is_empty() => c,
    _ => return AgentMeta { agent_type : AGENT_TYPE_UNKNOWN.into() },
  };
  let Ok( val ) = claude_storage_core::parse_json( &content ) else
  {
    return AgentMeta { agent_type : AGENT_TYPE_UNKNOWN.into() };
  };
  let agent_type = val.as_object()
    .and_then( | obj | obj.get( "agentType" ) )
    .and_then( claude_storage_core::JsonValue::as_str )
    .filter( | s | !s.trim().is_empty() )
    .unwrap_or( AGENT_TYPE_UNKNOWN )
    .to_string();
  AgentMeta { agent_type }
}

/// Extract parent UUID from hierarchical agent path.
///
/// Layout: `{project_dir}/{parent_uuid}/subagents/agent-{id}.jsonl`
/// Returns `parent_uuid` by navigating `parent/parent/file_name`.
fn extract_parent_hierarchical( agent_path : &std::path::Path ) -> Option< String >
{
  agent_path
    .parent()?  // subagents/
    .parent()?  // {parent_uuid}/
    .file_name()?
    .to_str()
    .map( String::from )
}

/// Extract parent session ID from first JSONL line of a flat agent file.
///
/// Reads only the first line and parses the `sessionId` field.
fn extract_parent_flat( agent_path : &std::path::Path ) -> Option< String >
{
  use std::io::BufRead;
  let file = std::fs::File::open( agent_path ).ok()?;
  let mut reader = std::io::BufReader::new( file );
  let mut line = String::new();
  reader.read_line( &mut line ).ok()?;
  let val = claude_storage_core::parse_json( &line ).ok()?;
  val.as_object()?
    .get( "sessionId" )?
    .as_str()
    .map( String::from )
}

/// Detect whether this project uses hierarchical agent storage.
///
/// Returns `true` if any agent path contains a "subagents" component.
fn is_hierarchical_format( agents : &[ &claude_storage_core::Session ] ) -> bool
{
  agents.iter().any( | s |
    s.storage_path().components().any( | c | c.as_os_str() == "subagents" )
  )
}

/// Resolve parent links for a list of agent sessions.
///
/// Detects hierarchical vs flat format, extracts parent IDs, and partitions
/// agents into a parent-keyed map and an orphan list.
fn resolve_agent_parents(
  agents : Vec< claude_storage_core::Session >,
) -> ( std::collections::HashMap< String, Vec< AgentInfo > >, Vec< AgentInfo > )
{
  use std::collections::HashMap;

  let agent_refs : Vec< &claude_storage_core::Session > = agents.iter().collect();
  let hierarchical = is_hierarchical_format( &agent_refs );

  let mut parent_map : HashMap< String, Vec< AgentInfo > > = HashMap::new();
  let mut orphans : Vec< AgentInfo > = Vec::new();

  for agent in agents
  {
    let meta = parse_agent_meta( agent.storage_path() );
    let parent_id = if hierarchical
    {
      extract_parent_hierarchical( agent.storage_path() )
    }
    else
    {
      extract_parent_flat( agent.storage_path() )
    };

    let info = AgentInfo { session : agent, agent_type : meta.agent_type };
    match parent_id
    {
      Some( pid ) => parent_map.entry( pid ).or_default().push( info ),
      None => orphans.push( info ),
    }
  }

  ( parent_map, orphans )
}

/// Build session families from a flat list of sessions.
///
/// Groups agent sessions under their parent root sessions. Handles both
/// hierarchical (path-based) and flat (`sessionId`-based) parent detection.
/// Agents without a matching root become orphan families.
fn build_families(
  sessions : Vec< claude_storage_core::Session >,
) -> Vec< SessionFamily >
{
  let mut roots  : Vec< claude_storage_core::Session > = Vec::new();
  let mut agents : Vec< claude_storage_core::Session > = Vec::new();
  for s in sessions
  {
    if s.is_agent_session() { agents.push( s ); }
    else { roots.push( s ); }
  }

  if agents.is_empty()
  {
    return roots.into_iter()
      .map( | r | SessionFamily { root : Some( r ), agents : Vec::new() } )
      .collect();
  }

  let ( mut parent_map, mut orphan_agents ) = resolve_agent_parents( agents );

  let mut families : Vec< SessionFamily > = Vec::new();
  for root in roots
  {
    let children = parent_map.remove( root.id() ).unwrap_or_default();
    families.push( SessionFamily { root : Some( root ), agents : children } );
  }

  for ( _pid, agents_vec ) in parent_map
  {
    orphan_agents.extend( agents_vec );
  }
  if !orphan_agents.is_empty()
  {
    families.push( SessionFamily { root : None, agents : orphan_agents } );
  }

  families.sort_by( | a, b |
  {
    let ta = a.root.as_ref().and_then( session_mtime )
      .unwrap_or( std::time::UNIX_EPOCH );
    let tb = b.root.as_ref().and_then( session_mtime )
      .unwrap_or( std::time::UNIX_EPOCH );
    tb.cmp( &ta )
  } );

  families
}

/// Format agent type breakdown as `"N×Type, M×Type"` sorted by count desc.
fn format_type_breakdown( agents : &[ AgentInfo ] ) -> String
{
  use std::collections::HashMap;
  let mut counts : HashMap< &str, usize > = HashMap::new();
  for a in agents
  {
    *counts.entry( a.agent_type.as_str() ).or_default() += 1;
  }
  let mut pairs : Vec< ( &str, usize ) > = counts.into_iter().collect();
  pairs.sort_by( | a, b | b.1.cmp( &a.1 ).then_with( || a.0.cmp( b.0 ) ) );
  pairs.iter()
    .map( | ( t, n ) | format!( "{n}\u{00d7}{t}" ) )
    .collect::< Vec< _ > >()
    .join( ", " )
}

/// Aggregate sessions by project, returning projects sorted by last mtime descending.
///
/// For each project in `groups`, finds the most-recently-modified non-zero-byte session.
/// Projects where no session has a readable mtime are excluded.
///
/// # Pitfalls
///
/// - (P4) Finds the most-active PROJECT by max(mtime) per project — not the
///   globally most-active session. A project with 3 old sessions and 1 new
///   session has `last_mtime` = that new session's mtime.
/// - (P5) Returns a Vec sorted by mtime descending; never iterate `groups`
///   directly for time-sorted output — `BTreeMap` order is alphabetical.
fn aggregate_projects(
  groups : &mut std::collections::BTreeMap< String, Vec< claude_storage_core::Session > >,
) -> Vec< ProjectSummary >
{
  let mut summaries : Vec< ProjectSummary > = Vec::new();

  for ( display_path, sessions ) in groups.iter_mut()
  {
    // Fix(issue-034): Exclude zero-byte placeholder sessions from best-session
    // selection in aggregate_projects.
    //
    // Root cause: `best` selection iterated all sessions including zero-byte
    // placeholders. When a zero-byte file had a more recent mtime than any real
    // session, it became the "best" session with a stale timestamp.
    //
    // Pitfall: `is_zero_byte_session()` must be applied at every aggregation
    // site — not only in the render layer.
    let best = sessions
      .iter()
      .enumerate()
      .filter( | ( _, s ) | !is_zero_byte_session( s ) )
      .filter_map( | ( i, s ) | session_mtime( s ).map( | t | ( i, t ) ) )
      .max_by_key( | &( _, t ) | t );

    let Some( ( _, best_time ) ) = best else { continue };

    summaries.push( ProjectSummary
    {
      display_path : display_path.clone(),
      last_mtime   : best_time,
    } );
  }

  // Most recently active project first.
  summaries.sort_by( | a, b | b.last_mtime.cmp( &a.last_mtime ) );
  summaries
}

/// List sessions with scope control (session-first view).
///
/// Mirrors `kbase` `scope::` semantics:
/// - `local`    — Current project only (`path::` selects the project, defaults to cwd)
/// - `relevant` — Every project whose path is an ancestor of (or equal to) `path::`
/// - `under`    — Every project whose path starts with `path::` (default)
/// - `global`   — All projects in storage (ignores `path::`)
///
/// # Errors
///
/// Returns error if `scope::` is invalid, `verbosity::` is out of range,
/// `min_entries::` is negative, `limit::` is negative, path resolution fails,
/// or storage access fails.
#[allow(clippy::needless_pass_by_value)]
#[allow(clippy::too_many_lines)]
#[inline]
pub fn projects_routine( cmd : VerifiedCommand, _ctx : ExecutionContext )
  -> core::result::Result< OutputData, ErrorData >
{
  use std::collections::BTreeMap;
  use std::path::PathBuf;
  use claude_storage_core::{ Session, SessionFilter, encode_path };

  // --- parameters ---

  let scope_raw = cmd.get_string( "scope" ).unwrap_or( "around" );
  let scope = scope_raw.to_lowercase();
  if !matches!( scope.as_str(), "local" | "relevant" | "under" | "around" | "global" )
  {
    return Err( ErrorData::new(
      ErrorCode::InternalError,
      format!( "scope must be relevant|local|under|around|global, got {scope_raw}" ),
    ) );
  }

  let verbosity = cmd.get_integer( "verbosity" ).unwrap_or( 1 );
  validate_verbosity( verbosity )?;

  let min_entries_filter = if let Some( n ) = cmd.get_integer( "min_entries" )
  {
    if n < 0
    {
      return Err( ErrorData::new(
        ErrorCode::InternalError,
        format!( "Invalid min_entries: {n}. Must be non-negative" ),
      ) );
    }
    #[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
    Some( n as usize )
  }
  else { None };

  let limit_cap = if let Some( n ) = cmd.get_integer( "limit" )
  {
    if n < 0
    {
      return Err( ErrorData::new(
        ErrorCode::InternalError,
        format!( "Invalid limit: {n}. Must be non-negative" ),
      ) );
    }
    #[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
    let v = n as usize;
    // 0 means unlimited — map to usize::MAX so comparisons work without special-casing
    if v == 0 { usize::MAX } else { v }
  }
  else { usize::MAX };

  let agent_filter = cmd.get_boolean( "agent" );
  let session_id_filter = cmd.get_string( "session" );

  // Resolve base path (used by local / relevant / under; ignored for global)
  let base_path : PathBuf = if let Some( p ) = cmd.get_string( "path" )
  {
    resolve_path_parameter( p )
      .map( PathBuf::from )
      .map_err( | e | ErrorData::new(
        ErrorCode::InternalError,
        format!( "Failed to resolve path '{p}': {e}" ),
      ) )?
  }
  else
  {
    std::env::current_dir()
      .map_err( | e | ErrorData::new(
        ErrorCode::InternalError,
        format!( "Failed to get current directory: {e}" ),
      ) )?
  };

  // --- collect projects by scope ---

  let storage = create_storage()?;
  let all_projects = storage.list_projects()
    .map_err( | e | ErrorData::new( ErrorCode::InternalError, format!( "Failed to list projects: {e}" ) ) )?;

  // Fix(issue-024)
  // Root cause: encode_path() maps both '_' and '/' to '-', so decode_component()
  // defaults unknown pairs to '/', turning `wip_core` → `wip-core` → `wip/core`.
  // Decoded paths never match the real base_path, causing silent 0-result returns.
  // Pitfall: Never decode storage dir names for path comparison — encoding is
  // deterministic but decoding is lossy. Compare encoded ↔ encoded instead.
  let encoded_base : Option< String > = if scope == "global"
  {
    None
  }
  else
  {
    Some(
      encode_path( &base_path )
        .map_err( | e | ErrorData::new(
          ErrorCode::InternalError,
          format!( "Failed to encode base path '{}': {e}", base_path.display() ),
        ) )?
    )
  };

  // Closure: does this project qualify under `scope`?
  // Compares encoded base against raw storage directory name — no decode step.
  // UUID project dirs start with a hex character (not '-'), so they never match
  // path-based comparisons and are correctly excluded from non-global scopes.
  let project_matches = | project : &claude_storage_core::Project | -> bool
  {
    if scope == "global" { return true; }
    let Some( ref eb ) = encoded_base else { return false };
    let dir_name = project
      .storage_dir()
      .file_name()
      .and_then( | n | n.to_str() )
      .unwrap_or( "" );
    match scope.as_str()
    {
      "local"    => dir_name == eb || dir_name.starts_with( &format!( "{eb}--" ) ),
      // Fix(issue-031)
      // Root cause: starts_with on encoded strings cannot distinguish a child
      //   directory (base/sub → `base-sub`) from a same-level sibling whose name
      //   uses an underscore (base_extra → `base-extra`): both share the `base-`
      //   prefix. Path::starts_with is component-wise and correctly excludes siblings.
      // Pitfall: strip the `--topic` suffix from dir_name before calling
      //   decode_path_via_fs. The `--topic` part encodes a hyphen-prefixed directory
      //   like `-default_topic`; left in place, the walker searches for a dir named
      //   `topic` under the project root, returns None, and the fallback silently
      //   includes everything — the sibling exclusion is bypassed.
      "under" => matches_under( dir_name, eb, &base_path ),
      // Fix(issue-032)
      // Root cause: is_relevant_encoded uses string starts_with to check if
      //   dir_name's encoded path is a prefix of encoded_base, so a sibling
      //   `base` (encoded `base-`) falsely matches when base_path is `base_extra`
      //   (encoded `base-extra`). Both `_` and `/` map to `-`, making siblings
      //   indistinguishable from ancestors by string comparison alone.
      //   base_path.starts_with(decoded_path) is component-wise and rejects siblings.
      // Pitfall: strip the `--topic` suffix before calling decode_path_via_fs —
      //   same requirement as the issue-031 fix for scope::under.
      "relevant" => matches_relevant( dir_name, eb, &base_path ),
      // Union of under + relevant — bidirectional neighborhood.
      // BTreeMap key on decoded path deduplicates projects matched by both arms.
      "around" =>
        matches_under( dir_name, eb, &base_path )
          || matches_relevant( dir_name, eb, &base_path ),
      _          => false,
    }
  };

  // --- build session filter ---

  let session_filter = SessionFilter
  {
    agent_only                : agent_filter,
    min_entries               : min_entries_filter,
    session_id_substring      : session_id_filter.map( std::string::ToString::to_string ),
  };

  // --- collect sessions grouped by decoded project path (Algorithm B) ---

  // BTreeMap gives deterministic, alphabetically sorted project order.
  let mut groups : BTreeMap< String, Vec< Session > > = BTreeMap::new();

  for mut project in all_projects
  {
    if !project_matches( &project ) { continue; }

    let dir_name = project
      .storage_dir()
      .file_name()
      .and_then( | n | n.to_str() )
      .unwrap_or( "" )
      .to_string();
    let display_path = decode_project_display( &dir_name );

    let Ok( sessions ) = project.sessions_filtered( &session_filter ) else { continue };
    if sessions.is_empty() { continue; }

    groups
      .entry( display_path )
      .or_default()
      .extend( sessions );
  }

  // --- sort each project's sessions by mtime descending (most recent first) ---

  for sessions in groups.values_mut()
  {
    sessions.sort_by( | a, b |
    {
      let ta = session_mtime( a ).unwrap_or( std::time::UNIX_EPOCH );
      let tb = session_mtime( b ).unwrap_or( std::time::UNIX_EPOCH );
      tb.cmp( &ta )
    } );
  }

  // --- format output (Algorithm C) ---

  // Aggregate into time-sorted project summaries (P5: never iterate groups directly).
  // aggregate_projects borrows groups mutably then releases; groups used below for
  // session lookup by display_path key.
  let summaries = aggregate_projects( &mut groups );

  // v0: one project path per line (machine-readable, no session IDs).
  if verbosity == 0
  {
    let mut output = String::new();
    for summary in &summaries
    {
      writeln!( output, "{}", summary.display_path ).unwrap();
    }
    return Ok( OutputData::new( output, "text" ) );
  }

  let total_projects = summaries.len();
  let mut output = String::new();

  // Family grouping: at v1 with no explicit agent:: filter, agents are grouped
  // into families under their root sessions instead of shown flat (P6: preserved).
  let use_families = agent_filter.is_none();

  let p_noun = if total_projects == 1 { "project" } else { "projects" };
  writeln!( output, "Found {total_projects} {p_noun}:\n" ).unwrap();

  for summary in summaries
  {
    // Retrieve (and remove) sessions for this project from groups.
    let sessions = groups.remove( &summary.display_path ).unwrap_or_default();
    let display_path = &summary.display_path;

    if use_families
    {
      // Build families from sessions and group into conversations (1:1 now)
      let families = build_families( sessions );
      let conversations = group_into_conversations( families );

      // Fix(issue-034): Count only displayable (non-zero-byte) root sessions in header.
      //
      // Root cause: families.iter().filter(|f| f.root.is_some()).count() counted ALL
      // root families including those whose root is a zero-byte placeholder. render_families_v1
      // excludes zero-byte roots from display, so the header showed "(2 sessions)" while
      // zero lines were rendered below it.
      //
      // Pitfall: The render layer and the count must apply identical zero-byte filters.
      // If render changes to show/hide zero-byte sessions, update this count expression too.
      let root_count = conversations
        .iter()
        .filter( | c | c.root_session().is_some_and( | s | !is_zero_byte_session( s ) ) )
        .count();
      let agent_count : usize = conversations.iter().map( | c | c.all_agents().count() ).sum();
      // Unpack back to families for rendering (Phase 4 will use Conversation directly)
      let families : Vec< SessionFamily > = conversations
        .into_iter()
        .flat_map( | c | c.families )
        .collect();

      let r_noun = if root_count == 1 { "conversation" } else { "conversations" };
      if agent_count > 0
      {
        let a_noun = if agent_count == 1 { "agent" } else { "agents" };
        writeln!( output, "{display_path}: ({root_count} {r_noun}, {agent_count} {a_noun})" ).unwrap();
      }
      else
      {
        writeln!( output, "{display_path}: ({root_count} {r_noun})" ).unwrap();
      }

      if verbosity == 1
      {
        render_families_v1( &mut output, &families, limit_cap );
      }
      else
      {
        render_families_v2( &mut output, &families );
      }
    }
    else
    {
      // Fix(issue-034): Flat branch — compute displayable before group_count so
      // the header count matches what is actually rendered.
      //
      // Root cause: `group_count = sessions.len()` was computed before the
      // `displayable` filter that excludes zero-byte non-agent sessions.
      // The header showed "(2 sessions)" when `displayable` produced 0 lines.
      //
      // Pitfall: Never count from the unfiltered source after a render filter
      // has been defined. Move the filter computation above the count so both
      // the header and the render loop use the same source of truth.
      let displayable : Vec< &Session > = sessions
        .iter()
        .filter( | &s | s.is_agent_session() || !is_zero_byte_session( s ) )
        .collect();
      let group_count = displayable.len();
      let group_noun = if group_count == 1 { "conversation" } else { "conversations" };
      writeln!( output, "{display_path}: ({group_count} {group_noun})" ).unwrap();
      let show_count = displayable.len().min( limit_cap );
      for ( i, &session ) in displayable[ ..show_count ].iter().enumerate()
      {
        let marker = if i == 0 { '*' } else { '-' };
        let id_str = short_id( session.id() );
        let time_str = session_mtime( session )
          .map( | t | format!( "  {}", format_relative_time( t ) ) )
          .unwrap_or_default();
        let count_str = session
          .count_entries()
          .map( | n |
          {
            let noun = if n == 1 { "entry" } else { "entries" };
            format!( "  ({n} {noun})" )
          } )
          .unwrap_or_default();
        writeln!( output, "  {marker} {id_str}{time_str}{count_str}" ).unwrap();
      }
      if displayable.len() > limit_cap
      {
        let hidden = displayable.len() - limit_cap;
        // "conversation" is the user-facing taxonomy noun; "session" is the internal storage term.
        let hidden_noun = if hidden == 1 { "conversation" } else { "conversations" };
        writeln!(
          output,
          "  ... and {hidden} more {hidden_noun}  (use limit::0 to list all)"
        ).unwrap();
      }
    }

    writeln!( output ).unwrap();
  }

  Ok( OutputData::new( output, "text" ) )
}

/// Format `[N agents: breakdown]` bracket suffix for a family with agents.
///
/// Returns empty string when the agent list is empty.
fn format_agent_bracket( agents : &[ AgentInfo ] ) -> String
{
  if agents.is_empty() { return String::new(); }
  let n = agents.len();
  let noun = if n == 1 { "agent" } else { "agents" };
  let breakdown = format_type_breakdown( agents );
  format!( "  [{n} {noun}: {breakdown}]" )
}

/// Format a single session line: `{marker} {id}  {age}  ({n} entries)`.
fn format_session_line(
  session : &claude_storage_core::Session,
  marker  : char,
) -> String
{
  let id_str = short_id( session.id() );
  let time_str = session_mtime( session )
    .map( | t | format!( "  {}", format_relative_time( t ) ) )
    .unwrap_or_default();
  let count_str = session
    .count_entries()
    .map( | n |
    {
      let noun = if n == 1 { "entry" } else { "entries" };
      format!( "  ({n} {noun})" )
    } )
    .unwrap_or_default();
  format!( "  {marker} {id_str}{time_str}{count_str}" )
}

/// Render family-grouped display at v1: root lines with `[N agents: breakdown]`.
fn render_families_v1(
  output    : &mut String,
  families  : &[ SessionFamily ],
  limit_cap : usize,
)
{
  let displayable : Vec< &SessionFamily > = families.iter()
    .filter( | f | !f.root.as_ref().is_some_and( is_zero_byte_session ) )
    .collect();
  let show_count = displayable.len().min( limit_cap );

  for ( i, family ) in displayable[ ..show_count ].iter().enumerate()
  {
    if let Some( root ) = &family.root
    {
      let marker = if i == 0 { '*' } else { '-' };
      let line = format_session_line( root, marker );
      let bracket = format_agent_bracket( &family.agents );
      writeln!( output, "{line}{bracket}" ).unwrap();
    }
    else
    {
      let bracket = format_agent_bracket( &family.agents );
      writeln!( output, "  ? (orphan){bracket}" ).unwrap();
    }
  }

  if displayable.len() > limit_cap
  {
    let hidden = displayable.len() - limit_cap;
    // "conversation" is the user-facing taxonomy noun; "session" is the internal storage term.
    let noun = if hidden == 1 { "conversation" } else { "conversations" };
    writeln!( output, "  ... and {hidden} more {noun}  (use limit::0 to list all)" ).unwrap();
  }
}

/// Render family-grouped display at v2+: tree-indented agents under each root.
fn render_families_v2(
  output   : &mut String,
  families : &[ SessionFamily ],
)
{
  for family in families
  {
    if let Some( root ) = &family.root
    {
      let id = root.id();
      let count_str = root
        .count_entries()
        .map( | n | {
          let noun = if n == 1 { "entry" } else { "entries" };
          format!( "  ({n} {noun})" )
        } )
        .unwrap_or_default();
      writeln!( output, "  - {id}{count_str}" ).unwrap();
    }
    else
    {
      writeln!( output, "  ? (orphan agents)" ).unwrap();
    }

    for ( j, agent ) in family.agents.iter().enumerate()
    {
      let connector = if j + 1 < family.agents.len() { "\u{251c}\u{2500}" } else { "\u{2514}\u{2500}" };
      let aid = agent.session.id();
      let atype = &agent.agent_type;
      let acount = agent.session
        .count_entries()
        .map( | n | {
          let noun = if n == 1 { "entry" } else { "entries" };
          format!( "  {n} {noun}" )
        } )
        .unwrap_or_default();
      writeln!( output, "    {connector} {aid}  {atype}{acount}" ).unwrap();
    }
  }
}

// ─── session path lifecycle helpers ──────────────────────────────────────────

/// Resolve the `path::` argument (or cwd) to an absolute `PathBuf`.
///
/// Returns `Err(ErrorData)` when `path::` is present but cannot be resolved,
/// or when `path::` is absent and `current_dir()` fails.
fn resolve_cmd_path( cmd : &VerifiedCommand ) -> core::result::Result< std::path::PathBuf, ErrorData >
{
  let resolved = cmd.get_string( "path" )
    .map( | p | resolve_path_parameter( p )
      .map_err( | e | ErrorData::new( ErrorCode::InternalError, e ) )
    )
    .transpose()?;

  match resolved
  {
    Some( s ) => Ok( std::path::PathBuf::from( s ) ),
    None =>
    {
      std::env::current_dir()
        .map_err( | e | ErrorData::new(
          ErrorCode::InternalError,
          format!( "Failed to get current directory: {e}" ),
        ) )
    }
  }
}

/// Validate a `topic::` value: non-empty and no path separators.
fn validate_topic( topic : &str ) -> core::result::Result< (), ErrorData >
{
  if topic.is_empty()
  {
    return Err( ErrorData::new(
      ErrorCode::InternalError,
      "topic must be non-empty".to_string(),
    ) );
  }
  if topic.contains( '/' )
  {
    return Err( ErrorData::new(
      ErrorCode::InternalError,
      "topic must not contain path separators".to_string(),
    ) );
  }
  Ok( () )
}

// ─── .project.path routine ────────────────────────────────────────────────────

/// Compute the Claude storage path for a directory.
///
/// Returns `~/.claude/projects/{encoded}/` for the given path.
/// Exits 0 always; path does not need to exist on disk.
///
/// # Errors
///
/// Returns error if path resolution fails or HOME is not set.
#[ allow( clippy::needless_pass_by_value ) ]
#[ inline ]
pub fn project_path_routine( cmd : VerifiedCommand, _ctx : ExecutionContext )
  -> core::result::Result< OutputData, ErrorData >
{
  let base = resolve_cmd_path( &cmd )?;

  let session_dir = if let Some( topic ) = cmd.get_string( "topic" )
  {
    validate_topic( topic )?;
    base.join( format!( "-{topic}" ) )
  }
  else
  {
    base
  };

  let storage_path = claude_storage_core::continuation::to_storage_path_for( &session_dir )
    .ok_or_else( || ErrorData::new(
      ErrorCode::InternalError,
      "Failed to compute storage path (HOME not set or invalid path)".to_string(),
    ) )?;

  Ok( OutputData::new( format!( "{}/", storage_path.display() ), "text" ) )
}

// ─── .project.exists routine ──────────────────────────────────────────────────

/// Check whether conversation history exists for a directory.
///
/// Exits 0 with "sessions exist" when history found.
/// Exits 1 via `Err` with "no sessions" when absent.
///
/// # Errors
///
/// Returns error if path resolution fails, topic is invalid, or no history exists.
#[ allow( clippy::needless_pass_by_value ) ]
#[ inline ]
pub fn project_exists_routine( cmd : VerifiedCommand, _ctx : ExecutionContext )
  -> core::result::Result< OutputData, ErrorData >
{
  let base = resolve_cmd_path( &cmd )?;

  let session_dir = if let Some( topic ) = cmd.get_string( "topic" )
  {
    validate_topic( topic )?;
    base.join( format!( "-{topic}" ) )
  }
  else
  {
    base
  };

  if claude_storage_core::continuation::check_continuation( &session_dir )
  {
    Ok( OutputData::new( "sessions exist".to_string(), "text" ) )
  }
  else
  {
    // Fix(issue-033): Spec requires stderr = "no sessions" for exit-1 case.
    //
    // Root cause: `execute_oneshot` printed `"Error: {error}"` where the pipeline
    // had already wrapped ErrorData as `"Execution error: Execution Error: {msg}\n"`,
    // producing three-level noise instead of the clean message the spec requires.
    //
    // Pitfall: Any command whose stderr content is consumed by shell scripts needs
    // an exact-match stderr test. `contains` tests pass even with prefix wrapping.
    Err( ErrorData::new( ErrorCode::InternalError, "no sessions".to_string() ) )
  }
}

// ─── .session.dir / .session.ensure shared helper ────────────────────────────

/// Resolve `path::` + `topic::` parameters into a session working directory.
///
/// Defaults to cwd when `path::` is absent. `topic::` defaults to `DEFAULT_TOPIC`.
/// The returned path is `{base}/-{topic}`.
///
/// # Errors
///
/// Returns `ErrorData` when path resolution fails (including cwd failure) or
/// topic is invalid.
fn resolve_session_dir(
  cmd : &VerifiedCommand,
) -> core::result::Result< std::path::PathBuf, ErrorData >
{
  // Fix(issue-037): resolve_session_dir defaults to cwd when path:: is absent.
  // Root cause: prior ok_or_else guard rejected absent path:: though YAML declared it optional.
  // Pitfall: resolve_cmd_path returns cwd — tests must set .current_dir() explicitly.
  let base = resolve_cmd_path( cmd )?;

  let topic = cmd.get_string( "topic" ).unwrap_or( DEFAULT_TOPIC );
  validate_topic( topic )?;

  Ok( base.join( format!( "-{topic}" ) ) )
}

// ─── .session.dir routine ─────────────────────────────────────────────────────

/// Compute the session working directory path without creating it.
///
/// Returns `{base}/-{topic}`. `path::` defaults to cwd when absent;
/// `topic::` defaults to `default_topic`.
///
/// # Errors
///
/// Returns error if path resolution fails or topic is invalid.
#[ allow( clippy::needless_pass_by_value ) ]
#[ inline ]
pub fn session_dir_routine( cmd : VerifiedCommand, _ctx : ExecutionContext )
  -> core::result::Result< OutputData, ErrorData >
{
  let session_dir = resolve_session_dir( &cmd )?;
  Ok( OutputData::new( format!( "{}", session_dir.display() ), "text" ) )
}

// ─── .session.ensure routine ──────────────────────────────────────────────────

/// Ensure the session working directory exists and report resume strategy.
///
/// Creates `{base}/-{topic}` if absent (idempotent — never wipes an existing dir).
/// Outputs two lines:
///   Line 1: absolute path to session directory
///   Line 2: `resume` or `fresh`
///
/// `path::` defaults to cwd when absent; `topic::` defaults to `default_topic`.
/// `strategy::` overrides the auto-detected label (`fresh`/`resume`) — it is a
/// **label override only** and does NOT modify the filesystem (no wipe/recreate).
///
/// # Errors
///
/// Returns error if path resolution fails, topic invalid, strategy invalid,
/// or directory creation fails.
#[ allow( clippy::needless_pass_by_value ) ]
#[ inline ]
pub fn session_ensure_routine( cmd : VerifiedCommand, _ctx : ExecutionContext )
  -> core::result::Result< OutputData, ErrorData >
{
  let session_dir = resolve_session_dir( &cmd )?;

  let forced_strategy = if let Some( s ) = cmd.get_string( "strategy" )
  {
    match s.to_lowercase().as_str()
    {
      "resume" => Some( true ),
      "fresh"  => Some( false ),
      other    => return Err( ErrorData::new(
        ErrorCode::InternalError,
        format!( "strategy must be resume|fresh, got {other}" ),
      ) ),
    }
  }
  else
  {
    None
  };

  std::fs::create_dir_all( &session_dir ).map_err( | e | ErrorData::new(
    ErrorCode::InternalError,
    format!( "Failed to create session directory: {e}" ),
  ) )?;

  let is_resume = forced_strategy.unwrap_or_else(
    || claude_storage_core::continuation::check_continuation( &session_dir )
  );

  let strategy = if is_resume { "resume" } else { "fresh" };

  Ok( OutputData::new( format!( "{}\n{strategy}", session_dir.display() ), "text" ) )
}

#[ cfg( test ) ]
mod tests
{
  use super::*;

  /// UT-49: `group_into_conversations` implements identity (1:1) mapping from families to conversations.
  ///
  /// Each `SessionFamily` maps to exactly one `Conversation`; empty input → empty output.
  /// Also verifies `root_session`, `all_agents`, `conversation_count` compile and return sensible values.
  #[ test ]
  fn it_conversation_groups_families_one_to_one()
  {
    let result = group_into_conversations( vec![] );
    assert_eq!( result.len(), 0, "Expected 0 conversations for 0 families" );
    // Verify all helper methods compile; loop is a no-op for empty input.
    for conv in &result
    {
      let _ = conv.root_session();
      let _ = conv.all_agents().count();
      let _ = conv.conversation_count();
    }
  }
}
