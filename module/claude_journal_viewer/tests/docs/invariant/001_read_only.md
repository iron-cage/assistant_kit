# Test: Invariant — Read-Only

Test case planning for [invariant/001_read_only.md](../../../docs/invariant/001_read_only.md). Tests validate that viewer commands never open `.jsonl` files in write mode, and that `.export` writes only to the target output file, not to any journal file.

**Source:** [invariant/001_read_only.md](../../../docs/invariant/001_read_only.md)
**Related:** [feature/001_cli_viewing.md](../../../docs/feature/001_cli_viewing.md)

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| IN-1 | Source: `src/cli/` contains no `OpenOptions::write`, `fs::write`, `File::create` calls on `.jsonl` paths (excluding `prune.rs`) | Structural |
| IN-2 | `.export` writes to target file; journal `.jsonl` files are unmodified afterward | Behavioral |

## Test Coverage Summary

- Structural: 1 test (IN-1)
- Behavioral: 1 test (IN-2)

**Total:** 2 invariant test cases

## Architectural Constraint

IN-1 is a structural test: scan `src/cli/*.rs` (excluding `prune.rs`) for the forbidden write-mode patterns. The measurement method is specified directly in the invariant doc.

IN-2 computes a checksum (or records byte count + modification time) of the journal `.jsonl` files before and after running `.export`. Both values must be identical.

---

### IN-1: No write-mode file opens in viewer commands (excluding prune)

- **Given:** source files `src/cli/*.rs` excluding `src/cli/prune.rs`, read as strings
- **When:** scan each for: `OpenOptions`, `write(true)`, `fs::write(`, `File::create(`
- **Then:** zero matches — no viewer command implementation opens files in write mode on journal paths
- **Source:** [invariant/001_read_only.md](../../../docs/invariant/001_read_only.md) Measurement

---

### IN-2: `.export` does not modify journal files

- **Given:** journal dir with `YYYY-MM-DD.jsonl`; record file size and content hash before export
- **When:** `clj .export format::jsonl output::/tmp/export_test.jsonl --journal-dir <dir>`
- **Then:** journal file size and content hash are identical after export; output file at `/tmp/export_test.jsonl` contains the exported events
- **Source:** [invariant/001_read_only.md](../../../docs/invariant/001_read_only.md) Rule: viewer never modifies journal file content
