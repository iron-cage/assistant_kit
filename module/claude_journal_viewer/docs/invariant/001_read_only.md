# Read-Only

**Status**: Planned | **Since**: 1.3.0

### Scope

- **Purpose**: Guarantee the viewer never modifies journal file content.
- **Responsibility**: Documents which viewer commands may touch journal files and in what mode.
- **In Scope**: Read-only file access for all viewing commands and whole-file (not content) deletion by `.prune`.
- **Out of Scope**: Journal writing (→ `claude_journal` `docs/api/001_journal_writer.md`), network exposure of served data (→ `docs/invariant/002_localhost_only.md`).

## Description

The viewer never modifies journal file content. All viewing commands (`.list`, `.tail`, `.stats`, `.search`, `.serve`, `.status`, `.export`) open journal files in read-only mode. The `.prune` command deletes whole files but never modifies their content. No command truncates, seeks, or writes to any `.jsonl` file.

## Measurement

- **Threshold**: 0 write-mode file opens on `.jsonl` files across all viewer commands (measured by code review — no `OpenOptions::write`, `fs::write`, `File::create` on journal paths outside `prune.rs`)
- **Method**: `grep -rn "OpenOptions\|fs::write\|File::create" src/cli/ | grep -v prune.rs` must return zero matches

## Sources

- `src/cli/*.rs` — all command implementations use `JournalReader` (read-only)
- `src/cli/prune.rs` — uses `std::fs::remove_file` (whole-file delete, not content modification)
