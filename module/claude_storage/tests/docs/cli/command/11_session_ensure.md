# Command :: `.session.ensure`

Integration tests for the `.session.ensure` command. Tests verify directory creation, strategy detection, strategy forcing, two-line output format, and validation.

**Source:** [command/11_session_ensure.md](../../../../docs/cli/command/11_session_ensure.md)

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| INT-1 | path:: required — missing returns error | Validation |
| INT-2 | Creates directory when it does not exist | Directory Creation |
| INT-3 | Does not fail if directory already exists | Idempotency |
| INT-4 | Auto-detects resume when history exists | Strategy Detection |
| INT-5 | Auto-detects fresh when no history | Strategy Detection |
| INT-6 | Output line 1 is absolute session dir path | Output Format |
| INT-7 | Output line 2 is strategy (resume or fresh) | Output Format |
| INT-8 | strategy::resume forces resume even when no history | Strategy Forcing |
| INT-9 | strategy::fresh forces fresh even when history exists | Strategy Forcing |
| INT-10 | Default topic is default_topic | Topic Defaults |
| INT-11 | Custom topic produces {base}/-{topic} | Topic Handling |
| INT-12 | Empty topic:: rejected | Validation |
| INT-13 | topic:: with slash rejected | Validation |
| INT-14 | Invalid strategy:: rejected | Validation |
| INT-15 | Exits with code 0 on success | Exit Codes |

## Test Coverage Summary

- Validation: 4 tests (INT-1, INT-12, INT-13, INT-14)
- Directory Creation: 1 test (INT-2)
- Idempotency: 1 test (INT-3)
- Strategy Detection: 2 tests (INT-4, INT-5)
- Output Format: 2 tests (INT-6, INT-7)
- Strategy Forcing: 2 tests (INT-8, INT-9)
- Topic Defaults: 1 test (INT-10)
- Topic Handling: 1 test (INT-11)
- Exit Codes: 1 test (INT-15)

## Test Cases

---

### INT-1: path:: required — missing returns error

**Command:**
```
clg .session.ensure
```

**Expected behavior:**
- Error message on stderr about missing path
- Exit code: 1
- **Source:** [command/11_session_ensure.md](../../../../docs/cli/command/11_session_ensure.md)

---

### INT-2: Creates directory when it does not exist

**Command:**
```
clg .session.ensure path::{base}
```

**Expected behavior:**
- Fixture: create a TempDir as HOME and as base path; session dir does not exist yet
- Two lines on stdout; the session directory is created on disk
- Exit code: 0
- **Source:** [command/11_session_ensure.md](../../../../docs/cli/command/11_session_ensure.md)

---

### INT-3: Does not fail if directory already exists

**Command:**
```
clg .session.ensure path::{base}
```

**Expected behavior:**
- Fixture: pre-create `{base}/-default_topic` directory
- Two lines on stdout; no failure on re-invocation with existing directory
- Exit code: 0
- **Source:** [command/11_session_ensure.md](../../../../docs/cli/command/11_session_ensure.md)

---

### INT-4: Auto-detects resume when history exists

**Command:**
```
clg .session.ensure path::{base} topic::{topic}
```

**Expected behavior:**
- Fixture: create a TempDir as HOME; create `~/.claude/projects/{encoded_session_dir}/` with a non-empty `.jsonl` file
- Output:
  ```
  {base}/-{topic}
  resume
  ```
- Exit code: 0
- **Source:** [command/11_session_ensure.md](../../../../docs/cli/command/11_session_ensure.md)

---

### INT-5: Auto-detects fresh when no history

**Command:**
```
clg .session.ensure path::{base}
```

**Expected behavior:**
- Fixture: create a TempDir as HOME; no matching project in storage
- Output:
  ```
  {base}/-default_topic
  fresh
  ```
- Exit code: 0
- **Source:** [command/11_session_ensure.md](../../../../docs/cli/command/11_session_ensure.md)

---

### INT-6: Output line 1 is absolute session dir path

**Command:**
```
clg .session.ensure path::/home/user/project topic::work
```

**Expected behavior:**
- Line 1 is `/home/user/project/-work`
- Exit code: 0
- **Source:** [command/11_session_ensure.md](../../../../docs/cli/command/11_session_ensure.md)

---

### INT-7: Output line 2 is strategy (resume or fresh)

**Command:**
```
clg .session.ensure path::{base}
```

**Expected behavior:**
- Fixture: any valid invocation
- Line 2 is either `resume` or `fresh` (no other values)
- Exit code: 0
- **Source:** [command/11_session_ensure.md](../../../../docs/cli/command/11_session_ensure.md)

---

### INT-8: strategy::resume forces resume even when no history

**Command:**
```
clg .session.ensure path::{base} strategy::resume
```

**Expected behavior:**
- Fixture: create a TempDir as HOME; no matching project in storage (would normally be `fresh`)
- Output:
  ```
  {base}/-default_topic
  resume
  ```
- Exit code: 0
- **Source:** [command/11_session_ensure.md](../../../../docs/cli/command/11_session_ensure.md)

---

### INT-9: strategy::fresh forces fresh even when history exists

**Command:**
```
clg .session.ensure path::{base} topic::{topic} strategy::fresh
```

**Expected behavior:**
- Fixture: create a TempDir as HOME; create storage history for the session directory
- Output:
  ```
  {base}/-{topic}
  fresh
  ```
- Exit code: 0
- **Source:** [command/11_session_ensure.md](../../../../docs/cli/command/11_session_ensure.md)

---

### INT-10: Default topic is default_topic

**Command:**
```
clg .session.ensure path::/home/user/project
```

**Expected behavior:**
- Line 1 ends with `/-default_topic`
- Exit code: 0
- **Source:** [command/11_session_ensure.md](../../../../docs/cli/command/11_session_ensure.md)

---

### INT-11: Custom topic produces {base}/-{topic}

**Command:**
```
clg .session.ensure path::/home/user/project topic::work
```

**Expected behavior:**
- Line 1 is `/home/user/project/-work`
- Exit code: 0
- **Source:** [command/11_session_ensure.md](../../../../docs/cli/command/11_session_ensure.md)

---

### INT-12: Empty topic:: rejected

**Command:**
```
clg .session.ensure path::/home/user/project topic::
```

**Expected behavior:**
- Error message on stderr about empty topic
- Exit code: 1
- **Source:** [command/11_session_ensure.md](../../../../docs/cli/command/11_session_ensure.md)

---

### INT-13: topic:: with slash rejected

**Command:**
```
clg .session.ensure path::/home/user/project topic::a/b
```

**Expected behavior:**
- Error message on stderr about path separators in topic
- Exit code: 1
- **Source:** [command/11_session_ensure.md](../../../../docs/cli/command/11_session_ensure.md)

---

### INT-14: Invalid strategy:: rejected

**Command:**
```
clg .session.ensure path::/home/user/project strategy::auto
```

**Expected behavior:**
- Error message on stderr about invalid strategy value
- Exit code: 1
- **Source:** [command/11_session_ensure.md](../../../../docs/cli/command/11_session_ensure.md)

---

### INT-15: Exits with code 0 on success

**Command:**
```
clg .session.ensure path::{valid_base}
```

**Expected behavior:**
- Fixture: valid base directory; any topic
- Two lines on stdout
- Exit code: 0
- **Source:** [command/11_session_ensure.md](../../../../docs/cli/command/11_session_ensure.md)
