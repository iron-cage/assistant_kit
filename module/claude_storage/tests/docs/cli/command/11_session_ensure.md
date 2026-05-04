# Command :: `.session.ensure`

Integration tests for the `.session.ensure` command. Tests verify directory creation, strategy detection, strategy forcing, two-line output format, and validation.

**Source:** [commands.md#command--13-sessionensure](../../../../docs/cli/commands.md#command--13-sessionensure)

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| IT-1 | path:: required — missing returns error | Validation |
| IT-2 | Creates directory when it does not exist | Directory Creation |
| IT-3 | Does not fail if directory already exists | Idempotency |
| IT-4 | Auto-detects resume when history exists | Strategy Detection |
| IT-5 | Auto-detects fresh when no history | Strategy Detection |
| IT-6 | Output line 1 is absolute session dir path | Output Format |
| IT-7 | Output line 2 is strategy (resume or fresh) | Output Format |
| IT-8 | strategy::resume forces resume even when no history | Strategy Forcing |
| IT-9 | strategy::fresh forces fresh even when history exists | Strategy Forcing |
| IT-10 | Default topic is default_topic | Topic Defaults |
| IT-11 | Custom topic produces {base}/-{topic} | Topic Handling |
| IT-12 | Empty topic:: rejected | Validation |
| IT-13 | topic:: with slash rejected | Validation |
| IT-14 | Invalid strategy:: rejected | Validation |
| IT-15 | Exits with code 0 on success | Exit Codes |

## Test Coverage Summary

- Validation: 4 tests (IT-1, IT-12, IT-13, IT-14)
- Directory Creation: 1 test (IT-2)
- Idempotency: 1 test (IT-3)
- Strategy Detection: 2 tests (IT-4, IT-5)
- Output Format: 2 tests (IT-6, IT-7)
- Strategy Forcing: 2 tests (IT-8, IT-9)
- Topic Defaults: 1 test (IT-10)
- Topic Handling: 1 test (IT-11)
- Exit Codes: 1 test (IT-15)

## Test Cases

---

### IT-1: path:: required — missing returns error

- **Given:** clean environment
- **When:** `clg .session.ensure`
- **Then:** Error on stderr about missing path; exit code 1.; error about required path
- **Exit:** 1
- **Source:** [commands.md](../../../../docs/cli/commands.md)

---

### IT-2: Creates directory when it does not exist

- **Given:** Create a TempDir as HOME and as base path. Session dir does not exist yet.
- **When:** `clg .session.ensure path::{base}`
- **Then:** Two lines; the session directory is created.; + directory created
- **Exit:** 0
- **Source:** [commands.md](../../../../docs/cli/commands.md)

---

### IT-3: Does not fail if directory already exists

- **Given:** Pre-create `{base}/-default_topic` directory.
- **When:** `clg .session.ensure path::{base}`
- **Then:** Two lines; exit code 0.; on re-invocation with existing directory
- **Exit:** 0
- **Source:** [commands.md](../../../../docs/cli/commands.md)

---

### IT-4: Auto-detects resume when history exists

- **Given:** Create a TempDir as HOME; create `~/.claude/projects/{encoded_session_dir}/` with a non-empty `.jsonl` file.
- **When:** `clg .session.ensure path::{base} topic::{topic}`
- **Then:** ```
{base}/-{topic}
resume
```; + line 2 is "resume"
- **Exit:** 0
- **Source:** [commands.md](../../../../docs/cli/commands.md)

---

### IT-5: Auto-detects fresh when no history

- **Given:** Create a TempDir as HOME; no matching project in storage.
- **When:** `clg .session.ensure path::{base}`
- **Then:** ```
{base}/-default_topic
fresh
```; + line 2 is "fresh"
- **Exit:** 0
- **Source:** [commands.md](../../../../docs/cli/commands.md)

---

### IT-6: Output line 1 is absolute session dir path

- **Given:** clean environment
- **When:** `clg .session.ensure path::/home/user/project topic::work`
- **Then:** Line 1 is `/home/user/project/-work`.; line 1 is absolute filesystem path to session dir
- **Exit:** 0
- **Source:** [commands.md](../../../../docs/cli/commands.md)

---

### IT-7: Output line 2 is strategy (resume or fresh)

- **Given:** Any valid invocation.
- **When:** `clg .session.ensure path::{base}`
- **Then:** Line 2 is either `resume` or `fresh` (no other values).; line 2 is exactly "resume" or "fresh"
- **Exit:** 0
- **Source:** [commands.md](../../../../docs/cli/commands.md)

---

### IT-8: strategy::resume forces resume even when no history

- **Given:** Create a TempDir as HOME; no matching project in storage (would normally be `fresh`).
- **When:** `clg .session.ensure path::{base} strategy::resume`
- **Then:** ```
{base}/-default_topic
resume
```; + line 2 is "resume" despite no history
- **Exit:** 0
- **Source:** [commands.md](../../../../docs/cli/commands.md)

---

### IT-9: strategy::fresh forces fresh even when history exists

- **Given:** Create a TempDir as HOME; create storage history for the session directory.
- **When:** `clg .session.ensure path::{base} topic::{topic} strategy::fresh`
- **Then:** ```
{base}/-{topic}
fresh
```; + line 2 is "fresh" despite existing history
- **Exit:** 0
- **Source:** [commands.md](../../../../docs/cli/commands.md)

---

### IT-10: Default topic is default_topic

- **Given:** clean environment
- **When:** `clg .session.ensure path::/home/user/project`
- **Then:** Line 1 ends with `/-default_topic`.; line 1 uses `default_topic` as default topic suffix
- **Exit:** 0
- **Source:** [commands.md](../../../../docs/cli/commands.md)

---

### IT-11: Custom topic produces {base}/-{topic}

- **Given:** clean environment
- **When:** `clg .session.ensure path::/home/user/project topic::work`
- **Then:** Line 1 is `/home/user/project/-work`.; line 1 reflects custom topic
- **Exit:** 0
- **Source:** [commands.md](../../../../docs/cli/commands.md)

---

### IT-12: Empty topic:: rejected

- **Given:** clean environment
- **When:** `clg .session.ensure path::/home/user/project topic::`
- **Then:** Error about empty topic; exit code 1.; + error about empty topic
- **Exit:** 1
- **Source:** [commands.md](../../../../docs/cli/commands.md)

---

### IT-13: topic:: with slash rejected

- **Given:** clean environment
- **When:** `clg .session.ensure path::/home/user/project topic::a/b`
- **Then:** Error about path separators; exit code 1.; + error about slash in topic
- **Exit:** 1
- **Source:** [commands.md](../../../../docs/cli/commands.md)

---

### IT-14: Invalid strategy:: rejected

- **Given:** clean environment
- **When:** `clg .session.ensure path::/home/user/project strategy::auto`
- **Then:** Error about invalid strategy; exit code 1.; + error about invalid strategy value
- **Exit:** 1
- **Source:** [commands.md](../../../../docs/cli/commands.md)

---

### IT-15: Exits with code 0 on success

- **Given:** Valid base directory; any topic.
- **When:** `clg .session.ensure path::{valid_base}`
- **Then:** Two lines; exit code 0.; on success
- **Exit:** 0
- **Source:** [commands.md](../../../../docs/cli/commands.md)
