//! Journal reader — query and tail events from daily JSONL files.

use core::time::Duration;
use std::{ fs, io, io::BufRead, path::{ Path, PathBuf }, time::SystemTime };
use crate::event::{ EventRecord, EventType };

/// Filter applied by [`JournalReader::query`] and [`JournalReader::tail`].
///
/// All non-`None` fields are AND-combined: every field must match for an event
/// to be included in the result. `None` means "no constraint on this field."
#[ derive( Debug, Clone, Default ) ]
pub struct JournalFilter
{
  /// Include only events whose timestamp is within this duration of now.
  ///
  /// E.g. `Some(Duration::from_secs(3600))` = "events from the last hour".
  pub since      : Option< Duration >,
  /// Include only events whose timestamp is at or before this instant.
  pub until      : Option< SystemTime >,
  /// Include only events of this type (exact match).
  pub event_type : Option< EventType >,
  /// Include only events where `fields.command` equals this string (exact match).
  pub command    : Option< String >,
  /// Include only events where `fields.exit_code` equals this value.
  pub exit_code  : Option< i32 >,
  /// Include only events where `fields.model` contains this substring (case-sensitive).
  pub model      : Option< String >,
  /// Include only events where `fields.dir` contains this substring (case-sensitive).
  pub dir        : Option< String >,
  /// Include only events where `fields.creds` contains this substring (case-sensitive).
  pub creds      : Option< String >,
  /// Stop after collecting this many matching events.
  pub limit      : Option< usize >,
}

/// Read-side API for querying and tailing journal events.
///
/// Opens a journal directory and iterates over daily JSONL files in
/// chronological order, applying a [`JournalFilter`] to select matching events.
#[ derive( Debug ) ]
pub struct JournalReader
{
  dir : PathBuf,
}

impl JournalReader
{
  /// Open a journal directory for reading.
  ///
  /// Infallible — missing directory is detected only on first [`JournalReader::query`]
  /// or [`JournalReader::tail`] call, which returns empty results rather than an error.
  #[ inline ]
  #[ must_use ]
  pub fn open( dir : PathBuf ) -> Self
  {
    Self { dir }
  }

  /// Return all matching events in chronological order (oldest first).
  ///
  /// Reads only daily files whose date falls within the `since`/`until` range.
  /// Skips lines that fail JSON parsing (crash-safety — see `docs/invariant/002_crash_safety.md`).
  /// Applies all non-`None` filter fields as AND conditions.
  /// Stops after `filter.limit` matches if set.
  #[ inline ]
  #[ must_use ]
  pub fn query( &self, filter : &JournalFilter ) -> Vec< EventRecord >
  {
    let mut results = Vec::new();
    let now = SystemTime::now();
    let since_cutoff : Option< SystemTime > = filter.since.map( | d | now - d );

    let mut files = match collect_jsonl_files( &self.dir )
    {
      Ok( f ) => f,
      Err( _ ) => return results,
    };
    files.sort();

    for path in &files
    {
      // Skip files whose date range cannot contain any matching events.
      let stem = file_stem( path );
      if !file_date_in_range( &stem, since_cutoff.as_ref(), filter.until.as_ref() )
      {
        continue;
      }

      let lines = match read_lines( path )
      {
        Ok( l ) => l,
        Err( _ ) => continue,
      };

      for line in lines
      {
        if let Some( limit ) = filter.limit
        {
          if results.len() >= limit { return results; }
        }
        let event : EventRecord = match serde_json::from_str( &line )
        {
          Ok( e ) => e,
          Err( _ ) => continue, // skip malformed lines
        };
        if event_matches( &event, filter, since_cutoff.as_ref() )
        {
          results.push( event );
        }
      }
    }
    results
  }

  /// Stream new events matching `filter` as they are appended to the journal.
  ///
  /// Polls the current UTC day's file for new lines at ~500 ms intervals.
  /// Rolls over to the next day's file at UTC midnight.
  /// Skips lines that fail JSON parsing.
  /// This is a blocking iterator — it yields events as they appear and does
  /// not return `None` until the [`TailIter`] is dropped.
  #[ inline ]
  #[ must_use ]
  pub fn tail< 'a >( &'a self, filter : &'a JournalFilter ) -> TailIter< 'a >
  {
    TailIter
    {
      dir    : &self.dir,
      filter,
      offset : 0,
      date   : crate::rotation::today_filename(),
    }
  }

  /// Count of `.jsonl` files in the journal directory.
  #[ inline ]
  #[ must_use ]
  pub fn file_count( &self ) -> usize
  {
    collect_jsonl_files( &self.dir ).map( | f | f.len() ).unwrap_or( 0 )
  }

  /// Total bytes across all `.jsonl` files.
  #[ inline ]
  #[ must_use ]
  pub fn total_bytes( &self ) -> u64
  {
    collect_jsonl_files( &self.dir )
      .unwrap_or_default()
      .iter()
      .filter_map( | p | fs::metadata( p ).ok() )
      .map( | m | m.len() )
      .sum()
  }

  /// Date string of the oldest journal file (from filename, not content).
  ///
  /// Returns `None` if the directory is absent or contains no `.jsonl` files.
  #[ inline ]
  #[ must_use ]
  pub fn oldest_date( &self ) -> Option< String >
  {
    let mut files = collect_jsonl_files( &self.dir ).ok()?;
    files.sort();
    files.first().map( | p | file_stem( p ) )
  }

  /// Date string of the newest journal file (from filename, not content).
  ///
  /// Returns `None` if the directory is absent or contains no `.jsonl` files.
  #[ inline ]
  #[ must_use ]
  pub fn newest_date( &self ) -> Option< String >
  {
    let mut files = collect_jsonl_files( &self.dir ).ok()?;
    files.sort();
    files.last().map( | p | file_stem( p ) )
  }
}

// ── helpers ───────────────────────────────────────────────────────────────────

/// Return all `.jsonl` file paths in `dir` (non-recursive).
fn collect_jsonl_files( dir : &Path ) -> io::Result< Vec< PathBuf > >
{
  let mut files = Vec::new();
  for entry in fs::read_dir( dir )?
  {
    let entry = entry?;
    let path = entry.path();
    if path.extension().and_then( | e | e.to_str() ) == Some( "jsonl" )
    {
      files.push( path );
    }
  }
  Ok( files )
}

/// Read all non-empty lines from a file.
fn read_lines( path : &Path ) -> io::Result< Vec< String > >
{
  let file = fs::File::open( path )?;
  let reader = io::BufReader::new( file );
  let lines = reader
    .lines()
    .filter_map( | l | l.ok() )
    .filter( | l | !l.trim().is_empty() )
    .collect();
  Ok( lines )
}

/// Extract the date stem from a `.jsonl` filename (e.g. `"2026-06-27"` from `"2026-06-27.jsonl"`).
fn file_stem( path : &Path ) -> String
{
  path
    .file_stem()
    .and_then( | s | s.to_str() )
    .unwrap_or( "" )
    .to_owned()
}

/// Parse a file date stem (`"YYYY-MM-DD"`) into a `SystemTime`.
///
/// Returns `None` if the stem is not in the expected format.
fn parse_file_date( stem : &str ) -> Option< SystemTime >
{
  use chrono::{ NaiveDate, TimeZone, Utc };
  let parts : Vec< &str > = stem.split( '-' ).collect();
  if parts.len() != 3 { return None; }
  let year  = parts[ 0 ].parse::< i32 >().ok()?;
  let month = parts[ 1 ].parse::< u32 >().ok()?;
  let day   = parts[ 2 ].parse::< u32 >().ok()?;
  let date = NaiveDate::from_ymd_opt( year, month, day )?;
  let dt = Utc.from_utc_datetime( &date.and_hms_opt( 0, 0, 0 )? );
  let secs = dt.timestamp();
  if secs < 0 { return None; } // pre-epoch dates are not valid journal file dates
  Some( SystemTime::UNIX_EPOCH + std::time::Duration::from_secs( secs as u64 ) )
}

/// Return `true` if a file (identified by its date stem) falls within the
/// `[since_cutoff, until]` range. Either bound may be `None` (unbounded).
fn file_date_in_range(
  stem         : &str,
  since_cutoff : Option< &SystemTime >,
  until        : Option< &SystemTime >,
) -> bool
{
  // End-of-day for this file is stem date + 86400s.
  let file_start = match parse_file_date( stem )
  {
    Some( t ) => t,
    None      => return true, // don't skip files we can't parse
  };
  let file_end = file_start + Duration::from_secs( 86400 );

  if let Some( cutoff ) = since_cutoff
  {
    // File must end after the since cutoff (file contains events newer than cutoff)
    if file_end <= *cutoff { return false; }
  }
  if let Some( unt ) = until
  {
    // File must start before the until bound
    if file_start > *unt { return false; }
  }
  true
}

/// Parse the timestamp from an event record into a `SystemTime`.
fn parse_event_time( ts : &str ) -> Option< SystemTime >
{
  use chrono::DateTime;
  let dt = DateTime::parse_from_rfc3339( ts ).ok()?;
  let secs = dt.timestamp();
  let nanos = dt.timestamp_subsec_nanos();
  if secs < 0 { return None; }
  Some( SystemTime::UNIX_EPOCH + Duration::new( secs as u64, nanos ) )
}

/// Return `true` if `event` matches all non-`None` fields in `filter`.
fn event_matches(
  event        : &EventRecord,
  filter       : &JournalFilter,
  since_cutoff : Option< &SystemTime >,
) -> bool
{
  // since / until timestamp checks
  if since_cutoff.is_some() || filter.until.is_some()
  {
    if let Some( event_time ) = parse_event_time( &event.ts )
    {
      if let Some( cutoff ) = since_cutoff
      {
        if event_time < *cutoff { return false; }
      }
      if let Some( ref until ) = filter.until
      {
        if event_time > *until { return false; }
      }
    }
  }

  if let Some( ref et ) = filter.event_type
  {
    if event.event_type != *et { return false; }
  }

  if let Some( ref cmd ) = filter.command
  {
    match &event.fields.command
    {
      Some( c ) if c == cmd => {},
      _ => return false,
    }
  }

  if let Some( code ) = filter.exit_code
  {
    match event.fields.exit_code
    {
      Some( c ) if c == code => {},
      _ => return false,
    }
  }

  if let Some( ref model ) = filter.model
  {
    match &event.fields.model
    {
      Some( m ) if m.contains( model.as_str() ) => {},
      _ => return false,
    }
  }

  if let Some( ref dir ) = filter.dir
  {
    match &event.fields.dir
    {
      Some( d ) if d.contains( dir.as_str() ) => {},
      _ => return false,
    }
  }

  if let Some( ref creds ) = filter.creds
  {
    match &event.fields.creds
    {
      Some( c ) if c.contains( creds.as_str() ) => {},
      _ => return false,
    }
  }

  true
}

// ── TailIter ──────────────────────────────────────────────────────────────────

/// Blocking iterator returned by [`JournalReader::tail`].
///
/// Polls the current UTC day's file for new lines at ~500 ms intervals.
/// Rolls over to the next day's file at UTC midnight.
pub struct TailIter< 'a >
{
  dir    : &'a Path,
  filter : &'a JournalFilter,
  /// Byte offset of the next unread position in the current file.
  offset : u64,
  /// Filename of the current day being tailed (e.g. `"2026-06-27.jsonl"`).
  date   : String,
}

impl< 'a > core::fmt::Debug for TailIter< 'a >
{
  #[ inline ]
  fn fmt( &self, f : &mut core::fmt::Formatter< '_ > ) -> core::fmt::Result
  {
    f.debug_struct( "TailIter" )
      .field( "dir", &self.dir )
      .field( "offset", &self.offset )
      .field( "date", &self.date )
      .finish()
  }
}

impl< 'a > Iterator for TailIter< 'a >
{
  type Item = EventRecord;

  #[ inline ]
  fn next( &mut self ) -> Option< Self::Item >
  {
    loop
    {
      // Detect UTC-day rollover.
      let today = crate::rotation::today_filename();
      if today != self.date
      {
        self.date   = today;
        self.offset = 0;
      }

      let path = self.dir.join( &self.date );
      if let Ok( mut file ) = fs::File::open( &path )
      {
        use io::{ Read, Seek, SeekFrom };
        let meta = file.metadata().ok();
        let size = meta.map( | m | m.len() ).unwrap_or( 0 );
        if size > self.offset
        {
          if file.seek( SeekFrom::Start( self.offset ) ).is_ok()
          {
            let mut buf = String::new();
            let _ = file.read_to_string( &mut buf );
            self.offset = size;
            for line in buf.lines()
            {
              let line = line.trim();
              if line.is_empty() { continue; }
              if let Ok( event ) = serde_json::from_str::< EventRecord >( line )
              {
                if event_matches( &event, self.filter, None )
                {
                  return Some( event );
                }
              }
            }
          }
        }
      }

      std::thread::sleep( Duration::from_millis( 500 ) );
    }
  }
}
