# Command :: `.show.project` (deprecated)

Integration tests for the `.show.project` command. Tests verify deprecated behavior matches `.show project::` and deprecation messaging.

**Source:** [commands.md#command--9-showproject](../../commands.md#command--9-showproject)

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| IT-1 | Default shows current project (same as .show) | Behavior |
| IT-2 | project:: shows named project | Behavior |
| IT-3 | Output matches equivalent .show project:: output | Deprecation |
| IT-4 | Deprecation warning emitted on stderr | Deprecation |
| IT-5 | project:: with UUID identifier | Behavior |
| IT-6 | project:: with path-encoded ID | Behavior |
| IT-7 | Exit code 2 on project not found | Exit Codes |
| IT-8 | verbosity:: works same as in .show | Output Format |

## Test Coverage Summary

- Behavior: 4 tests (IT-1, IT-2, IT-5, IT-6)
- Deprecation: 2 tests (IT-3, IT-4)
- Exit Codes: 1 test (IT-7)
- Output Format: 1 test (IT-8)

## Test Cases

### IT-1: Default shows current project (same as .show)

**Goal:** Verify `.show.project` with no arguments shows the session list for the cwd-resolved project, identical to `.show`.
**Setup:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture` (fixture includes a project matching test cwd); run from that cwd.
**Command:** `clg .show.project`
**Expected Output:** Session list for the cwd-resolved project — same content as `clg .show` would produce for the same cwd.
**Verification:**
- stdout contains session entries for the cwd-resolved project
- output matches `clg .show` output for the same cwd (apart from any deprecation warning on stderr)
- stderr contains a deprecation warning
**Pass Criteria:** exit 0 + session list matching `.show` output for cwd

**Source:** [commands.md](../../commands.md)

---

### IT-2: project:: shows named project

**Goal:** Verify `.show.project project::PROJECT` displays the session list for the named project.
**Setup:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture` (fixture: projects `alpha` and `beta`)
**Command:** `clg .show.project project::alpha`
**Expected Output:** Session list for project `alpha`; no sessions from `beta`; deprecation warning on stderr.
**Verification:**
- stdout contains session entries for project `alpha`
- stdout does not contain session entries for project `beta`
- stderr contains a deprecation warning
**Pass Criteria:** exit 0 + `alpha` session list shown

**Source:** [commands.md](../../commands.md)

---

### IT-3: Output matches equivalent .show project:: output

**Goal:** Verify `.show.project project::PROJECT` produces the same stdout as `clg .show project::PROJECT`.
**Setup:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture` (fixture: project `alpha` with 2 sessions)
**Command (deprecated):** `clg .show.project project::alpha`
**Command (replacement):** `clg .show project::alpha`
**Expected Output:** stdout of both commands is identical.
**Verification:**
- capture stdout of both commands into temp files
- `diff` of the two stdout files shows no differences
- stderr of `.show.project` contains deprecation warning; stderr of `.show` does not
**Pass Criteria:** exit 0 + stdout byte-identical between the two commands

**Source:** [commands.md](../../commands.md)

---

### IT-4: Deprecation warning emitted on stderr

**Goal:** Verify that invoking `.show.project` always emits a deprecation warning on stderr regardless of other parameters.
**Setup:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture`
**Command:** `clg .show.project`
**Expected Output:** stderr contains a deprecation message directing users to `.show` or `.show project::PATH`.
**Verification:**
- stderr is non-empty
- stderr contains the word "deprecated" or a migration hint (e.g., "use `.show`")
- deprecation message references the replacement command (`.show`)
- stdout contains normal project output (warning does not suppress results)
**Pass Criteria:** exit 0 + deprecation warning on stderr present

**Source:** [commands.md](../../commands.md)

---

### IT-5: project:: with UUID identifier

**Goal:** Verify `.show.project` accepts a UUID-format project identifier and resolves it correctly.
**Setup:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture` (fixture: UUID project `a1b2c3d4-0000-0000-0000-000000000001` with 1 session)
**Command:** `clg .show.project project::a1b2c3d4-0000-0000-0000-000000000001`
**Expected Output:** Session list for the UUID-identified project; deprecation warning on stderr.
**Verification:**
- stdout contains the session entry for the UUID project
- stderr contains a deprecation warning
- no error about invalid project identifier format
**Pass Criteria:** exit 0 + sessions for UUID project shown

**Source:** [commands.md](../../commands.md)

---

### IT-6: project:: with path-encoded ID

**Goal:** Verify `.show.project` accepts a path-encoded project identifier and resolves it correctly.
**Setup:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture` (fixture: project stored with path-encoded ID `-home-user1-pro-alpha`)
**Command:** `clg .show.project project::-home-user1-pro-alpha`
**Expected Output:** Session list for the path-encoded project `-home-user1-pro-alpha`; deprecation warning on stderr.
**Verification:**
- stdout contains session entries for the `-home-user1-pro-alpha` project
- stderr contains a deprecation warning
- no error about invalid project identifier format
**Pass Criteria:** exit 0 + sessions for path-encoded project shown

**Source:** [commands.md](../../commands.md)

---

### IT-7: Exit code 2 on project not found

**Goal:** Verify `.show.project` exits with code `2` when the specified project does not exist in storage.
**Setup:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture` (fixture with known projects; `00000000-0000-0000-0000-000000000000` does not exist)
**Command:** `clg .show.project project::00000000-0000-0000-0000-000000000000`
**Expected Output:** Error message on stderr indicating project not found; no project content on stdout.
**Verification:**
- `$?` is `2`
- stderr contains an error message indicating project not found
- stdout is empty
**Pass Criteria:** exit 2 + error message on stderr for nonexistent project

**Source:** [commands.md](../../commands.md)

---

### IT-8: verbosity:: works same as in .show

**Goal:** Verify the `verbosity::` parameter in `.show.project` behaves identically to `verbosity::` in `.show project::`.
**Setup:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture` (fixture: project `alpha` with sessions having known entry counts)
**Command:** `clg .show.project project::alpha verbosity::2`
**Expected Output:** More detailed output than default verbosity, matching `clg .show project::alpha verbosity::2` output.
**Verification:**
- stdout at `verbosity::2` is more detailed than at `verbosity::1` for the same project
- stdout matches the output of `clg .show project::alpha verbosity::2` (same detail level)
- stderr contains deprecation warning
**Pass Criteria:** exit 0 + verbosity::2 produces expanded output matching `.show` equivalent

**Source:** [commands.md](../../commands.md)
