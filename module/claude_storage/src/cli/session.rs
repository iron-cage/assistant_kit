//! `.project.path`, `.project.exists`, `.session.dir`, `.session.ensure`,
//! `.session.path` commands.

use unilang::{ VerifiedCommand, ExecutionContext, OutputData, ErrorData, ErrorCode };
use super::storage::resolve_path_parameter;

/// Default session topic when no `topic::` param is supplied.
const DEFAULT_TOPIC : &str = "default_topic";

// ─── shared path helpers ──────────────────────────────────────────────────────

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

// ─── .session.path routine ────────────────────────────────────────────────────

/// Resolve `path::` (or cwd) to the canonical physical base plus its Claude
/// session storage directory, shared by every `.session.path` selector branch.
///
/// Canonicalization goes through `claude_storage_core::physical_abs` — the same
/// rule `clr` keys storage names and topic identities on — so a relative or
/// symlinked `path::` resolves to the identical storage dir the real run used.
/// Storage resolution goes through `to_storage_path_for` (CLAUDE_HOME-aware),
/// matching what `topic_session_file` uses internally.
fn resolve_storage_for_cmd(
  cmd : &VerifiedCommand,
) -> core::result::Result< ( std::path::PathBuf, std::path::PathBuf ), ErrorData >
{
  let base = resolve_cmd_path( cmd )?;
  let canonical_base = claude_storage_core::physical_abs( &base );
  let storage = claude_storage_core::continuation::to_storage_path_for( &canonical_base )
    .ok_or_else( || ErrorData::new(
      ErrorCode::InternalError,
      "Failed to compute storage path (HOME not set or invalid path)".to_string(),
    ) )?;
  Ok( ( canonical_base, storage ) )
}

/// Print the absolute session `.jsonl` file path for a directory.
///
/// Three mutually exclusive selectors; `latest` is the default when none is given:
///   - `latest::1` (default) — the most recent qualifying session in the
///     directory's storage. The one disk-reading selector; when the storage
///     holds no qualifying session, prints a note to stderr and exits 2
///     (nothing-found = usage outcome, matching `.status`'s exit-2 precedent).
///   - `session::UUID` — `{storage}/{UUID}.jsonl`, pure computation; the file
///     need not exist.
///   - `topic::NAME` — the FORK-mode topic session file:
///     `{storage}/{UUIDv5(canonical base, NAME)}.jsonl` via the shared
///     `claude_storage_core::topic_session_file` rule. Pure computation,
///     byte-identical to `clr topics --file NAME` run from the same directory.
///
/// TOPIC SENSE COLLISION — deliberate and documented: every OTHER command's
/// `topic::` (`.project.path`, `.project.exists`, `.session.dir`,
/// `.session.ensure`) means the legacy dir-mode suffix `{base}/-{topic}` — a
/// WORKING-DIRECTORY transform. This command's `topic::` instead names a
/// fork-mode topic — a SESSION-FILE identity inside the base's own storage,
/// with no `-{topic}` dir involved. The two senses are different mechanisms for
/// different topic kinds; do not "fix" one to match the other.
///
/// # Errors
///
/// Returns error when more than one selector is given, path resolution fails,
/// `session::`/`topic::` is malformed, or the storage dir cannot be computed.
#[ allow( clippy::needless_pass_by_value ) ]
#[ inline ]
pub fn session_path_routine( cmd : VerifiedCommand, _ctx : ExecutionContext )
  -> core::result::Result< OutputData, ErrorData >
{
  let session = cmd.get_string( "session" );
  let topic = cmd.get_string( "topic" );
  let latest = cmd.get_boolean( "latest" ).unwrap_or( false );

  let selector_count = usize::from( session.is_some() )
    + usize::from( topic.is_some() )
    + usize::from( latest );
  if selector_count > 1
  {
    return Err( ErrorData::new(
      ErrorCode::InternalError,
      "session::, latest::, and topic:: are mutually exclusive selectors".to_string(),
    ) );
  }

  if let Some( id ) = session
  {
    if id.is_empty() || id.contains( '/' )
    {
      return Err( ErrorData::new(
        ErrorCode::InternalError,
        "session must be a non-empty UUID (no path separators)".to_string(),
      ) );
    }
    let ( _canonical_base, storage ) = resolve_storage_for_cmd( &cmd )?;
    let file = storage.join( format!( "{id}.jsonl" ) );
    return Ok( OutputData::new( format!( "{}", file.display() ), "text" ) );
  }

  if let Some( name ) = topic
  {
    validate_topic( name )?;
    let ( canonical_base, _storage ) = resolve_storage_for_cmd( &cmd )?;
    let file = claude_storage_core::topic_session_file( &canonical_base, name )
      .ok_or_else( || ErrorData::new(
        ErrorCode::InternalError,
        "Failed to compute storage path (HOME not set or invalid path)".to_string(),
      ) )?;
    return Ok( OutputData::new( format!( "{}", file.display() ), "text" ) );
  }

  // Default / latest::1 — the only selector that reads the disk.
  let ( _canonical_base, storage ) = resolve_storage_for_cmd( &cmd )?;
  match claude_storage_core::continuation::most_recent_session_in_dir( &storage )
  {
    Some( id ) =>
    {
      let file = storage.join( format!( "{}.jsonl", id.as_str() ) );
      Ok( OutputData::new( format!( "{}", file.display() ), "text" ) )
    }
    None =>
    {
      eprintln!( "no sessions in {}", storage.display() );
      std::process::exit( 2 );
    }
  }
}
