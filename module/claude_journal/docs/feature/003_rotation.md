# Rotation

**Status**: Planned | **Since**: 1.3.0

### Scope

- **Purpose**: Provide daily file rotation and retention pruning for journal storage.
- **Responsibility**: Documents the age-based and size-based pruning strategies and the filename-driven file listing they rely on.
- **In Scope**: Explicit `.prune`-triggered pruning, filename-date ordering, and non-matching-filename handling.
- **Out of Scope**: Journal file creation/writing (→ `docs/feature/001_event_journaling.md`), the CLI `.prune` command surface (→ `claude_journal_viewer` `docs/feature/001_cli_viewing.md`).

## Description

Daily file rotation and retention pruning for journal storage. Journal files are named by UTC date (`YYYY-MM-DD.jsonl`) — one file per day, created on first write. Two pruning strategies are supported: age-based (delete files older than N days) and size-based (delete oldest files until total size is under a threshold).

Pruning is invoked explicitly via `clj .prune` — there is no automatic background pruning. The `JournalWriter` never deletes files; it only appends. The `rotation` module provides pruning functions consumed by the viewer's `.prune` command.

File listing uses the filename date for ordering and age calculation — no filesystem metadata dependency. Filenames that do not match the `YYYY-MM-DD.jsonl` pattern are ignored (not deleted, not listed).

## Acceptance Criteria

- AC-001: `list_journal_files()` returns files sorted by date (oldest first), filtering to `YYYY-MM-DD.jsonl` pattern only
- AC-002: `prune_by_age(dir, keep_days)` deletes all `.jsonl` files older than `keep_days` and returns the count deleted
- AC-003: `prune_by_size(dir, max_bytes)` deletes oldest files first until total directory size is under `max_bytes`
- AC-004: Both pruning functions skip non-matching filenames (non-JSONL, non-date-pattern)
- AC-005: Pruning an empty or nonexistent directory returns `Ok(0)` (no error)
- AC-006: Today's file is never deleted by age-based pruning (age = 0 days)
- AC-007: Size-based pruning stops if only today's file remains, even if it exceeds `max_bytes`

## Sources

- `src/rotation.rs` — `list_journal_files()`, `prune_by_age()`, `prune_by_size()`
