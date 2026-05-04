# Parameter Group :: Scope Configuration

Interaction tests for the Scope Configuration group (`scope::`, `path::`). Tests verify scope × path interaction semantics for the `.projects` command.

**Source:** [parameter_groups.md#scope-configuration](../../../../docs/cli/parameter_groups.md#scope-configuration)

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| EC-1 | scope::local uses path:: as directory anchor | Scope × Path |
| EC-2 | scope::relevant starts ancestor walk from path:: | Scope × Path |
| EC-3 | scope::under searches subtree rooted at path:: | Scope × Path |
| EC-4 | scope::global ignores path:: value | Scope × Path |
| EC-5 | scope::under without path:: defaults to cwd | Default Behavior |
| EC-6 | path:: without scope:: defaults to under scope | Default Behavior |

## Test Coverage Summary

- Scope × Path: 4 tests (EC-1, EC-2, EC-3, EC-4)
- Default Behavior: 2 tests (EC-5, EC-6)

## Test Cases

---

### EC-1: scope::local uses path:: as directory anchor

- **Given:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture` with projects at `/a/b/c`, `/a/b`, and `/a`. Run from `/a/x` (no project there).
- **When:** `clg .projects scope::local path::/a/b/c`
- **Then:** Only the project at `/a/b/c` appears in the project list; ancestor projects `/a/b` and `/a` are absent.; only the path-anchored project in output
- **Exit:** 0
- **Source:** [parameter_groups.md#scope-configuration](../../../../docs/cli/parameter_groups.md#scope-configuration)

---

### EC-2: scope::relevant starts ancestor walk from path::

- **Given:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture` with projects at `/a/b/c`, `/a/b`, and `/a`.
- **When:** `clg .projects scope::relevant path::/a/b/c`
- **Then:** All three projects appear in output: `/a/b/c`, `/a/b`, and `/a`.; all ancestor projects up to filesystem root in output
- **Exit:** 0
- **Source:** [parameter_groups.md#scope-configuration](../../../../docs/cli/parameter_groups.md#scope-configuration)

---

### EC-3: scope::under searches subtree rooted at path::

- **Given:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture` with projects at `/a/b`, `/a/b/c`, `/a/b/c/d`, and `/z` (unrelated).
- **When:** `clg .projects scope::under path::/a/b`
- **Then:** Projects `/a/b`, `/a/b/c`, and `/a/b/c/d` appear in output; project `/z` is absent.; all projects in the subtree in output
- **Exit:** 0
- **Source:** [parameter_groups.md#scope-configuration](../../../../docs/cli/parameter_groups.md#scope-configuration)

---

### EC-4: scope::global ignores path:: value

- **Given:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture` with projects at `/a/b`, `/c/d`, and `/e/f`.
- **When:** `clg .projects scope::global path::/a/b`
- **Then:** All projects appear (`/a/b`, `/c/d`, `/e/f`), not just `/a/b`.; all projects in storage in output
- **Exit:** 0
- **Source:** [parameter_groups.md#scope-configuration](../../../../docs/cli/parameter_groups.md#scope-configuration)

---

### EC-5: scope::under without path:: defaults to cwd

- **Given:** `cd /a/b && export CLAUDE_STORAGE_ROOT=/tmp/test-fixture` with projects at `/a/b`, `/a/b/c`, and `/z`.
- **When:** `clg .projects scope::under` (no `path::` param)
- **Then:** Only projects `/a/b` and `/a/b/c` appear; project `/z` is absent.; subtree projects anchored at cwd in output
- **Exit:** 0
- **Source:** [parameter_groups.md#scope-configuration](../../../../docs/cli/parameter_groups.md#scope-configuration)

---

### EC-6: path:: without scope:: defaults to under scope

- **Given:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture` with projects at `/a/b/c`, `/a/b/c/sub`, and `/z`.
- **When:** `clg .projects path::/a/b/c` (no `scope::` param)
- **Then:** Projects `/a/b/c` and `/a/b/c/sub` appear in output (under scope default); project `/z` is absent.; projects at path and all sub-paths in output (under scope default)
- **Exit:** 0
- **Source:** [parameter_groups.md#scope-configuration](../../../../docs/cli/parameter_groups.md#scope-configuration)
