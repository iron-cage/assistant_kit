# Command :: `.exists`

Integration tests for the `.exists` command. Tests verify exit code semantics, output format, topic handling, and shell composability.

**Source:** [commands.md#command--11-exists](../../../../../docs/cli/commands.md#command--11-exists)

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| IT-1 | cwd with history exits 0 | Exit Codes |
| IT-2 | cwd without history exits 1 | Exit Codes |
| IT-3 | path:: with history exits 0 | Exit Codes |
| IT-4 | path:: without history exits 1 | Exit Codes |
| IT-5 | Exit 0 prints "sessions exist" to stdout | Output Format |
| IT-6 | Exit 1 prints "no sessions" to stderr | Output Format |
| IT-7 | topic:: filters to topic-specific storage | Topic Handling |
| IT-8 | topic:: no history exits 1 | Topic Handling |
| IT-9 | Nonexistent path exits 1 (not error) | Exit Codes |
| IT-10 | Empty topic:: rejected with exit 1 | Validation |

## Test Coverage Summary

- Exit Codes: 5 tests (IT-1, IT-2, IT-3, IT-4, IT-9)
- Output Format: 2 tests (IT-5, IT-6)
- Topic Handling: 2 tests (IT-7, IT-8)
- Validation: 1 test (IT-10)

## Test Cases

### IT-1: cwd with history exits 0

**Goal:** Verify `.exists` exits with code `0` when the current directory has conversation history.
**Setup:** Create a TempDir as HOME; create `~/.claude/projects/{encoded_cwd}/` with a non-empty `.jsonl` file. Run from the cwd corresponding to the encoded path.
**Command:** `clg .exists`
**Expected Output:** `"sessions exist\n"` on stdout; exit code 0.
**Verification:**
- `$?` is `0`
- stdout is `"sessions exist\n"`
- stderr is empty
**Pass Criteria:** exit 0 + stdout is "sessions exist"

**Source:** [commands.md](../../../../../docs/cli/commands.md)

---

### IT-2: cwd without history exits 1

**Goal:** Verify `.exists` exits with code `1` when the current directory has no conversation history.
**Setup:** Create a TempDir as HOME with no matching project in `~/.claude/projects/`. Run from a directory with no storage entry.
**Command:** `clg .exists`
**Expected Output:** `"no sessions"` on stderr; exit code 1.
**Verification:**
- `$?` is `1`
- stderr contains `"no sessions"`
- stdout is empty
**Pass Criteria:** exit 1 + stderr is "no sessions"

**Source:** [commands.md](../../../../../docs/cli/commands.md)

---

### IT-3: path:: with history exits 0

**Goal:** Verify `.exists path::PATH` exits `0` when the path has history.
**Setup:** Create a TempDir as HOME; create a project entry for `/home/user1/pro/alpha`.
**Command:** `clg .exists path::/home/user1/pro/alpha`
**Expected Output:** stdout `"sessions exist\n"`; exit code 0.
**Verification:**
- `$?` is `0`
- stdout is `"sessions exist\n"`
- stderr is empty
**Pass Criteria:** exit 0 + stdout is "sessions exist"

**Source:** [commands.md](../../../../../docs/cli/commands.md)

---

### IT-4: path:: without history exits 1

**Goal:** Verify `.exists path::PATH` exits `1` when the path has no history.
**Setup:** Create a TempDir as HOME; path has no matching project entry.
**Command:** `clg .exists path::/home/user1/pro/no-history`
**Expected Output:** stderr `"no sessions"`; exit code 1.
**Verification:**
- `$?` is `1`
- stderr contains `"no sessions"`
- stdout is empty
**Pass Criteria:** exit 1 + stderr is "no sessions"

**Source:** [commands.md](../../../../../docs/cli/commands.md)

---

### IT-5: Exit 0 prints "sessions exist" to stdout

**Goal:** Verify the exact output text when history is found.
**Setup:** Directory with history in storage.
**Command:** `clg .exists path::PATH_WITH_HISTORY`
**Expected Output:** stdout is exactly `"sessions exist\n"` (one line).
**Verification:**
- stdout equals `"sessions exist\n"` exactly
- stderr is empty
- exit code 0
**Pass Criteria:** exact stdout match + empty stderr

**Source:** [commands.md](../../../../../docs/cli/commands.md)

---

### IT-6: Exit 1 prints "no sessions" to stderr

**Goal:** Verify the exact stderr output text when no history is found.
**Setup:** Directory with no history in storage.
**Command:** `clg .exists path::PATH_WITHOUT_HISTORY`
**Expected Output:** stderr contains `"no sessions"`.
**Verification:**
- stderr contains `"no sessions"`
- stdout is empty
- exit code 1
**Pass Criteria:** exit 1 + "no sessions" on stderr + empty stdout

**Source:** [commands.md](../../../../../docs/cli/commands.md)

---

### IT-7: topic:: filters to topic-specific storage

**Goal:** Verify that `topic::` checks the storage path `{encoded_path}--{topic}` rather than the base path.
**Setup:** Create a TempDir as HOME; create `~/.claude/projects/-home-user-project--work/` with a non-empty `.jsonl`. The base path `-home-user-project` directory does NOT have history.
**Command:** `clg .exists path::/home/user/project topic::work`
**Expected Output:** `"sessions exist\n"` on stdout; exit code 0.
**Verification:**
- `$?` is `0`
- stdout is `"sessions exist\n"`
- stderr is empty
**Pass Criteria:** exit 0 — topic-specific storage directory detected

**Source:** [commands.md](../../../../../docs/cli/commands.md)

---

### IT-8: topic:: no history exits 1

**Goal:** Verify that `topic::` with no matching storage exits 1.
**Setup:** Create a TempDir as HOME; the base directory has history but the topic-specific directory does not.
**Command:** `clg .exists path::/home/user/project topic::nonexistent`
**Expected Output:** stderr `"no sessions"`; exit code 1.
**Verification:**
- `$?` is `1`
- stderr contains `"no sessions"`
- stdout is empty
**Pass Criteria:** exit 1 — base history does not satisfy topic-specific check

**Source:** [commands.md](../../../../../docs/cli/commands.md)

---

### IT-9: Nonexistent path exits 1 (not error)

**Goal:** Verify that a nonexistent filesystem path produces exit 1 (no history), not exit 2 (error).
**Setup:** None specific; path does not exist on disk.
**Command:** `clg .exists path::/tmp/nonexistent-xyz-abc`
**Expected Output:** exit 1; `"no sessions"` on stderr.
**Verification:**
- `$?` is `1` (not `2`)
- stderr contains `"no sessions"`
**Pass Criteria:** exit 1 (not 2) — nonexistent path treated as no history, not storage error

**Source:** [commands.md](../../../../../docs/cli/commands.md)

---

### IT-10: Empty topic:: rejected with exit 1

**Goal:** Verify that an empty `topic::` value produces an error.
**Setup:** None specific.
**Command:** `clg .exists topic::`
**Expected Output:** Error message about empty topic; exit code 1.
**Verification:**
- `$?` is `1`
- stderr contains an error message about empty topic
- stdout is empty
**Pass Criteria:** exit 1 + error about empty topic

**Source:** [commands.md](../../../../../docs/cli/commands.md)
