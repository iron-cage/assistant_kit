//! Unix-only integration tests.
#![ cfg( unix ) ]
//! Journal auto-prune tests (JP-1..JP-5)
//!
//! ## Purpose
//!
//! Verify the once-daily journal retention prune wired into journal resolution:
//! filename-date-based deletion with a 30-day default, the `-last_prune` stamp's
//! same-UTC-day gate, and the `CLR_JOURNAL_KEEP` override (`"N"`/`"Nd"` days,
//! `"0"`/`"off"` disables, invalid values warn and fall back to the default).
//!
//! ## Test Layout
//!
//! - JP-1: default 30d window — ancient dated file deleted, non-date `.jsonl` and
//!   today's file survive, stamp written
//! - JP-2: same-day stamp gate — a second run on the same UTC day prunes nothing
//! - JP-3: `CLR_JOURNAL_KEEP=off` — no prune, no stamp (re-enabling acts next run)
//! - JP-4: `CLR_JOURNAL_KEEP=999999d` — window covers the ancient file; it survives
//! - JP-5: invalid `CLR_JOURNAL_KEEP` — stderr warning, 30d default still applied

mod cli_binary_test_helpers;
use cli_binary_test_helpers::{ read_journal_content, run_with_journal };
use std::path::Path;

/// Plant an ancient dated journal file plus a non-date-named `.jsonl` file.
fn plant_fixture_files( jdir : &Path )
{
  std::fs::write( jdir.join( "2020-01-01.jsonl" ), "{}\n" ).expect( "plant dated file" );
  std::fs::write( jdir.join( "notes.jsonl" ), "not a journal rotation file\n" ).expect( "plant notes" );
}

// ── JP-1: default window prunes ancient file, spares everything else ─────────

/// JP-1: with no `CLR_JOURNAL_KEEP`, one run deletes a file dated far outside
/// the 30-day default, leaves non-date-pattern `.jsonl` files untouched, writes
/// today's journal normally, and drops the `-last_prune` stamp.
#[ test ]
fn jp1_default_prune_deletes_ancient_keeps_rest()
{
  let jdir   = tempfile::TempDir::new().expect( "tmpdir" );
  let jdir_s = jdir.path().to_str().expect( "utf-8" ).to_owned();
  plant_fixture_files( jdir.path() );

  let ( out, _fake ) = run_with_journal(
    &[ "--journal", "full", "--journal-dir", &jdir_s ],
    &[],
    "printf done\nexit 0",
  );
  assert!( out.status.success(), "exit must be 0. stderr: {}", String::from_utf8_lossy( &out.stderr ) );

  assert!(
    !jdir.path().join( "2020-01-01.jsonl" ).exists(),
    "ancient dated file must be pruned by the 30d default",
  );
  assert!(
    jdir.path().join( "notes.jsonl" ).exists(),
    "non-date-pattern .jsonl must never be auto-pruned",
  );
  assert!(
    read_journal_content( jdir.path() ).contains( "execution" ),
    "today's journal file must be written and survive the prune",
  );
  let stamp = std::fs::read_to_string( jdir.path().join( "-last_prune" ) )
    .expect( "-last_prune stamp must exist after an enabled prune run" );
  assert!( !stamp.trim().is_empty(), "stamp must carry the prune date" );
}

// ── JP-2: same-UTC-day stamp gate ────────────────────────────────────────────

/// JP-2: after one run writes the stamp, a second run on the same UTC day skips
/// pruning entirely — a file planted between the runs survives.
#[ test ]
fn jp2_same_day_stamp_gates_second_prune()
{
  let jdir   = tempfile::TempDir::new().expect( "tmpdir" );
  let jdir_s = jdir.path().to_str().expect( "utf-8" ).to_owned();

  let ( first, _fake ) = run_with_journal(
    &[ "--journal", "full", "--journal-dir", &jdir_s ],
    &[],
    "printf done\nexit 0",
  );
  assert!( first.status.success(), "first run must succeed" );
  assert!( jdir.path().join( "-last_prune" ).exists(), "first run must write the stamp" );

  std::fs::write( jdir.path().join( "2020-01-01.jsonl" ), "{}\n" ).expect( "plant dated file" );
  let ( second, _fake2 ) = run_with_journal(
    &[ "--journal", "full", "--journal-dir", &jdir_s ],
    &[],
    "printf done\nexit 0",
  );
  assert!( second.status.success(), "second run must succeed" );
  assert!(
    jdir.path().join( "2020-01-01.jsonl" ).exists(),
    "same-day second run must not prune (stamp gate)",
  );
}

// ── JP-3: CLR_JOURNAL_KEEP=off disables pruning ──────────────────────────────

/// JP-3: `off` disables the prune and writes no stamp, so re-enabling takes
/// effect on the very next invocation instead of waiting out the day.
#[ test ]
fn jp3_keep_off_disables_prune_and_stamp()
{
  let jdir   = tempfile::TempDir::new().expect( "tmpdir" );
  let jdir_s = jdir.path().to_str().expect( "utf-8" ).to_owned();
  plant_fixture_files( jdir.path() );

  let ( out, _fake ) = run_with_journal(
    &[ "--journal", "full", "--journal-dir", &jdir_s ],
    &[ ( "CLR_JOURNAL_KEEP", "off" ) ],
    "printf done\nexit 0",
  );
  assert!( out.status.success(), "exit must be 0. stderr: {}", String::from_utf8_lossy( &out.stderr ) );
  assert!(
    jdir.path().join( "2020-01-01.jsonl" ).exists(),
    "CLR_JOURNAL_KEEP=off must disable pruning",
  );
  assert!(
    !jdir.path().join( "-last_prune" ).exists(),
    "disabled prune must not write a stamp",
  );
}

// ── JP-4: numeric CLR_JOURNAL_KEEP widens the window ─────────────────────────

/// JP-4: a keep window large enough to cover the ancient file keeps it.
#[ test ]
fn jp4_keep_override_widens_window()
{
  let jdir   = tempfile::TempDir::new().expect( "tmpdir" );
  let jdir_s = jdir.path().to_str().expect( "utf-8" ).to_owned();
  plant_fixture_files( jdir.path() );

  let ( out, _fake ) = run_with_journal(
    &[ "--journal", "full", "--journal-dir", &jdir_s ],
    &[ ( "CLR_JOURNAL_KEEP", "999999d" ) ],
    "printf done\nexit 0",
  );
  assert!( out.status.success(), "exit must be 0. stderr: {}", String::from_utf8_lossy( &out.stderr ) );
  assert!(
    jdir.path().join( "2020-01-01.jsonl" ).exists(),
    "a 999999d window must keep the ancient file",
  );
  assert!( jdir.path().join( "-last_prune" ).exists(), "enabled prune must write the stamp" );
}

// ── JP-5: invalid CLR_JOURNAL_KEEP warns and falls back to default ───────────

/// JP-5: an unparsable value emits a stderr warning and the 30d default still
/// applies — misconfiguration is loud, not a silent retention change.
#[ test ]
fn jp5_invalid_keep_warns_and_uses_default()
{
  let jdir   = tempfile::TempDir::new().expect( "tmpdir" );
  let jdir_s = jdir.path().to_str().expect( "utf-8" ).to_owned();
  plant_fixture_files( jdir.path() );

  let ( out, _fake ) = run_with_journal(
    &[ "--journal", "full", "--journal-dir", &jdir_s ],
    &[ ( "CLR_JOURNAL_KEEP", "bogus" ) ],
    "printf done\nexit 0",
  );
  assert!( out.status.success(), "exit must be 0. stderr: {}", String::from_utf8_lossy( &out.stderr ) );
  assert!(
    String::from_utf8_lossy( &out.stderr ).contains( "invalid CLR_JOURNAL_KEEP" ),
    "invalid value must warn on stderr. stderr: {}",
    String::from_utf8_lossy( &out.stderr ),
  );
  assert!(
    !jdir.path().join( "2020-01-01.jsonl" ).exists(),
    "30d default must still apply after an invalid override",
  );
}
