# Rotation

**Status**: Implemented | **Since**: 1.3.0

### Scope

- **Purpose**: Provide daily file rotation and retention pruning for journal storage.
- **Responsibility**: Documents the age-based pruning strategy and the filename-driven file listing it relies on.
- **In Scope**: Age-based pruning, filename-date ordering, dry-run reporting, and non-matching-filename handling.
- **Out of Scope**: Journal file creation/writing (→ `docs/feature/001_event_journaling.md`), the CLI `.prune` command surface (→ `claude_journal_viewer` `docs/feature/001_cli_viewing.md`), the runner's once-daily auto-prune policy — stamp cadence and `CLR_JOURNAL_KEEP` (→ `claude_runner` `docs/feature/002_journaling_integration.md`).

## Description

Daily file rotation and retention pruning for journal storage. Journal files are named by UTC date (`YYYY-MM-DD.jsonl`) — one file per day, created on first write. Retention is age-based: delete files whose filename date falls strictly before `today - keep_days`. (A size-based strategy was considered and dropped — no consumer needs it.)

Pruning is always explicitly invoked — the `JournalWriter` never deletes files; it only appends. Two consumers call the `rotation` module's pruning functions: the viewer's `clj .prune` command (on demand, with `dry_run` support) and the runner's once-daily auto-prune at journal resolution (policy documented in `claude_runner`).

File listing and age calculation use the filename date exclusively — no filesystem metadata dependency, so copies and restores cannot change what gets pruned. Filenames that do not match the `YYYY-MM-DD.jsonl` pattern exactly (including calendar-invalid dates like `2026-02-30`) are ignored: not deleted, not listed.

## Acceptance Criteria

- AC-001: `list_journal_files()` returns files sorted by date (oldest first), filtering to the strict `YYYY-MM-DD.jsonl` pattern only
- AC-002: `prune_by_age( dir, keep_days, today, dry_run )` deletes exactly the pattern-matching files dated strictly before `today - keep_days`, reporting one `( path, PruneAction )` entry per qualifying file
- AC-003: With `dry_run`, qualifying files are reported as `WouldDelete` and nothing is touched
- AC-004: Both functions skip non-matching filenames (non-JSONL, non-date-pattern, calendar-invalid)
- AC-005: Listing or pruning an empty or nonexistent directory yields an empty result (no error)
- AC-006: Today's file is never deleted — the cutoff is at most `today` and only strictly-older dates qualify, so even `keep_days = 0` spares it
- AC-007: A per-file deletion failure is reported as `Failed` and the sweep continues (best-effort)

## Sources

- `src/rotation.rs` — `parse_date_filename()`, `list_journal_files()`, `prune_by_age()`, `PruneAction`
- `docs/api/004_rotation.md` — interface contract
- `tests/rotation_test.rs` — RT-1..RT-12 coverage of every AC
