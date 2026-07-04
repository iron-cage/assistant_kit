# User Story :: 6. Quick Context Refresh

Acceptance criteria tests for the developer persona resuming work in a known directory without running a lookup command.
Source: [006_quick_context_refresh.md](../../../../docs/cli/user_story/006_quick_context_refresh.md)

## Test Case Index

| ID | Test Name | Criteria |
|----|-----------|---------|
| RWS-1 | Zero-parameter tail shows recent entries in current directory | AC: view last few entries with zero parameters |
| RWS-2 | tail:: controls how many entries are shown | AC: control how many recent entries are shown |
| RWS-3 | topic:: views a non-default session | AC: view recent entries for a non-default session topic |
| RWS-4 | No history reports a clear error | AC: clear error when directory has no history |

---

### RWS-1: Zero-parameter tail shows recent entries in current directory

**Scenario:** Developer returns to a project directory and wants an immediate content refresher.

**Fixture:** Project matching cwd with a `-default_topic` session of 6 known entries.

**Command:**
```bash
clg .tail
```

**Expected:**
- Stdout shows the last 4 entries, oldest-first, without any parameters supplied

**Exit:** `0`

---

### RWS-2: tail:: controls how many entries are shown

**Scenario:** Developer wants more context than the default 4 entries.

**Fixture:** Same project, `-default_topic` session with 6 known entries.

**Command:**
```bash
clg .tail tail::6
```

**Expected:**
- Stdout shows all 6 entries, oldest-first

**Exit:** `0`

---

### RWS-3: topic:: views a non-default session

**Scenario:** Developer works across multiple topics in the same directory and wants to check a specific one.

**Fixture:** Project with `-default_topic` and `-work` sessions, each with distinct content.

**Command:**
```bash
clg .tail topic::work
```

**Expected:**
- Stdout shows only `-work` session content, not `-default_topic`

**Exit:** `0`

---

### RWS-4: No history reports a clear error

**Scenario:** Developer runs `.tail` in a directory with no recorded conversation history.

**Fixture:** cwd with no corresponding project in storage.

**Command:**
```bash
clg .tail
```

**Expected:**
- Stderr reports that no history was found for the current directory

**Exit:** `2`
