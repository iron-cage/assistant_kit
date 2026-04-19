# Command :: `.session.ensure`

Integration tests for the `.session.ensure` command. Tests verify directory creation, strategy detection, strategy forcing, two-line output format, and validation.

**Source:** [commands.md#command--13-sessionensure](../../../../../docs/cli/commands.md#command--13-sessionensure)

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

### IT-1: path:: required — missing returns error

**Goal:** Verify that omitting `path::` produces an error (exit 1).
**Setup:** None specific.
**Command:** `clg .session.ensure`
**Expected Output:** Error on stderr about missing path; exit code 1.
**Verification:**
- `$?` is `1`
- stderr contains message about required `path::` parameter
- stdout is empty
**Pass Criteria:** exit 1 + error about required path

**Source:** [commands.md](../../../../../docs/cli/commands.md)

---

### IT-2: Creates directory when it does not exist

**Goal:** Verify `.session.ensure` creates the session directory when it does not exist.
**Setup:** Create a TempDir as HOME and as base path. Session dir does not exist yet.
**Command:** `clg .session.ensure path::{base}`
**Expected Output:** Two lines; the session directory is created.
**Verification:**
- Exit code is `0`
- `{base}/-default_topic` directory EXISTS after the command
**Pass Criteria:** exit 0 + directory created

**Source:** [commands.md](../../../../../docs/cli/commands.md)

---

### IT-3: Does not fail if directory already exists

**Goal:** Verify `.session.ensure` is idempotent — calling it when the directory already exists does not produce an error.
**Setup:** Pre-create `{base}/-default_topic` directory.
**Command:** `clg .session.ensure path::{base}`
**Expected Output:** Two lines; exit code 0.
**Verification:**
- `$?` is `0`
- No error on stderr
- Directory still exists
**Pass Criteria:** exit 0 on re-invocation with existing directory

**Source:** [commands.md](../../../../../docs/cli/commands.md)

---

### IT-4: Auto-detects resume when history exists

**Goal:** Verify strategy is `resume` when the session directory has conversation history.
**Setup:** Create a TempDir as HOME; create `~/.claude/projects/{encoded_session_dir}/` with a non-empty `.jsonl` file.
**Command:** `clg .session.ensure path::{base} topic::{topic}`
**Expected Output:**
```
{base}/-{topic}
resume
```
**Verification:**
- Exit code is `0`
- Line 2 is `resume`
**Pass Criteria:** exit 0 + line 2 is "resume"

**Source:** [commands.md](../../../../../docs/cli/commands.md)

---

### IT-5: Auto-detects fresh when no history

**Goal:** Verify strategy is `fresh` when no conversation history exists for the session directory.
**Setup:** Create a TempDir as HOME; no matching project in storage.
**Command:** `clg .session.ensure path::{base}`
**Expected Output:**
```
{base}/-default_topic
fresh
```
**Verification:**
- Exit code is `0`
- Line 2 is `fresh`
**Pass Criteria:** exit 0 + line 2 is "fresh"

**Source:** [commands.md](../../../../../docs/cli/commands.md)

---

### IT-6: Output line 1 is absolute session dir path

**Goal:** Verify that the first output line is the absolute path to the session directory.
**Setup:** None specific.
**Command:** `clg .session.ensure path::/home/user/project topic::work`
**Expected Output:** Line 1 is `/home/user/project/-work`.
**Verification:**
- First line of stdout is the absolute session directory path
- Starts with `/`
- Does not include storage root or encoded path
**Pass Criteria:** line 1 is absolute filesystem path to session dir

**Source:** [commands.md](../../../../../docs/cli/commands.md)

---

### IT-7: Output line 2 is strategy (resume or fresh)

**Goal:** Verify that the second output line is exactly `resume` or `fresh`.
**Setup:** Any valid invocation.
**Command:** `clg .session.ensure path::{base}`
**Expected Output:** Line 2 is either `resume` or `fresh` (no other values).
**Verification:**
- Second line of stdout is `"resume"` or `"fresh"` (lowercase, no extra whitespace)
**Pass Criteria:** line 2 is exactly "resume" or "fresh"

**Source:** [commands.md](../../../../../docs/cli/commands.md)

---

### IT-8: strategy::resume forces resume even when no history

**Goal:** Verify that `strategy::resume` forces the strategy output to `resume` even when no conversation history exists.
**Setup:** Create a TempDir as HOME; no matching project in storage (would normally be `fresh`).
**Command:** `clg .session.ensure path::{base} strategy::resume`
**Expected Output:**
```
{base}/-default_topic
resume
```
**Verification:**
- `$?` is `0`
- Line 2 is `resume` (not `fresh`)
**Pass Criteria:** exit 0 + line 2 is "resume" despite no history

**Source:** [commands.md](../../../../../docs/cli/commands.md)

---

### IT-9: strategy::fresh forces fresh even when history exists

**Goal:** Verify that `strategy::fresh` forces the strategy output to `fresh` even when conversation history exists.
**Setup:** Create a TempDir as HOME; create storage history for the session directory.
**Command:** `clg .session.ensure path::{base} topic::{topic} strategy::fresh`
**Expected Output:**
```
{base}/-{topic}
fresh
```
**Verification:**
- `$?` is `0`
- Line 2 is `fresh` (not `resume`)
**Pass Criteria:** exit 0 + line 2 is "fresh" despite existing history

**Source:** [commands.md](../../../../../docs/cli/commands.md)

---

### IT-10: Default topic is default_topic

**Goal:** Verify that omitting `topic::` uses `default_topic` as the default.
**Setup:** None specific.
**Command:** `clg .session.ensure path::/home/user/project`
**Expected Output:** Line 1 ends with `/-default_topic`.
**Verification:**
- Line 1 is `/home/user/project/-default_topic`
**Pass Criteria:** line 1 uses `default_topic` as default topic suffix

**Source:** [commands.md](../../../../../docs/cli/commands.md)

---

### IT-11: Custom topic produces {base}/-{topic}

**Goal:** Verify that a custom `topic::` produces the correct directory path.
**Setup:** None specific.
**Command:** `clg .session.ensure path::/home/user/project topic::work`
**Expected Output:** Line 1 is `/home/user/project/-work`.
**Verification:**
- Line 1 is `/home/user/project/-work`
**Pass Criteria:** line 1 reflects custom topic

**Source:** [commands.md](../../../../../docs/cli/commands.md)

---

### IT-12: Empty topic:: rejected

**Goal:** Verify that `topic::` with empty value returns an error.
**Setup:** None specific.
**Command:** `clg .session.ensure path::/home/user/project topic::`
**Expected Output:** Error about empty topic; exit code 1.
**Verification:**
- `$?` is `1`
- stderr contains error about empty topic
**Pass Criteria:** exit 1 + error about empty topic

**Source:** [commands.md](../../../../../docs/cli/commands.md)

---

### IT-13: topic:: with slash rejected

**Goal:** Verify that a topic containing `/` is rejected.
**Setup:** None specific.
**Command:** `clg .session.ensure path::/home/user/project topic::a/b`
**Expected Output:** Error about path separators; exit code 1.
**Verification:**
- `$?` is `1`
- stderr contains error about path separators
**Pass Criteria:** exit 1 + error about slash in topic

**Source:** [commands.md](../../../../../docs/cli/commands.md)

---

### IT-14: Invalid strategy:: rejected

**Goal:** Verify that an invalid `strategy::` value returns an error.
**Setup:** None specific.
**Command:** `clg .session.ensure path::/home/user/project strategy::auto`
**Expected Output:** Error about invalid strategy; exit code 1.
**Verification:**
- `$?` is `1`
- stderr contains `"strategy must be resume|fresh"`
**Pass Criteria:** exit 1 + error about invalid strategy value

**Source:** [commands.md](../../../../../docs/cli/commands.md)

---

### IT-15: Exits with code 0 on success

**Goal:** Verify exit code is 0 for all normal successful invocations.
**Setup:** Valid base directory; any topic.
**Command:** `clg .session.ensure path::{valid_base}`
**Expected Output:** Two lines; exit code 0.
**Verification:**
- `$?` is `0`
- Two lines of output
- No error on stderr
**Pass Criteria:** exit 0 on success

**Source:** [commands.md](../../../../../docs/cli/commands.md)
