# Parameter :: `detail::`

Edge case tests for the `detail::` parameter on `.projects` and `.show`. Tests validate the `projects`/`sessions` output-verbosity toggle, its default, error handling, and its interaction with other output-shaping parameters.

**Source:** [param/30_detail.md](../../../../docs/cli/param/30_detail.md)

> **Note:** New in `.projects`, absorbed `.list`'s former project-only default view and its `show_sessions::` toggle into a single explicit parameter (see [`15_sessions.md`](15_sessions.md)). Coverage below is `.projects`-only — `.show`'s `detail::` branch is out of scope for this file (→ `command/03_show.md`'s own test surface).

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| EC-1 | `detail::projects` shows header-only output, no session/family lines | Behavior |
| EC-2 | Omitted matches explicit `detail::sessions` byte-for-byte | Default |
| EC-3 | Invalid value rejected | Error Handling |
| EC-4 | `limit::`/`show_tree::`/`show_topic::` are no-ops under `detail::projects` | Composition |
| EC-5 | Mixed-case value (`detail::PROJECTS`) matches lowercase byte-for-byte | Case Insensitivity |

## Test Coverage Summary

- Behavior: 1 test (EC-1)
- Default: 1 test (EC-2)
- Error Handling: 1 test (EC-3)
- Composition: 1 test (EC-4)
- Case Insensitivity: 1 test (EC-5)

**Total:** 5 edge cases

## Test Cases

---

### EC-1: `detail::projects` shows header-only output, no session/family lines

- **Commands:** `.projects`
- **Given:** one project with a root session plus 2 agent sessions (family), one plain path-based project with a single session
- **When:** `clg .projects scope::global detail::projects`
- **Then:** stdout shows the `Found N projects` header; no session ids, agent ids, or `[N agents...]` bracket breakdowns appear anywhere in the body
- **Exit:** 0
- **Source:** [param/30_detail.md](../../../../docs/cli/param/30_detail.md); same test as [command/07_projects.md INT-53](../command/07_projects.md) (`int_53_detail_projects_header_only_no_body_lines`)

---

### EC-2: Omitted matches explicit `detail::sessions` byte-for-byte

- **Commands:** `.projects`
- **Given:** one path-based project with one session
- **When:** `clg .projects scope::global` compared against `clg .projects scope::global detail::sessions`
- **Then:** stdout is byte-identical between the two invocations
- **Exit:** 0
- **Source:** [param/30_detail.md](../../../../docs/cli/param/30_detail.md); same test as [command/07_projects.md INT-54](../command/07_projects.md) (`int_54_detail_omitted_matches_explicit_sessions`)

---

### EC-3: Invalid value rejected

- **Commands:** `.projects`
- **Given:** clean environment
- **When:** `clg .projects detail::bogus`
- **Then:** stderr contains `detail must be projects|sessions, got bogus`; stdout is empty
- **Exit:** 1
- **Source:** [param/30_detail.md](../../../../docs/cli/param/30_detail.md); same test as [command/07_projects.md INT-55](../command/07_projects.md) (`int_55_detail_invalid_value_rejected`)

---

### EC-4: `limit::`/`show_tree::`/`show_topic::` are no-ops under `detail::projects`

- **Commands:** `.projects`
- **Given:** one project with a root session plus one agent session (family)
- **When:** `clg .projects scope::global detail::projects` compared against the same command plus `limit::1 show_tree::1 show_topic::1`
- **Then:** stdout is byte-identical between the two invocations — these three parameters have nothing to act on once body lines are suppressed
- **Exit:** 0
- **Source:** [param/30_detail.md](../../../../docs/cli/param/30_detail.md); same test as [command/07_projects.md INT-65](../command/07_projects.md) (`int_65_limit_show_tree_show_topic_noop_under_detail_projects`)

---

### EC-5: Mixed-case value (`detail::PROJECTS`) matches lowercase byte-for-byte

- **Commands:** `.projects`
- **Given:** two projects with sessions in scope (one family, one plain path project)
- **When:** `clg .projects scope::global detail::PROJECTS` compared against `clg .projects scope::global detail::projects`
- **Then:** stdout is byte-identical between the two invocations — `validate_detail_level` lowercases input before matching
- **Exit:** 0
- **Source:** [param/30_detail.md](../../../../docs/cli/param/30_detail.md); same test as [command/07_projects.md INT-67](../command/07_projects.md) (`int_67_detail_uppercase_matches_lowercase`)
