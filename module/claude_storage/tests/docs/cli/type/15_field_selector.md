# Type :: 15. `FieldSelector`

Type constraint tests for `FieldSelector` — comma-separated attribute-projection field list (or `all`).

**Source:** [type/15_field_selector.md](../../../../docs/cli/type/15_field_selector.md)

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| TC-1 | Single valid token accepted | Valid Input |
| TC-2 | Multi-token comma list accepted, request order preserved | Valid Input |
| TC-3 | `all` expands to the full 18-name canonical vocabulary, in canonical order | Valid Input |
| TC-4 | Invalid token rejected, error lists all 18 valid names | Invalid Input |
| TC-5 | `all` combined with another token rejected | Invalid Input |
| TC-6 | Case-insensitive per-token parsing (`"UUID"` == `"uuid"`) | Case Insensitivity |
| TC-7 | Whitespace trimmed around tokens and commas | Normalization |
| TC-8 | Duplicate token collapses to one occurrence | Normalization |
| TC-9 | Empty string rejected | Invalid Input |

## Test Coverage Summary

- Valid Input: 3 tests (TC-1, TC-2, TC-3)
- Invalid Input: 3 tests (TC-4, TC-5, TC-9)
- Case Insensitivity: 1 test (TC-6)
- Normalization: 2 tests (TC-7, TC-8)

**Total:** 9 cases

## Test Cases

---

### TC-1: Single valid token accepted

- **Given:** Input string `"timestamp"`
- **When:** `FieldSelector` is parsed
- **Then:** Accepted; `fields()` returns `["timestamp"]`
- **Source:** same test as [param/32_fields.md](../param/32_fields.md) EC-1

---

### TC-2: Multi-token comma list accepted, request order preserved

- **Given:** Input string `"uuid,model"`
- **When:** `FieldSelector` is parsed
- **Then:** Accepted; `fields()` returns `["uuid", "model"]` in that exact order (not canonical-vocabulary order)
- **Source:** same test as [param/32_fields.md](../param/32_fields.md) EC-2

---

### TC-3: `all` expands to the full 18-name canonical vocabulary, in canonical order

- **Given:** Input string `"all"`
- **When:** `FieldSelector` is parsed
- **Then:** Accepted; `fields()` returns all 18 canonical names in the order documented in [type/15_field_selector.md](../../../../docs/cli/type/15_field_selector.md) — `uuid, parent_uuid, timestamp, entry_type, cwd, session_id, version, git_branch, user_type, is_sidechain, content, thinking_level, thinking_disabled, model, message_id, stop_reason, stop_sequence, request_id`
- **Source:** same test as [param/32_fields.md](../param/32_fields.md) EC-3

---

### TC-4: Invalid token rejected, error lists all 18 valid names

- **Given:** Input string `"bogus"`
- **When:** `FieldSelector` is parsed
- **Then:** Rejected; error message is `unknown field 'bogus' — valid fields: uuid, parent_uuid, timestamp, entry_type, cwd, session_id, version, git_branch, user_type, is_sidechain, content, thinking_level, thinking_disabled, model, message_id, stop_reason, stop_sequence, request_id, or all`
- **Source:** same test as [param/32_fields.md](../param/32_fields.md) EC-4

---

### TC-5: `all` combined with another token rejected

- **Given:** Input string `"all,uuid"`
- **When:** `FieldSelector` is parsed
- **Then:** Rejected; error message is `'all' cannot be combined with other fields`
- **Source:** same test as [param/32_fields.md](../param/32_fields.md) EC-5

---

### TC-6: Case-insensitive per-token parsing (`"UUID"` == `"uuid"`)

- **Given:** Input string `"UUID,Timestamp"`
- **When:** `FieldSelector` is parsed
- **Then:** Accepted; `fields()` returns `["uuid", "timestamp"]` — identical to lowercase input
- **Source:** same test as [param/32_fields.md](../param/32_fields.md) EC-7

---

### TC-7: Whitespace trimmed around tokens and commas

- **Given:** Input string `" uuid , timestamp "`
- **When:** `FieldSelector` is parsed
- **Then:** Accepted; `fields()` returns `["uuid", "timestamp"]` — identical to the untrimmed form
- **Source:** same test as [param/32_fields.md](../param/32_fields.md) EC-7

---

### TC-8: Duplicate token collapses to one occurrence

- **Given:** Input string `"uuid,uuid"`
- **When:** `FieldSelector` is parsed
- **Then:** Accepted; `fields()` returns `["uuid"]` — single occurrence, not two
- **Source:** same test as [param/32_fields.md](../param/32_fields.md) EC-8

---

### TC-9: Empty string rejected

- **Given:** Input string `""`
- **When:** `FieldSelector` is parsed
- **Then:** Rejected; error message is `fields must be non-empty`
- **Source:** unit-level only — not reachable via the CLI, where unilang's own argument parser rejects a missing `fields::` value before parse ever runs (see [param/32_fields.md](../param/32_fields.md) EC-6)
