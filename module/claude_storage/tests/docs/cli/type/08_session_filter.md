# Type :: 8. `SessionFilter`

Type constraint tests for `SessionFilter` — case-insensitive session ID substring matcher.

**Source:** [type/08_session_filter.md](../../../../docs/cli/type/08_session_filter.md)

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| TC-1 | Non-empty string accepted | Valid Input |
| TC-2 | Empty string rejected | Invalid Input |
| TC-3 | Substring match is case-insensitive | Matching Semantics |
| TC-4 | Partial match against filename stem | Matching Semantics |

## Test Coverage Summary

- Valid Input: 1 test (TC-1)
- Invalid Input: 1 test (TC-2)
- Matching Semantics: 2 tests (TC-3, TC-4)

**Total:** 4 cases

## Test Cases

---

### TC-1: Non-empty string accepted

- **Given:** Input string `"commit"`
- **When:** `SessionFilter` is parsed
- **Then:** Accepted as `SessionFilter("commit")`

---

### TC-2: Empty string rejected

- **Given:** Input string `""`
- **When:** `SessionFilter` is parsed
- **Then:** Rejected; error message is `session filter must be non-empty`

---

### TC-3: Substring match is case-insensitive

- **Given:** `SessionFilter("COMMIT")` and session filename stem `auto-commit`
- **When:** `matches()` is called
- **Then:** Returns true — uppercase filter matches lowercase session ID

---

### TC-4: Partial match against filename stem

- **Given:** `SessionFilter("commit")` and session filename stem `auto-commit`
- **When:** `matches()` is called
- **Then:** Returns true — partial substring within the stem is sufficient; the `.jsonl` extension is excluded from matching
