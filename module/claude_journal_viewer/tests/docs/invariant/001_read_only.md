# Test: Invariant — Read-Only

Test case planning for [invariant/001_read_only.md](../../../docs/invariant/001_read_only.md). Tests validate that viewer commands never open `.jsonl` files in write mode, and that `.export` writes only to the target output file, not to any journal file.

**Source:** [invariant/001_read_only.md](../../../docs/invariant/001_read_only.md)
**Related:** [feature/001_cli_viewing.md](../../../docs/feature/001_cli_viewing.md)

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| IN-1 | Source: `src/` contains zero `remove_file` calls and exactly one write-mode call — `.export`'s write to its user-specified `output::` path, never a `.jsonl` path | Structural |
| IN-2 | `.export` writes to target file; journal `.jsonl` files are unmodified afterward | Behavioral |

## Test Coverage Summary

- Structural: 1 test (IN-1)
- Behavioral: 1 test (IN-2)

**Total:** 2 invariant test cases

## Architectural Constraint

IN-1 is a structural test: recursively scan `src/` for the forbidden deletion and write-mode patterns, using the measurement method specified directly in the invariant doc. The crate has no `src/cli/` subdirectory and no `prune.rs` — all four sources (`cli_main.rs`, `lib.rs`, `output.rs`, `routines.rs`) sit flat in `src/`, and `.prune` is `prune_output()` in `src/output.rs`, which delegates whole-file deletion to `claude_journal::rotation::prune_by_age` rather than deleting locally. The scan therefore covers `src/` whole, with no exclusions.

IN-2 computes a checksum (or records byte count + modification time) of the journal `.jsonl` files before and after running `.export`. Both values must be identical.

---

### IN-1: No deletion and no unexpected write-mode file opens in viewer source

- **Given:** every `.rs` file under `src/` — currently `cli_main.rs`, `lib.rs`, `output.rs`, `routines.rs`
- **When:** run from the crate root (`module/claude_journal_viewer`):

```sh
# Vacuity guard: a scan that matches zero files must fail, not pass silently.
n_src=$( find src -name '*.rs' | wc -l )
[ "$n_src" -ge 4 ] || { echo "IN-1 FAIL: scanned $n_src source files, expected >= 4"; exit 1; }

# (a) zero deletion calls anywhere in viewer source
n_del=$( grep -rn 'remove_file' src/ | wc -l )
[ "$n_del" -eq 0 ] || { echo "IN-1 FAIL: $n_del remove_file call(s)"; exit 1; }

# (b) exactly one write-mode call — export_output()'s write to the user's output:: path
n_write=$( grep -rn 'OpenOptions\|fs::write\|File::create' src/ | wc -l )
[ "$n_write" -eq 1 ] || { echo "IN-1 FAIL: $n_write write-mode call(s), expected exactly 1"; exit 1; }

echo "IN-1 PASS: $n_src files scanned, 0 deletions, 1 permitted write"
```

- **Then:** the guard confirms at least 4 sources were actually scanned; `remove_file` returns zero matches (deletion lives in `claude_journal::rotation`, not here); the write-mode scan returns exactly one match — `std::fs::write( &output, &content )` in `src/output.rs` `export_output()`, the user-specified `output::` path, which is never a journal file. Any additional match, or a scan that sees fewer than 4 files, fails the invariant.
- **Source:** [invariant/001_read_only.md](../../../docs/invariant/001_read_only.md) Measurement

---

### IN-2: `.export` does not modify journal files

- **Given:** journal dir with `YYYY-MM-DD.jsonl`; record file size and content hash before export
- **When:** `clj .export format::jsonl output::/tmp/export_test.jsonl --journal-dir <dir>`
- **Then:** journal file size and content hash are identical after export; output file at `/tmp/export_test.jsonl` contains the exported events
- **Source:** [invariant/001_read_only.md](../../../docs/invariant/001_read_only.md) Rule: viewer never modifies journal file content
