# Test: Invariant — Session Source Isolation

Test case planning for [invariant/011_session_source_isolation.md](../../../../docs/invariant/011_session_source_isolation.md). Tests verify that session reads use the source directory's storage, Claude runs in the target directory, source files are never modified, cross-loading is one-time, and `--session-dir` takes precedence over `--session-from`.

**Source:** [invariant/011_session_source_isolation.md](../../../../docs/invariant/011_session_source_isolation.md)

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| IT-1 | Session UUID is read from source dir's `CLAUDE_SESSION_DIR`, not target | Read isolation |
| IT-2 | Subprocess working directory is target dir, not source dir | Run isolation |
| IT-3 | Source session file mtime and size unchanged after cross-loaded run | Write isolation |
| IT-4 | `--session-dir` takes precedence over `--session-from` (raw path wins) | Precedence |
| IT-5 | `--session-from` + `--to`: session UUID from source, cwd is target | Combined |

## Test Coverage Summary

- Read isolation: 1 test (IT-1)
- Run isolation: 1 test (IT-2)
- Write isolation: 1 test (IT-3)
- Precedence: 1 test (IT-4)
- Combined: 1 test (IT-5)

**Total:** 5 invariant test cases

---

### IT-1: Session UUID is read from source dir's `CLAUDE_SESSION_DIR`, not target

- **Given:** source dir `/tmp/011it1_src` has session `lll-001.jsonl`; target dir `/tmp/011it1_tgt` has no `.jsonl` files; fake claude binary in PATH
- **When:** `clr --session-from /tmp/011it1_src --to /tmp/011it1_tgt --dry-run "task"`
- **Then:** dry-run output contains `-c lll-001` (from source); no UUID from target is used
- **Exit:** 0
- **Source:** [invariant/011_session_source_isolation.md](../../../../docs/invariant/011_session_source_isolation.md) point 1

---

### IT-2: Subprocess working directory is target dir, not source dir

- **Given:** source dir `/tmp/011it2_src` has session `mmm-002.jsonl`; target dir `/tmp/011it2_tgt` exists; fake claude binary in PATH
- **When:** `clr --session-from /tmp/011it2_src --to /tmp/011it2_tgt --dry-run "task"`
- **Then:** dry-run output shows subprocess working directory as `/tmp/011it2_tgt`; `/tmp/011it2_src` does NOT appear as the working directory
- **Exit:** 0
- **Source:** [invariant/011_session_source_isolation.md](../../../../docs/invariant/011_session_source_isolation.md) point 2

---

### IT-3: Source session file mtime and size unchanged after cross-loaded run

- **Given:** source dir `/tmp/011it3_src` has session `nnn-003.jsonl` with recorded mtime T1 and known file size; target dir `/tmp/011it3_tgt`; fake claude binary in PATH
- **When:** `clr --session-from /tmp/011it3_src --to /tmp/011it3_tgt --dry-run "Continue"`; run completes
- **Then:** `nnn-003.jsonl` mtime is still T1; file size is unchanged; source dir contents are identical to before the run
- **Exit:** 0
- **Source:** [invariant/011_session_source_isolation.md](../../../../docs/invariant/011_session_source_isolation.md) point 3

---

### IT-4: `--session-dir` takes precedence over `--session-from` (raw path wins)

- **Given:** source dir `/tmp/011it4_src` has session `ooo-004.jsonl`; raw session dir `/tmp/011it4_raw` has session `ppp-005.jsonl`; fake claude binary in PATH
- **When:** `clr --session-from /tmp/011it4_src --session-dir /tmp/011it4_raw --dry-run "test"`
- **Then:** dry-run output contains `-c ppp-005`; `ooo-004` is NOT used; `--session-dir` raw path wins over `--session-from` computed path
- **Exit:** 0
- **Source:** [invariant/011_session_source_isolation.md](../../../../docs/invariant/011_session_source_isolation.md) point 5

---

### IT-5: `--session-from` + `--to`: session UUID from source, cwd is target

- **Given:** source dir `/tmp/011it5_src` has session `qqq-006.jsonl`; target dir `/tmp/011it5_tgt` exists; fake claude binary in PATH
- **When:** `clr --to /tmp/011it5_tgt --session-from /tmp/011it5_src --dry-run "Continue"`
- **Then:** dry-run output contains `-c qqq-006`; subprocess working directory is `/tmp/011it5_tgt`; source UUID is injected into a subprocess that runs in target — confirming both read isolation (source) and run isolation (target) hold simultaneously
- **Exit:** 0
- **Source:** [invariant/011_session_source_isolation.md](../../../../docs/invariant/011_session_source_isolation.md) points 1–2

## Notes

IT-3 (write isolation) is the most critical property: if source files are written to during a cross-loaded run, the isolation contract is broken and source session history would be polluted with unrelated conversation turns. The dry-run mode used in IT-3 prevents actual subprocess execution, so this test validates that `clr` itself does not touch source files during setup.
