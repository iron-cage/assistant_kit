# Command :: `.project.exists`

Integration tests for the `.project.exists` command. Tests verify exit code semantics, output format, topic handling, and shell composability.

**Source:** [command/09_project_exists.md](../../../../docs/cli/command/09_project_exists.md)

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| INT-1 | cwd with history exits 0 | Exit Codes |
| INT-2 | cwd without history exits 1 | Exit Codes |
| INT-3 | path:: with history exits 0 | Exit Codes |
| INT-4 | path:: without history exits 1 | Exit Codes |
| INT-5 | Exit 0 prints "sessions exist" to stdout | Output Format |
| INT-6 | Exit 1 prints "no sessions" to stderr | Output Format |
| INT-7 | topic:: filters to topic-specific storage | Topic Handling |
| INT-8 | topic:: no history exits 1 | Topic Handling |
| INT-9 | Nonexistent path exits 1 (not error) | Exit Codes |
| INT-10 | Empty topic:: rejected with exit 1 | Validation |

## Test Coverage Summary

- Exit Codes: 5 tests (INT-1, INT-2, INT-3, INT-4, INT-9)
- Output Format: 2 tests (INT-5, INT-6)
- Topic Handling: 2 tests (INT-7, INT-8)
- Validation: 1 test (INT-10)

## Test Cases

---

### INT-1: cwd with history exits 0

**Command:**
```
clg .project.exists
```

**Expected behavior:**
- Fixture: create a TempDir as HOME; create `~/.claude/projects/{encoded_cwd}/` with a non-empty `.jsonl` file; run from the cwd corresponding to the encoded path
- `"sessions exist\n"` on stdout
- Exit code: 0
- **Source:** [command/09_project_exists.md](../../../../docs/cli/command/09_project_exists.md)

---

### INT-2: cwd without history exits 1

**Command:**
```
clg .project.exists
```

**Expected behavior:**
- Fixture: create a TempDir as HOME with no matching project in `~/.claude/projects/`; run from a directory with no storage entry
- `"no sessions"` on stderr
- Exit code: 1
- **Source:** [command/09_project_exists.md](../../../../docs/cli/command/09_project_exists.md)

---

### INT-3: path:: with history exits 0

**Command:**
```
clg .project.exists path::/home/alice/projects/alpha
```

**Expected behavior:**
- Fixture: create a TempDir as HOME; create a project entry for `/home/alice/projects/alpha`
- stdout: `"sessions exist\n"`
- Exit code: 0
- **Source:** [command/09_project_exists.md](../../../../docs/cli/command/09_project_exists.md)

---

### INT-4: path:: without history exits 1

**Command:**
```
clg .project.exists path::/home/alice/projects/no-history
```

**Expected behavior:**
- Fixture: create a TempDir as HOME; path has no matching project entry
- stderr: `"no sessions"`
- Exit code: 1
- **Source:** [command/09_project_exists.md](../../../../docs/cli/command/09_project_exists.md)

---

### INT-5: Exit 0 prints "sessions exist" to stdout

**Command:**
```
clg .project.exists path::PATH_WITH_HISTORY
```

**Expected behavior:**
- Fixture: directory with history in storage
- stdout is exactly `"sessions exist\n"` (one line); stderr is empty
- Exit code: 0
- **Source:** [command/09_project_exists.md](../../../../docs/cli/command/09_project_exists.md)

---

### INT-6: Exit 1 prints "no sessions" to stderr

**Command:**
```
clg .project.exists path::PATH_WITHOUT_HISTORY
```

**Expected behavior:**
- Fixture: directory with no history in storage
- stderr contains `"no sessions"`; stdout is empty
- Exit code: 1
- **Source:** [command/09_project_exists.md](../../../../docs/cli/command/09_project_exists.md)

---

### INT-7: topic:: filters to topic-specific storage

**Command:**
```
clg .project.exists path::/home/user/project topic::work
```

**Expected behavior:**
- Fixture: create a TempDir as HOME; create `~/.claude/projects/-home-user-project--work/` with a non-empty `.jsonl`; the base path `-home-user-project` directory does NOT have history
- `"sessions exist\n"` on stdout; topic-specific storage directory detected
- Exit code: 0
- **Source:** [command/09_project_exists.md](../../../../docs/cli/command/09_project_exists.md)

---

### INT-8: topic:: no history exits 1

**Command:**
```
clg .project.exists path::/home/user/project topic::nonexistent
```

**Expected behavior:**
- Fixture: create a TempDir as HOME; the base directory has history but the topic-specific directory does not
- stderr: `"no sessions"`; base history does not satisfy topic-specific check
- Exit code: 1
- **Source:** [command/09_project_exists.md](../../../../docs/cli/command/09_project_exists.md)

---

### INT-9: Nonexistent path exits 1 (not error)

**Command:**
```
clg .project.exists path::/tmp/nonexistent-xyz-abc
```

**Expected behavior:**
- `"no sessions"` on stderr; nonexistent path treated as no history, not storage error (exit 1, not 2)
- Exit code: 1
- **Source:** [command/09_project_exists.md](../../../../docs/cli/command/09_project_exists.md)

---

### INT-10: Empty topic:: rejected with exit 1

**Command:**
```
clg .project.exists topic::
```

**Expected behavior:**
- Error message on stderr about empty topic
- Exit code: 1
- **Source:** [command/09_project_exists.md](../../../../docs/cli/command/09_project_exists.md)
