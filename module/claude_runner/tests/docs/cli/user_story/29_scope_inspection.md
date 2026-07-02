# User Story :: Scope Inspection

Test case spec for [029_scope_inspection.md](../../../../docs/cli/user_story/029_scope_inspection.md).

## Test Case Index

| ID | Test Name | AC | Status |
|----|-----------|-----|--------|
| US-1 | `clr scope` (no args) prints 6 vars for CWD | AC-1 | ⏳ |
| US-2 | `clr scope --dir <path>` prints 6 vars for that directory | AC-2 | ⏳ |
| US-3 | `CLAUDE_SESSION_FILE` populated when session file exists | AC-3 | ⏳ |
| US-4 | `CLAUDE_SESSION_FILE=` printed empty when no session history | AC-4 | ⏳ |
| US-5 | `CLAUDE_HOME` override reflected in all 6 variables | AC-5 | ⏳ |
| US-6 | `CLAUDE_COWORK_MEMORY_PATH_OVERRIDE` reflected in memory vars | AC-6 | ⏳ |
| US-7 | Output is eval-safe `key=value` format, one var per line | AC-7 | ⏳ |
| US-8 | Exit 0 on success; exit 1 when `--dir` path does not exist | AC-8 | ⏳ |

---

### US-1: `clr scope` (no args) prints 6 vars for CWD

- **Given:** CWD is an existing directory; no `--dir` flag
- **When:** `clr scope`
- **Then:** stdout contains exactly 6 `CLAUDE_*` variable lines: `CLAUDE_HOME`, `CLAUDE_PROJECTS_DIR`, `CLAUDE_SESSION_DIR`, `CLAUDE_MEMORY_DIR`, `CLAUDE_MEMORY_FILE`, `CLAUDE_SESSION_FILE`; all values non-empty (except optionally `CLAUDE_SESSION_FILE`)
- **Exit:** 0
- **Verifies:** AC-1

---

### US-2: `clr scope --dir <path>` prints 6 vars for that directory

- **Given:** directory `/tmp/scope_test_proj_us2` exists; no `.jsonl` files in its Claude storage
- **When:** `clr scope --dir /tmp/scope_test_proj_us2`
- **Then:** stdout contains `CLAUDE_SESSION_DIR` with a path derived from `/tmp/scope_test_proj_us2`; all 6 vars printed
- **Exit:** 0
- **Verifies:** AC-2

---

### US-3: `CLAUDE_SESSION_FILE` populated when session file exists

- **Given:** directory `/tmp/scope_test_proj_us3` has a non-empty `.jsonl` session file in its Claude storage; fake claude binary in PATH
- **When:** `clr scope --dir /tmp/scope_test_proj_us3`
- **Then:** stdout line `CLAUDE_SESSION_FILE=` has a non-empty path value ending in `.jsonl`
- **Exit:** 0
- **Verifies:** AC-3

---

### US-4: `CLAUDE_SESSION_FILE=` empty when no session history

- **Given:** directory `/tmp/scope_test_proj_us4` exists; Claude storage for that dir contains no `.jsonl` files
- **When:** `clr scope --dir /tmp/scope_test_proj_us4`
- **Then:** stdout contains `CLAUDE_SESSION_FILE=` (empty value, no path after `=`)
- **Exit:** 0
- **Verifies:** AC-4

---

### US-5: `CLAUDE_HOME` override reflected in all 6 variables

- **Given:** `CLAUDE_HOME=/tmp/custom_home_us5` set in environment; directory `/tmp/scope_test_proj_us5` exists
- **When:** `CLAUDE_HOME=/tmp/custom_home_us5 clr scope --dir /tmp/scope_test_proj_us5`
- **Then:** stdout line `CLAUDE_HOME=/tmp/custom_home_us5`; `CLAUDE_PROJECTS_DIR` starts with `/tmp/custom_home_us5`; `CLAUDE_SESSION_DIR` starts with `/tmp/custom_home_us5`; all path vars reflect custom home
- **Exit:** 0
- **Verifies:** AC-5

---

### US-6: `CLAUDE_COWORK_MEMORY_PATH_OVERRIDE` reflected in memory vars

- **Given:** `CLAUDE_COWORK_MEMORY_PATH_OVERRIDE=/tmp/shared_memory_us6` set in environment; directory `/tmp/scope_test_proj_us6` exists
- **When:** `CLAUDE_COWORK_MEMORY_PATH_OVERRIDE=/tmp/shared_memory_us6 clr scope --dir /tmp/scope_test_proj_us6`
- **Then:** stdout line `CLAUDE_MEMORY_DIR=/tmp/shared_memory_us6`; `CLAUDE_MEMORY_FILE=/tmp/shared_memory_us6/MEMORY.md`; `CLAUDE_SESSION_DIR` still uses normal Df() derivation
- **Exit:** 0
- **Verifies:** AC-6

---

### US-7: Output is eval-safe `key=value` format, one var per line

- **Given:** `clr scope` executed in any valid directory
- **When:** output captured and passed to `eval`
- **Then:** each output line matches `^[A-Z_]+=.*$`; no shell metacharacters (spaces, quotes, semicolons) in variable values; `eval "$(clr scope)"` sets all 6 env vars without error
- **Exit:** 0
- **Verifies:** AC-7

---

### US-8: Exit 0 on success; exit 1 on nonexistent `--dir`

- **Given:** `/tmp/nonexistent_scope_dir_us8` does not exist
- **When:** `clr scope --dir /tmp/nonexistent_scope_dir_us8`
- **Then:** exit code is 1; stderr contains an error message referencing the path
- **Exit:** 1
- **Verifies:** AC-8
