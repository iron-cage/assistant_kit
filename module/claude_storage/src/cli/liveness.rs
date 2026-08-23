//! Session liveness — which conversations have a Claude Code process attached.
//!
//! `.projects` sorts by write recency, but recency is not liveness. A session
//! whose terminal sits open and idle can stay quiet for the better part of an
//! hour, while a session that ended thirty seconds ago still looks freshest.
//! Answering "is anything running right now, and is it mid-turn" needs signals
//! the storage tree does not carry.
//!
//! ## Signals
//!
//! Claude Code writes no per-session lock file, does not hold the session JSONL
//! open between appends, and does not carry the session id in its process
//! environment — so liveness is inferred, from two signals it does leave behind:
//!
//! 1. **Attached processes.** The main `claude` process keeps its working
//!    directory at the session's project root for the life of the session, so
//!    every `/proc/<pid>/cwd` under a process named `claude` names a live
//!    project. Authoritative for *whether* a project is live; silent about
//!    *which* of its conversations.
//! 2. **`~/.claude/history.jsonl`.** One record per submitted prompt, carrying
//!    `{ project, sessionId, timestamp }` — where `project` is the *unencoded*
//!    path. The newest record for a project names the exact session receiving
//!    input, which is what turns project-level liveness into session-level.
//!
//! ## Known Pitfalls
//!
//! ### Write recency is not a liveness substitute
//!
//! **Issue**: An mtime cutoff looks like it would answer the same question far
//! more cheaply, and it does not. Measured against the process table on a store
//! of 914 projects with 38 live sessions, 11 of those 38 had been idle longer
//! than five minutes and 8 longer than fifteen — a five-minute "recently
//! written means active" rule mislabels 29% of genuinely live sessions as dead,
//! and mislabels every just-ended session as alive.
//!
//! **Solution**: Recency and liveness are rendered as separate facts. `LAST`
//! keeps carrying mtime; `STATUS` carries process attachment. Recency is
//! consulted only to split an already-attached session into
//! [`Liveness::Working`] and [`Liveness::Waiting`].
//!
//! **Prevention**: Never widen [`WORKING_WINDOW`] into a liveness test of its
//! own — outside an attached project it proves nothing.
//!
//! ### Detection never claims a negative
//!
//! **Issue**: The process table is Linux-only, and inside a container it lists
//! the container's processes rather than the host's. In both cases every
//! project reads as not-attached, which is indistinguishable from a store where
//! nothing happens to be running.
//!
//! **Solution**: Absence is never rendered. The `STATUS` column appears only
//! when at least one live process was actually found ([`LivenessMap::any_attached`]),
//! so a blank column means "nothing detected" and never asserts "nothing live".
//! `live::` refuses rather than silently filtering everything away.
//!
//! **Prevention**: Any new consumer must branch on [`LivenessMap::any_attached`]
//! before presenting attachment state as information.

use core::time::Duration;
use std::collections::HashMap;
use std::io::{ Read, Seek, SeekFrom };
use std::path::Path;
use std::time::SystemTime;

use claude_storage_core::JsonValue;
use super::scope::decode_project_display;

// ─── constants ─────────────────────────────────────────────────────────────

/// An attached session that wrote inside this window is mid-turn, not waiting.
///
/// Sized to outlast a slow model response but not a human reading the output.
const WORKING_WINDOW : Duration = Duration::from_secs( 60 );

/// Bytes read from the tail of `history.jsonl`.
///
/// The file grows without bound (megabytes within weeks) and only its newest
/// records can name a live session, so it is never read whole.
const HISTORY_TAIL_BYTES : u64 = 512 * 1024;

/// Process name Claude Code's main process reports in `/proc/<pid>/comm`.
const PROCESS_NAME : &str = "claude";

/// Default process table.
const PROC_DIR : &str = "/proc";

/// Path of the prompt history relative to `$HOME`.
const HISTORY_RELATIVE : &str = ".claude/history.jsonl";

// ─── state ─────────────────────────────────────────────────────────────────

/// Attachment state of one conversation or project.
///
/// Only ever constructed for something with a live process attached — the
/// absence of a `Liveness` is the "no process" case, not a third variant.
#[ derive( Debug, Clone, Copy, PartialEq, Eq ) ]
pub( super ) enum Liveness
{
  /// A process is attached and wrote within [`WORKING_WINDOW`] — mid-turn.
  Working,
  /// A process is attached and has gone quiet — waiting on the user.
  Waiting,
}

impl Liveness
{
  /// Rendered cell text, glyph included.
  pub( super ) fn label( self ) -> &'static str
  {
    match self
    {
      Self::Working => "● working",
      Self::Waiting => "○ waiting",
    }
  }

  /// Width of the widest label, for sizing a column before its cells exist.
  pub( super ) fn column_width() -> usize
  {
    Self::Working.label().chars().count().max( Self::Waiting.label().chars().count() )
  }

  /// Split an attached row by write recency.
  ///
  /// An mtime ahead of the local clock — skew against an NFS or container host,
  /// a restored archive, a deliberate `touch -d` — is the freshest write there
  /// can be, not the oldest. `duration_since` signals it as `Err`, so folding
  /// that error in with "too old" would rank the newest possible write least
  /// active.
  fn from_mtime( mtime : SystemTime ) -> Self
  {
    let fresh = match SystemTime::now().duration_since( mtime )
    {
      Ok( age ) => age <= WORKING_WINDOW,
      Err( _ )  => true,
    };
    if fresh { Self::Working } else { Self::Waiting }
  }
}

// ─── detection ─────────────────────────────────────────────────────────────

/// Which projects have Claude Code processes attached, and which of their
/// conversations those processes are driving.
///
/// Keys are display paths in the same `~/…` form `.projects` renders, produced
/// by the same encode-then-decode round trip the rows go through — so an
/// ambiguous decode lands identically on both sides and always matches.
#[ derive( Debug, Default ) ]
pub( super ) struct LivenessMap
{
  /// Display path → number of attached processes.
  attached : HashMap< String, usize >,
  /// Display path → session ids receiving input, newest first.
  driving  : HashMap< String, Vec< String > >,
}

impl LivenessMap
{
  /// Probe the running system.
  ///
  /// Never fails: an unreadable process table or absent history yields an empty
  /// map, which reports nothing rather than reporting a negative.
  pub( super ) fn detect() -> Self
  {
    let history = std::env::var_os( "HOME" )
      .map( | home | Path::new( &home ).join( HISTORY_RELATIVE ) );
    Self::probe( Path::new( PROC_DIR ), history.as_deref() )
  }

  /// Probe an explicit process table and history file.
  ///
  /// Both are ordinary filesystem paths, so a test supplies a real directory
  /// laid out like `/proc` (numeric subdirectories holding a `comm` file and a
  /// `cwd` symlink) and a real history file, exercising the same reads and the
  /// same parser the live path uses.
  pub( super ) fn probe( proc_dir : &Path, history_path : Option< &Path > ) -> Self
  {
    let attached = read_attached( proc_dir );
    let driving = history_path
      .map( | p | read_driving( p, &attached ) )
      .unwrap_or_default();
    Self { attached, driving }
  }

  /// Whether any attached process was found at all.
  ///
  /// Gates every rendering decision: false means detection found nothing, which
  /// is not the same as "nothing is live" and must never be displayed as such.
  pub( super ) fn any_attached( &self ) -> bool
  {
    !self.attached.is_empty()
  }

  /// Attachment state of a whole project, or `None` when no process is attached.
  ///
  /// `last_mtime` is the project's newest session write — the same value the
  /// `LAST` column renders.
  pub( super ) fn project_state( &self, display_path : &str, last_mtime : SystemTime )
    -> Option< Liveness >
  {
    self.attached.get( display_path ).filter( | n | **n > 0 )?;
    Some( Liveness::from_mtime( last_mtime ) )
  }

  /// Attachment state of one conversation, or `None` when it is not the one
  /// being driven.
  ///
  /// `rank` is the session's position in its project's mtime-descending order
  /// (0 = most recent), used only for the headless fallback below.
  pub( super ) fn session_state(
    &self,
    display_path : &str,
    session_id   : &str,
    rank         : usize,
    mtime        : SystemTime,
  ) -> Option< Liveness >
  {
    let attached = *self.attached.get( display_path ).filter( | n | **n > 0 )?;

    // History names the driven session exactly and is preferred whenever it has
    // anything to say: the newest session by mtime is frequently *not* the live
    // one, precisely because a live session can idle longer than a dead one has
    // been dead. Sessions started headlessly (`--print`) never write history, so
    // for a project history knows nothing about, mtime rank is the only signal
    // left and the newest `attached` sessions are taken as the live ones.
    let driven = match self.driving.get( display_path )
    {
      Some( ids ) if !ids.is_empty() => ids.iter().any( | id | id == session_id ),
      _ => rank < attached,
    };
    if !driven { return None; }

    Some( Liveness::from_mtime( mtime ) )
  }
}

// ─── signal readers ────────────────────────────────────────────────────────

/// Display-path key for an absolute path, matching how `.projects` builds rows.
///
/// Encoding then decoding is deliberate rather than wasteful: rows carry a path
/// that was already decoded out of a storage directory name, and that decode is
/// lossy (`_` and `/` both encode to `-`). Putting the probe's path through the
/// identical round trip makes both sides agree on the same guess, so keys match
/// even where the guess is wrong.
fn display_key( path : &Path ) -> Option< String >
{
  let encoded = claude_storage_core::encode_path( path ).ok()?;
  Some( decode_project_display( &encoded ) )
}

/// Count attached `claude` processes per project.
fn read_attached( proc_dir : &Path ) -> HashMap< String, usize >
{
  let mut attached : HashMap< String, usize > = HashMap::new();
  let Ok( entries ) = std::fs::read_dir( proc_dir ) else { return attached };

  for entry in entries.flatten()
  {
    let name = entry.file_name();
    let Some( name ) = name.to_str() else { continue };
    if name.is_empty() || !name.bytes().all( | b | b.is_ascii_digit() ) { continue; }

    let dir = entry.path();
    // `comm` is the process name alone; matching it avoids the false positives a
    // command-line scan collects (wrappers, `grep claude`, this process itself).
    let Ok( comm ) = std::fs::read_to_string( dir.join( "comm" ) ) else { continue };
    if comm.trim_end() != PROCESS_NAME { continue; }

    let Ok( cwd ) = std::fs::read_link( dir.join( "cwd" ) ) else { continue };
    let Some( key ) = display_key( &cwd ) else { continue };
    *attached.entry( key ).or_insert( 0 ) += 1;
  }

  attached
}

/// Session ids currently receiving input, per attached project, newest first.
///
/// Only attached projects are recorded — a driven session in a project with no
/// process is a session that has since exited, and collecting it would cost a
/// key per project in the entire store for no gain.
fn read_driving( path : &Path, attached : &HashMap< String, usize > )
  -> HashMap< String, Vec< String > >
{
  let mut driving : HashMap< String, Vec< String > > = HashMap::new();
  if attached.is_empty() { return driving; }

  let Some( text ) = read_tail( path ) else { return driving };
  let mut remaining : usize = attached.values().sum();

  for line in text.lines().rev()
  {
    if remaining == 0 { break; }

    let Ok( record ) = claude_storage_core::parse_json( line ) else { continue };
    let Some( project ) = record.get( "project" ).and_then( JsonValue::as_str ) else { continue };
    let Some( session_id ) = record.get( "sessionId" ).and_then( JsonValue::as_str ) else { continue };

    let Some( key ) = display_key( Path::new( project ) ) else { continue };
    let Some( &want ) = attached.get( &key ) else { continue };

    let ids = driving.entry( key ).or_default();
    if ids.len() >= want { continue; }
    if ids.iter().any( | id | id == session_id ) { continue; }
    ids.push( session_id.to_string() );
    remaining -= 1;
  }

  driving
}

/// Read the last [`HISTORY_TAIL_BYTES`] of a file as whole lines.
///
/// A mid-file seek lands inside a record, so the leading fragment is dropped.
fn read_tail( path : &Path ) -> Option< String >
{
  let mut file = std::fs::File::open( path ).ok()?;
  let len = file.metadata().ok()?.len();
  let from = len.saturating_sub( HISTORY_TAIL_BYTES );
  file.seek( SeekFrom::Start( from ) ).ok()?;

  let mut buf = Vec::new();
  file.read_to_end( &mut buf ).ok()?;
  let text = String::from_utf8_lossy( &buf ).into_owned();

  if from == 0 { return Some( text ); }
  let first_break = text.find( '\n' )?;
  Some( text[ first_break + 1 .. ].to_string() )
}

// ─── tests ─────────────────────────────────────────────────────────────────

#[ cfg( test ) ]
mod liveness_tests
{
  use super::*;
  use core::time::Duration;

  /// Build a real `/proc`-shaped directory: one numeric subdirectory per
  /// process, each holding a `comm` file and a `cwd` symlink.
  ///
  /// This is the genuine article rather than a stand-in — `read_attached`
  /// performs the same `read_dir`/`read_to_string`/`read_link` calls it makes
  /// against the kernel's own filesystem.
  fn fake_proc( root : &Path, processes : &[ ( &str, &str, &Path ) ] )
  {
    for ( pid, comm, cwd ) in processes
    {
      let dir = root.join( pid );
      std::fs::create_dir_all( &dir ).unwrap();
      std::fs::write( dir.join( "comm" ), format!( "{comm}\n" ) ).unwrap();
      std::os::unix::fs::symlink( cwd, dir.join( "cwd" ) ).unwrap();
    }
  }

  fn history_line( project : &str, session_id : &str ) -> String
  {
    format!( r#"{{"display":"hi","project":"{project}","sessionId":"{session_id}","timestamp":"1"}}"# )
  }

  fn now() -> SystemTime { SystemTime::now() }
  fn ago( secs : u64 ) -> SystemTime { SystemTime::now() - Duration::from_secs( secs ) }
  fn ahead( secs : u64 ) -> SystemTime { SystemTime::now() + Duration::from_secs( secs ) }

  /// An empty probe reports nothing rather than reporting everything dead.
  #[ test ]
  fn test_absent_process_table_reports_nothing()
  {
    let tmp = tempfile::tempdir().unwrap();
    let map = LivenessMap::probe( &tmp.path().join( "no-such-proc" ), None );

    assert!( !map.any_attached(), "an unreadable process table must not claim knowledge" );
    assert_eq!( map.project_state( "~/anything", now() ), None );
  }

  /// A `claude` process' cwd marks its project attached; unrelated processes do not.
  #[ test ]
  fn test_attached_project_detected_from_process_cwd()
  {
    let tmp = tempfile::tempdir().unwrap();
    let proc_dir = tmp.path().join( "proc" );
    let live = tmp.path().join( "live_project" );
    let other = tmp.path().join( "other_project" );
    std::fs::create_dir_all( &live ).unwrap();
    std::fs::create_dir_all( &other ).unwrap();
    fake_proc( &proc_dir, &[ ( "101", "claude", &live ), ( "102", "bash", &other ) ] );

    let map = LivenessMap::probe( &proc_dir, None );
    let live_key = display_key( &live ).unwrap();
    let other_key = display_key( &other ).unwrap();

    assert!( map.any_attached() );
    assert!( map.project_state( &live_key, now() ).is_some(), "cwd of a claude process is live" );
    assert_eq!( map.project_state( &other_key, now() ), None, "a non-claude process must not mark a project" );
  }

  /// Recency splits an attached project into working and waiting — and, crucially,
  /// a long-idle attached project stays live rather than decaying to nothing.
  #[ test ]
  fn test_attached_project_splits_working_from_waiting()
  {
    let tmp = tempfile::tempdir().unwrap();
    let proc_dir = tmp.path().join( "proc" );
    let project = tmp.path().join( "project" );
    std::fs::create_dir_all( &project ).unwrap();
    fake_proc( &proc_dir, &[ ( "101", "claude", &project ) ] );

    let map = LivenessMap::probe( &proc_dir, None );
    let key = display_key( &project ).unwrap();

    assert_eq!( map.project_state( &key, now() ), Some( Liveness::Working ) );
    assert_eq!( map.project_state( &key, ago( 3_600 ) ), Some( Liveness::Waiting ),
      "an hour of silence with a process attached is waiting, never absent" );
  }

  /// An mtime ahead of the local clock is the freshest write there is, not the
  /// oldest.
  ///
  /// `duration_since` reports a future timestamp as `Err`, and the obvious
  /// reading of that error — "no measurable age, so not fresh" — inverts the
  /// answer: clock skew against an NFS or container host would make the one
  /// session being actively written the only one reported quiet.
  #[ test ]
  fn test_future_mtime_is_working_not_waiting()
  {
    let tmp = tempfile::tempdir().unwrap();
    let proc_dir = tmp.path().join( "proc" );
    let project = tmp.path().join( "project" );
    std::fs::create_dir_all( &project ).unwrap();
    fake_proc( &proc_dir, &[ ( "101", "claude", &project ) ] );

    let map = LivenessMap::probe( &proc_dir, None );
    let key = display_key( &project ).unwrap();

    assert_eq!( map.project_state( &key, ahead( 5 ) ), Some( Liveness::Working ),
      "a few seconds of skew must not read as an idle terminal" );
    assert_eq!( map.project_state( &key, ahead( 86_400 ) ), Some( Liveness::Working ),
      "and neither must a wholly wrong clock — the direction of the error is what matters" );
  }

  /// History pins the driven session even when it is not the newest by mtime —
  /// the case a recency heuristic gets backwards.
  #[ test ]
  fn test_history_pins_driven_session_over_newer_sibling()
  {
    let tmp = tempfile::tempdir().unwrap();
    let proc_dir = tmp.path().join( "proc" );
    let project = tmp.path().join( "project" );
    std::fs::create_dir_all( &project ).unwrap();
    fake_proc( &proc_dir, &[ ( "101", "claude", &project ) ] );

    let history = tmp.path().join( "history.jsonl" );
    std::fs::write( &history, format!( "{}\n", history_line( project.to_str().unwrap(), "driven-id" ) ) ).unwrap();

    let map = LivenessMap::probe( &proc_dir, Some( &history ) );
    let key = display_key( &project ).unwrap();

    // rank 1 — an older session by mtime, yet the one actually driven.
    assert_eq!( map.session_state( &key, "driven-id", 1, ago( 3_000 ) ), Some( Liveness::Waiting ) );
    // rank 0 — the newest session, but history says it is not the live one.
    assert_eq!( map.session_state( &key, "newer-id", 0, now() ), None );
  }

  /// With no history record (a headless `--print` session), the newest session
  /// by mtime stands in, bounded by the attached process count.
  #[ test ]
  fn test_missing_history_falls_back_to_mtime_rank()
  {
    let tmp = tempfile::tempdir().unwrap();
    let proc_dir = tmp.path().join( "proc" );
    let project = tmp.path().join( "project" );
    std::fs::create_dir_all( &project ).unwrap();
    fake_proc( &proc_dir, &[ ( "101", "claude", &project ) ] );

    let map = LivenessMap::probe( &proc_dir, None );
    let key = display_key( &project ).unwrap();

    assert_eq!( map.session_state( &key, "any-id", 0, now() ), Some( Liveness::Working ) );
    assert_eq!( map.session_state( &key, "any-id", 1, now() ), None,
      "only as many sessions as there are processes may be called live" );
  }

  /// Two processes in one project mark two driven sessions, not one.
  #[ test ]
  fn test_two_processes_drive_two_sessions()
  {
    let tmp = tempfile::tempdir().unwrap();
    let proc_dir = tmp.path().join( "proc" );
    let project = tmp.path().join( "project" );
    std::fs::create_dir_all( &project ).unwrap();
    fake_proc( &proc_dir, &[ ( "101", "claude", &project ), ( "102", "claude", &project ) ] );

    let path = project.to_str().unwrap();
    let history = tmp.path().join( "history.jsonl" );
    std::fs::write(
      &history,
      format!( "{}\n{}\n{}\n",
        history_line( path, "oldest-id" ),
        history_line( path, "second-id" ),
        history_line( path, "newest-id" ) ),
    ).unwrap();

    let map = LivenessMap::probe( &proc_dir, Some( &history ) );
    let key = display_key( &project ).unwrap();

    assert!( map.session_state( &key, "newest-id", 0, now() ).is_some() );
    assert!( map.session_state( &key, "second-id", 1, now() ).is_some() );
    assert_eq!( map.session_state( &key, "oldest-id", 2, now() ), None,
      "history is read newest-first and capped at the attached process count" );
  }

  /// History for a project with no attached process is ignored entirely.
  #[ test ]
  fn test_history_without_attached_process_is_ignored()
  {
    let tmp = tempfile::tempdir().unwrap();
    let proc_dir = tmp.path().join( "proc" );
    let attached = tmp.path().join( "attached" );
    let exited = tmp.path().join( "exited" );
    std::fs::create_dir_all( &attached ).unwrap();
    std::fs::create_dir_all( &exited ).unwrap();
    fake_proc( &proc_dir, &[ ( "101", "claude", &attached ) ] );

    let history = tmp.path().join( "history.jsonl" );
    std::fs::write( &history, format!( "{}\n", history_line( exited.to_str().unwrap(), "ghost-id" ) ) ).unwrap();

    let map = LivenessMap::probe( &proc_dir, Some( &history ) );
    let exited_key = display_key( &exited ).unwrap();

    assert_eq!( map.session_state( &exited_key, "ghost-id", 0, now() ), None );
  }

  /// A malformed history line is skipped without discarding the records around it.
  #[ test ]
  fn test_malformed_history_line_is_skipped()
  {
    let tmp = tempfile::tempdir().unwrap();
    let proc_dir = tmp.path().join( "proc" );
    let project = tmp.path().join( "project" );
    std::fs::create_dir_all( &project ).unwrap();
    fake_proc( &proc_dir, &[ ( "101", "claude", &project ) ] );

    let history = tmp.path().join( "history.jsonl" );
    std::fs::write(
      &history,
      format!( "not json at all\n{{\"partial\":true}}\n{}\n", history_line( project.to_str().unwrap(), "good-id" ) ),
    ).unwrap();

    let map = LivenessMap::probe( &proc_dir, Some( &history ) );
    let key = display_key( &project ).unwrap();

    assert!( map.session_state( &key, "good-id", 5, now() ).is_some(),
      "a valid record must survive malformed neighbours" );
  }

  /// Labels stay in step with the width the column reserves for them.
  #[ test ]
  fn test_labels_fit_the_reserved_column_width()
  {
    let width = Liveness::column_width();
    for state in [ Liveness::Working, Liveness::Waiting ]
    {
      assert!( state.label().chars().count() <= width, "{} overflows reserved width", state.label() );
    }
  }
}
