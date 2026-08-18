//! Rotation retention tests (RT-1..RT-12) — `parse_date_filename`, `list_journal_files`,
//! `prune_by_age` (`docs/feature/003_rotation.md` AC-001..AC-006).
//!
//! All pruning tests inject a fixed `today` tuple, so nothing here depends on the
//! wall clock — no UTC-midnight flakiness by construction.

use claude_journal::rotation::{ list_journal_files, parse_date_filename, prune_by_age, PruneAction };
use std::path::Path;

/// Create an empty journal file named for the given date inside `dir`.
fn touch_dated( dir : &Path, name : &str )
{
  std::fs::write( dir.join( name ), "{}\n" ).expect( "write fixture journal file" );
}

// ── parse_date_filename ───────────────────────────────────────────────────────

/// RT-1: strict round-trip — a `date_filename` output parses back to its inputs.
#[ test ]
fn rt1_parse_date_filename_round_trip()
{
  let name = claude_journal::rotation::date_filename( 2026, 8, 18 );
  assert_eq!( parse_date_filename( &name ), Some( ( 2026, 8, 18 ) ) );
}

/// RT-2: non-matching shapes are rejected — wrong extension, extra characters,
/// non-digits, and calendar-invalid dates all yield `None` (AC-004's foundation).
#[ test ]
fn rt2_parse_date_filename_rejects_non_matching()
{
  for bad in
  [
    "2026-08-18.json",      // wrong extension
    "2026-08-18.jsonl.bak", // trailing junk
    "x2026-08-18.jsonl",    // leading junk
    "2026-8-18.jsonl",      // unpadded month
    "2026_08_18.jsonl",     // wrong separators
    "20260818.jsonl",       // no separators
    "2026-13-01.jsonl",     // invalid month
    "2026-02-30.jsonl",     // invalid day
    "-last_prune",          // runner stamp file
    "notes.jsonl",          // non-date jsonl
  ]
  {
    assert_eq!( parse_date_filename( bad ), None, "must reject {bad:?}" );
  }
}

// ── list_journal_files ────────────────────────────────────────────────────────

/// RT-3 (AC-001): listing returns only pattern-matching files, sorted oldest first.
#[ test ]
fn rt3_list_journal_files_sorted_and_filtered()
{
  let dir = tempfile::TempDir::new().expect( "tmpdir" );
  touch_dated( dir.path(), "2026-01-05.jsonl" );
  touch_dated( dir.path(), "2025-12-31.jsonl" );
  touch_dated( dir.path(), "2026-01-04.jsonl" );
  touch_dated( dir.path(), "notes.jsonl" );
  touch_dated( dir.path(), "2026-02-30.jsonl" ); // calendar-invalid → ignored

  let files = list_journal_files( dir.path() );
  let dates : Vec< ( i32, u32, u32 ) > = files.iter().map( | ( _, d ) | *d ).collect();
  assert_eq!(
    dates,
    vec![ ( 2025, 12, 31 ), ( 2026, 1, 4 ), ( 2026, 1, 5 ) ],
    "must list only valid date-pattern files, ascending",
  );
}

/// RT-4 (AC-005): a nonexistent directory lists as empty — no error, no panic.
#[ test ]
fn rt4_list_journal_files_missing_dir_empty()
{
  assert!( list_journal_files( Path::new( "/nonexistent/journal/dir" ) ).is_empty() );
}

// ── prune_by_age ──────────────────────────────────────────────────────────────

/// RT-5 (AC-002): files dated strictly before `today - keep_days` are deleted;
/// files on or after the cutoff survive.
#[ test ]
fn rt5_prune_by_age_deletes_only_older_than_cutoff()
{
  let dir = tempfile::TempDir::new().expect( "tmpdir" );
  touch_dated( dir.path(), "2026-08-10.jsonl" ); // 8 days before today → prune (keep 7)
  touch_dated( dir.path(), "2026-08-11.jsonl" ); // exactly cutoff → keep
  touch_dated( dir.path(), "2026-08-15.jsonl" ); // inside window → keep
  touch_dated( dir.path(), "2026-08-18.jsonl" ); // today → keep

  let report = prune_by_age( dir.path(), 7, ( 2026, 8, 18 ), false );
  assert_eq!( report.len(), 1, "exactly one file qualifies, got: {report:?}" );
  assert!( report[ 0 ].0.ends_with( "2026-08-10.jsonl" ) );
  assert_eq!( report[ 0 ].1, PruneAction::Deleted );
  assert!( !dir.path().join( "2026-08-10.jsonl" ).exists(), "qualifying file must be gone" );
  assert!( dir.path().join( "2026-08-11.jsonl" ).exists(), "cutoff-day file must survive" );
  assert!( dir.path().join( "2026-08-15.jsonl" ).exists() );
  assert!( dir.path().join( "2026-08-18.jsonl" ).exists() );
}

/// RT-6 (AC-006): `keep_days = 0` deletes everything dated before today — but
/// never today's file, structurally (cutoff = today, comparison is strict `<`).
#[ test ]
fn rt6_prune_by_age_zero_keep_never_deletes_today()
{
  let dir = tempfile::TempDir::new().expect( "tmpdir" );
  touch_dated( dir.path(), "2026-08-17.jsonl" );
  touch_dated( dir.path(), "2026-08-18.jsonl" );

  let report = prune_by_age( dir.path(), 0, ( 2026, 8, 18 ), false );
  assert_eq!( report.len(), 1 );
  assert!( !dir.path().join( "2026-08-17.jsonl" ).exists() );
  assert!( dir.path().join( "2026-08-18.jsonl" ).exists(), "today's file must never be pruned" );
}

/// RT-7 (AC-004): non-matching filenames are untouchable — even a `.jsonl` file
/// whose name isn't a valid date survives every prune.
#[ test ]
fn rt7_prune_by_age_skips_non_matching_files()
{
  let dir = tempfile::TempDir::new().expect( "tmpdir" );
  touch_dated( dir.path(), "notes.jsonl" );
  touch_dated( dir.path(), "-last_prune" );
  touch_dated( dir.path(), "2026-02-30.jsonl" );

  let report = prune_by_age( dir.path(), 0, ( 2026, 8, 18 ), false );
  assert!( report.is_empty(), "no pattern-matching file qualifies, got: {report:?}" );
  assert!( dir.path().join( "notes.jsonl" ).exists() );
  assert!( dir.path().join( "-last_prune" ).exists() );
  assert!( dir.path().join( "2026-02-30.jsonl" ).exists() );
}

/// RT-8 (AC-005): pruning a nonexistent directory reports nothing and does not error.
#[ test ]
fn rt8_prune_by_age_missing_dir_reports_nothing()
{
  assert!( prune_by_age( Path::new( "/nonexistent/journal/dir" ), 30, ( 2026, 8, 18 ), false ).is_empty() );
}

/// RT-9: `dry_run` reports `WouldDelete` for qualifying files and touches nothing.
#[ test ]
fn rt9_prune_by_age_dry_run_touches_nothing()
{
  let dir = tempfile::TempDir::new().expect( "tmpdir" );
  touch_dated( dir.path(), "2020-01-01.jsonl" );

  let report = prune_by_age( dir.path(), 30, ( 2026, 8, 18 ), true );
  assert_eq!( report.len(), 1 );
  assert_eq!( report[ 0 ].1, PruneAction::WouldDelete );
  assert!( dir.path().join( "2020-01-01.jsonl" ).exists(), "dry_run must not delete" );
}

/// RT-10: the cutoff is a true date subtraction — a keep window crossing a month
/// boundary prunes by calendar distance, not by same-month day arithmetic.
#[ test ]
fn rt10_prune_by_age_month_boundary_cutoff()
{
  let dir = tempfile::TempDir::new().expect( "tmpdir" );
  touch_dated( dir.path(), "2026-07-28.jsonl" ); // 5 days before 2026-08-02 → prune (keep 3)
  touch_dated( dir.path(), "2026-07-30.jsonl" ); // exactly cutoff → keep

  let report = prune_by_age( dir.path(), 3, ( 2026, 8, 2 ), false );
  assert_eq!( report.len(), 1, "got: {report:?}" );
  assert!( !dir.path().join( "2026-07-28.jsonl" ).exists() );
  assert!( dir.path().join( "2026-07-30.jsonl" ).exists() );
}

/// RT-11: a keep window larger than representable time keeps everything.
#[ test ]
fn rt11_prune_by_age_giant_keep_window_keeps_all()
{
  let dir = tempfile::TempDir::new().expect( "tmpdir" );
  touch_dated( dir.path(), "2020-01-01.jsonl" );

  let report = prune_by_age( dir.path(), u32::MAX, ( 2026, 8, 18 ), false );
  assert!( report.is_empty() );
  assert!( dir.path().join( "2020-01-01.jsonl" ).exists() );
}

/// RT-12: `today_ymd` agrees with `today_filename` — the two public "today"
/// views can never disagree on the rotation date.
#[ test ]
fn rt12_today_ymd_matches_today_filename()
{
  // Retry once to absorb the (astronomically unlikely) UTC-midnight boundary
  // between the two calls; a genuine disagreement fails on both attempts.
  for attempt in 0..2
  {
    let ( y, m, d ) = claude_journal::rotation::today_ymd();
    let derived     = claude_journal::rotation::date_filename( y, m, d );
    if derived == claude_journal::rotation::today_filename() { return; }
    assert!( attempt == 0, "today_ymd and today_filename disagree: {derived}" );
  }
}
