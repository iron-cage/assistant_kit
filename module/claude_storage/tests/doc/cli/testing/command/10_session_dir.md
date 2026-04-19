# Command :: `.session.dir`

Integration tests for the `.session.dir` command. Tests verify session directory path computation, topic handling, and validation.

**Source:** [commands.md#command--12-sessiondir](../../../../../docs/cli/commands.md#command--12-sessiondir)

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| IT-1 | path:: with default topic produces {base}/-default_topic | Basic Behavior |
| IT-2 | path:: with custom topic produces {base}/-{topic} | Topic Handling |
| IT-3 | path:: required — missing path:: returns error | Validation |
| IT-4 | Output is a single line (absolute path) | Output Format |
| IT-5 | ~ prefix expanded in path:: | Path Resolution |
| IT-6 | path::. resolves to cwd | Path Resolution |
| IT-7 | Empty topic:: rejected | Validation |
| IT-8 | topic:: with slash rejected | Validation |
| IT-9 | Does not create directory | Behavior Boundary |
| IT-10 | Exits 0 even if path does not exist on disk | Exit Codes |

## Test Coverage Summary

- Basic Behavior: 1 test (IT-1)
- Topic Handling: 1 test (IT-2)
- Validation: 3 tests (IT-3, IT-7, IT-8)
- Output Format: 1 test (IT-4)
- Path Resolution: 2 tests (IT-5, IT-6)
- Behavior Boundary: 1 test (IT-9)
- Exit Codes: 1 test (IT-10)

## Test Cases

### IT-1: path:: with default topic produces {base}/-default_topic

**Goal:** Verify `.session.dir path::PATH` (no topic) returns `{base}/-default_topic`.
**Setup:** None specific.
**Command:** `clg .session.dir path::/home/user/project`
**Expected Output:** `/home/user/project/-default_topic`
**Verification:**
- Exit code is `0`
- stdout is `/home/user/project/-default_topic\n`
- stderr is empty
**Pass Criteria:** exit 0 + correct default topic path

**Source:** [commands.md](../../../../../docs/cli/commands.md)

---

### IT-2: path:: with custom topic produces {base}/-{topic}

**Goal:** Verify `.session.dir path::PATH topic::TOPIC` returns `{base}/-{topic}`.
**Setup:** None specific.
**Command:** `clg .session.dir path::/home/user/project topic::work`
**Expected Output:** `/home/user/project/-work`
**Verification:**
- Exit code is `0`
- stdout is `/home/user/project/-work\n`
- stderr is empty
**Pass Criteria:** exit 0 + `/-work` suffix in output

**Source:** [commands.md](../../../../../docs/cli/commands.md)

---

### IT-3: path:: required — missing path:: returns error

**Goal:** Verify that omitting `path::` produces an error (exit 1).
**Setup:** None specific.
**Command:** `clg .session.dir`
**Expected Output:** Error on stderr; exit code 1.
**Verification:**
- `$?` is `1`
- stderr contains an error about missing `path::` parameter
- stdout is empty
**Pass Criteria:** exit 1 + error message about required path

**Source:** [commands.md](../../../../../docs/cli/commands.md)

---

### IT-4: Output is a single line (absolute path)

**Goal:** Verify the output is exactly one line containing an absolute path.
**Setup:** None specific.
**Command:** `clg .session.dir path::/home/user/project`
**Expected Output:** Exactly one line starting with `/`.
**Verification:**
- Line count is exactly 1
- The line starts with `/`
- No extra whitespace or trailing content beyond the path
**Pass Criteria:** single-line absolute path output

**Source:** [commands.md](../../../../../docs/cli/commands.md)

---

### IT-5: ~ prefix expanded in path::

**Goal:** Verify `~`-prefixed paths are expanded to absolute paths in the output.
**Setup:** None specific.
**Command:** `clg .session.dir path::~/pro/lib/myapp`
**Expected Output:** An absolute path without `~`; ends with `/-default_topic`.
**Verification:**
- Exit code is `0`
- Output does not contain literal `~`
- Output contains the home directory path
- Output ends with `/-default_topic`
**Pass Criteria:** exit 0 + `~` expanded in output

**Source:** [commands.md](../../../../../docs/cli/commands.md)

---

### IT-6: path::. resolves to cwd

**Goal:** Verify `path::.` resolves to the current working directory.
**Setup:** None specific; run from a known directory.
**Command:** `clg .session.dir path::.`
**Expected Output:** `{cwd}/-default_topic`
**Verification:**
- Exit code is `0`
- Output matches `{cwd}/-default_topic` where cwd is the actual working directory
**Pass Criteria:** exit 0 + output is absolute cwd-based path

**Source:** [commands.md](../../../../../docs/cli/commands.md)

---

### IT-7: Empty topic:: rejected

**Goal:** Verify that `topic::` with an empty value returns an error.
**Setup:** None specific.
**Command:** `clg .session.dir path::/home/user/project topic::`
**Expected Output:** Error about empty topic; exit code 1.
**Verification:**
- `$?` is `1`
- stderr contains an error about empty topic
**Pass Criteria:** exit 1 + error about empty topic

**Source:** [commands.md](../../../../../docs/cli/commands.md)

---

### IT-8: topic:: with slash rejected

**Goal:** Verify that a topic containing `/` is rejected.
**Setup:** None specific.
**Command:** `clg .session.dir path::/home/user/project topic::sub/dir`
**Expected Output:** Error about path separators in topic; exit code 1.
**Verification:**
- `$?` is `1`
- stderr contains message about path separators
**Pass Criteria:** exit 1 + error about slash in topic

**Source:** [commands.md](../../../../../docs/cli/commands.md)

---

### IT-9: Does not create directory

**Goal:** Verify `.session.dir` only computes the path and does not create the directory.
**Setup:** Use a base path that definitely does not have a `-default_topic` subdirectory.
**Command:** `clg .session.dir path::/tmp/no-such-base-abc topic::default_topic`
**Expected Output:** Single-line path output; the directory `/tmp/no-such-base-abc/-default_topic` is NOT created.
**Verification:**
- Exit code is `0`
- `/tmp/no-such-base-abc/-default_topic` does NOT exist after the command
**Pass Criteria:** exit 0 + no directory created on disk

**Source:** [commands.md](../../../../../docs/cli/commands.md)

---

### IT-10: Exits 0 even if path does not exist on disk

**Goal:** Verify exit code is 0 even when the base path does not exist.
**Setup:** Use a path that does not exist.
**Command:** `clg .session.dir path::/tmp/nonexistent-xyz-abc`
**Expected Output:** Single-line path; exit code 0.
**Verification:**
- `$?` is `0`
- stdout contains the computed path
- No error about nonexistent path
**Pass Criteria:** exit 0 — path computation is filesystem-independent

**Source:** [commands.md](../../../../../docs/cli/commands.md)
