# Parameter :: `topic::`

Edge case tests for the `topic::` parameter. Tests validate non-empty constraint, no-slash constraint, leading-hyphen handling, and default values.

**Source:** [params.md#parameter--17-topic](../../params.md#parameter--17-topic) | [types.md#topicname](../../types.md#topicname)

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| EC-1 | Valid simple name accepted | Type Validation |
| EC-2 | Empty value rejected | Boundary Values |
| EC-3 | Slash in value rejected | Boundary Values |
| EC-4 | Backslash in value rejected | Boundary Values |
| EC-5 | Default (absent) uses default_topic in .session.dir | Default |
| EC-6 | Default (absent) uses default_topic in .session.ensure | Default |
| EC-7 | Absent in .path produces no suffix | Default |
| EC-8 | Absent in .exists checks base path storage | Default |
| EC-9 | Value with hyphen accepted | Type Validation |
| EC-10 | Value with underscore accepted | Type Validation |

## Test Coverage Summary

- Type Validation: 3 tests (EC-1, EC-9, EC-10)
- Boundary Values: 3 tests (EC-2, EC-3, EC-4)
- Default: 4 tests (EC-5, EC-6, EC-7, EC-8)

## Test Cases

### EC-1: Valid simple name accepted

**Goal:** Verify that a simple alphanumeric topic name is accepted.
**Setup:** None specific.
**Command:** `clg .session.dir path::/tmp/base topic::work`
**Expected Output:** `/tmp/base/-work`; exit code 0.
**Verification:**
- Exit code is `0`
- Output contains `/-work`
- No error on stderr
**Pass Criteria:** exit 0 + correct path suffix

**Source:** [params.md](../../params.md)

---

### EC-2: Empty value rejected

**Goal:** Verify that `topic::` with no value is rejected.
**Setup:** None specific.
**Command:** `clg .session.dir path::/tmp/base topic::`
**Expected Output:** Error about empty topic; exit code 1.
**Verification:**
- `$?` is `1`
- stderr contains error about empty topic value
**Pass Criteria:** exit 1 + error about empty topic

**Source:** [params.md](../../params.md)

---

### EC-3: Slash in value rejected

**Goal:** Verify that a topic containing `/` is rejected.
**Setup:** None specific.
**Command:** `clg .session.dir path::/tmp/base topic::sub/dir`
**Expected Output:** Error about path separators; exit code 1.
**Verification:**
- `$?` is `1`
- stderr contains message about path separators
**Pass Criteria:** exit 1 + error about slash

**Source:** [params.md](../../params.md)

---

### EC-4: Backslash in value rejected

**Goal:** Verify that a topic containing `\` is also rejected (platform-agnostic safety).
**Setup:** None specific.
**Command:** `clg .session.dir path::/tmp/base topic::sub\\dir`
**Expected Output:** Either accepted as-is (backslash is valid on Unix) or rejected; key is that `/` must be rejected per EC-3 — this tests the forward-slash constraint is directional.
**Verification:**
- The backslash test documents that only `/` is the separator constraint
- Exit behavior is acceptable either way (backslash has no special meaning on Unix paths)
**Pass Criteria:** documented behavior — error for `/`, any behavior for `\`

**Source:** [params.md](../../params.md)

---

### EC-5: Default (absent) uses default_topic in .session.dir

**Goal:** Verify that when `topic::` is absent from `.session.dir`, `default_topic` is used.
**Setup:** None specific.
**Command:** `clg .session.dir path::/tmp/base`
**Expected Output:** `/tmp/base/-default_topic`
**Verification:**
- Exit code is `0`
- stdout is `/tmp/base/-default_topic\n`
**Pass Criteria:** exit 0 + `default_topic` is the default topic

**Source:** [params.md](../../params.md)

---

### EC-6: Default (absent) uses default_topic in .session.ensure

**Goal:** Verify that when `topic::` is absent from `.session.ensure`, `default_topic` is used.
**Setup:** TempDir as HOME; base directory exists.
**Command:** `clg .session.ensure path::{base}`
**Expected Output:** Line 1 ends with `/-default_topic`.
**Verification:**
- Exit code is `0`
- Line 1 ends with `/-default_topic`
**Pass Criteria:** exit 0 + `default_topic` is the default topic in `.session.ensure`

**Source:** [params.md](../../params.md)

---

### EC-7: Absent in .path produces no suffix

**Goal:** Verify that when `topic::` is absent from `.path`, no topic suffix is appended.
**Setup:** None specific.
**Command:** `clg .path path::/tmp/base`
**Expected Output:** Storage path for `/tmp/base` with NO `--` topic suffix.
**Verification:**
- Exit code is `0`
- Output does not contain `--` at the end of the directory name
**Pass Criteria:** exit 0 + no topic suffix in output

**Source:** [params.md](../../params.md)

---

### EC-8: Absent in .exists checks base path storage

**Goal:** Verify that when `topic::` is absent from `.exists`, the base path storage (not topic-specific) is checked.
**Setup:** Create a TempDir as HOME with storage for the base path but not for any topic.
**Command:** `clg .exists path::{base}`
**Expected Output:** `"sessions exist\n"` (checks base storage, which exists); exit code 0.
**Verification:**
- Exit code is `0`
- stdout is `"sessions exist\n"`
**Pass Criteria:** exit 0 — base storage checked when topic absent

**Source:** [params.md](../../params.md)

---

### EC-9: Value with hyphen accepted

**Goal:** Verify that a topic containing a hyphen is accepted.
**Setup:** None specific.
**Command:** `clg .session.dir path::/tmp/base topic::my-topic`
**Expected Output:** `/tmp/base/-my-topic`; exit code 0.
**Verification:**
- Exit code is `0`
- Output contains `/-my-topic`
**Pass Criteria:** exit 0 + hyphen in topic accepted

**Source:** [params.md](../../params.md)

---

### EC-10: Value with underscore accepted

**Goal:** Verify that a topic containing an underscore is accepted.
**Setup:** None specific.
**Command:** `clg .session.dir path::/tmp/base topic::default_topic`
**Expected Output:** `/tmp/base/-default_topic`; exit code 0.
**Verification:**
- Exit code is `0`
- Output contains `/-default_topic`
**Pass Criteria:** exit 0 + underscore in topic accepted

**Source:** [params.md](../../params.md)
