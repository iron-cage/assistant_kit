# Test: Feature — Session Path Resolution

Test case planning for [feature/005_session_path_resolution.md](../../../../docs/feature/005_session_path_resolution.md). Tests validate `scope_for()` output correctness, CLAUDE_HOME and memory path override handling, git-root anchoring for memory dir, the `clr scope` command, and the `--session-from`/`--to` cross-loading behaviors.

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| SF-1 | `scope_for()` default: uses `$HOME/.claude` when CLAUDE_HOME unset | scope_for |
| SF-2 | `scope_for()` respects `CLAUDE_HOME` env var override | scope_for |
| SF-3 | `scope_for()` respects `CLAUDE_COWORK_MEMORY_PATH_OVERRIDE` | scope_for |
| SF-4 | `scope_for()` anchors memory dir to git root, not subdirectory | scope_for |
| SF-5 | `scope_for()` returns `None` for session file when dir is empty | scope_for |
| SF-6 | `clr scope` prints 6 `CLAUDE_*` vars in `key=value` format | clr scope |
| SF-7 | `--session-from` resumes most recent session from source dir | cross-loading |
| SF-8 | `--to` + `--session-from`: Claude runs in target, loads from source | cross-loading |
| SF-9 | `--to` is an alias for `--dir`; behavior is identical | alias |
| SF-10 | `--session-dir` takes precedence over `--session-from` | precedence |

## Test Coverage Summary

- scope_for: 5 tests (SF-1–SF-5)
- clr scope: 1 test (SF-6)
- cross-loading: 2 tests (SF-7, SF-8)
- alias: 1 test (SF-9)
- precedence: 1 test (SF-10)

**Total:** 10 tests

---

### SF-1: `scope_for()` default: uses `$HOME/.claude` when CLAUDE_HOME unset

- **Given:** `CLAUDE_HOME` is unset in environment; target dir `/tmp/sf1_proj`
- **When:** `clr scope --dir /tmp/sf1_proj` (no CLAUDE_HOME override)
- **Then:** `CLAUDE_HOME` line in output is `<HOME>/.claude`; `CLAUDE_PROJECTS_DIR` is `<HOME>/.claude/projects/`
- **Exit:** 0
- **Source:** [feature/005_session_path_resolution.md](../../../../docs/feature/005_session_path_resolution.md) AC-1

---

### SF-2: `scope_for()` respects `CLAUDE_HOME` env var override

- **Given:** `CLAUDE_HOME=/tmp/sf2_claude_home`; target dir `/tmp/sf2_proj`
- **When:** `CLAUDE_HOME=/tmp/sf2_claude_home clr scope --dir /tmp/sf2_proj`
- **Then:** all 6 variables reflect the custom home; `CLAUDE_HOME=/tmp/sf2_claude_home`; `CLAUDE_SESSION_DIR` starts with `/tmp/sf2_claude_home`
- **Exit:** 0
- **Source:** [feature/005_session_path_resolution.md](../../../../docs/feature/005_session_path_resolution.md) AC-2

---

### SF-3: `scope_for()` respects `CLAUDE_COWORK_MEMORY_PATH_OVERRIDE`

- **Given:** `CLAUDE_COWORK_MEMORY_PATH_OVERRIDE=/tmp/sf3_shared`; target dir `/tmp/sf3_proj`
- **When:** `CLAUDE_COWORK_MEMORY_PATH_OVERRIDE=/tmp/sf3_shared clr scope --dir /tmp/sf3_proj`
- **Then:** `CLAUDE_MEMORY_DIR=/tmp/sf3_shared`; `CLAUDE_MEMORY_FILE=/tmp/sf3_shared/MEMORY.md`; `CLAUDE_SESSION_DIR` unchanged (uses normal derivation)
- **Exit:** 0
- **Source:** [feature/005_session_path_resolution.md](../../../../docs/feature/005_session_path_resolution.md) AC-3

---

### SF-4: `scope_for()` anchors memory dir to git root, not subdirectory

- **Given:** git repo at `/tmp/sf4_repo` with `.git`; target dir is `/tmp/sf4_repo/src` (a subdirectory)
- **When:** `clr scope --dir /tmp/sf4_repo/src`
- **Then:** `CLAUDE_MEMORY_DIR` contains the encoded form of `/tmp/sf4_repo` (git root), NOT `/tmp/sf4_repo/src`; `CLAUDE_SESSION_DIR` uses `/tmp/sf4_repo/src` encoding
- **Exit:** 0
- **Source:** [feature/005_session_path_resolution.md](../../../../docs/feature/005_session_path_resolution.md) AC-4

---

### SF-5: `scope_for()` returns `None` for session file when dir has no sessions

- **Given:** target dir `/tmp/sf5_empty_proj` has no `.jsonl` files in its Claude storage
- **When:** `clr scope --dir /tmp/sf5_empty_proj`
- **Then:** `CLAUDE_SESSION_FILE=` is an empty assignment (no path after `=`)
- **Exit:** 0
- **Source:** [feature/005_session_path_resolution.md](../../../../docs/feature/005_session_path_resolution.md) AC-5

---

### SF-6: `clr scope` prints 6 `CLAUDE_*` vars in `key=value` format

- **Given:** any valid directory
- **When:** `clr scope --dir /tmp`
- **Then:** stdout has exactly 6 lines matching `^CLAUDE_[A-Z_]+=.*$`; output is valid for `eval`; printed in order: HOME, PROJECTS_DIR, SESSION_DIR, MEMORY_DIR, MEMORY_FILE, SESSION_FILE
- **Exit:** 0
- **Source:** [feature/005_session_path_resolution.md](../../../../docs/feature/005_session_path_resolution.md) AC-6

---

### SF-7: `--session-from` resumes most recent session from source dir

- **Given:** source dir `/tmp/sf7_src` has session `hhh-101.jsonl` (highest mtime); CWD is `/tmp/sf7_cwd`; fake claude binary in PATH
- **When:** `clr --session-from /tmp/sf7_src --dry-run "Continue"`
- **Then:** dry-run output includes `-c hhh-101`; subprocess working directory is CWD (not `/tmp/sf7_src`)
- **Exit:** 0
- **Source:** [feature/005_session_path_resolution.md](../../../../docs/feature/005_session_path_resolution.md) AC-7

---

### SF-8: `--to` + `--session-from`: Claude runs in target, loads from source

- **Given:** source dir `/tmp/sf8_src` has session `iii-202.jsonl`; target dir `/tmp/sf8_tgt` exists; fake claude binary in PATH
- **When:** `clr --to /tmp/sf8_tgt --session-from /tmp/sf8_src --dry-run "Continue"`
- **Then:** dry-run output includes `-c iii-202`; subprocess working directory is `/tmp/sf8_tgt`
- **Exit:** 0
- **Source:** [feature/005_session_path_resolution.md](../../../../docs/feature/005_session_path_resolution.md) AC-8

---

### SF-9: `--to` is an alias for `--dir`; behavior is identical

- **Given:** target dir `/tmp/sf9_tgt` exists; fake claude binary in PATH
- **When (a):** `clr --dir /tmp/sf9_tgt --dry-run "task"` and **When (b):** `clr --to /tmp/sf9_tgt --dry-run "task"`
- **Then:** both produce identical dry-run output; subprocess working directory is `/tmp/sf9_tgt` in both cases
- **Exit:** 0 both cases
- **Source:** [feature/005_session_path_resolution.md](../../../../docs/feature/005_session_path_resolution.md) AC-9

---

### SF-10: `--session-dir` takes precedence over `--session-from`

- **Given:** source dir `/tmp/sf10_src` has session `jjj-303.jsonl`; raw session dir `/tmp/sf10_raw` has session `kkk-404.jsonl`; fake claude binary in PATH
- **When:** `clr --session-from /tmp/sf10_src --session-dir /tmp/sf10_raw --dry-run "test"`
- **Then:** dry-run output contains `-c kkk-404`; `jjj-303` is NOT used; `--session-dir` (raw path) wins
- **Exit:** 0
- **Source:** [feature/005_session_path_resolution.md](../../../../docs/feature/005_session_path_resolution.md) AC-10
