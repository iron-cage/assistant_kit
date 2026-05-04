# Test: `.paths`

Integration test planning for the `.paths` command. See [commands.md](../../../../docs/cli/commands.md#command--8-paths) for specification.

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

- **Given:** `HOME` is set to a valid directory (e.g., `/home/testuser`). No special account or credential state required.
- **When:** `clp .paths`
- **Then:** Seven labeled lines on stdout covering: `credentials`, `credential_store`, `projects`, `stats`, `settings`, `session-env`, `sessions`. Exit 0.; all 7 labeled paths present in output
- **Exit:** 0
- **Source:** [commands.md — .paths](../../../../docs/cli/commands.md#command--8-paths)

---

### IT-2: JSON format returns object with all path keys

- **Given:** `HOME` set to `/home/testuser`.
- **When:** `clp .paths format::json`
- **Then:** Valid JSON object on stdout with keys for all 7 paths plus `base`, exit 0.; valid JSON object with all expected path keys
- **Exit:** 0
- **Source:** [commands.md — .paths](../../../../docs/cli/commands.md#command--8-paths)

---

### IT-3: Quiet verbosity shows bare base path only

- **Given:** `HOME` set to `/home/testuser`.
- **When:** `clp .paths v::0`
- **Then:** Single line on stdout: `/home/testuser/.claude`, exit 0.; output is single bare base path
- **Exit:** 0
- **Source:** [commands.md — .paths](../../../../docs/cli/commands.md#command--8-paths)

---

### IT-4: Verbose output shows paths with existence markers

- **Given:** `HOME` set to `/home/testuser`. Create `~/.claude/.credentials.json` and `~/.persistent/claude/credential/` so some paths exist and some do not.
- **When:** `clp .paths v::2`
- **Then:** Seven or more labeled lines on stdout, each with a path and an existence marker, exit 0.; all paths shown with accurate existence markers
- **Exit:** 0
- **Source:** [commands.md — .paths](../../../../docs/cli/commands.md#command--8-paths)

---

### IT-5: HOME not set exits 2

- **Given:** Unset the `HOME` environment variable before invocation.
- **When:** `env -u HOME clp .paths`
- **Then:** Error message on stderr indicating `HOME` is not set, exit 2.; stderr contains error about HOME not set
- **Exit:** 2
- **Source:** [commands.md — .paths](../../../../docs/cli/commands.md#command--8-paths)

---

### IT-6: All output paths are absolute

- **Given:** `HOME` set to `/home/testuser`.
- **When:** `clp .paths`
- **Then:** Seven labeled lines, each path starting with `/`, exit 0.; all paths are absolute
- **Exit:** 0
- **Source:** [commands.md — .paths](../../../../docs/cli/commands.md#command--8-paths)

---

### IT-7: Paths resolve from actual HOME value

- **Given:** Set `HOME` to a custom temporary directory (e.g., `/tmp/test_home_12345`). Create `$HOME/.claude/` directory.
- **When:** `HOME=/tmp/test_home_12345 clp .paths`
- **Then:** All paths rooted under `/tmp/test_home_12345/.claude/`, exit 0.; all paths rooted under the custom HOME value
- **Exit:** 0
- **Source:** [commands.md — .paths](../../../../docs/cli/commands.md#command--8-paths)

---

### IT-8: JSON format with quiet verbosity returns full JSON

- **Given:** `HOME` set to `/home/testuser`.
- **When:** `clp .paths format::json v::0`
- **Then:** Valid JSON object on stdout with all path keys (same as IT-2), exit 0.; full JSON object returned despite `v::0`
- **Exit:** 0
- **Source:** [commands.md — .paths](../../../../docs/cli/commands.md#command--8-paths)
