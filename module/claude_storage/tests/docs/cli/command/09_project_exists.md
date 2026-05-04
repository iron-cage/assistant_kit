# Command :: `.exists`

Integration tests for the `.exists` command. Tests verify exit code semantics, output format, topic handling, and shell composability.

**Source:** [commands.md#command--11-exists](../../../../docs/cli/commands.md#command--11-exists)

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

---

### IT-1: cwd with history exits 0

- **Given:** Create a TempDir as HOME; create `~/.claude/projects/{encoded_cwd}/` with a non-empty `.jsonl` file. Run from the cwd corresponding to the encoded path.
- **When:** `clg .exists`
- **Then:** `"sessions exist\n"` on stdout; exit code 0.; stdout is "sessions exist"
- **Exit:** 0
- **Source:** [commands.md](../../../../docs/cli/commands.md)

---

### IT-2: cwd without history exits 1

- **Given:** Create a TempDir as HOME with no matching project in `~/.claude/projects/`. Run from a directory with no storage entry.
- **When:** `clg .exists`
- **Then:** `"no sessions"` on stderr; exit code 1.; + stderr is "no sessions"
- **Exit:** 1
- **Source:** [commands.md](../../../../docs/cli/commands.md)

---

### IT-3: path:: with history exits 0

- **Given:** Create a TempDir as HOME; create a project entry for `/home/user1/pro/alpha`.
- **When:** `clg .exists path::/home/user1/pro/alpha`
- **Then:** stdout `"sessions exist\n"`; exit code 0.; + stdout is "sessions exist"
- **Exit:** 0
- **Source:** [commands.md](../../../../docs/cli/commands.md)

---

### IT-4: path:: without history exits 1

- **Given:** Create a TempDir as HOME; path has no matching project entry.
- **When:** `clg .exists path::/home/user1/pro/no-history`
- **Then:** stderr `"no sessions"`; exit code 1.; + stderr is "no sessions"
- **Exit:** 1
- **Source:** [commands.md](../../../../docs/cli/commands.md)

---

### IT-5: Exit 0 prints "sessions exist" to stdout

- **Given:** Directory with history in storage.
- **When:** `clg .exists path::PATH_WITH_HISTORY`
- **Then:** stdout is exactly `"sessions exist\n"` (one line).; exact stdout match + empty stderr
- **Exit:** 0
- **Source:** [commands.md](../../../../docs/cli/commands.md)

---

### IT-6: Exit 1 prints "no sessions" to stderr

- **Given:** Directory with no history in storage.
- **When:** `clg .exists path::PATH_WITHOUT_HISTORY`
- **Then:** stderr contains `"no sessions"`.; + "no sessions" on stderr + empty stdout
- **Exit:** 1
- **Source:** [commands.md](../../../../docs/cli/commands.md)

---

### IT-7: topic:: filters to topic-specific storage

- **Given:** Create a TempDir as HOME; create `~/.claude/projects/-home-user-project--work/` with a non-empty `.jsonl`. The base path `-home-user-project` directory does NOT have history.
- **When:** `clg .exists path::/home/user/project topic::work`
- **Then:** `"sessions exist\n"` on stdout; exit code 0.; — topic-specific storage directory detected
- **Exit:** 0
- **Source:** [commands.md](../../../../docs/cli/commands.md)

---

### IT-8: topic:: no history exits 1

- **Given:** Create a TempDir as HOME; the base directory has history but the topic-specific directory does not.
- **When:** `clg .exists path::/home/user/project topic::nonexistent`
- **Then:** stderr `"no sessions"`; exit code 1.; — base history does not satisfy topic-specific check
- **Exit:** 1
- **Source:** [commands.md](../../../../docs/cli/commands.md)

---

### IT-9: Nonexistent path exits 1 (not error)

- **Given:** clean environment
- **When:** `clg .exists path::/tmp/nonexistent-xyz-abc`
- **Then:** exit 1; `"no sessions"` on stderr.; (not 2) — nonexistent path treated as no history, not storage error
- **Exit:** 1
- **Source:** [commands.md](../../../../docs/cli/commands.md)

---

### IT-10: Empty topic:: rejected with exit 1

- **Given:** clean environment
- **When:** `clg .exists topic::`
- **Then:** Error message about empty topic; exit code 1.; + error about empty topic
- **Exit:** 1
- **Source:** [commands.md](../../../../docs/cli/commands.md)
