# Parameter :: `scope::`

Edge case tests for the `scope::` parameter. Tests validate enum parsing and per-variant behavior.

**Source:** [params.md#parameter--12-scope](../../../../docs/cli/params.md#parameter--12-scope) | [types.md#scopevalue](../../../../docs/cli/types.md#scopevalue) | [parameter_groups.md#scope-configuration](../../../../docs/cli/parameter_groups.md#scope-configuration)

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| EC-1 | Value "local" accepted | Enum Values |
| EC-2 | Value "relevant" accepted | Enum Values |
| EC-3 | Value "under" accepted | Enum Values |
| EC-4 | Value "global" accepted | Enum Values |
| EC-5 | Value "RELEVANT" accepted (case-insensitive) | Case Insensitivity |
| EC-6 | Invalid value "all" rejected with error | Error Handling |
| EC-7 | Omitted defaults to "under" | Default |
| EC-8 | scope::global ignores path:: | Behavior |

## Test Coverage Summary

- Enum Values: 4 tests (EC-1, EC-2, EC-3, EC-4)
- Case Insensitivity: 1 test (EC-5)
- Error Handling: 1 test (EC-6)
- Default: 1 test (EC-7)
- Behavior: 1 test (EC-8)

**Total:** 8 edge cases

**Behavioral Divergence Pair:** EC-1 (valid/expected path) ↔ EC-2 (invalid/rejected path)

## Test Cases

---

### EC-1: Value "local" accepted

- **Given:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture`
- **When:** `clg .list scope::local`
- **Then:** stdout lists sessions belonging to the current directory project only; sessions from parent or sibling projects are absent.; output scoped to current project only
- **Exit:** 0
- **Source:** [params.md](../../../../docs/cli/params.md)

---

### EC-2: Value "relevant" accepted

- **Given:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture`
- **When:** `clg .list scope::relevant`
- **Then:** stdout lists sessions from the current project and sessions from projects at ancestor path levels.; output includes ancestor-level sessions (broader than `scope::local`)
- **Exit:** 0
- **Source:** [params.md](../../../../docs/cli/params.md)

---

### EC-3: Value "under" accepted

- **Given:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture`
- **When:** `clg .list scope::under path::/tmp/test-fixture`
- **Then:** stdout lists sessions from all projects whose path is under the given base path.; output includes sessions from descendant projects
- **Exit:** 0
- **Source:** [params.md](../../../../docs/cli/params.md)

---

### EC-4: Value "global" accepted

- **Given:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture`
- **When:** `clg .list scope::global`
- **Then:** stdout lists sessions from all projects in storage, regardless of path hierarchy.; output includes all sessions across all projects in storage
- **Exit:** 0
- **Source:** [params.md](../../../../docs/cli/params.md)

---

### EC-5: Value "RELEVANT" accepted (case-insensitive)

- **Given:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture`
- **When:** `clg .list scope::RELEVANT`
- **Then:** No error; output is identical to using lowercase `scope::relevant`.; output identical to lowercase variant (case normalization applied)
- **Exit:** 0
- **Source:** [params.md](../../../../docs/cli/params.md)

---

### EC-6: Invalid value "all" rejected with error

- **Given:** clean environment
- **When:** `clg .list scope::all`
- **Then:** stderr contains `scope must be relevant|local|under|global, got all`; error message `scope must be relevant|local|under|global, got all`
- **Exit:** 1
- **Source:** [params.md](../../../../docs/cli/params.md)

---

### EC-7: Omitted defaults to "under" scope (summary mode output)

- **Given:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture` with a parent project at `/tmp/test-fixture/parent` and a child project at `/tmp/test-fixture/parent/child`. The most-recent session is in the child project. Run from `/tmp/test-fixture/parent`.
- **When:** `clg .projects` (run from `/tmp/test-fixture/parent`)
- **Then:** ```
Active project  ~/test-fixture/parent/child  (N sessions, last active Xs ago)
Last session:  {8-char-id}  Xs ago  (N entries)

Last message:
  {last message text}
```
The project path in the summary header belongs to the child project, confirming that `scope::under` is active (sub-project sessions are in scope). stdout does NOT contain `Found N projects:`.; `Active project` header present + `Found N projects:` absent + child project path in header
- **Exit:** 0
- **Source:** [params.md](../../../../docs/cli/params.md)

---

### EC-8: scope::global ignores path::

- **Given:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture`
- **When:** `clg .list scope::global path::/tmp/nonexistent-subpath`
- **Then:** stdout lists sessions from all projects in storage; the `path::` value has no filtering effect with `scope::global`.; output unaffected by path parameter when scope is global
- **Exit:** 0
- **Source:** [params.md](../../../../docs/cli/params.md)
