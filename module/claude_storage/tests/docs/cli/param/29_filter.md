# Parameter :: `filter::`

Edge case tests for the `filter::` parameter on `.projects`. Tests validate substring matching, empty-result handling, and composition with `type::`.

**Source:** [param/29_filter.md](../../../../docs/cli/param/29_filter.md)

> **Note:** New in `.projects`, absorbed from `.list`'s former `path::` role (see [`09_path.md`](09_path.md), [`command/02_list.md`](../command/02_list.md)) — kept as a distinct name because `.projects`'s own `path::` already means the scope anchor.

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| EC-1 | Substring match narrows to matching projects only | Behavior |
| EC-2 | No matching substring shows empty listing, not an error | Empty State |
| EC-3 | Composes with `type::` — both filters apply together | Composition |
| EC-4 | Mixed-case substring (`filter::ALPHA`) matches lowercase-equivalent projects | Case Insensitivity |

## Test Coverage Summary

- Behavior: 1 test (EC-1)
- Empty State: 1 test (EC-2)
- Composition: 1 test (EC-3)
- Case Insensitivity: 1 test (EC-4)

**Total:** 4 edge cases

## Test Cases

---

### EC-1: Substring match narrows to matching projects only

- **Commands:** `.projects`
- **Given:** three path-based projects whose decoded paths contain `alpha`, `beta`, `gamma` respectively
- **When:** `clg .projects scope::global filter::alpha`
- **Then:** stdout includes only the `alpha` project; `beta` and `gamma` projects are absent
- **Exit:** 0
- **Source:** [param/29_filter.md](../../../../docs/cli/param/29_filter.md); same test as [command/07_projects.md INT-56](../command/07_projects.md) (`int_56_filter_narrows_to_matching_substring`)

---

### EC-2: No matching substring shows empty listing, not an error

- **Commands:** `.projects`
- **Given:** one path-based project, none matching the filter substring
- **When:** `clg .projects scope::global filter::nonexistent-substring`
- **Then:** stdout shows a `Found 0 projects` header; exit succeeds (not an error); the non-matching project is absent
- **Exit:** 0
- **Source:** [param/29_filter.md](../../../../docs/cli/param/29_filter.md); same test as [command/07_projects.md INT-57](../command/07_projects.md) (`int_57_filter_no_match_shows_empty_listing`)

---

### EC-3: Composes with `type::` — both filters apply together

- **Commands:** `.projects`
- **Given:** a path project matching both `type::path` and `filter::alpha`, a second path project matching only `filter::alpha`, and a UUID-named project whose raw id also matches `filter::alpha`
- **When:** `clg .projects scope::global type::path filter::alpha`
- **Then:** stdout includes only the project matching both constraints; the UUID project (fails `type::path`) and the mismatched path project (fails `filter::alpha`) are both absent
- **Exit:** 0
- **Source:** [param/29_filter.md](../../../../docs/cli/param/29_filter.md); same test as [command/07_projects.md INT-64](../command/07_projects.md) (`int_64_type_and_filter_compose`)

---

### EC-4: Mixed-case substring (`filter::ALPHA`) matches lowercase-equivalent projects

- **Commands:** `.projects`
- **Given:** two path-based projects whose decoded paths contain `alpha-int68` and `beta-int68` respectively
- **When:** `clg .projects scope::global filter::ALPHA-INT68`
- **Then:** stdout includes only the `alpha` project; `beta` is absent — both the supplied substring and the decoded display path are lowercased before comparison, so casing never affects the match
- **Exit:** 0
- **Source:** [param/29_filter.md](../../../../docs/cli/param/29_filter.md); same test as [command/07_projects.md INT-68](../command/07_projects.md) (`int_68_filter_uppercase_matches_lowercase`)
