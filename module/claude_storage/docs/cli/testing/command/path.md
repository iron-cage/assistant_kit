# Command :: `.path`

Integration tests for the `.path` command. Tests verify storage path computation, topic suffix handling, and path resolution.

**Source:** [commands.md#command--10-path](../../commands.md#command--10-path)

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| IT-1 | Default (cwd) computes correct storage path | Basic Behavior |
| IT-2 | path:: override computes path for given directory | Basic Behavior |
| IT-3 | topic:: appended as suffix to encoded path | Topic Handling |
| IT-4 | path:: with topic:: combines both | Topic Handling |
| IT-5 | Output is a single line ending with / | Output Format |
| IT-6 | Exits with code 0 for nonexistent path | Exit Codes |
| IT-7 | ~ prefix expanded in path:: | Path Resolution |
| IT-8 | path::. resolves to cwd | Path Resolution |
| IT-9 | Empty topic:: rejected | Validation |
| IT-10 | topic:: with slash rejected | Validation |

## Test Coverage Summary

- Basic Behavior: 2 tests (IT-1, IT-2)
- Topic Handling: 2 tests (IT-3, IT-4)
- Output Format: 1 test (IT-5)
- Exit Codes: 1 test (IT-6)
- Path Resolution: 2 tests (IT-7, IT-8)
- Validation: 2 tests (IT-9, IT-10)

## Test Cases

### IT-1: Default (cwd) computes correct storage path

**Goal:** Verify `.path` with no arguments returns the storage path for the current working directory.
**Setup:** Run from `/home/user1/pro/lib/consumer`; `CLAUDE_STORAGE_ROOT` is not relevant (`.path` computes the path, not the root).
**Command:** `clg .path`
**Expected Output:** Single line: `~/.claude/projects/-home-user1-pro-lib-consumer/` (or the absolute equivalent).
**Verification:**
- Exit code is `0`
- stdout contains the path-encoded project directory for cwd
- Output includes `/.claude/projects/`
- stderr is empty
**Pass Criteria:** exit 0 + correct path-encoded directory for cwd

**Source:** [commands.md](../../commands.md)

---

### IT-2: path:: override computes path for given directory

**Goal:** Verify `.path path::PATH` returns the storage path for the given directory.
**Setup:** None specific.
**Command:** `clg .path path::/home/user1/pro/lib/consumer`
**Expected Output:** Single line containing `/.claude/projects/-home-user1-pro-lib-consumer/`.
**Verification:**
- Exit code is `0`
- Output contains the correct path-encoded directory
- stderr is empty
**Pass Criteria:** exit 0 + path-encoded directory for the given path

**Source:** [commands.md](../../commands.md)

---

### IT-3: topic:: appended as suffix to encoded path

**Goal:** Verify `topic::` appends `--{topic}` to the encoded path directory.
**Setup:** None specific.
**Command:** `clg .path path::/home/user1/pro/lib/consumer topic::default_topic`
**Expected Output:** Single line containing `/-home-user1-pro-lib-consumer--default-topic/`.
**Verification:**
- Exit code is `0`
- Output ends with `--default-topic/` (topic encoded with double hyphen separator)
- stderr is empty
**Pass Criteria:** exit 0 + topic suffix present in output

**Source:** [commands.md](../../commands.md)

---

### IT-4: path:: with topic:: combines both

**Goal:** Verify that specifying both `path::` and `topic::` produces the combined storage path.
**Setup:** None specific.
**Command:** `clg .path path::~/pro/lib/myapp topic::work`
**Expected Output:** Single line containing `{encoded_home_pro_lib_myapp}--work/`.
**Verification:**
- Exit code is `0`
- Output contains the encoded base path + `--work` suffix
- stderr is empty
**Pass Criteria:** exit 0 + both base path and topic suffix reflected in output

**Source:** [commands.md](../../commands.md)

---

### IT-5: Output is a single line ending with /

**Goal:** Verify the output format is a single line (the storage directory path).
**Setup:** None specific.
**Command:** `clg .path path::/tmp/test-dir`
**Expected Output:** Exactly one non-empty line; the path ends with `/`.
**Verification:**
- Line count is exactly 1
- The line ends with `/`
- No trailing newlines beyond the single line
**Pass Criteria:** Single-line output ending with `/`

**Source:** [commands.md](../../commands.md)

---

### IT-6: Exits with code 0 for nonexistent path

**Goal:** Verify `.path` exits with code `0` even when the given path does not exist on the filesystem.
**Setup:** Use a path that definitely does not exist.
**Command:** `clg .path path::/tmp/nonexistent-project-abc123`
**Expected Output:** Single line with computed storage path; exit code 0.
**Verification:**
- `$?` is `0`
- No error about nonexistent path on stderr
- Output contains the computed storage directory for the given (nonexistent) path
**Pass Criteria:** exit 0 + storage path computed regardless of existence

**Source:** [commands.md](../../commands.md)

---

### IT-7: ~ prefix expanded in path::

**Goal:** Verify `~`-prefixed paths are expanded correctly.
**Setup:** None specific.
**Command:** `clg .path path::~/pro/lib/consumer`
**Expected Output:** Storage path containing the expanded home directory (not literal `~`).
**Verification:**
- Exit code is `0`
- Output does not contain literal `~`
- Output contains the absolute home directory path
**Pass Criteria:** exit 0 + `~` expanded to absolute home path

**Source:** [commands.md](../../commands.md)

---

### IT-8: path::. resolves to cwd

**Goal:** Verify `path::.` is equivalent to omitting `path::` (both resolve to cwd).
**Setup:** Same working directory for both commands.
**Command:** `clg .path path::.`
**Expected Output:** Same output as `clg .path` from the same directory.
**Verification:**
- Exit code is `0`
- Output from `clg .path path::.` equals output from `clg .path`
**Pass Criteria:** exit 0 + identical output to bare `.path`

**Source:** [commands.md](../../commands.md)

---

### IT-9: Empty topic:: rejected

**Goal:** Verify that an empty `topic::` value returns an error.
**Setup:** None specific.
**Command:** `clg .path topic::`
**Expected Output:** Error message about empty topic; exit code 1.
**Verification:**
- `$?` is `1`
- stderr contains a message about empty topic
- stdout is empty
**Pass Criteria:** exit 1 + error message about empty topic

**Source:** [commands.md](../../commands.md)

---

### IT-10: topic:: with slash rejected

**Goal:** Verify that a topic containing `/` is rejected.
**Setup:** None specific.
**Command:** `clg .path topic::my/topic`
**Expected Output:** Error message about path separators in topic; exit code 1.
**Verification:**
- `$?` is `1`
- stderr contains message about path separators
- stdout is empty
**Pass Criteria:** exit 1 + error message about slash in topic

**Source:** [commands.md](../../commands.md)
