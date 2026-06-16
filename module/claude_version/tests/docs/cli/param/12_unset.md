# Test: `unset::`

Edge case coverage for the `unset::` parameter. See [param/12_unset.md](../../../../docs/cli/param/12_unset.md) for specification.

### Scope

- **Purpose**: Edge case tests for the `unset::` parameter.
- **Responsibility**: Boundary values, invalid inputs, mutual exclusion, and default behavior for `unset::`.
- **Commands:** `.config`
- **In Scope**: Single-parameter edge cases, validation errors, mutual exclusion with `value::`.
- **Out of Scope**: Command integration (-> `../command/`), group interactions (-> `../param_group/`).

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| EC-1 | `key::K unset::1` removes key from settings | Valid: unset |
| EC-2 | `key::K unset::1` for nonexistent key -> exit 0 (idempotent) | Valid: idempotent |
| EC-3 | `unset::1` without `key::` -> exit 1, key required | Missing Required |
| EC-4 | `key::K value::V unset::1` -> exit 1, mutually exclusive | Mutual Exclusion |
| EC-5 | `unset::0` (explicit disable) -> no effect, treated as normal mode | Valid: disabled |
| EC-6 | `unset::2` -> exit 1, boolean must be 0 or 1 | Invalid Value |
| EC-7 | `key::K unset::1 dry::1` -> preview without deleting | Dry Run |

## Test Coverage Summary

- Valid (unset): 1 test
- Valid (idempotent): 1 test
- Valid (disabled): 1 test
- Missing Required: 1 test
- Mutual Exclusion: 1 test
- Invalid Value: 1 test
- Dry Run: 1 test

**Total:** 7 edge cases

**Behavioral Divergence Pair:** EC-1 (`key::K unset::1` → removes key, exit 0) ↔ EC-5 (`key::K unset::0` → no effect on settings, exit 0)

---

### EC-1: `key::K unset::1` removes key from settings

- **Given:** `~/.claude/settings.json` exists and contains key `K`
- **When:** `clv .config key::K unset::1`
- **Then:** exit 0; key `K` removed from `~/.claude/settings.json`; remaining keys unchanged
- **Exit:** 0
- **Source:** [param/12_unset.md](../../../../docs/cli/param/12_unset.md)

---

### EC-2: `key::K unset::1` for nonexistent key -> exit 0 (idempotent)

- **Given:** `~/.claude/settings.json` does not contain key `K` (or file is absent)
- **When:** `clv .config key::K unset::1`
- **Then:** exit 0; file unchanged; no error output; idempotent
- **Exit:** 0
- **Source:** [param/12_unset.md](../../../../docs/cli/param/12_unset.md)

---

### EC-3: `unset::1` without `key::` -> exit 1, key required

- **Given:** no `key::` supplied
- **When:** `clv .config unset::1`
- **Then:** exit 1; error: `key::` required when `unset::1`; no file modified
- **Exit:** 1
- **Source:** [param/12_unset.md](../../../../docs/cli/param/12_unset.md)

---

### EC-4: `key::K value::V unset::1` -> exit 1, mutually exclusive

- **Given:** `key::`, `value::`, and `unset::1` all supplied
- **When:** `clv .config key::theme value::dark unset::1`
- **Then:** exit 1; error: `value::` and `unset::` are mutually exclusive; no file modified
- **Exit:** 1
- **Source:** [param/12_unset.md](../../../../docs/cli/param/12_unset.md)

---

### EC-5: `unset::0` (explicit disable) -> no effect, treated as normal mode

- **Given:** `key::theme` and `value::dark` supplied; `unset::0` present
- **When:** `clv .config key::theme value::dark unset::0`
- **Then:** exit 0; write proceeds normally; `"theme": "dark"` stored in settings
- **Exit:** 0
- **Source:** [param/12_unset.md](../../../../docs/cli/param/12_unset.md)

---

### EC-6: `unset::2` -> exit 1, boolean must be 0 or 1

- **Given:** `key::theme` supplied; `unset::2` present
- **When:** `clv .config key::theme unset::2`
- **Then:** exit 1; error: `unset::` accepts only 0 or 1; no file modified
- **Exit:** 1
- **Source:** [param/12_unset.md](../../../../docs/cli/param/12_unset.md)

---

### EC-7: `key::K unset::1 dry::1` -> preview without deleting

- **Given:** `~/.claude/settings.json` exists and contains key `K`
- **When:** `clv .config key::K unset::1 dry::1`
- **Then:** exit 0; output shows `[dry-run] would remove key K`; settings file not modified
- **Exit:** 0
- **Source:** [param/12_unset.md](../../../../docs/cli/param/12_unset.md)
