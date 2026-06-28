# Type :: 12. `TopicName`

Type constraint tests for `TopicName` — session topic identifier string.

**Source:** [type/12_topic_name.md](../../../../docs/cli/type/12_topic_name.md)

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| TC-1 | Simple name accepted | Valid Input |
| TC-2 | "default_topic" accepted | Valid Input |
| TC-3 | Empty string rejected | Invalid Input |
| TC-4 | Path separator rejected | Invalid Input |
| TC-5 | Dir suffix construction | Method Output |

## Test Coverage Summary

- Valid Input: 2 tests (TC-1, TC-2)
- Invalid Input: 2 tests (TC-3, TC-4)
- Method Output: 1 test (TC-5)

**Total:** 5 cases

## Test Cases

---

### TC-1: Simple name accepted

- **Given:** Input string `"work"`
- **When:** `TopicName` is parsed
- **Then:** Accepted as `TopicName("work")`; `get()` returns `"work"`

---

### TC-2: "default_topic" accepted

- **Given:** Input string `"default_topic"`
- **When:** `TopicName` is parsed
- **Then:** Accepted as `TopicName("default_topic")` — matches the system default topic name

---

### TC-3: Empty string rejected

- **Given:** Input string `""`
- **When:** `TopicName` is parsed
- **Then:** Rejected; error message is `topic must be non-empty`

---

### TC-4: Path separator rejected

- **Given:** Input string `"my/topic"`
- **When:** `TopicName` is parsed
- **Then:** Rejected; error message is `topic must not contain path separators`

---

### TC-5: Dir suffix construction

- **Given:** `TopicName("work")` already parsed
- **When:** `as_dir_suffix()` is called
- **Then:** Returns `"-work"` — leading dash prepended; used as directory path suffix
