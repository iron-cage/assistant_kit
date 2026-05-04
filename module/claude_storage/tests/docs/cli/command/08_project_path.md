# Command :: `.path`

Integration tests for the `.path` command. Tests verify storage path computation, topic suffix handling, and path resolution.

**Source:** [commands.md#command--10-path](../../../../docs/cli/commands.md#command--10-path)

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

---

### IT-1: Default (cwd) computes correct storage path

- **Given:** Run from `/home/user1/pro/lib/consumer`; `CLAUDE_STORAGE_ROOT` is not relevant (`.path` computes the path, not the root).
- **When:** `clg .path`
- **Then:** Single line: `~/.claude/projects/-home-user1-pro-lib-consumer/` (or the absolute equivalent).; correct path-encoded directory for cwd
- **Exit:** 0
- **Source:** [commands.md](../../../../docs/cli/commands.md)

---

### IT-2: path:: override computes path for given directory

- **Given:** clean environment
- **When:** `clg .path path::/home/user1/pro/lib/consumer`
- **Then:** Single line containing `/.claude/projects/-home-user1-pro-lib-consumer/`.; + path-encoded directory for the given path
- **Exit:** 0
- **Source:** [commands.md](../../../../docs/cli/commands.md)

---

### IT-3: topic:: appended as suffix to encoded path

- **Given:** clean environment
- **When:** `clg .path path::/home/user1/pro/lib/consumer topic::default_topic`
- **Then:** Single line containing `/-home-user1-pro-lib-consumer--default-topic/`.; + topic suffix present in output
- **Exit:** 0
- **Source:** [commands.md](../../../../docs/cli/commands.md)

---

### IT-4: path:: with topic:: combines both

- **Given:** clean environment
- **When:** `clg .path path::~/pro/lib/myapp topic::work`
- **Then:** Single line containing `{encoded_home_pro_lib_myapp}--work/`.; + both base path and topic suffix reflected in output
- **Exit:** 0
- **Source:** [commands.md](../../../../docs/cli/commands.md)

---

### IT-5: Output is a single line ending with /

- **Given:** clean environment
- **When:** `clg .path path::/tmp/test-dir`
- **Then:** Exactly one non-empty line; the path ends with `/`.; Single-line output ending with `/`
- **Exit:** 0
- **Source:** [commands.md](../../../../docs/cli/commands.md)

---

### IT-6: Exits with code 0 for nonexistent path

- **Given:** Use a path that definitely does not exist.
- **When:** `clg .path path::/tmp/nonexistent-project-abc123`
- **Then:** Single line with computed storage path; exit code 0.; + storage path computed regardless of existence
- **Exit:** 0
- **Source:** [commands.md](../../../../docs/cli/commands.md)

---

### IT-7: ~ prefix expanded in path::

- **Given:** clean environment
- **When:** `clg .path path::~/pro/lib/consumer`
- **Then:** Storage path containing the expanded home directory (not literal `~`).; + `~` expanded to absolute home path
- **Exit:** 0
- **Source:** [commands.md](../../../../docs/cli/commands.md)

---

### IT-8: path::. resolves to cwd

- **Given:** Same working directory for both commands.
- **When:** `clg .path path::.`
- **Then:** Same output as `clg .path` from the same directory.; + identical output to bare `.path`
- **Exit:** 0
- **Source:** [commands.md](../../../../docs/cli/commands.md)

---

### IT-9: Empty topic:: rejected

- **Given:** clean environment
- **When:** `clg .path topic::`
- **Then:** Error message about empty topic; exit code 1.; + error message about empty topic
- **Exit:** 1
- **Source:** [commands.md](../../../../docs/cli/commands.md)

---

### IT-10: topic:: with slash rejected

- **Given:** clean environment
- **When:** `clg .path topic::my/topic`
- **Then:** Error message about path separators in topic; exit code 1.; + error message about slash in topic
- **Exit:** 1
- **Source:** [commands.md](../../../../docs/cli/commands.md)
