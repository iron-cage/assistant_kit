# Command :: `.session.dir`

Integration tests for the `.session.dir` command. Tests verify session directory path computation, topic handling, and validation.

**Source:** [commands.md#command--12-sessiondir](../../../../docs/cli/commands.md#command--12-sessiondir)

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

---

### IT-1: path:: with default topic produces {base}/-default_topic

- **Given:** clean environment
- **When:** `clg .session.dir path::/home/user/project`
- **Then:** `/home/user/project/-default_topic`; correct default topic path
- **Exit:** 0
- **Source:** [commands.md](../../../../docs/cli/commands.md)

---

### IT-2: path:: with custom topic produces {base}/-{topic}

- **Given:** clean environment
- **When:** `clg .session.dir path::/home/user/project topic::work`
- **Then:** `/home/user/project/-work`; + `/-work` suffix in output
- **Exit:** 0
- **Source:** [commands.md](../../../../docs/cli/commands.md)

---

### IT-3: path:: required — missing path:: returns error

- **Given:** clean environment
- **When:** `clg .session.dir`
- **Then:** Error on stderr; exit code 1.; + error message about required path
- **Exit:** 1
- **Source:** [commands.md](../../../../docs/cli/commands.md)

---

### IT-4: Output is a single line (absolute path)

- **Given:** clean environment
- **When:** `clg .session.dir path::/home/user/project`
- **Then:** Exactly one line starting with `/`.; single-line absolute path output
- **Exit:** 0
- **Source:** [commands.md](../../../../docs/cli/commands.md)

---

### IT-5: ~ prefix expanded in path::

- **Given:** clean environment
- **When:** `clg .session.dir path::~/pro/lib/myapp`
- **Then:** An absolute path without `~`; ends with `/-default_topic`.; + `~` expanded in output
- **Exit:** 0
- **Source:** [commands.md](../../../../docs/cli/commands.md)

---

### IT-6: path::. resolves to cwd

- **Given:** clean environment
- **When:** `clg .session.dir path::.`
- **Then:** `{cwd}/-default_topic`; + output is absolute cwd-based path
- **Exit:** 0
- **Source:** [commands.md](../../../../docs/cli/commands.md)

---

### IT-7: Empty topic:: rejected

- **Given:** clean environment
- **When:** `clg .session.dir path::/home/user/project topic::`
- **Then:** Error about empty topic; exit code 1.; + error about empty topic
- **Exit:** 1
- **Source:** [commands.md](../../../../docs/cli/commands.md)

---

### IT-8: topic:: with slash rejected

- **Given:** clean environment
- **When:** `clg .session.dir path::/home/user/project topic::sub/dir`
- **Then:** Error about path separators in topic; exit code 1.; + error about slash in topic
- **Exit:** 1
- **Source:** [commands.md](../../../../docs/cli/commands.md)

---

### IT-9: Does not create directory

- **Given:** Use a base path that definitely does not have a `-default_topic` subdirectory.
- **When:** `clg .session.dir path::/tmp/no-such-base-abc topic::default_topic`
- **Then:** Single-line path output; the directory `/tmp/no-such-base-abc/-default_topic` is NOT created.; + no directory created on disk
- **Exit:** 0
- **Source:** [commands.md](../../../../docs/cli/commands.md)

---

### IT-10: Exits 0 even if path does not exist on disk

- **Given:** Use a path that does not exist.
- **When:** `clg .session.dir path::/tmp/nonexistent-xyz-abc`
- **Then:** Single-line path; exit code 0.; — path computation is filesystem-independent
- **Exit:** 0
- **Source:** [commands.md](../../../../docs/cli/commands.md)
