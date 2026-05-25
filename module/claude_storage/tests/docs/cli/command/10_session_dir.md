# Command :: `.session.dir`

Integration tests for the `.session.dir` command. Tests verify session directory path computation, topic handling, and validation.

**Source:** [command/10_session_dir.md](../../../../docs/cli/command/10_session_dir.md)

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| INT-1 | path:: with default topic produces {base}/-default_topic | Basic Behavior |
| INT-2 | path:: with custom topic produces {base}/-{topic} | Topic Handling |
| INT-3 | path:: required — missing path:: returns error | Validation |
| INT-4 | Output is a single line (absolute path) | Output Format |
| INT-5 | ~ prefix expanded in path:: | Path Resolution |
| INT-6 | path::. resolves to cwd | Path Resolution |
| INT-7 | Empty topic:: rejected | Validation |
| INT-8 | topic:: with slash rejected | Validation |
| INT-9 | Does not create directory | Behavior Boundary |
| INT-10 | Exits 0 even if path does not exist on disk | Exit Codes |

## Test Coverage Summary

- Basic Behavior: 1 test (INT-1)
- Topic Handling: 1 test (INT-2)
- Validation: 3 tests (INT-3, INT-7, INT-8)
- Output Format: 1 test (INT-4)
- Path Resolution: 2 tests (INT-5, INT-6)
- Behavior Boundary: 1 test (INT-9)
- Exit Codes: 1 test (INT-10)

## Test Cases

---

### INT-1: path:: with default topic produces {base}/-default_topic

**Command:**
```
clg .session.dir path::/home/user/project
```

**Expected behavior:**
- Output: `/home/user/project/-default_topic`
- Exit code: 0
- **Source:** [command/10_session_dir.md](../../../../docs/cli/command/10_session_dir.md)

---

### INT-2: path:: with custom topic produces {base}/-{topic}

**Command:**
```
clg .session.dir path::/home/user/project topic::work
```

**Expected behavior:**
- Output: `/home/user/project/-work`
- Exit code: 0
- **Source:** [command/10_session_dir.md](../../../../docs/cli/command/10_session_dir.md)

---

### INT-3: path:: required — missing path:: returns error

**Command:**
```
clg .session.dir
```

**Expected behavior:**
- Error message on stderr about required path
- Exit code: 1
- **Source:** [command/10_session_dir.md](../../../../docs/cli/command/10_session_dir.md)

---

### INT-4: Output is a single line (absolute path)

**Command:**
```
clg .session.dir path::/home/user/project
```

**Expected behavior:**
- Exactly one non-empty line starting with `/`
- Exit code: 0
- **Source:** [command/10_session_dir.md](../../../../docs/cli/command/10_session_dir.md)

---

### INT-5: ~ prefix expanded in path::

**Command:**
```
clg .session.dir path::~/projects/myapp
```

**Expected behavior:**
- An absolute path without `~`; ends with `/-default_topic`
- Exit code: 0
- **Source:** [command/10_session_dir.md](../../../../docs/cli/command/10_session_dir.md)

---

### INT-6: path::. resolves to cwd

**Command:**
```
clg .session.dir path::.
```

**Expected behavior:**
- Output: `{cwd}/-default_topic` (absolute cwd-based path)
- Exit code: 0
- **Source:** [command/10_session_dir.md](../../../../docs/cli/command/10_session_dir.md)

---

### INT-7: Empty topic:: rejected

**Command:**
```
clg .session.dir path::/home/user/project topic::
```

**Expected behavior:**
- Error message on stderr about empty topic
- Exit code: 1
- **Source:** [command/10_session_dir.md](../../../../docs/cli/command/10_session_dir.md)

---

### INT-8: topic:: with slash rejected

**Command:**
```
clg .session.dir path::/home/user/project topic::sub/dir
```

**Expected behavior:**
- Error message on stderr about path separators in topic
- Exit code: 1
- **Source:** [command/10_session_dir.md](../../../../docs/cli/command/10_session_dir.md)

---

### INT-9: Does not create directory

**Command:**
```
clg .session.dir path::/tmp/no-such-base-abc topic::default_topic
```

**Expected behavior:**
- Fixture: use a base path that definitely does not have a `-default_topic` subdirectory
- Single-line path output; the directory `/tmp/no-such-base-abc/-default_topic` is NOT created
- Exit code: 0
- **Source:** [command/10_session_dir.md](../../../../docs/cli/command/10_session_dir.md)

---

### INT-10: Exits 0 even if path does not exist on disk

**Command:**
```
clg .session.dir path::/tmp/nonexistent-xyz-abc
```

**Expected behavior:**
- Fixture: use a path that does not exist on disk
- Single-line path output; path computation is filesystem-independent
- Exit code: 0
- **Source:** [command/10_session_dir.md](../../../../docs/cli/command/10_session_dir.md)
