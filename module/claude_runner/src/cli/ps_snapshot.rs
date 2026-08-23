//! `clr ps` temporal-diff state: persists the process list from one `clr ps`
//! invocation to the next, so the current call can tell which sessions are
//! 🆕 (absent from the prior snapshot) and which have ended (present in the
//! prior snapshot but no longer running).
//!
//! State lives at `CLR_PS_STATE_DIR` (env override) or `$HOME/.clr/ps` by
//! default, mirroring `config.rs`'s `user_config_dir()` 2-tier resolution
//! shape. All I/O here is best-effort: a missing, unreadable, or malformed
//! snapshot file must never abort `clr ps` itself — it just means no prior
//! state to compare against, exactly like a first-ever invocation.

use claude_core::process::ProcessInfo;
use claude_runner_core::ps_table::{ duration_label, render_headed_table, resolve_task, shorten_path };
use data_fmt::{ Heading, RowBuilder };
use std::collections::{ BTreeMap, HashSet };

/// One tracked session as of the last `clr ps` snapshot.
#[ derive( Debug, Clone, serde::Serialize, serde::Deserialize ) ]
pub( crate ) struct PsSnapshotEntry
{
  /// Process start time (Unix epoch seconds); `None` on non-Linux, where no
  /// `/proc`-derived start time is available (mirrors this crate's existing
  /// "-" convention for every other metrics-derived column).
  pub( crate ) started_at : Option< u64 >,
  /// Raw (unshortened) working directory — `$PRO` shortening is applied at
  /// render time, not storage time, consistent with how the live table always
  /// shortens on display rather than on capture.
  pub( crate ) path       : String,
  /// Last human message extracted from the session JSONL, or "interactive".
  pub( crate ) task       : String,
}

/// A full `clr ps` snapshot: every tracked session as of `captured_at`.
#[ derive( Debug, Clone, serde::Serialize, serde::Deserialize ) ]
pub( crate ) struct PsSnapshot
{
  /// Unix epoch seconds when this snapshot was written.
  pub( crate ) captured_at : u64,
  /// PID (as a string — JSON object keys must be strings) → tracked session.
  /// `BTreeMap` for deterministic serialization order.
  pub( crate ) sessions    : BTreeMap< String, PsSnapshotEntry >,
}

impl PsSnapshot
{
  /// The set of PIDs this snapshot tracked, for 🆕-flag comparison.
  #[ must_use ]
  pub( crate ) fn pid_set( &self ) -> HashSet< u32 >
  {
    self.sessions.keys().filter_map( | k | k.parse().ok() ).collect()
  }
}

// Private copy of ps_table.rs's unix_now() — that module documents why each
// small current-time consumer keeps its own copy rather than a shared pub
// helper (gate.rs already established the precedent for this crate).
fn unix_now() -> u64
{
  std::time::SystemTime::now()
    .duration_since( std::time::UNIX_EPOCH )
    .map_or( 0, | d | d.as_secs() )
}

// Resolve the directory for `clr ps`'s snapshot state file.
//
// Mirrors config.rs's user_config_dir() 2-tier shape (env var > $HOME
// default; no CLI-arg tier, since snapshot tracking is always-on, not
// user-configurable per invocation).
fn ps_state_dir() -> std::path::PathBuf
{
  if let Ok( v ) = std::env::var( "CLR_PS_STATE_DIR" )
  {
    if !v.is_empty() { return std::path::PathBuf::from( v ); }
  }
  std::env::var( "HOME" )
    .map_or_else(
      | _ | std::path::PathBuf::from( ".clr" ).join( "ps" ),
      | h | std::path::PathBuf::from( h ).join( ".clr" ).join( "ps" ),
    )
}

fn snapshot_path() -> std::path::PathBuf
{
  ps_state_dir().join( "last_snapshot.json" )
}

/// Read the previous `clr ps` snapshot, if one exists.
///
/// Best-effort: a missing file, an unreadable file, or malformed JSON all
/// resolve to `None` — the very first `clr ps` invocation (or state-dir loss)
/// must never crash or block the command; it just means no prior state.
#[ must_use ]
pub( crate ) fn read_snapshot() -> Option< PsSnapshot >
{
  let content = std::fs::read_to_string( snapshot_path() ).ok()?;
  serde_json::from_str( &content ).ok()
}

// Build a snapshot entry for one currently-live process.
fn build_entry( proc : &ProcessInfo ) -> PsSnapshotEntry
{
  #[ cfg( target_os = "linux" ) ]
  let started_at = claude_core::process::read_process_metrics( proc.pid ).map( | m | m.started_at );
  #[ cfg( not( target_os = "linux" ) ) ]
  let started_at : Option< u64 > = None;

  PsSnapshotEntry
  {
    started_at,
    path : proc.cwd.display().to_string(),
    task : resolve_task( proc ),
  }
}

/// Write the current process list as the new `clr ps` snapshot, for the
/// *next* invocation's 🆕/Ended comparison.
///
/// Best-effort: I/O failures (unwritable state dir, full disk) must never
/// abort `clr ps` itself — the command's own table output is already
/// complete by the time this runs. Writes via a temp file + rename so a
/// concurrent reader never observes a partially-written snapshot.
pub( crate ) fn write_snapshot( procs : &[ ProcessInfo ] )
{
  let sessions : BTreeMap< String, PsSnapshotEntry > = procs.iter()
    .map( | p | ( p.pid.to_string(), build_entry( p ) ) )
    .collect();
  let snapshot = PsSnapshot { captured_at : unix_now(), sessions };

  let Ok( json ) = serde_json::to_string_pretty( &snapshot ) else { return; };
  let dir = ps_state_dir();
  if std::fs::create_dir_all( &dir ).is_err() { return; }
  let tmp_path = dir.join( "last_snapshot.json.tmp" );
  if std::fs::write( &tmp_path, json ).is_err() { return; }
  let _ = std::fs::rename( &tmp_path, snapshot_path() );
}

/// Build the "Ended Since Last Check" table: sessions present in `prior` but
/// absent from the current `procs` list.
///
/// Returns `None` when there is no prior snapshot to compare against (`clr
/// ps`'s very first invocation) or when nothing has ended since. PID-ascending
/// order — unlike the Active table's oldest-first, there is no finer-grained
/// "when did it end" signal available (only one prior snapshot is ever
/// retained), so PID order mirrors the Queued table's own "not naturally
/// orderable by staleness" convention.
#[ must_use ]
pub( crate ) fn build_ended_table( procs : &[ ProcessInfo ], prior : Option< &PsSnapshot > ) -> Option< String >
{
  let prior = prior?;
  let live_pids : HashSet< u32 > = procs.iter().map( | p | p.pid ).collect();

  let mut ended : Vec< ( u32, &PsSnapshotEntry ) > = prior.sessions.iter()
    .filter_map( | ( pid_str, entry ) |
    {
      let pid : u32 = pid_str.parse().ok()?;
      if live_pids.contains( &pid ) { None } else { Some( ( pid, entry ) ) }
    } )
    .collect();

  if ended.is_empty() { return None; }
  ended.sort_by_key( | ( pid, _ ) | *pid );

  let headers = vec![
    "#".to_string(), "PID".to_string(), "Ran for".to_string(),
    "Absolute Path".to_string(), "Task".to_string(),
  ];
  let mut builder = RowBuilder::new( headers );
  for ( idx, ( pid, entry ) ) in ended.iter().enumerate()
  {
    let ran_for = entry.started_at
      .map_or_else( || "-".to_string(), | s | duration_label( prior.captured_at.saturating_sub( s ) ) );
    let row = vec![
      ( idx + 1 ).to_string(),
      pid.to_string(),
      ran_for,
      shorten_path( &entry.path ),
      entry.task.clone(),
    ];
    builder = builder.add_row( row.into_iter().map( Into::into ).collect() );
  }

  let count = ended.len();
  let ago   = duration_label( unix_now().saturating_sub( prior.captured_at ) );
  let heading = Heading::new( "Ended Since Last Check" )
    .with_field( format!( "{count} ended (last check {ago} ago)" ) );
  Some( render_headed_table( builder, heading ) )
}
