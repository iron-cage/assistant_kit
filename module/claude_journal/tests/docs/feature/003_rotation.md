# Test: Feature — Rotation

Test case planning for [feature/003_rotation.md](../../../docs/feature/003_rotation.md). Tests validate daily filename generation, file listing order, age-based pruning, size-based pruning, non-matching file filtering, and empty-directory behavior.

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| FT-1 | `date_filename(y,m,d)` generates `YYYY-MM-DD.jsonl`; different dates produce different filenames | Filename Format |
| FT-2 | `list_journal_files()` returns files sorted oldest-first; skips non-matching filenames | File Listing |
| FT-3 | `prune_by_age(dir, 30)` deletes files older than 30 days; returns count deleted | Age Pruning |
| FT-4 | `prune_by_age` never deletes today's file (age = 0 days) | Age Pruning Guard |
| FT-5 | `prune_by_size(dir, max)` deletes oldest files until total size is under `max` | Size Pruning |
| FT-6 | Pruning an empty directory returns `Ok(0)` without error | Empty Dir |
| FT-7 | Size pruning stops when only today's file remains, even if it still exceeds `max` | Size Pruning Guard |

## Test Coverage Summary

- Filename Format: 1 test (FT-1)
- File Listing: 1 test (FT-2)
- Age Pruning: 1 test (FT-3)
- Age Pruning Guard: 1 test (FT-4)
- Size Pruning: 1 test (FT-5)
- Empty Dir: 1 test (FT-6)
- Size Pruning Guard: 1 test (FT-7)

**Total:** 7 tests

## Architectural Constraint

FT-3 through FT-7 require creating synthetic `.jsonl` files with controlled dates in a temp directory. Date-based tests use filenames like `2020-01-01.jsonl` (clearly in the past) to avoid flakiness from real wall-clock proximity.

FT-2 also checks that a file named `notes.txt` or `2026-13-01.jsonl` (invalid date) in the journal dir is not returned by `list_journal_files()`.

---

### FT-1: `date_filename()` generates `YYYY-MM-DD.jsonl`

- **Given:** calls `date_filename(2026, 6, 27)` and `date_filename(2026, 6, 28)`
- **When:** compare the two return values
- **Then:** `date_filename(2026, 6, 27) == "2026-06-27.jsonl"`; the two are not equal; `today_filename()` ends with `.jsonl` and has a date part with 4-digit year, 2-digit month, 2-digit day
- **Source:** [feature/003_rotation.md](../../../docs/feature/003_rotation.md) AC-001 (implied by naming convention)

---

### FT-2: `list_journal_files()` sorts oldest-first and filters

- **Given:** temp dir containing `2023-01-01.jsonl`, `2025-06-15.jsonl`, `2026-01-01.jsonl`, plus a non-matching `readme.txt`
- **When:** `list_journal_files(&dir)`
- **Then:** returns exactly 3 entries in order: `2023-01-01.jsonl`, `2025-06-15.jsonl`, `2026-01-01.jsonl`; `readme.txt` is absent
- **Source:** [feature/003_rotation.md](../../../docs/feature/003_rotation.md) AC-001, AC-004

---

### FT-3: `prune_by_age()` deletes old files

- **Given:** temp dir containing `2020-01-01.jsonl`, `2020-06-01.jsonl`, plus today's file
- **When:** `prune_by_age(&dir, 30)` (keep 30 days)
- **Then:** returns `Ok(2)` (two old files deleted); `2020-01-01.jsonl` and `2020-06-01.jsonl` no longer exist; today's file still exists
- **Source:** [feature/003_rotation.md](../../../docs/feature/003_rotation.md) AC-002

---

### FT-4: `prune_by_age()` never deletes today's file

- **Given:** temp dir containing only today's file with some content
- **When:** `prune_by_age(&dir, 0)` (keep 0 days — would delete everything if guard absent)
- **Then:** returns `Ok(0)`; today's file still exists
- **Source:** [feature/003_rotation.md](../../../docs/feature/003_rotation.md) AC-006

---

### FT-5: `prune_by_size()` deletes oldest files first

- **Given:** temp dir with `2020-01-01.jsonl` (10 KB), `2021-01-01.jsonl` (10 KB), today's file (1 KB); max_bytes = 15_000
- **When:** `prune_by_size(&dir, 15_000)`
- **Then:** `2020-01-01.jsonl` is deleted; total size drops below 15 KB; today's file is present
- **Source:** [feature/003_rotation.md](../../../docs/feature/003_rotation.md) AC-003

---

### FT-6: Pruning an empty dir returns `Ok(0)`

- **Given:** empty temporary directory (no `.jsonl` files)
- **When:** `prune_by_age(&dir, 30)` and `prune_by_size(&dir, 1000)`
- **Then:** both return `Ok(0)` without error; no panic; dir still exists
- **Source:** [feature/003_rotation.md](../../../docs/feature/003_rotation.md) AC-005

---

### FT-7: Size pruning stops when only today's file remains

- **Given:** temp dir containing only today's file with 50 KB of content; max_bytes = 1_000 (far below file size)
- **When:** `prune_by_size(&dir, 1_000)`
- **Then:** returns `Ok(0)` (nothing deleted); today's file still present — guard prevents last-file deletion
- **Source:** [feature/003_rotation.md](../../../docs/feature/003_rotation.md) AC-007
