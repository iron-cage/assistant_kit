# Parameter :: `scope::`

Edge case tests for the `scope::` parameter. Tests validate enum parsing and per-variant behavior.

**Source:** [param/12_scope.md](../../../../docs/cli/param/12_scope.md) | [type/07_scope_value.md](../../../../docs/cli/type/07_scope_value.md) | [param_group/05_scope_configuration.md](../../../../docs/cli/param_group/05_scope_configuration.md)

> **Note:** `.list` is deprecated (see [`command/02_list.md`](../command/02_list.md)), superseded by [`.projects`](../command/07_projects.md). EC-1–EC-6, EC-8 exercise `scope::` enum parsing and behavior via `.list`; `.projects` accepts the same five values with the same semantics (EC-7/EC-9/EC-10/EC-11 below already test `.projects` directly), except `.projects`' own default is `around`, not `.list`'s former `global`.

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
| EC-9 | Invalid value rejected with documented word order | Error Handling |
| EC-10 | Value "AROUND" accepted (case-insensitive) | Case Insensitivity |
| EC-11 | Global help describes scope:: with real value set | Help |

## Test Coverage Summary

- Enum Values: 4 tests (EC-1, EC-2, EC-3, EC-4)
- Case Insensitivity: 2 tests (EC-5, EC-10)
- Error Handling: 2 tests (EC-6, EC-9)
- Default: 1 test (EC-7)
- Behavior: 1 test (EC-8)
- Help: 1 test (EC-11)

**Total:** 11 edge cases

**Behavioral Divergence Pair:** EC-1 (scope::local, current project only) ↔ EC-2 (scope::relevant, ancestors included)

## Test Cases

---

### EC-1: Value "local" accepted

- **Commands:** `.list`
- **Given:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture`
- **When:** `clg .list scope::local`
- **Then:** stdout lists sessions belonging to the current directory project only; sessions from parent or sibling projects are absent.; output scoped to current project only
- **Exit:** 0
- **Source:** [param/12_scope.md](../../../../docs/cli/param/12_scope.md)

---

### EC-2: Value "relevant" accepted

- **Commands:** `.list`
- **Given:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture`
- **When:** `clg .list scope::relevant`
- **Then:** stdout lists sessions from the current project and sessions from projects at ancestor path levels.; output includes ancestor-level sessions (broader than `scope::local`)
- **Exit:** 0
- **Source:** [param/12_scope.md](../../../../docs/cli/param/12_scope.md)

---

### EC-3: Value "under" accepted

- **Commands:** `.list`
- **Given:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture`
- **When:** `clg .list scope::under path::/tmp/test-fixture`
- **Then:** stdout lists sessions from all projects whose path is under the given base path.; output includes sessions from descendant projects
- **Exit:** 0
- **Source:** [param/12_scope.md](../../../../docs/cli/param/12_scope.md)

---

### EC-4: Value "global" accepted

- **Commands:** `.list`
- **Given:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture`
- **When:** `clg .list scope::global`
- **Then:** stdout lists sessions from all projects in storage, regardless of path hierarchy.; output includes all sessions across all projects in storage
- **Exit:** 0
- **Source:** [param/12_scope.md](../../../../docs/cli/param/12_scope.md)

---

### EC-5: Value "RELEVANT" accepted (case-insensitive)

- **Commands:** `.list`
- **Given:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture`
- **When:** `clg .list scope::RELEVANT`
- **Then:** No error; output is identical to using lowercase `scope::relevant`.; output identical to lowercase variant (case normalization applied)
- **Exit:** 0
- **Source:** [param/12_scope.md](../../../../docs/cli/param/12_scope.md)

---

### EC-6: Invalid value "all" rejected with error

- **Commands:** `.list`
- **Given:** clean environment
- **When:** `clg .list scope::all`
- **Then:** stderr contains `scope must be relevant|local|under|global|around, got all`; error message `scope must be relevant|local|under|global|around, got all`
- **Exit:** 1
- **Source:** [param/12_scope.md](../../../../docs/cli/param/12_scope.md)

---

### EC-7: Omitted defaults to "under" scope (summary mode output)

- **Commands:** `.projects`
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
- **Source:** [param/12_scope.md](../../../../docs/cli/param/12_scope.md)

---

### EC-8: scope::global ignores path::

- **Commands:** `.list`
- **Given:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture`
- **When:** `clg .list scope::global path::/tmp/nonexistent-subpath`
- **Then:** stdout lists sessions from all projects in storage; the `path::` value has no filtering effect with `scope::global`.; output unaffected by path parameter when scope is global
- **Exit:** 0
- **Source:** [param/12_scope.md](../../../../docs/cli/param/12_scope.md)

---

### EC-9: Invalid value rejected with documented word order

- **Commands:** `.projects`
- **Given:** clean environment (validation precedes storage access)
- **When:** `clg .projects scope::bogus`
- **Then:** stderr contains the canonical message `scope must be relevant|local|under|global|around, got bogus` — the value list ends `global|around` exactly as documented at [param/12_scope.md](../../../../docs/cli/param/12_scope.md) line 19, not the pre-Task-512 `around|global` drift
- **Exit:** 1
- **Source:** [param/12_scope.md](../../../../docs/cli/param/12_scope.md)

---

### EC-10: Value "AROUND" accepted (case-insensitive)

- **Commands:** `.projects`
- **Given:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture` with one path-encoded project; run from that project's directory
- **When:** `clg .projects scope::AROUND`
- **Then:** No error; uppercase input normalizes to `around` — the same case-insensitivity EC-5 pins for `RELEVANT`, applied to the value `.projects` defaults to
- **Exit:** 0
- **Source:** [param/12_scope.md](../../../../docs/cli/param/12_scope.md)

---

### EC-11: Global help describes scope:: with real value set

- **Commands:** global help (`.help`)
- **Given:** clean environment
- **When:** `clg .help`
- **Then:** stdout's `scope::VALUE` option line contains the value token `relevant|local|under|global|around`; no fragment of the stale `(all, cli, web, ide)` set remains
- **Exit:** 0
- **Source:** [param/12_scope.md](../../../../docs/cli/param/12_scope.md)
