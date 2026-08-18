# Type :: 14. `DetailLevel`

Type constraint tests for `DetailLevel` — output verbosity enum (`projects`/`sessions`).

**Source:** [type/14_detail_level.md](../../../../docs/cli/type/14_detail_level.md)

**Known gap:** the production type's `Case-insensitive on parse` constraint has no dedicated regression test — every existing `detail::` test (CLI-level and here) exercises only lowercase input. Unlike `ProjectType`'s case-insensitivity (covered indirectly by `.list type::PATH`, see [`param/18_type.md`](../param/18_type.md) EC-4), `detail::` has no pre-existing `.list`-era test to carry the same coverage, since it's new to both `.projects` and `.show`. Not counted below; candidate for a future edge case under task 525's own scope.

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| TC-1 | "projects" variant accepted | Valid Enum |
| TC-2 | "sessions" variant accepted (default) | Valid Enum |
| TC-3 | Invalid value rejected | Invalid Input |

## Test Coverage Summary

- Valid Enum: 2 tests (TC-1, TC-2)
- Invalid Input: 1 test (TC-3)

**Total:** 3 cases

## Test Cases

---

### TC-1: "projects" variant accepted

- **Given:** Input string `"projects"`
- **When:** `DetailLevel` is parsed
- **Then:** Accepted as `DetailLevel::Projects`; `is_projects()` returns true — header-only view selected
- **Source:** same test as [command/07_projects.md INT-53](../command/07_projects.md) (`int_53_detail_projects_header_only_no_body_lines`)

---

### TC-2: "sessions" variant accepted (default)

- **Given:** Input string `"sessions"`, or the parameter omitted entirely
- **When:** `DetailLevel` is parsed
- **Then:** Accepted as `DetailLevel::Sessions`; `is_projects()` returns false — full session/family detail selected; matches the omitted-parameter default on `.projects`
- **Source:** same test as [command/07_projects.md INT-54](../command/07_projects.md) (`int_54_detail_omitted_matches_explicit_sessions`)

---

### TC-3: Invalid value rejected

- **Given:** Input string `"bogus"`
- **When:** `DetailLevel` is parsed
- **Then:** Rejected; error message is `detail must be projects|sessions, got bogus`
- **Source:** same test as [command/07_projects.md INT-55](../command/07_projects.md) (`int_55_detail_invalid_value_rejected`)
