# Test: `--version` / `-V`

Integration test planning for the `--version` and `-V` meta-flags. See [`src/lib.rs`](../../../../src/lib.rs) for implementation.

### Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| IT-1 | `--version` prints binary name and version | Content |
| IT-2 | `-V` alias behaves identically to `--version` | Alias |
| IT-3 | `--version` exits 0 | Exit Code |
| IT-4 | `--version` output goes to stdout only | Output Stream |
| IT-5 | `--version` output is stable across repeated invocations | Stability |
| IT-6 | `--version` with trailing unknown flags still prints version | Robustness |
| IT-7 | `--version` output is exactly one non-empty line | Output Format |
| IT-8 | Version string matches `clp X.Y.Z` semver format exactly | Output Format |

### Test Coverage Summary

- Content: 1 test
- Alias: 1 test
- Exit Code: 1 test
- Output Stream: 1 test
- Stability: 1 test
- Robustness: 1 test
- Output Format: 2 tests

**Total:** 8 integration tests

---

### IT-1: `--version` prints binary name and version

- **Given:** clean environment, `clp` on PATH
- **When:** `clp --version`
- **Then:** stdout contains one non-empty line matching `clp \d+\.\d+\.\d+`
- **Exit:** 0
- **Source:** [commands.md — --version](../../../../docs/cli/commands.md#meta-flag----version---v)

---

### IT-2: `-V` alias behaves identically to `--version`

- **Given:** clean environment, `clp` on PATH
- **When:** `clp -V`
- **Then:** stdout is byte-identical to output of `clp --version`
- **Exit:** 0
- **Source:** [commands.md — --version](../../../../docs/cli/commands.md#meta-flag----version---v)

---

### IT-3: `--version` exits 0

- **Given:** clean environment, `clp` on PATH
- **When:** `clp --version`
- **Then:** process exits with code 0
- **Exit:** 0
- **Source:** [commands.md — --version](../../../../docs/cli/commands.md#meta-flag----version---v)

---

### IT-4: `--version` output goes to stdout only

- **Given:** clean environment, `clp` on PATH
- **When:** `clp --version`
- **Then:** stdout is non-empty; stderr is empty
- **Exit:** 0
- **Source:** [commands.md — --version](../../../../docs/cli/commands.md#meta-flag----version---v)

---

### IT-5: `--version` output is stable across repeated invocations

- **Given:** clean environment, `clp` on PATH
- **When:** `clp --version` (run 3 times)
- **Then:** all 3 stdout captures are byte-identical
- **Exit:** 0
- **Source:** [commands.md — --version](../../../../docs/cli/commands.md#meta-flag----version---v)

---

### IT-6: `--version` with trailing unknown flags still prints version

- **Given:** clean environment, `clp` on PATH
- **When:** `clp --version --unknown-flag`
- **Then:** stdout contains the version string; process does not error out
- **Exit:** 0
- **Source:** [commands.md — --version](../../../../docs/cli/commands.md#meta-flag----version---v)

---

### IT-7: `--version` output is exactly one non-empty line

- **Given:** clean environment, `clp` on PATH
- **When:** `clp --version`
- **Then:** stdout contains exactly one line and that line is non-empty; no blank lines before or after
- **Exit:** 0
- **Source:** [commands.md — --version](../../../../docs/cli/commands.md#meta-flag----version---v)

---

### IT-8: Version string matches `clp X.Y.Z` semver format

- **Given:** clean environment, `clp` on PATH
- **When:** `clp --version`
- **Then:** stdout matches the regex `^clp \d+\.\d+\.\d+$`
- **Exit:** 0
- **Source:** [commands.md — --version](../../../../docs/cli/commands.md#meta-flag----version---v)
