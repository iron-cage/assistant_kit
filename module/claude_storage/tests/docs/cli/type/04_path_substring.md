# Type :: 4. `PathSubstring`

Type constraint tests for `PathSubstring` — case-insensitive path filter string.

**Source:** [type/04_path_substring.md](../../../../docs/cli/type/04_path_substring.md)

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| TC-1 | Non-empty string accepted | Valid Input |
| TC-2 | Empty string rejected | Invalid Input |
| TC-3 | Substring match is case-insensitive | Matching Semantics |
| TC-4 | Partial substring match is sufficient | Matching Semantics |

## Test Coverage Summary

- Valid Input: 1 test (TC-1)
- Invalid Input: 1 test (TC-2)
- Matching Semantics: 2 tests (TC-3, TC-4)

**Total:** 4 cases

## Test Cases

---

### TC-1: Non-empty string accepted

- **Given:** Input string `"myproject"`
- **When:** `PathSubstring` is parsed
- **Then:** Accepted as `PathSubstring("myproject")`

---

### TC-2: Empty string rejected

- **Given:** Input string `""`
- **When:** `PathSubstring` is parsed
- **Then:** Rejected; error message is `path filter must be non-empty`

---

### TC-3: Substring match is case-insensitive

- **Given:** `PathSubstring("ASSISTANT")` and path `/home/alice/projects/assistant`
- **When:** `matches()` is called
- **Then:** Returns true — uppercase filter matches lowercase path segment

---

### TC-4: Partial substring match is sufficient

- **Given:** `PathSubstring("proj")` and path `/home/alice/projects/my-app`
- **When:** `matches()` is called
- **Then:** Returns true — partial match within path is enough; exact equality not required
