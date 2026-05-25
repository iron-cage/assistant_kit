# Type :: 3. `ExportFormat`

Type constraint tests for `ExportFormat` — output serialization format enum.

**Source:** [type/03_export_format.md](../../../../docs/cli/type/03_export_format.md)

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| TC-1 | "markdown" variant accepted (default) | Valid Enum |
| TC-2 | "json" variant accepted | Valid Enum |
| TC-3 | "text" variant accepted | Valid Enum |
| TC-4 | Case-insensitive parse accepted | Case Handling |
| TC-5 | Invalid value rejected | Invalid Input |

## Test Coverage Summary

- Valid Enum: 3 tests (TC-1, TC-2, TC-3)
- Case Handling: 1 test (TC-4)
- Invalid Input: 1 test (TC-5)

**Total:** 5 cases

## Test Cases

---

### TC-1: "markdown" variant accepted (default)

- **Given:** Input string `"markdown"`
- **When:** `ExportFormat` is parsed
- **Then:** Accepted as `ExportFormat::Markdown`; `file_extension()` returns `"md"`; `is_default()` returns true

---

### TC-2: "json" variant accepted

- **Given:** Input string `"json"`
- **When:** `ExportFormat` is parsed
- **Then:** Accepted as `ExportFormat::Json`; `file_extension()` returns `"json"`

---

### TC-3: "text" variant accepted

- **Given:** Input string `"text"`
- **When:** `ExportFormat` is parsed
- **Then:** Accepted as `ExportFormat::Text`; `file_extension()` returns `"txt"`

---

### TC-4: Case-insensitive parse accepted

- **Given:** Input string `"JSON"` (uppercase)
- **When:** `ExportFormat` is parsed
- **Then:** Accepted as `ExportFormat::Json` — parsing normalizes case before matching

---

### TC-5: Invalid value rejected

- **Given:** Input string `"csv"`
- **When:** `ExportFormat` is parsed
- **Then:** Rejected; error message is `format must be markdown|json|text, got csv`
