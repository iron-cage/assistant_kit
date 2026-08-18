# Type :: 14. `DetailLevel`

Type constraint tests for `DetailLevel` — output verbosity enum (`projects`/`sessions`).

**Source:** [type/14_detail_level.md](../../../../docs/cli/type/14_detail_level.md)

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| TC-1 | "projects" variant accepted | Valid Enum |
| TC-2 | "sessions" variant accepted (default) | Valid Enum |
| TC-3 | Invalid value rejected | Invalid Input |
| TC-4 | Mixed-case value (`"PROJECTS"`) accepted, matches lowercase | Case Insensitivity |

## Test Coverage Summary

- Valid Enum: 2 tests (TC-1, TC-2)
- Invalid Input: 1 test (TC-3)
- Case Insensitivity: 1 test (TC-4)

**Total:** 4 cases

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

---

### TC-4: Mixed-case value (`"PROJECTS"`) accepted, matches lowercase

- **Given:** Input string `"PROJECTS"`
- **When:** `DetailLevel` is parsed
- **Then:** Accepted as `DetailLevel::Projects` — identical to lowercase `"projects"`; output byte-identical between the two invocations. Closes the gap flagged in this file's earlier "Known gap" note; unlike `ProjectType`'s case-insensitivity (covered indirectly by `.list type::PATH`, see [`param/18_type.md`](../param/18_type.md) EC-4), `detail::` needed its own dedicated test since it has no pre-existing `.list`-era test to carry the coverage
- **Source:** same test as [command/07_projects.md INT-67](../command/07_projects.md) (`int_67_detail_uppercase_matches_lowercase`)
