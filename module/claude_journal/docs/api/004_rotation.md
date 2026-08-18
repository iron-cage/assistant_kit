# Rotation

**Status**: Implemented | **Since**: 1.3.0

### Scope

- **Purpose**: Provide the UTC-date-based daily filename scheme that journal rotation is built on, and the retention pruning functions layered on it.
- **Responsibility**: Documents the `rotation` module — the single authority for the `YYYY-MM-DD.jsonl` naming contract (formatting and strict parsing) and for journal file deletion.
- **In Scope**: Filename formatting/parsing for arbitrary and current UTC dates, filename-ordered listing, age-based pruning with dry-run reporting.
- **Out of Scope**: File creation and append mechanics (→ `docs/api/001_journal_writer.md`), rotation behavior and acceptance criteria (→ `docs/feature/003_rotation.md`), consumer pruning policy — `clj .prune` params and the runner's once-daily cadence (→ consumer docs).

## Description

Rotation in this crate is passive — nothing moves or renames files; the writer simply targets whichever filename the current UTC date maps to, so a new file "rotates in" at UTC midnight. The naming functions define that mapping; both writer and reader derive daily filenames from here, which is what keeps the two sides agreeing on the naming contract.

Deletion lives here exclusively — the `JournalWriter` never deletes. `prune_by_age` derives age from the filename date (never filesystem metadata) and takes `today` as an argument, so cutoff math is deterministic and testable. Only names `parse_date_filename` accepts are visible to listing and pruning; everything else in the directory is untouchable by construction.

## Interface

```rust
/// Return the JSONL filename for the given UTC year/month/day.
///
/// Format: `YYYY-MM-DD.jsonl` (zero-padded: `{year:04}-{month:02}-{day:02}`).
pub fn date_filename( year : i32, month : u32, day : u32 ) -> String;

/// Return the JSONL filename for today's UTC date.
pub fn today_filename() -> String;

/// Return today's UTC date as `( year, month, day )` — the `today` argument
/// callers pass to `prune_by_age`.
pub fn today_ymd() -> ( i32, u32, u32 );

/// Strict inverse of `date_filename`: exact `YYYY-MM-DD.jsonl` shape AND a
/// calendar-valid date, else `None`.
pub fn parse_date_filename( name : &str ) -> Option< ( i32, u32, u32 ) >;

/// List journal rotation files in `dir`, sorted by date ascending.
/// Missing/unreadable dir → empty; non-matching names ignored.
pub fn list_journal_files( dir : &Path ) -> Vec< ( PathBuf, ( i32, u32, u32 ) ) >;

/// Outcome of one file considered by `prune_by_age`.
pub enum PruneAction { Deleted, WouldDelete, Failed( String ) }

/// Delete journal files dated strictly before `today - keep_days`.
/// One report entry per qualifying file; `dry_run` reports without deleting.
pub fn prune_by_age( dir : &Path, keep_days : u32, today : ( i32, u32, u32 ), dry_run : bool )
  -> Vec< ( PathBuf, PruneAction ) >;
```

## Behavioral Contract

- Naming functions are pure and infallible — no I/O; `date_filename` does not validate calendar plausibility (an out-of-range month is formatted as given), while `parse_date_filename` DOES reject calendar-invalid dates — formatting is permissive, parsing is strict
- Zero-padding is fixed-width: 4-digit year, 2-digit month and day
- `today_filename()`/`today_ymd()` use UTC (`chrono::Utc`), never local time — the rotation boundary is UTC midnight on every machine
- `prune_by_age` cutoff is strict `<`: files dated exactly `today - keep_days` survive; today's file structurally survives every window including `keep_days = 0`
- Deletion is best-effort per file: a failure is reported as `Failed( error )` and the sweep continues
- `prune_by_age` panics only on a calendar-invalid `today` tuple (programmer error); a `keep_days` window reaching past representable time keeps everything

## Sources

- `src/rotation.rs` — implementation
- `docs/feature/003_rotation.md` — rotation behavior and acceptance criteria
- `tests/rotation_test.rs` — RT-1..RT-12 naming/listing/pruning coverage
- `tests/journal_integration_test.rs` — filename scheme exercised end-to-end
