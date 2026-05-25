# Parameter :: `topic::`

Edge case tests for the `topic::` parameter. Tests validate non-empty constraint, no-slash constraint, leading-hyphen handling, and default values.

**Source:** [param/17_topic.md](../../../../docs/cli/param/17_topic.md) | [type/13_topic_name.md](../../../../docs/cli/type/13_topic_name.md)

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| EC-1 | Valid simple name accepted | Behavioral Divergence |
| EC-2 | Empty value rejected | Boundary Values |
| EC-3 | Slash in value rejected | Boundary Values |
| EC-4 | Backslash in value rejected | Boundary Values |
| EC-5 | Default (absent) uses default_topic in .session.dir | Default |
| EC-6 | Default (absent) uses default_topic in .session.ensure | Default |
| EC-7 | Absent in .path produces no suffix | Default |
| EC-8 | Absent in .exists checks base path storage | Default |
| EC-9 | Value with hyphen accepted | Behavioral Divergence |
| EC-10 | Value with underscore accepted | Type Validation |

## Test Coverage Summary

- Behavioral Divergence: 2 tests (EC-1, EC-9)
- Type Validation: 1 test (EC-10)
- Boundary Values: 3 tests (EC-2, EC-3, EC-4)
- Default: 4 tests (EC-5, EC-6, EC-7, EC-8)

**Total:** 10 edge cases

**Behavioral Divergence Pair:** EC-1 (topic::work → /-work path suffix) ↔ EC-9 (topic::my-topic → /-my-topic path suffix)

## Test Cases

---

### EC-1: Valid simple name accepted

- **Commands:** `.session.dir`
- **Given:** clean environment
- **When:** `clg .session.dir path::/tmp/base topic::work`
- **Then:** `/tmp/base/-work`; exit code 0.; correct path suffix
- **Exit:** 0
- **Source:** [param/17_topic.md](../../../../docs/cli/param/17_topic.md)

---

### EC-2: Empty value rejected

- **Commands:** `.session.dir`
- **Given:** clean environment
- **When:** `clg .session.dir path::/tmp/base topic::`
- **Then:** Error about empty topic; exit code 1.; + error about empty topic
- **Exit:** 1
- **Source:** [param/17_topic.md](../../../../docs/cli/param/17_topic.md)

---

### EC-3: Slash in value rejected

- **Commands:** `.session.dir`
- **Given:** clean environment
- **When:** `clg .session.dir path::/tmp/base topic::sub/dir`
- **Then:** Error about path separators; exit code 1.; + error about slash
- **Exit:** 1
- **Source:** [param/17_topic.md](../../../../docs/cli/param/17_topic.md)

---

### EC-4: Backslash in value rejected

- **Commands:** `.session.dir`
- **Given:** clean environment
- **When:** `clg .session.dir path::/tmp/base topic::sub\\dir`
- **Then:** Either accepted as-is (backslash is valid on Unix) or rejected; key is that `/` must be rejected per EC-3 — this tests the forward-slash constraint is directional.; documented behavior — error for `/`, any behavior for `\`
- **Exit:** 0
- **Source:** [param/17_topic.md](../../../../docs/cli/param/17_topic.md)

---

### EC-5: Default (absent) uses default_topic in .session.dir

- **Commands:** `.session.dir`
- **Given:** clean environment
- **When:** `clg .session.dir path::/tmp/base`
- **Then:** `/tmp/base/-default_topic`; + `default_topic` is the default topic
- **Exit:** 0
- **Source:** [param/17_topic.md](../../../../docs/cli/param/17_topic.md)

---

### EC-6: Default (absent) uses default_topic in .session.ensure

- **Commands:** `.session.ensure`
- **Given:** TempDir as HOME; base directory exists.
- **When:** `clg .session.ensure path::{base}`
- **Then:** Line 1 ends with `/-default_topic`.; + `default_topic` is the default topic in `.session.ensure`
- **Exit:** 0
- **Source:** [param/17_topic.md](../../../../docs/cli/param/17_topic.md)

---

### EC-7: Absent in .path produces no suffix

- **Commands:** `.project.path`
- **Given:** clean environment
- **When:** `clg .project.path path::/tmp/base`
- **Then:** Storage path for `/tmp/base` with NO `--` topic suffix.; + no topic suffix in output
- **Exit:** 0
- **Source:** [param/17_topic.md](../../../../docs/cli/param/17_topic.md)

---

### EC-8: Absent in .exists checks base path storage

- **Commands:** `.project.exists`
- **Given:** Create a TempDir as HOME with storage for the base path but not for any topic.
- **When:** `clg .project.exists path::{base}`
- **Then:** `"sessions exist\n"` (checks base storage, which exists); exit code 0.; — base storage checked when topic absent
- **Exit:** 0
- **Source:** [param/17_topic.md](../../../../docs/cli/param/17_topic.md)

---

### EC-9: Value with hyphen accepted

- **Commands:** `.session.dir`
- **Given:** clean environment
- **When:** `clg .session.dir path::/tmp/base topic::my-topic`
- **Then:** `/tmp/base/-my-topic`; exit code 0.; + hyphen in topic accepted
- **Exit:** 0
- **Source:** [param/17_topic.md](../../../../docs/cli/param/17_topic.md)

---

### EC-10: Value with underscore accepted

- **Commands:** `.session.dir`
- **Given:** clean environment
- **When:** `clg .session.dir path::/tmp/base topic::default_topic`
- **Then:** `/tmp/base/-default_topic`; exit code 0.; + underscore in topic accepted
- **Exit:** 0
- **Source:** [param/17_topic.md](../../../../docs/cli/param/17_topic.md)
