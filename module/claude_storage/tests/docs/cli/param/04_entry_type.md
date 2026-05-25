# Parameter :: `entry_type::`

Edge case tests for the `entry_type::` parameter. Tests validate enum parsing, case-insensitivity, and default behavior.

**Source:** [param/04_entry_type.md](../../../../docs/cli/param/04_entry_type.md) | [type/02_entry_type.md](../../../../docs/cli/type/02_entry_type.md)

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| EC-1 | Value "user" accepted | Enum Values |
| EC-2 | Value "assistant" accepted | Enum Values |
| EC-3 | Value "all" accepted | Enum Values |
| EC-4 | Value "USER" accepted (case-insensitive) | Case Insensitivity |
| EC-5 | Invalid value "both" rejected with error | Error Handling |
| EC-6 | Invalid value "system" rejected with error | Error Handling |
| EC-7 | Omitted defaults to "all" | Default |

## Test Coverage Summary

- Enum Values: 3 tests (EC-1, EC-2, EC-3)
- Case Insensitivity: 1 test (EC-4)
- Error Handling: 2 tests (EC-5, EC-6)
- Default: 1 test (EC-7)

**Total:** 7 edge cases

**Behavioral Divergence Pair:** EC-1 (entry_type::user) ↔ EC-2 (entry_type::assistant)

## Test Cases

---

### EC-1: Value "user" accepted

- **Commands:** `.search`
- **Given:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture`
- **When:** `clg .search query::error entry_type::user`
- **Then:** Search results containing only user-turn entries (not assistant responses) that match `error`.; only user-authored entries appear in results
- **Exit:** 0
- **Source:** [param/04_entry_type.md](../../../../docs/cli/param/04_entry_type.md)

---

### EC-2: Value "assistant" accepted

- **Commands:** `.search`
- **Given:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture`
- **When:** `clg .search query::error entry_type::assistant`
- **Then:** Search results containing only assistant-turn entries (not user messages) that match `error`.; + only assistant-authored entries appear in results
- **Exit:** 0
- **Source:** [param/04_entry_type.md](../../../../docs/cli/param/04_entry_type.md)

---

### EC-3: Value "all" accepted

- **Commands:** `.search`
- **Given:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture`
- **When:** `clg .search query::error entry_type::all`
- **Then:** Search results containing both user and assistant entries matching `error`; identical to omitting `entry_type::`.; + both user and assistant entries appear in results
- **Exit:** 0
- **Source:** [param/04_entry_type.md](../../../../docs/cli/param/04_entry_type.md)

---

### EC-4: Value "USER" accepted (case-insensitive)

- **Commands:** `.search`
- **Given:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture`
- **When:** `clg .search query::error entry_type::USER`
- **Then:** Same results as `entry_type::user` — case difference is normalized on parse.; + results match `entry_type::user` results exactly
- **Exit:** 0
- **Source:** [param/04_entry_type.md](../../../../docs/cli/param/04_entry_type.md)

---

### EC-5: Invalid value "both" rejected with error

- **Commands:** `.search`
- **Given:** clean environment
- **When:** `clg .search query::error entry_type::both`
- **Then:** `entry_type must be user|assistant|all, got both`; + error message `entry_type must be user|assistant|all, got both`
- **Exit:** 1
- **Source:** [param/04_entry_type.md](../../../../docs/cli/param/04_entry_type.md)

---

### EC-6: Invalid value "system" rejected with error

- **Commands:** `.search`
- **Given:** clean environment
- **When:** `clg .search query::error entry_type::system`
- **Then:** `entry_type must be user|assistant|all, got system`; + error message `entry_type must be user|assistant|all, got system`
- **Exit:** 1
- **Source:** [param/04_entry_type.md](../../../../docs/cli/param/04_entry_type.md)

---

### EC-7: Omitted defaults to "all"

- **Commands:** `.search`
- **Given:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture`
- **When:** `clg .search query::error`
- **Then:** Search results including both user and assistant entries matching `error`; same as `entry_type::all`.; + results are equivalent to `entry_type::all` (no implicit filter applied)
- **Exit:** 0
- **Source:** [param/04_entry_type.md](../../../../docs/cli/param/04_entry_type.md)
