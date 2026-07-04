# User Story :: 5. Resume Claude Session

Acceptance criteria tests for the developer persona setting up a Claude Code session.
Source: [005_resume_claude_session.md](../../../../docs/cli/user_story/005_resume_claude_session.md)

## Test Case Index

| ID | Test Name | Criteria |
|----|-----------|---------|
| RWS-1 | project.exists exits 0 when project has history | AC: verify project has existing conversation history |
| RWS-2 | project.exists exits 1 when project has no history | AC: verify project has existing conversation history |
| RWS-3 | project.path outputs encoded storage path | AC: compute Claude storage path for any project directory |
| RWS-4 | session.dir outputs session working directory path | AC: compute session working directory for a given topic |
| RWS-5 | session.ensure creates directory and reports strategy | AC: create session directory; report resume vs fresh |

---

### RWS-1: project.exists exits 0 when project has history

**Scenario:** Script checks whether a project has conversation history before resuming.

**Fixture:** One project with 1 session file in `CLAUDE_STORAGE_ROOT/projects/`.

**Command:**
```bash
clg .project.exists path::{project-dir}
```

**Expected:**
- Stdout: `sessions exist`
- Shell conditional `if clg .project.exists path::...` evaluates to true

**Exit:** `0`

---

### RWS-2: project.exists exits 1 when project has no history

**Scenario:** Script detects a fresh project with no previous Claude sessions.

**Fixture:** No session files for the target project directory.

**Command:**
```bash
clg .project.exists path::/tmp/nonexistent-project-xyz
```

**Expected:**
- Stderr: `no sessions`
- Shell conditional `if clg .project.exists path::...` evaluates to false

**Exit:** `1`

---

### RWS-3: project.path outputs encoded storage path

**Scenario:** Script needs the encoded storage path to locate project data.

**Fixture:** Any project directory path (does not need to exist in storage).

**Command:**
```bash
clg .project.path path::/home/user/myproject
```

**Expected:**
- Stdout is a single line: the absolute path to `~/.claude/projects/{encoded}/`
- Path ends with the encoded form of `/home/user/myproject`
- No trailing newline issues

**Exit:** `0`

---

### RWS-4: session.dir outputs session working directory path

**Scenario:** Script computes the session directory path without creating it.

**Fixture:** None required — `.session.dir` is read-only.

**Command:**
```bash
clg .session.dir path::/home/user/myproject topic::work
```

**Expected:**
- Stdout is a single line: `/home/user/myproject/-work`
- Directory is NOT created on disk

**Exit:** `0`

---

### RWS-5: session.ensure creates directory and reports strategy

**Scenario:** Script ensures the session directory exists and learns whether to resume or start fresh.

**Fixture:** Project directory exists but session directory `-work` does not.

**Command:**
```bash
clg .session.ensure path::/home/user/myproject topic::work
```

**Expected:**
- Stdout line 1: absolute path to the session directory (e.g., `/home/user/myproject/-work`)
- Stdout line 2: `fresh` (no prior conversation history)
- Session directory `/home/user/myproject/-work` is created on disk

**Exit:** `0`
