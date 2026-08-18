# Read-Only

**Status**: Implemented | **Since**: 1.3.0

### Scope

- **Purpose**: Guarantee the viewer never modifies journal file content.
- **Responsibility**: Documents which viewer commands may touch journal files and in what mode.
- **In Scope**: Read-only file access for all viewing commands and whole-file (not content) deletion by `.prune`.
- **Out of Scope**: Journal writing (→ `claude_journal` `docs/api/001_journal_writer.md`), network exposure of served data (→ `docs/invariant/002_localhost_only.md`).

## Description

The viewer never modifies journal file content. All viewing commands (`.list`, `.tail`, `.stats`, `.search`, `.serve`, `.status`, `.export`) open journal files in read-only mode. The `.prune` command deletes whole files but never modifies their content — and even that deletion is not implemented in viewer source: `prune_output` delegates to `claude_journal::rotation::prune_by_age`. No command truncates, seeks, or writes to any `.jsonl` file.

## Measurement

- **Threshold**: 0 deletion calls and 0 write-mode file opens on journal paths anywhere in viewer source
- **Method**: `grep -rn "remove_file" src/` must return zero matches (deletion lives in `claude_journal::rotation`, not here), and `grep -rn "OpenOptions\|fs::write\|File::create" src/` must return exactly one match — `.export`'s write to its user-specified `output::` path, which is never a journal file

## Sources

- `src/cli_main.rs`, `src/output.rs` — all command implementations read via `JournalReader` (read-only)
- `src/output.rs` `prune_output()` — delegates deletion to `claude_journal::rotation::prune_by_age` (whole-file delete, not content modification)
