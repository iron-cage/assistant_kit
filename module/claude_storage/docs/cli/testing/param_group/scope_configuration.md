# Parameter Group :: Scope Configuration

Interaction tests for the Scope Configuration group (`scope::`, `path::`). Tests verify scope × path interaction semantics for the `.sessions` command.

**Source:** [parameter_groups.md#scope-configuration](../../parameter_groups.md#scope-configuration)

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
**Command:** `clg .sessions scope::local path::/a/b/c`
**Expected Output:** Sessions from only the project at `/a/b/c`; no sessions from ancestor projects.
**Verification:**
- Only sessions from the `/a/b/c` project are returned
- Sessions from `/a/b` or `/a` are not included
- Result is scoped to exactly the directory specified by `path::`
**Pass Criteria:** exit 0 + only sessions from the path-anchored project
**Source:** [parameter_groups.md#scope-configuration](../../parameter_groups.md#scope-configuration)

### CC-2: scope::relevant starts ancestor walk from path::

**Goal:** Verify that `scope::relevant` walks up from `path::` and returns sessions from all ancestor projects.
**Setup:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture` with projects at `/a/b/c`, `/a/b`, and `/a`.
**Command:** `clg .sessions scope::relevant path::/a/b/c`
**Expected Output:** Sessions from all three projects: `/a/b/c`, `/a/b`, and `/a`.
**Verification:**
- Sessions from `/a/b/c` are included
- Sessions from `/a/b` are included
- Sessions from `/a` are included
- Sessions from unrelated paths are not included
**Pass Criteria:** exit 0 + sessions from all ancestor projects up to filesystem root
**Source:** [parameter_groups.md#scope-configuration](../../parameter_groups.md#scope-configuration)

### CC-3: scope::under searches subtree rooted at path::

**Goal:** Verify that `scope::under` returns sessions from all projects descending from `path::`.
**Setup:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture` with projects at `/a/b`, `/a/b/c`, `/a/b/c/d`, and `/z` (unrelated).
**Command:** `clg .sessions scope::under path::/a/b`
**Expected Output:** Sessions from `/a/b`, `/a/b/c`, and `/a/b/c/d`; not from `/z`.
**Verification:**
- Sessions from `/a/b` are included (root of subtree)
- Sessions from `/a/b/c` are included (child)
- Sessions from `/a/b/c/d` are included (grandchild)
- Sessions from `/z` are not included (outside subtree)
**Pass Criteria:** exit 0 + sessions from all projects in the subtree
**Source:** [parameter_groups.md#scope-configuration](../../parameter_groups.md#scope-configuration)

### CC-4: scope::global ignores path:: value

**Goal:** Verify that `scope::global` returns all sessions regardless of any `path::` value provided.
**Setup:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture` with projects at `/a/b`, `/c/d`, and `/e/f`.
**Command:** `clg .sessions scope::global path::/a/b`
**Expected Output:** Sessions from all projects (`/a/b`, `/c/d`, `/e/f`), not just `/a/b`.
**Verification:**
- Sessions from `/c/d` are present (unrelated to `path::`)
- Sessions from `/e/f` are present (unrelated to `path::`)
- Total session count matches the full fixture total
**Pass Criteria:** exit 0 + sessions from all projects in storage
**Source:** [parameter_groups.md#scope-configuration](../../parameter_groups.md#scope-configuration)

### CD-1: scope::under without path:: defaults to cwd

**Goal:** Verify that `scope::under` without an explicit `path::` uses the current working directory as the subtree root.
**Setup:** `cd /a/b && export CLAUDE_STORAGE_ROOT=/tmp/test-fixture` with projects at `/a/b`, `/a/b/c`, and `/z`.
**Command:** `clg .sessions scope::under` (no `path::` param)
**Expected Output:** Sessions from `/a/b` and `/a/b/c` only; not from `/z`.
**Verification:**
- Sessions from `/a/b` are included (cwd)
- Sessions from `/a/b/c` are included (child of cwd)
- Sessions from `/z` are not included (outside subtree)
**Pass Criteria:** exit 0 + subtree results anchored at cwd
**Source:** [parameter_groups.md#scope-configuration](../../parameter_groups.md#scope-configuration)

### CD-2: path:: without scope:: defaults to under scope

**Goal:** Verify that providing `path::` without `scope::` defaults to `scope::under` behavior.
**Setup:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture` with projects at `/a/b/c`, `/a/b/c/sub`, and `/z`.
**Command:** `clg .sessions path::/a/b/c` (no `scope::` param)
**Expected Output:** Sessions from `/a/b/c` and `/a/b/c/sub` (under scope default); no sessions from `/z`.
**Verification:**
- Sessions from `/a/b/c` are returned
- Sessions from `/a/b/c/sub` are returned (subtree child)
- Sessions from `/z` are not included (outside subtree)
- Behavior is identical to `clg .sessions scope::under path::/a/b/c`
**Pass Criteria:** exit 0 + sessions from path and all sub-paths included (under scope default)
**Source:** [parameter_groups.md#scope-configuration](../../parameter_groups.md#scope-configuration)
