# Parameter :: `model`

Edge case tests for the `model` parameter. Tests validate absence
behavior (all models) and substring matching.

**Source:** [param/06_model.md](../../../../docs/cli/param/06_model.md)

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| EC-1 | Absent -> all models shown | Default |
| EC-2 | `model::opus` -> substring matches `claude-opus-4-8` | Substring Match |
| EC-3 | `model::sonnet` -> substring matches `claude-sonnet-5` | Substring Match |

## Test Coverage Summary

- Default: 1 test (EC-1)
- Substring Match: 2 tests (EC-2, EC-3)

**Total:** 3 edge cases

## Test Cases

---

### EC-1: Absent -> all models shown

- **Given:** journal with events across multiple models
- **When:** `clj .list`
- **Then:** exit 0; events for all models are shown
- **Exit:** 0
- **Source:** [param/06_model.md](../../../../docs/cli/param/06_model.md)

---

### EC-2: `model::opus` -> substring matches `claude-opus-4-8`

- **Given:** journal with events recording `model::claude-opus-4-8`
- **When:** `clj .list model::opus`
- **Then:** exit 0; events with a model field containing `opus` are shown
- **Exit:** 0
- **Source:** [param/06_model.md](../../../../docs/cli/param/06_model.md)

---

### EC-3: `model::sonnet` -> substring matches `claude-sonnet-5`

- **Given:** journal with events recording `model::claude-sonnet-5`
- **When:** `clj .list model::sonnet`
- **Then:** exit 0; events with a model field containing `sonnet` are shown
- **Exit:** 0
- **Source:** [param/06_model.md](../../../../docs/cli/param/06_model.md)
