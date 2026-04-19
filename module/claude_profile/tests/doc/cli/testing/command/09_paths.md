# Test: `.paths`

Integration test planning for the `.paths` command. See [commands.md](../../../../../docs/cli/commands.md#command--8-paths) for specification.

### Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| IT-1 | Shows all 7 canonical paths with labels | Basic Invocation |
| IT-2 | `format::json` returns JSON object with all path keys | Output Format |
| IT-3 | `v::0` shows bare base path only | Verbosity |
| IT-4 | `v::2` shows all paths with labels and existence markers | Verbosity |
| IT-5 | HOME not set exits 2 | Error Handling |
| IT-6 | All output paths are absolute | Path Correctness |
| IT-7 | Paths resolve from actual HOME value | Path Correctness |
| IT-8 | `format::json` with `v::0` returns full JSON (format overrides verbosity) | Interaction |

### Test Coverage Summary

- Basic Invocation: 1 test
- Output Format: 1 test
- Verbosity: 2 tests
- Error Handling: 1 test
- Path Correctness: 2 tests
- Interaction: 1 test

**Total:** 8 integration tests

---

### IT-1: Shows all 7 canonical paths with labels

**Goal:** Verify that the default invocation lists all 7 canonical paths, each with its label.
**Setup:** `HOME` is set to a valid directory (e.g., `/home/testuser`). No special account or credential state required.
**Command:** `clp .paths`
**Expected Output:** Seven labeled lines on stdout covering: `credentials`, `accounts`, `projects`, `stats`, `settings`, `session-env`, `sessions`. Exit 0.
**Verification:**
- Stdout contains exactly 7 non-empty lines (one per canonical path)
- Each line contains a label followed by a colon and a path
- Labels present: `credentials`, `accounts`, `projects`, `stats`, `settings`, `session-env`, `sessions`
- Exit code is 0
**Pass Criteria:** Exit 0; all 7 labeled paths present in output.
**Source:** [commands.md — .paths](../../../../../docs/cli/commands.md#command--8-paths)

---

### IT-2: JSON format returns object with all path keys

**Goal:** Verify that `format::json` produces a valid JSON object containing all canonical path keys.
**Setup:** `HOME` set to `/home/testuser`.
**Command:** `clp .paths format::json`
**Expected Output:** Valid JSON object on stdout with keys for all 7 paths plus `base`, exit 0.
**Verification:**
- Stdout is valid JSON (parseable without error)
- Parsed JSON is an object (not an array)
- JSON contains key `base` with value ending in `/.claude`
- JSON contains keys: `credentials`, `accounts`, `projects`, `stats`, `settings`, `session_env` (or `session-env`), `sessions`
- All values are non-empty strings
- Exit code is 0
**Pass Criteria:** Exit 0; valid JSON object with all expected path keys.
**Source:** [commands.md — .paths](../../../../../docs/cli/commands.md#command--8-paths)

---

### IT-3: Quiet verbosity shows bare base path only

**Goal:** Verify that `v::0` reduces output to just the base `~/.claude` path with no labels or additional paths.
**Setup:** `HOME` set to `/home/testuser`.
**Command:** `clp .paths v::0`
**Expected Output:** Single line on stdout: `/home/testuser/.claude`, exit 0.
**Verification:**
- Stdout contains exactly 1 non-empty line
- That line is the base path (e.g., `/home/testuser/.claude`)
- No labels, no colons, no additional paths
- Exit code is 0
**Pass Criteria:** Exit 0; output is single bare base path.
**Source:** [commands.md — .paths](../../../../../docs/cli/commands.md#command--8-paths)

---

### IT-4: Verbose output shows paths with existence markers

**Goal:** Verify that `v::2` includes all paths with labels and an existence indicator for each path (e.g., whether the file/directory exists on disk).
**Setup:** `HOME` set to `/home/testuser`. Create `~/.claude/.credentials.json` and `~/.claude/accounts/` so some paths exist and some do not.
**Command:** `clp .paths v::2`
**Expected Output:** Seven or more labeled lines on stdout, each with a path and an existence marker, exit 0.
**Verification:**
- Stdout contains all 7 canonical path labels
- Each line includes an existence indicator (e.g., a checkmark, `[exists]`, `[missing]`, or similar marker)
- Paths that exist on disk are marked as existing
- Paths that do not exist are marked as missing
- Exit code is 0
**Pass Criteria:** Exit 0; all paths shown with accurate existence markers.
**Source:** [commands.md — .paths](../../../../../docs/cli/commands.md#command--8-paths)

---

### IT-5: HOME not set exits 2

**Goal:** Verify that when the `HOME` environment variable is unset, the command exits 2 with an error.
**Setup:** Unset the `HOME` environment variable before invocation.
**Command:** `env -u HOME clp .paths`
**Expected Output:** Error message on stderr indicating `HOME` is not set, exit 2.
**Verification:**
- Exit code is 2
- Stderr contains an error message (not empty)
- No path output on stdout
**Pass Criteria:** Exit 2; stderr contains error about HOME not set.
**Source:** [commands.md — .paths](../../../../../docs/cli/commands.md#command--8-paths)

---

### IT-6: All output paths are absolute

**Goal:** Verify that every path in the output starts with `/` (is absolute, not relative).
**Setup:** `HOME` set to `/home/testuser`.
**Command:** `clp .paths`
**Expected Output:** Seven labeled lines, each path starting with `/`, exit 0.
**Verification:**
- Extract all path values from stdout
- Every extracted path starts with `/`
- No path is relative (no path starts with `~`, `.`, or a bare directory name)
- Exit code is 0
**Pass Criteria:** Exit 0; all paths are absolute.
**Source:** [commands.md — .paths](../../../../../docs/cli/commands.md#command--8-paths)

---

### IT-7: Paths resolve from actual HOME value

**Goal:** Verify that paths are derived from the current `HOME` environment variable, not hardcoded.
**Setup:** Set `HOME` to a custom temporary directory (e.g., `/tmp/test_home_12345`). Create `$HOME/.claude/` directory.
**Command:** `HOME=/tmp/test_home_12345 clp .paths`
**Expected Output:** All paths rooted under `/tmp/test_home_12345/.claude/`, exit 0.
**Verification:**
- Every path in stdout starts with `/tmp/test_home_12345/.claude`
- No path contains the real user home directory
- Exit code is 0
**Pass Criteria:** Exit 0; all paths rooted under the custom HOME value.
**Source:** [commands.md — .paths](../../../../../docs/cli/commands.md#command--8-paths)

---

### IT-8: JSON format with quiet verbosity returns full JSON

**Goal:** Verify that when both `format::json` and `v::0` are specified, `format::` takes precedence and the full JSON object is returned (not reduced to a bare path).
**Setup:** `HOME` set to `/home/testuser`.
**Command:** `clp .paths format::json v::0`
**Expected Output:** Valid JSON object on stdout with all path keys (same as IT-2), exit 0.
**Verification:**
- Stdout is valid JSON (parseable without error)
- Parsed JSON contains `base` key and all 7 canonical path keys
- Output is not a bare string path (format overrides verbosity reduction)
- Exit code is 0
**Pass Criteria:** Exit 0; full JSON object returned despite `v::0`.
**Source:** [commands.md — .paths](../../../../../docs/cli/commands.md#command--8-paths)
