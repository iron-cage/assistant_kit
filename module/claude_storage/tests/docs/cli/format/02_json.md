# Format :: 2. JSON

Output format verification tests for JSON export.

**Source:** [format/02_json.md](../../../../docs/cli/format/02_json.md)

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| FM-1 | Top-level JSON object has required keys | Structure |
| FM-2 | Entries array contains all session entries | Structure |
| FM-3 | Original JSONL fields preserved verbatim | Content Fidelity |
| FM-4 | Output is valid JSON (parseable by jq) | Parsability |
| FM-5 | Pretty-printed with 2-space indentation | Formatting |

## Test Coverage Summary

- Structure: 2 tests (FM-1, FM-2)
- Content Fidelity: 1 test (FM-3)
- Parsability: 1 test (FM-4)
- Formatting: 1 test (FM-5)

**Total:** 5 cases

## Test Cases

---

### FM-1: Top-level JSON object has required keys

**Command:**
```
clg .export session_id::{id} format::json output::{file}.json
```

**Expected behavior:**
- Output file is a JSON object
- Top-level keys include `session_id`, `storage_path`, and `entries`
- `session_id` value matches the requested session ID
- `entries` value is a JSON array

---

### FM-2: Entries array contains all session entries

**Command:**
```
clg .export session_id::{id} format::json output::{file}.json
```

**Expected behavior:**
- `entries` array length equals the known entry count for the session
- Each array element is a JSON object
- Each entry object contains `uuid`, `timestamp`, and `type` keys

---

### FM-3: Original JSONL fields preserved verbatim

**Command:**
```
clg .export session_id::{id} format::json output::{file}.json
```

**Expected behavior:**
- Entry `uuid` values match the source JSONL uuid fields
- Entry `type` values match the source JSONL type fields (`"user"` or `"assistant"`)
- No fields are omitted or renamed from the original JSONL lines

---

### FM-4: Output is valid JSON (parseable by jq)

**Command:**
```
clg .export session_id::{id} format::json output::{file}.json
```

**Expected behavior:**
- File content passes `jq .` without error (exit 0)
- No trailing commas in arrays or objects
- All string values properly escaped per JSON spec

---

### FM-5: Pretty-printed with 2-space indentation

**Command:**
```
clg .export session_id::{id} format::json output::{file}.json
```

**Expected behavior:**
- Top-level `{` and `}` at column 0
- `session_id`, `storage_path`, `entries` keys indented 2 spaces
- Entry objects within `entries` array indented 4 spaces
- Entry fields indented 6 spaces
