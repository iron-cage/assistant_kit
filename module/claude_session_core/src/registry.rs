//! The live-session registry at `~/.claude/sessions/`.
//!
//! Claude Code writes one `<pid>.json` file per running process and reaps it on
//! exit. This is a *different* store from the conversation transcripts under
//! `~/.claude/projects/` that `claude_storage_core` owns: mutable rather than
//! append-only, ephemeral rather than permanent, and keyed by PID rather than by
//! session id. The two are joined by the `sessionId` field carried here.
//!
//! # Injected directory
//!
//! Every entry point takes the registry directory as a parameter rather than
//! resolving `~/.claude/sessions` internally. That keeps this crate at Layer 0
//! (no `claude_core` dependency) and, more usefully, lets tests point at a
//! `TempDir` instead of reading the developer's real home.

use std::path::{ Path, PathBuf };

use claude_storage_core::parse_json;

use crate::error::{ Error, Result };
use crate::liveness::pid_alive;

/// Status a Claude Code process reports for itself.
#[ derive( Debug, Clone, PartialEq, Eq ) ]
pub enum SessionStatus
{
  /// A turn is in flight.
  Busy,
  /// No turn is in flight.
  ///
  /// **Not the same as "the turn is finished."** A session that is waiting to be
  /// woken by an outstanding background task also reports `idle` unless it was
  /// started with `CLAUDE_CODE_BG_TASKS_REPORT_RUNNING=1`. See
  /// [`crate::turn`] before treating this as a turn boundary.
  Idle,
  /// A status this crate does not model — carried through verbatim rather than
  /// collapsed into `Idle`, so a future Claude Code state is never silently
  /// mistaken for quiescence.
  Other( String ),
}

impl SessionStatus
{
  /// Parse a raw registry `status` string.
  #[ inline ]
  #[ must_use ]
  pub fn from_raw( raw : &str ) -> Self
  {
    match raw
    {
      "busy" => Self::Busy,
      "idle" => Self::Idle,
      other => Self::Other( other.to_string() ),
    }
  }
}

/// One entry in the live-session registry.
#[ derive( Debug, Clone ) ]
#[ non_exhaustive ]
pub struct SessionRecord
{
  /// Process id of the Claude Code process.
  pub pid : u32,
  /// Conversation id — the join key to the transcript under `~/.claude/projects/`.
  pub session_id : String,
  /// Working directory the process was started in.
  pub cwd : PathBuf,
  /// Process start time in clock ticks since boot, as recorded by the writer.
  ///
  /// Stored by Claude Code as a JSON *string*, not a number. Paired with `pid`
  /// it identifies a process incarnation; `pid` alone never does — see
  /// [`crate::liveness`].
  pub proc_start : Option< u64 >,
  /// Claude Code version string.
  pub version : Option< String >,
  /// Session kind, e.g. `interactive`.
  pub kind : Option< String >,
  /// How the session was entered, e.g. `cli`.
  pub entrypoint : Option< String >,
  /// Human-readable session name.
  pub name : Option< String >,
  /// Self-reported status.
  pub status : SessionStatus,
  /// Milliseconds since the Unix epoch when the record was last written.
  pub updated_at : Option< u64 >,
}

impl SessionRecord
{
  /// Parse one registry file's contents.
  ///
  /// Returns `None` when the text is not parseable JSON or lacks the two fields
  /// that make a record usable at all (`pid`, `sessionId`). A corrupt or
  /// half-written file is skipped rather than failing the whole scan — Claude
  /// Code rewrites these files in place, so a reader can observe a torn write.
  #[ inline ]
  #[ must_use ]
  pub fn parse( text : &str ) -> Option< Self >
  {
    let json = parse_json( text ).ok()?;
    let pid = u32::try_from( json_u64( json.get_number( "pid" ) )? ).ok()?;
    let session_id = json.get_str( "sessionId" )?.to_string();

    Some( Self
    {
      pid,
      session_id,
      cwd : json.get_str( "cwd" ).map_or_else( PathBuf::new, PathBuf::from ),
      // `procStart` is a string in the on-disk format, not a number.
      proc_start : json.get_str( "procStart" ).and_then( | s | s.parse().ok() ),
      version : json.get_str( "version" ).map( str::to_string ),
      kind : json.get_str( "kind" ).map( str::to_string ),
      entrypoint : json.get_str( "entrypoint" ).map( str::to_string ),
      name : json.get_str( "name" ).map( str::to_string ),
      status : json.get_str( "status" ).map_or( SessionStatus::Idle, SessionStatus::from_raw ),
      updated_at : json_u64( json.get_number( "updatedAt" ) ),
    })
  }

  /// Whether the process named by this record is still running, and is the same
  /// incarnation that wrote it.
  #[ inline ]
  #[ must_use ]
  pub fn is_alive( &self ) -> bool
  {
    pid_alive( self.pid, self.proc_start )
  }
}

/// Read every record in `sessions_dir`.
///
/// A missing directory yields an empty vector, not an error — it means no Claude
/// Code process has ever registered here. Individual unreadable or malformed
/// files are skipped; only a directory that exists but cannot be enumerated is
/// an error.
///
/// Records are returned sorted by `pid` for deterministic iteration.
///
/// # Errors
///
/// Returns [`Error::ReadDir`] when `sessions_dir` exists but cannot be read.
#[ inline ]
pub fn scan( sessions_dir : &Path ) -> Result< Vec< SessionRecord > >
{
  if !sessions_dir.exists()
  {
    return Ok( Vec::new() );
  }
  let entries = std::fs::read_dir( sessions_dir )
    .map_err( | source | Error::ReadDir { path : sessions_dir.to_path_buf(), source } )?;

  let mut records = Vec::new();
  for entry in entries.flatten()
  {
    let path = entry.path();
    if path.extension().and_then( | e | e.to_str() ) != Some( "json" )
    {
      continue;
    }
    let Ok( text ) = std::fs::read_to_string( &path ) else { continue };
    if let Some( record ) = SessionRecord::parse( &text )
    {
      records.push( record );
    }
  }
  records.sort_by_key( | r | r.pid );
  Ok( records )
}

/// Read every record in `sessions_dir` whose process is still running.
///
/// The registry is reaped on clean exit, but a killed or crashed process leaves
/// its file behind — so a raw [`scan`] can report sessions that no longer exist.
///
/// # Errors
///
/// Returns [`Error::ReadDir`] when `sessions_dir` exists but cannot be read.
#[ inline ]
pub fn scan_live( sessions_dir : &Path ) -> Result< Vec< SessionRecord > >
{
  let mut records = scan( sessions_dir )?;
  records.retain( SessionRecord::is_alive );
  Ok( records )
}

/// Convert a JSON number to `u64`, rejecting values outside its range.
///
/// The hand-written parser in `claude_storage_core` represents every number as
/// `f64`, which is the only lossy step in this path: PIDs and millisecond
/// timestamps are both well inside `f64`'s exact-integer range (2^53), so the
/// conversion is exact for every value this registry actually carries.
#[ allow( clippy::cast_possible_truncation, clippy::cast_sign_loss ) ]
fn json_u64( value : Option< f64 > ) -> Option< u64 >
{
  let n = value?;
  if n < 0.0 || !n.is_finite() || n > 9_007_199_254_740_992.0
  {
    return None;
  }
  Some( n as u64 )
}
