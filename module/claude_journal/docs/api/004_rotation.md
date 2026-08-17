# Rotation Filenames

**Status**: Implemented | **Since**: 1.3.0

### Scope

- **Purpose**: Provide the UTC-date-based daily filename scheme that journal rotation is built on.
- **Responsibility**: Documents `date_filename()` and `today_filename()` — the single authority for the `YYYY-MM-DD.jsonl` naming contract.
- **In Scope**: Filename formatting for arbitrary and current UTC dates.
- **Out of Scope**: File creation and append mechanics (→ `docs/api/001_journal_writer.md`), rotation behavior and acceptance criteria (→ `docs/feature/003_rotation.md`).

## Description

Rotation in this crate is passive — nothing moves or renames files; the writer simply targets whichever filename the current UTC date maps to, so a new file "rotates in" at UTC midnight. These two pure functions define that mapping. Both writer and reader derive daily filenames from here, which is what keeps the two sides agreeing on the naming contract.

## Interface

```rust
/// Return the JSONL filename for the given UTC year/month/day.
///
/// Format: `YYYY-MM-DD.jsonl` (zero-padded: `{year:04}-{month:02}-{day:02}`).
pub fn date_filename( year : i32, month : u32, day : u32 ) -> String;

/// Return the JSONL filename for today's UTC date.
///
/// Equivalent to `date_filename` called with the current UTC year/month/day.
pub fn today_filename() -> String;
```

## Behavioral Contract

- Pure and infallible — no I/O, no validation of calendar plausibility (a caller-supplied out-of-range month is formatted as given, not rejected)
- Zero-padding is fixed-width: 4-digit year, 2-digit month and day
- `today_filename()` uses UTC (`chrono::Utc`), never local time — the rotation boundary is UTC midnight on every machine

## Sources

- `src/rotation.rs` — implementation
- `docs/feature/003_rotation.md` — rotation behavior and acceptance criteria
- `tests/journal_integration_test.rs` — filename scheme exercised end-to-end
