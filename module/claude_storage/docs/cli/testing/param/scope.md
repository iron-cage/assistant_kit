# Parameter :: `scope::`

Edge case tests for the `scope::` parameter. Tests validate enum parsing and per-variant behavior.

**Source:** [params.md#parameter--12-scope](../../params.md#parameter--12-scope) | [types.md#scopevalue](../../types.md#scopevalue) | [parameter_groups.md#scope-configuration](../../parameter_groups.md#scope-configuration)

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

## Test Cases

### EC-1: Value "local" accepted

**Goal:** Verify that `scope::local` is accepted and `.sessions` returns only sessions for the current project.
**Setup:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture`
**Command:** `clg .sessions scope::local`
**Expected Output:** stdout lists sessions belonging to the current directory project only; sessions from parent or sibling projects are absent.
**Verification:**
- Exit code is 0
- Output contains sessions from the current project only
- Sessions from unrelated projects are not listed
**Pass Criteria:** exit 0 + output scoped to current project only
**Source:** [params.md](../../params.md)

### EC-2: Value "relevant" accepted

**Goal:** Verify that `scope::relevant` is accepted and `.sessions` walks the ancestor chain, returning sessions from the current project and all parent projects up to `/`.
**Setup:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture`
**Command:** `clg .sessions scope::relevant`
**Expected Output:** stdout lists sessions from the current project and sessions from projects at ancestor path levels.
**Verification:**
- Exit code is 0
- Output contains sessions from the current project
- Output may also include sessions from ancestor-level projects
**Pass Criteria:** exit 0 + output includes ancestor-level sessions (broader than `scope::local`)
**Source:** [params.md](../../params.md)

### EC-3: Value "under" accepted

**Goal:** Verify that `scope::under` is accepted and `.sessions` returns sessions from all projects under the specified path.
**Setup:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture`
**Command:** `clg .sessions scope::under path::/tmp/test-fixture`
**Expected Output:** stdout lists sessions from all projects whose path is under the given base path.
**Verification:**
- Exit code is 0
- Output contains sessions from projects nested under the specified path
**Pass Criteria:** exit 0 + output includes sessions from descendant projects
**Source:** [params.md](../../params.md)

### EC-4: Value "global" accepted

**Goal:** Verify that `scope::global` is accepted and `.sessions` returns sessions from all projects in storage.
**Setup:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture`
**Command:** `clg .sessions scope::global`
**Expected Output:** stdout lists sessions from all projects in storage, regardless of path hierarchy.
**Verification:**
- Exit code is 0
- Output count is ≥ the count returned by `scope::local`
- Output includes sessions from projects unrelated to the current directory
**Pass Criteria:** exit 0 + output includes all sessions across all projects in storage
**Source:** [params.md](../../params.md)

### EC-5: Value "RELEVANT" accepted (case-insensitive)

**Goal:** Verify that enum parsing is case-insensitive and `scope::RELEVANT` is treated identically to `scope::relevant`.
**Setup:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture`
**Command:** `clg .sessions scope::RELEVANT`
**Expected Output:** No error; output is identical to using lowercase `scope::relevant`.
**Verification:**
- Exit code is 0
- Output matches the result of `clg .sessions scope::relevant`
**Pass Criteria:** exit 0 + output identical to lowercase variant (case normalization applied)
**Source:** [params.md](../../params.md)

### EC-6: Invalid value "all" rejected with error

**Goal:** Verify that `scope::all` is rejected with the exact error message `"scope must be relevant|local|under|global, got all"`.
**Setup:** None
**Command:** `clg .sessions scope::all`
**Expected Output:** stderr contains `scope must be relevant|local|under|global, got all`
**Verification:**
- Exit code is 1
- stderr contains the exact string `scope must be relevant|local|under|global, got all`
**Pass Criteria:** exit 1 + error message `scope must be relevant|local|under|global, got all`
**Source:** [params.md](../../params.md)

### EC-7: Omitted defaults to "under" scope (summary mode output)

**Goal:** Verify that omitting `scope::` defaults the session discovery scope to `under` — sessions from the entire subtree are candidates — while the output is **summary mode** (not a session list). The scope filter behaviour is `under`; the output format is distinct from `clg .sessions scope::under`, which uses an explicit parameter and always activates list mode.
**Setup:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture` with a parent project at `/tmp/test-fixture/parent` and a child project at `/tmp/test-fixture/parent/child`. The most-recent session is in the child project. Run from `/tmp/test-fixture/parent`.
**Command:** `clg .sessions` (run from `/tmp/test-fixture/parent`)
**Expected Output:**
```
Active session  {8-char-id}  Xs ago  N entries
Project  ~/test-fixture/parent/child

Last message:
  {last message text}
```
The project path in the summary belongs to the child project, confirming that `scope::under` is active (sub-project sessions are in scope). stdout does NOT contain `Found N sessions:`.
**Verification:**
- Exit code is 0
- stdout contains `Active session` (summary mode active)
- stdout does NOT contain `Found N sessions:` (list mode not triggered)
- The `Project` line path belongs to the sub-project (confirming under-scope, not just local)
**Pass Criteria:** exit 0 + summary header present + `Found N sessions:` absent + child project path visible
**Source:** [params.md](../../params.md)

### EC-8: scope::global ignores path::

**Goal:** Verify that `scope::global` includes all projects in storage even when `path::` is specified — the path is ignored for global scope.
**Setup:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture`
**Command:** `clg .sessions scope::global path::/tmp/nonexistent-subpath`
**Expected Output:** stdout lists sessions from all projects in storage; the `path::` value has no filtering effect with `scope::global`.
**Verification:**
- Exit code is 0
- Output is identical to `clg .sessions scope::global` without `path::`
- Output is broader than `clg .sessions scope::under path::/tmp/nonexistent-subpath` would be
**Pass Criteria:** exit 0 + output unaffected by path parameter when scope is global
**Source:** [params.md](../../params.md)
