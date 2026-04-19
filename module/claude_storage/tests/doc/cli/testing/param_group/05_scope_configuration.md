# Parameter Group :: Scope Configuration

Interaction tests for the Scope Configuration group (`scope::`, `path::`). Tests verify scope × path interaction semantics for the `.projects` command.

**Source:** [parameter_groups.md#scope-configuration](../../../../../docs/cli/parameter_groups.md#scope-configuration)

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| CC-1 | scope::local uses path:: as directory anchor | Scope × Path |
| CC-2 | scope::relevant starts ancestor walk from path:: | Scope × Path |
| CC-3 | scope::under searches subtree rooted at path:: | Scope × Path |
| CC-4 | scope::global ignores path:: value | Scope × Path |
| CD-1 | scope::under without path:: defaults to cwd | Default Behavior |
| CD-2 | path:: without scope:: defaults to under scope | Default Behavior |

## Test Coverage Summary

- Scope × Path: 4 tests (CC-1, CC-2, CC-3, CC-4)
- Default Behavior: 2 tests (CD-1, CD-2)

## Test Cases

### CC-1: scope::local uses path:: as directory anchor

**Goal:** Verify that `scope::local` uses `path::` as the anchor directory for project lookup rather than cwd.
**Setup:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture` with projects at `/a/b/c`, `/a/b`, and `/a`. Run from `/a/x` (no project there).
**Command:** `clg .projects scope::local path::/a/b/c`
**Expected Output:** Only the project at `/a/b/c` appears in the project list; ancestor projects `/a/b` and `/a` are absent.
**Verification:**
- Only the `/a/b/c` project row appears in output
- Projects `/a/b` and `/a` do not appear
- Result is scoped to exactly the directory specified by `path::`
**Pass Criteria:** exit 0 + only the path-anchored project in output
**Source:** [parameter_groups.md#scope-configuration](../../../../../docs/cli/parameter_groups.md#scope-configuration)

### CC-2: scope::relevant starts ancestor walk from path::

**Goal:** Verify that `scope::relevant` walks up from `path::` and returns sessions from all ancestor projects.
**Setup:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture` with projects at `/a/b/c`, `/a/b`, and `/a`.
**Command:** `clg .projects scope::relevant path::/a/b/c`
**Expected Output:** All three projects appear in output: `/a/b/c`, `/a/b`, and `/a`.
**Verification:**
- Project `/a/b/c` appears in output
- Project `/a/b` appears in output
- Project `/a` appears in output
- Projects at unrelated paths are not included
**Pass Criteria:** exit 0 + all ancestor projects up to filesystem root in output
**Source:** [parameter_groups.md#scope-configuration](../../../../../docs/cli/parameter_groups.md#scope-configuration)

### CC-3: scope::under searches subtree rooted at path::

**Goal:** Verify that `scope::under` returns sessions from all projects descending from `path::`.
**Setup:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture` with projects at `/a/b`, `/a/b/c`, `/a/b/c/d`, and `/z` (unrelated).
**Command:** `clg .projects scope::under path::/a/b`
**Expected Output:** Projects `/a/b`, `/a/b/c`, and `/a/b/c/d` appear in output; project `/z` is absent.
**Verification:**
- Project `/a/b` appears in output (root of subtree)
- Project `/a/b/c` appears in output (child)
- Project `/a/b/c/d` appears in output (grandchild)
- Project `/z` does not appear (outside subtree)
**Pass Criteria:** exit 0 + all projects in the subtree in output
**Source:** [parameter_groups.md#scope-configuration](../../../../../docs/cli/parameter_groups.md#scope-configuration)

### CC-4: scope::global ignores path:: value

**Goal:** Verify that `scope::global` returns all sessions regardless of any `path::` value provided.
**Setup:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture` with projects at `/a/b`, `/c/d`, and `/e/f`.
**Command:** `clg .projects scope::global path::/a/b`
**Expected Output:** All projects appear (`/a/b`, `/c/d`, `/e/f`), not just `/a/b`.
**Verification:**
- Project `/c/d` is present in output (unrelated to `path::`)
- Project `/e/f` is present in output (unrelated to `path::`)
- Total project count matches the full fixture total
**Pass Criteria:** exit 0 + all projects in storage in output
**Source:** [parameter_groups.md#scope-configuration](../../../../../docs/cli/parameter_groups.md#scope-configuration)

### CD-1: scope::under without path:: defaults to cwd

**Goal:** Verify that `scope::under` without an explicit `path::` uses the current working directory as the subtree root.
**Setup:** `cd /a/b && export CLAUDE_STORAGE_ROOT=/tmp/test-fixture` with projects at `/a/b`, `/a/b/c`, and `/z`.
**Command:** `clg .projects scope::under` (no `path::` param)
**Expected Output:** Only projects `/a/b` and `/a/b/c` appear; project `/z` is absent.
**Verification:**
- Project `/a/b` appears in output (cwd)
- Project `/a/b/c` appears in output (child of cwd)
- Project `/z` does not appear (outside subtree)
**Pass Criteria:** exit 0 + subtree projects anchored at cwd in output
**Source:** [parameter_groups.md#scope-configuration](../../../../../docs/cli/parameter_groups.md#scope-configuration)

### CD-2: path:: without scope:: defaults to under scope

**Goal:** Verify that providing `path::` without `scope::` defaults to `scope::under` behavior.
**Setup:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture` with projects at `/a/b/c`, `/a/b/c/sub`, and `/z`.
**Command:** `clg .projects path::/a/b/c` (no `scope::` param)
**Expected Output:** Projects `/a/b/c` and `/a/b/c/sub` appear in output (under scope default); project `/z` is absent.
**Verification:**
- Project `/a/b/c` appears in output
- Project `/a/b/c/sub` appears in output (subtree child)
- Project `/z` does not appear (outside subtree)
- Behavior is identical to `clg .projects scope::under path::/a/b/c` (explicit scope)
**Pass Criteria:** exit 0 + projects at path and all sub-paths in output (under scope default)
**Source:** [parameter_groups.md#scope-configuration](../../../../../docs/cli/parameter_groups.md#scope-configuration)
