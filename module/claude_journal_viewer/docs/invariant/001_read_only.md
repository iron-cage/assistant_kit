# Read-Only

**Status**: Planned | **Since**: 1.3.0

## Description

The viewer never modifies journal file content. All viewing commands (`.list`, `.tail`, `.stats`, `.search`, `.serve`, `.status`, `.export`) open journal files in read-only mode. The `.prune` command deletes whole files but never modifies their content. No command truncates, seeks, or writes to any `.jsonl` file.

## Measurement

- **Threshold**: 0 write-mode file opens on `.jsonl` files across all viewer commands (measured by code review — no `OpenOptions::write`, `fs::write`, `File::create` on journal paths outside `prune.rs`)
- **Method**: `grep -rn "OpenOptions\|fs::write\|File::create" src/cli/ | grep -v prune.rs` must return zero matches

## Sources

- `src/cli/*.rs` — all command implementations use `JournalReader` (read-only)
- `src/cli/prune.rs` — uses `std::fs::remove_file` (whole-file delete, not content modification)
