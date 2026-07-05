# Invariant :: Entry Type Format

Direct contract tests for the JSONL entry type field behavioral invariant.

**Source:** [invariant/003_entry_type_format.md](../../../docs/invariant/003_entry_type_format.md)

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| IN-1 | Top-level `"type": "user"` entry counted as conversation entry | Classification |
| IN-2 | Top-level `"type": "assistant"` entry counted as conversation entry | Classification |
| IN-3 | `"queue-operation"` top-level type skipped in conversation counts | Skip Rule |
| IN-4 | `"summary"` top-level type skipped in conversation counts | Skip Rule |
| IN-5 | Entry missing `"uuid"` field silently skipped | Skip Rule |
| IN-6 | Nested `"message.role"` field NOT used for entry classification | Violation Guard |

## Test Coverage Summary

- Classification: 2 tests (IN-1, IN-2)
- Skip Rule: 3 tests (IN-3, IN-4, IN-5)
- Violation Guard: 1 test (IN-6)

**Total:** 6 invariant contract cases

**Implementation target:** `tests/invariant_contracts_test.rs`

## Test Cases

---

### IN-1: Top-level `"type": "user"` counted as conversation entry

- **Given:** JSONL line `{"type":"user","uuid":"a1b2c3d4-...","message":{"role":"user","content":"hello"}}`
- **When:** entry parser classifies and counts the entry
- **Then:** entry is classified as a user conversation entry and included in conversation count

---

### IN-2: Top-level `"type": "assistant"` counted as conversation entry

- **Given:** JSONL line `{"type":"assistant","uuid":"b2c3d4e5-...","message":{"role":"assistant","content":"hello"}}`
- **When:** entry parser classifies and counts the entry
- **Then:** entry is classified as an assistant conversation entry and included in conversation count

---

### IN-3: `"queue-operation"` top-level type skipped

- **Given:** JSONL line `{"type":"queue-operation","uuid":"c3d4e5f6-...",...}`
- **When:** entry parser processes the line
- **Then:** entry is silently skipped; conversation count unchanged

---

### IN-4: `"summary"` top-level type skipped

- **Given:** JSONL line `{"type":"summary","uuid":"d4e5f6a7-...",...}`
- **When:** entry parser processes the line
- **Then:** entry is silently skipped; conversation count unchanged

---

### IN-5: Entry missing `"uuid"` silently skipped

- **Given:** JSONL line `{"type":"user","message":{"role":"user","content":"hello"}}` — no `"uuid"` field
- **When:** entry parser processes the line
- **Then:** entry is silently skipped without error; subsequent entries continue to parse normally

---

### IN-6: Nested `"message.role"` NOT used for entry classification

- **Given:** two JSONL entries: (A) `{"type":"user","uuid":"...","message":{"role":"assistant",...}}` and (B) `{"type":"assistant","uuid":"...","message":{"role":"user",...}}`
- **When:** entry parser classifies both entries
- **Then:** (A) is classified by top-level `"type": "user"` as a user entry; (B) by top-level `"type": "assistant"` as an assistant entry; `"message.role"` is ignored for classification
