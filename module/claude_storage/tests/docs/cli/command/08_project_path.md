# Command :: `.project.path`

Integration tests for the `.project.path` command. Tests verify storage path computation, topic suffix handling, and path resolution.

**Source:** [command/08_project_path.md](../../../../docs/cli/command/08_project_path.md)

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| INT-1 | Default (cwd) computes correct storage path | Basic Behavior |
| INT-2 | path:: override computes path for given directory | Basic Behavior |
| INT-3 | topic:: appended as suffix to encoded path | Topic Handling |
| INT-4 | path:: with topic:: combines both | Topic Handling |
| INT-5 | Output is a single line ending with / | Output Format |
| INT-6 | Exits with code 0 for nonexistent path | Exit Codes |
| INT-7 | ~ prefix expanded in path:: | Path Resolution |
| INT-8 | path::. resolves to cwd | Path Resolution |
| INT-9 | Empty topic:: rejected | Validation |
| INT-10 | topic:: with slash rejected | Validation |

## Test Coverage Summary

- Basic Behavior: 2 tests (INT-1, INT-2)
- Topic Handling: 2 tests (INT-3, INT-4)
- Output Format: 1 test (INT-5)
- Exit Codes: 1 test (INT-6)
- Path Resolution: 2 tests (INT-7, INT-8)
- Validation: 2 tests (INT-9, INT-10)

## Test Cases

---

### INT-1: Default (cwd) computes correct storage path

**Command:**
```
clg .project.path
```

**Expected behavior:**
- Run from `/home/alice/projects/consumer-app`
- Single line: `~/.claude/projects/-home-alice-projects-consumer-app/` (or the absolute equivalent)
- Exit code: 0
- **Source:** [command/08_project_path.md](../../../../docs/cli/command/08_project_path.md)

---

### INT-2: path:: override computes path for given directory

**Command:**
```
clg .project.path path::/home/alice/projects/consumer-app
```

**Expected behavior:**
- Single line containing `/.claude/projects/-home-alice-projects-consumer-app/`
- Exit code: 0
- **Source:** [command/08_project_path.md](../../../../docs/cli/command/08_project_path.md)

---

### INT-3: topic:: appended as suffix to encoded path

**Command:**
```
clg .project.path path::/home/alice/projects/consumer-app topic::default_topic
```

**Expected behavior:**
- Single line containing `/-home-alice-projects-consumer-app--default-topic/`
- Exit code: 0
- **Source:** [command/08_project_path.md](../../../../docs/cli/command/08_project_path.md)

---

### INT-4: path:: with topic:: combines both

**Command:**
```
clg .project.path path::~/projects/myapp topic::work
```

**Expected behavior:**
- Single line containing `{encoded_home_projects_myapp}--work/`
- Exit code: 0
- **Source:** [command/08_project_path.md](../../../../docs/cli/command/08_project_path.md)

---

### INT-5: Output is a single line ending with /

**Command:**
```
clg .project.path path::/tmp/test-dir
```

**Expected behavior:**
- Exactly one non-empty line; the path ends with `/`
- Exit code: 0
- **Source:** [command/08_project_path.md](../../../../docs/cli/command/08_project_path.md)

---

### INT-6: Exits with code 0 for nonexistent path

**Command:**
```
clg .project.path path::/tmp/nonexistent-project-abc123
```

**Expected behavior:**
- Fixture: use a path that definitely does not exist
- Single line with computed storage path; path computation succeeds regardless of existence
- Exit code: 0
- **Source:** [command/08_project_path.md](../../../../docs/cli/command/08_project_path.md)

---

### INT-7: ~ prefix expanded in path::

**Command:**
```
clg .project.path path::~/projects/consumer-app
```

**Expected behavior:**
- Storage path containing the expanded home directory (not literal `~`)
- Exit code: 0
- **Source:** [command/08_project_path.md](../../../../docs/cli/command/08_project_path.md)

---

### INT-8: path::. resolves to cwd

**Command:**
```
clg .project.path path::.
```

**Expected behavior:**
- Same output as `clg .project.path` from the same directory
- Exit code: 0
- **Source:** [command/08_project_path.md](../../../../docs/cli/command/08_project_path.md)

---

### INT-9: Empty topic:: rejected

**Command:**
```
clg .project.path topic::
```

**Expected behavior:**
- Error message on stderr about empty topic
- Exit code: 1
- **Source:** [command/08_project_path.md](../../../../docs/cli/command/08_project_path.md)

---

### INT-10: topic:: with slash rejected

**Command:**
```
clg .project.path topic::my/topic
```

**Expected behavior:**
- Error message on stderr about path separators in topic
- Exit code: 1
- **Source:** [command/08_project_path.md](../../../../docs/cli/command/08_project_path.md)
