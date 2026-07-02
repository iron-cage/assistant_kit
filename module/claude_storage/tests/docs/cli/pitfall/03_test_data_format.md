# Pitfall :: Test Data Must Match Production Format

Contract tests verifying that JSONL test fixtures use production-format fields and parser handles malformed data correctly.

**Source:** [cli/pitfall/03_test_data_format.md](../../../../docs/cli/pitfall/03_test_data_format.md)

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| PF-1 | JSONL with correct `"type": "user"` entries produces non-zero entry count | Correct Format |
| PF-2 | JSONL with `"type": "message"` entries produces entry count of 0 | Wrong Type Rejection |
| PF-3 | JSONL entry missing `"uuid"` field is silently skipped; count excludes it | UUID Requirement |
| PF-4 | JSONL entry with non-UUID `"uuid"` value is silently skipped | UUID Format |

## Test Coverage Summary

- Correct Format: 1 test (PF-1)
- Wrong Type Rejection: 1 test (PF-2)
- UUID Requirement: 1 test (PF-3)
- UUID Format: 1 test (PF-4)

**Total:** 4 pitfall contract cases

**Implementation target:** `tests/invariant_contracts_test.rs`

## Test Cases

---

### PF-1: JSONL with correct `"type": "user"` entries produces non-zero entry count

- **Given:** JSONL file with entries using `"type": "user"`, `"uuid": "<UUID>"`, `"sessionId": "<UUID>"`, and `"message": {"role": "user", "content": "..."}` (production format)
- **When:** the entry parser counts entries
- **Then:** count equals the number of well-formed entries (non-zero)

---

### PF-2: JSONL with `"type": "message"` entries produces entry count of 0

- **Given:** JSONL file with entries using `"type": "message"` (wrong type value) — a regression from issue-011
- **When:** the entry parser counts entries
- **Then:** count equals 0; no entries pass the type classifier

---

### PF-3: JSONL entry missing `"uuid"` field is silently skipped

- **Given:** JSONL file where one entry has `"type": "user"` but no `"uuid"` field; remaining entries are well-formed
- **When:** the entry parser counts entries
- **Then:** the entry without `"uuid"` is skipped; count equals the number of well-formed entries only

---

### PF-4: JSONL entry with non-UUID `"uuid"` value is silently skipped

- **Given:** JSONL file where one entry has `"uuid": "entry-1"` (simple identifier, not UUID format)
- **When:** the entry parser processes the line
- **Then:** the entry is silently skipped; no parse error is raised; subsequent entries continue to parse normally
