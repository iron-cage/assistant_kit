# Parameter :: `ids::`

Edge case tests for the `ids::` parameter on `.projects`. Tests validate the raw conversation-ID scripting output mode, its `count::` pairing, and its `project::` requirement.

**Source:** [param/31_ids.md](../../../../docs/cli/param/31_ids.md)

> **Note:** New in `.projects`, absorbed from `.list`'s former `type::conversation` early-dispatch path — same underlying algorithm, reachable through `.projects` instead of a separate `type::` value.

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| EC-1 | `project::X ids::1` outputs one conversation ID per line | Behavior |
| EC-2 | `project::X ids::1 count::1` outputs a single bare integer | Composition |
| EC-3 | `ids::1` without required `project::` rejected | Error Handling |

## Test Coverage Summary

- Behavior: 1 test (EC-1)
- Composition: 1 test (EC-2)
- Error Handling: 1 test (EC-3)

**Total:** 3 edge cases

## Test Cases

---

### EC-1: `project::X ids::1` outputs one conversation ID per line

- **Commands:** `.projects`
- **Given:** one project with 2 root conversations (one a family with 1 agent, one plain)
- **When:** `clg .projects project::{path} ids::1`
- **Then:** stdout is exactly 2 non-empty lines, one root conversation id per line; no `Found ...` header; no agent ids listed
- **Exit:** 0
- **Source:** [param/31_ids.md](../../../../docs/cli/param/31_ids.md); same test as [command/07_projects.md INT-61](../command/07_projects.md) (`int_61_ids_outputs_one_conversation_id_per_line`)

---

### EC-2: `project::X ids::1 count::1` outputs a single bare integer

- **Commands:** `.projects`
- **Given:** one project with 3 root conversations
- **When:** `clg .projects project::{path} ids::1 count::1`
- **Then:** stdout is exactly `3` (trimmed) and nothing else
- **Exit:** 0
- **Source:** [param/31_ids.md](../../../../docs/cli/param/31_ids.md); same test as [command/07_projects.md INT-62](../command/07_projects.md) (`int_62_ids_count_outputs_bare_integer`)

---

### EC-3: `ids::1` without required `project::` rejected

- **Commands:** `.projects`
- **Given:** clean environment, no `project::` given
- **When:** `clg .projects ids::1`
- **Then:** stderr contains the specific `project parameter required for ids` validation error (not a generic unknown-parameter hint); stdout is empty
- **Exit:** 1
- **Source:** [param/31_ids.md](../../../../docs/cli/param/31_ids.md); same test as [command/07_projects.md INT-63](../command/07_projects.md) (`int_63_ids_without_project_rejected`)
