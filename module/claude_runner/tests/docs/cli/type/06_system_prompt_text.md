# Type :: `SystemPromptText`

Validation tests for the `SystemPromptText` semantic type (free-form UTF-8 string). Tests validate pass-through of text including empty strings and missing-value handling.

**Source:** [type.md](../../../../docs/cli/type.md#type--6-systemprompttext)

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| TC-1 | Non-empty text → forwarded verbatim | Valid Input |
| TC-2 | Empty string `""` → accepted (forwarded without rejection) | Boundary |
| TC-3 | `--system-prompt` without value → exit 1 | Missing Value |
| TC-4 | `--append-system-prompt` without value → exit 1 | Missing Value |

## Test Coverage Summary

- Valid Input: 1 test (TC-1)
- Boundary: 1 test (TC-2)
- Missing Value: 2 tests (TC-3, TC-4)

**Total:** 4 test cases

## Test Cases

---

### TC-1: Non-empty text → forwarded verbatim

- **Given:** clean environment
- **When:** `clr --dry-run --system-prompt "You are a Rust expert." "test"`
- **Then:** Assembled command contains `--system-prompt` and the text; forwarded intact
- **Exit:** 0
- **Source:** [type.md — SystemPromptText](../../../../docs/cli/type.md#type--6-systemprompttext)

---

### TC-2: Empty string → accepted (not rejected)

- **Given:** clean environment
- **When:** `clr --dry-run --system-prompt "" "test"`
- **Then:** Exit 0; empty string accepted without error; forwarded to claude
- **Exit:** 0
- **Source:** [type.md — SystemPromptText](../../../../docs/cli/type.md#type--6-systemprompttext)

---

### TC-3: `--system-prompt` without value → exit 1

- **Given:** clean environment
- **When:** `clr --system-prompt`
- **Then:** Exit 1; error indicating `--system-prompt` requires a value
- **Exit:** 1
- **Source:** [type.md — SystemPromptText](../../../../docs/cli/type.md#type--6-systemprompttext)

---

### TC-4: `--append-system-prompt` without value → exit 1

- **Given:** clean environment
- **When:** `clr --append-system-prompt`
- **Then:** Exit 1; error indicating `--append-system-prompt` requires a value
- **Exit:** 1
- **Source:** [type.md — SystemPromptText](../../../../docs/cli/type.md#type--6-systemprompttext)
