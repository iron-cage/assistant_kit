//! Daily JSONL file rotation — UTC-date-based filename generation and retention pruning.
//!
//! The `JournalWriter` never deletes files; deletion lives here exclusively, invoked
//! explicitly by consumers (`clj .prune`, the runner's once-daily auto-prune). Age is
//! derived from the `YYYY-MM-DD.jsonl` filename — never filesystem metadata — so
//! copies and restores cannot change what gets pruned. Files whose names do not match
//! the pattern exactly are invisible to pruning: never listed, never deleted.

use chrono::{ Datelike, Utc };
use std::path::{ Path, PathBuf };

/// Return the JSONL filename for the given UTC year/month/day.
///
/// Format: `YYYY-MM-DD.jsonl`
#[ inline ]
#[ must_use ]
pub fn date_filename( year : i32, month : u32, day : u32 ) -> String
{
  format!( "{year:04}-{month:02}-{day:02}.jsonl" )
}

/// Return the JSONL filename for today's UTC date.
///
/// Equivalent to `date_filename` called with the current UTC year/month/day.
#[ inline ]
#[ must_use ]
pub fn today_filename() -> String
{
  let ( year, month, day ) = today_ymd();
  date_filename( year, month, day )
}

/// Return today's UTC date as `( year, month, day )`.
///
/// The date pruning cutoffs are computed against — callers pass it to
/// [`prune_by_age`], keeping that function deterministic and testable.
#[ inline ]
#[ must_use ]
pub fn today_ymd() -> ( i32, u32, u32 )
{
  let now = Utc::now();
  ( now.year(), now.month(), now.day() )
}

/// Parse a `YYYY-MM-DD.jsonl` filename into `( year, month, day )`.
///
/// Strict inverse of [`date_filename`]: exact shape (4-2-2 digits, dashes,
/// `.jsonl` suffix) AND a calendar-valid date. Anything else — other extensions,
/// extra characters, out-of-range months/days — returns `None`, which is what
/// keeps [`list_journal_files`] and [`prune_by_age`] blind to non-journal files.
#[ inline ]
#[ must_use ]
pub fn parse_date_filename( name : &str ) -> Option< ( i32, u32, u32 ) >
{
  let stem  = name.strip_suffix( ".jsonl" )?;
  let bytes = stem.as_bytes();
  if bytes.len() != 10 || bytes[ 4 ] != b'-' || bytes[ 7 ] != b'-' { return None; }
  let all_digits = | s : &str | !s.is_empty() && s.bytes().all( | b | b.is_ascii_digit() );
  if !( all_digits( &stem[ ..4 ] ) && all_digits( &stem[ 5..7 ] ) && all_digits( &stem[ 8..10 ] ) )
  {
    return None;
  }
  let year  : i32 = stem[ ..4 ].parse().ok()?;
  let month : u32 = stem[ 5..7 ].parse().ok()?;
  let day   : u32 = stem[ 8..10 ].parse().ok()?;
  chrono::NaiveDate::from_ymd_opt( year, month, day )?;
  Some( ( year, month, day ) )
}

/// List journal rotation files in `dir`, sorted by date ascending (oldest first).
///
/// Only names matching [`parse_date_filename`]'s strict pattern are returned;
/// everything else in the directory is ignored. A missing or unreadable
/// directory yields an empty list — listing is a read-only query and must not
/// invent an error path the caller can't act on.
#[ inline ]
#[ must_use ]
pub fn list_journal_files( dir : &Path ) -> Vec< ( PathBuf, ( i32, u32, u32 ) ) >
{
  let Ok( entries ) = std::fs::read_dir( dir ) else { return Vec::new(); };
  let mut files : Vec< ( PathBuf, ( i32, u32, u32 ) ) > = entries
    .flatten()
    .filter_map( | entry |
    {
      let name = entry.file_name();
      let date = parse_date_filename( name.to_str()? )?;
      Some( ( entry.path(), date ) )
    } )
    .collect();
  files.sort_by_key( | ( _, date ) | *date );
  files
}

/// Outcome of one file considered by [`prune_by_age`].
#[ derive( Debug, Clone, PartialEq, Eq ) ]
pub enum PruneAction
{
  /// File was deleted.
  Deleted,
  /// File would be deleted, but `dry_run` was set — nothing touched.
  WouldDelete,
  /// Deletion was attempted and failed; carries the OS error text.
  Failed( String ),
}

/// Delete journal files dated strictly before `today - keep_days` (UTC dates).
///
/// Age comes from the filename date, never filesystem metadata. Today's file is
/// structurally safe: the cutoff is at most `today`, and only strictly-older
/// dates qualify — even `keep_days = 0` deletes nothing dated today. Deletion
/// is best-effort per file: one failure is reported as [`PruneAction::Failed`]
/// and the sweep continues. With `dry_run`, qualifying files are reported as
/// [`PruneAction::WouldDelete`] and nothing is touched. Returns one entry per
/// qualifying file; an empty vec means nothing was old enough (or the
/// directory does not exist).
///
/// # Panics
///
/// Panics when `today` is not a valid calendar date — callers obtain it from
/// [`today_ymd`], so a bad tuple is a programmer error worth failing loudly on.
#[ inline ]
#[ must_use = "the report says what was deleted or failed — discard it explicitly if you don't care" ]
pub fn prune_by_age(
  dir       : &Path,
  keep_days : u32,
  today     : ( i32, u32, u32 ),
  dry_run   : bool,
) -> Vec< ( PathBuf, PruneAction ) >
{
  let ( year, month, day ) = today;
  let today_date = chrono::NaiveDate::from_ymd_opt( year, month, day )
    .expect( "prune_by_age: `today` is not a valid calendar date" );
  // A window reaching past representable time keeps everything.
  let Some( cutoff ) = today_date.checked_sub_days( chrono::Days::new( u64::from( keep_days ) ) )
  else { return Vec::new(); };
  let mut report = Vec::new();
  for ( path, ( y, m, d ) ) in list_journal_files( dir )
  {
    // Dates from parse_date_filename are always calendar-valid.
    let Some( date ) = chrono::NaiveDate::from_ymd_opt( y, m, d ) else { continue; };
    if date >= cutoff { continue; }
    if dry_run
    {
      report.push( ( path, PruneAction::WouldDelete ) );
    }
    else
    {
      match std::fs::remove_file( &path )
      {
        Ok( () )  => report.push( ( path, PruneAction::Deleted ) ),
        Err( e )  => report.push( ( path, PruneAction::Failed( e.to_string() ) ) ),
      }
    }
  }
  report
}
