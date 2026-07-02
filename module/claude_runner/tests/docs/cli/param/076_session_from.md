# Param :: `--session-from`

Edge case tests for the `--session-from <DIR>` parameter (alias `--from`), which enables session cross-loading by reading the source session from a different directory's Claude session storage.

**Source:** [param/076_session_from.md](../../../../docs/cli/param/076_session_from.md)

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| EC-1 | `--session-from` injects `-c <uuid>` when source has session | Behavioral |
| EC-2 | `--from` alias behaves identically to `--session-from` | Alias |
| EC-3 | Source dir with no `.jsonl` → no `-c` injected; fresh session | Behavioral |
| EC-4 | `--session-dir` takes precedence over `--session-from` | Precedence |
| EC-5 | `--new-session` takes precedence over `--session-from` | Precedence |
| EC-6 | `--to` + `--session-from`: Claude runs in target dir, loads from source | Behavioral |
| EC-7 | `CLR_SESSION_FROM` env var equivalent to `--session-from` | EnvFallback |
| EC-8 | `--dry-run` output contains `-c <uuid>` when source session present | Discovery |

## Test Coverage Summary

- Behavioral: 3 tests (EC-1, EC-3, EC-6)
- Alias: 1 test (EC-2)
- Precedence: 2 tests (EC-4, EC-5)
- EnvFallback: 1 test (EC-7)
- Discovery: 1 test (EC-8)

**Total:** 8 edge cases

## Test Cases

---

### EC-1: `--session-from` injects `-c <uuid>` when source has session

- **Given:** source dir `/tmp/076ec1_src` has a non-empty `.jsonl` file with stem `aaa-111`; target is CWD; fake claude binary in PATH
- **When:** `clr --session-from /tmp/076ec1_src --dry-run "Continue"`
- **Then:** dry-run output contains `-c aaa-111`
- **Exit:** 0
- **Source:** [param/076_session_from.md](../../../../docs/cli/param/076_session_from.md)

---

### EC-2: `--from` alias behaves identically to `--session-from`

- **Given:** same setup as EC-1 (source dir `/tmp/076ec2_src` has session `bbb-222.jsonl`)
- **When:** `clr --from /tmp/076ec2_src --dry-run "Continue"`
- **Then:** dry-run output contains `-c bbb-222`; behavior identical to `--session-from`
- **Exit:** 0
- **Source:** [param/076_session_from.md](../../../../docs/cli/param/076_session_from.md)

---

### EC-3: Source dir with no `.jsonl` → no `-c` injected; fresh session

- **Given:** source dir `/tmp/076ec3_empty_src` exists but contains no qualifying `.jsonl` files; fake claude binary in PATH
- **When:** `clr --session-from /tmp/076ec3_empty_src --dry-run "Start fresh"`
- **Then:** dry-run output does NOT contain `-c`; subprocess starts without session continuation
- **Exit:** 0
- **Source:** [param/076_session_from.md](../../../../docs/cli/param/076_session_from.md)

---

### EC-4: `--session-dir` takes precedence over `--session-from`

- **Given:** source dir `/tmp/076ec4_src` has session `ccc-333.jsonl`; raw session dir `/tmp/076ec4_override` has session `xyz-789.jsonl`; fake claude binary in PATH
- **When:** `clr --session-from /tmp/076ec4_src --session-dir /tmp/076ec4_override --dry-run "test"`
- **Then:** dry-run output contains `-c xyz-789`; `ccc-333` is NOT used; `--session-dir` wins
- **Exit:** 0
- **Source:** [param/076_session_from.md](../../../../docs/cli/param/076_session_from.md)

---

### EC-5: `--new-session` takes precedence over `--session-from`

- **Given:** source dir `/tmp/076ec5_src` has session `ddd-444.jsonl`; fake claude binary in PATH
- **When:** `clr --session-from /tmp/076ec5_src --new-session --dry-run "fresh"`
- **Then:** dry-run output does NOT contain `-c`; `--new-session` suppresses cross-loading
- **Exit:** 0
- **Source:** [param/076_session_from.md](../../../../docs/cli/param/076_session_from.md)

---

### EC-6: `--to` + `--session-from`: Claude runs in target dir, loads from source

- **Given:** source dir `/tmp/076ec6_src` has session `eee-555.jsonl`; target dir `/tmp/076ec6_tgt` exists; fake claude binary in PATH
- **When:** `clr --to /tmp/076ec6_tgt --session-from /tmp/076ec6_src --dry-run "Continue"`
- **Then:** dry-run output contains `-c eee-555`; subprocess working directory is `/tmp/076ec6_tgt` (not `/tmp/076ec6_src`)
- **Exit:** 0
- **Source:** [param/076_session_from.md](../../../../docs/cli/param/076_session_from.md)

---

### EC-7: `CLR_SESSION_FROM` env var equivalent to `--session-from`

- **Given:** source dir `/tmp/076ec7_src` has session `fff-666.jsonl`; `CLR_SESSION_FROM` set to that path; no `--session-from` on CLI
- **When:** `CLR_SESSION_FROM=/tmp/076ec7_src clr --dry-run "Continue"`
- **Then:** dry-run output contains `-c fff-666`; behavior identical to `--session-from /tmp/076ec7_src`
- **Exit:** 0
- **Source:** [param/076_session_from.md](../../../../docs/cli/param/076_session_from.md)

---

### EC-8: `--dry-run` output contains `-c <uuid>` when source session present

- **Given:** source dir `/tmp/076ec8_src` has session `ggg-777.jsonl` (highest mtime)
- **When:** `clr --session-from /tmp/076ec8_src --dry-run "task"`
- **Then:** dry-run output line includes `-c ggg-777`; WYSIWYG — dry-run accurately reflects what would be passed to the subprocess
- **Exit:** 0
- **Source:** [param/076_session_from.md](../../../../docs/cli/param/076_session_from.md)
