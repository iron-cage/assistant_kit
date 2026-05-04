# Test: `v::` / `verbosity::` (verbosity)

Edge case coverage for the `v::` alias and `verbosity::` canonical key. See [params.md](../../../../docs/cli/params.md) and [types.md](../../../../docs/cli/types.md) for specification.

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| TC-097 | `.status v::0` → 3 bare lines | Explicit 0 |
| TC-098 | `.status v::1` → labeled lines | Explicit 1 |
| TC-108 | `.version.show v::0` → bare semver | Explicit 0 |
| TC-109 | `.version.show v::1` → "Version: X.Y.Z" | Explicit 1 |
| TC-118 | `.version.list v::0` → names only | Explicit 0 |
| TC-119 | `.version.list v::1` → names + descriptions | Explicit 1 |
| TC-164 | `.settings.show v::0` → `key=value` format | Explicit 0 |
| TC-179 | `.settings.get v::0` → bare value only | Explicit 0 |
| EC-1 | Last `v::` wins when duplicated | Duplication |
| TC-428 | `.version.history v::0` → bare version+date | Explicit 0 |
| TC-430 | `.version.history v::2` → full changelog | Explicit 2 |
| EC-1 | Default (absent) resolves to `v::1` | Default Behavior |
| EC-1 | `v::0` consistently minimal across all commands | Cross-Command |
| EC-1 | `v::3` → exit 1, out of range | Invalid: out-of-range |
| EC-1 | `v::-1` → exit 1, out of range | Invalid: negative |
| EC-1 | `v::abc` → exit 1, non-integer | Format Violation |
| EC-1 | `v::` (empty) → exit 1 | Empty Value |
| EC-1 | `v::` accepted by 10 commands, rejected by 2 | Command Scope |
| EC-2 | `verbosity::3` → exit 1 (canonical key, over-range) | Invalid: canonical over-range |
| EC-3 | `verbosity::-1` → exit 1 (canonical key, negative) | Invalid: canonical negative |
| EC-4 | `verbosity::0` accepted via canonical key → exit 0 | Valid: canonical form |

## Test Coverage Summary

- Explicit 0 (minimal output): 5 tests
- Explicit 1 (default/labeled): 2 tests
- Explicit 2 (extended detail): 1 test
- Duplication (last-wins): 1 test
- Default Behavior: 1 test
- Cross-Command: 1 test
- Invalid (out-of-range): 2 tests
- Format Violation: 1 test
- Empty Value: 1 test
- Command Scope: 1 test
- Canonical form (valid): 1 test
- Canonical form (invalid): 2 tests

**Total:** 20 edge cases

**Behavioral Divergence Pair:** EC-1 (valid/expected path) ↔ EC-1 (invalid/rejected path)

---

### EC-1: Last `v::` wins when duplicated

- **Given:** clean environment
- **When:** `cm .version.list v::0 v::1` (last is v::1)
- **Then:** Output shows descriptions (v::1 behavior, not v::0).; Last `v::` value applied
- **Exit:** 0
- **Source:** [feature/005_cli_design.md](../../../../docs/feature/005_cli_design.md)

---

### EC-1: Default (absent) → `v::1`

- **Given:** clean environment
- **When:** `cm .version.list` (no v:: param).
- **Then:** Behavior identical to `v::1` (names with descriptions).; Default equals explicit v::1
- **Exit:** 0
- **Source:** [params.md — v:: default: 1](../../../../docs/cli/params.md)

---

### EC-1: `v::0` consistently minimal across commands

- **Given:** clean environment
- **When:** 
- **Then:** All produce bare data without labels.; Consistent minimum-output behavior across commands
- **Exit:** 0
- **Source:** [types.md — verbosity levels](../../../../docs/cli/types.md)

---

### EC-1: `v::3` → exit 1

- **Given:** clean environment
- **When:** `cm .version.list v::3`
- **Then:** exit code 1.
- **Exit:** 1
- **Source:** [feature/005_cli_design.md](../../../../docs/feature/005_cli_design.md)

---

### EC-1: `v::-1` → exit 1

- **Given:** clean environment
- **When:** `cm .version.list v::-1`
- **Then:** exit code 1.
- **Exit:** 1
- **Source:** [feature/005_cli_design.md](../../../../docs/feature/005_cli_design.md)

---

### EC-1: `v::abc` → exit 1

- **Given:** clean environment
- **When:** `cm .version.list v::abc`
- **Then:** exit code 1.
- **Exit:** 1
- **Source:** [feature/005_cli_design.md](../../../../docs/feature/005_cli_design.md)

---

### EC-1: `v::` (empty) → exit 1

- **Given:** clean environment
- **When:** `cm .version.list v::`
- **Then:** exit code 1; error about v:: requiring a value.
- **Exit:** 1
- **Source:** [feature/005_cli_design.md](../../../../docs/feature/005_cli_design.md)

---

### EC-1: `v::` only for output-formatting commands

- **Given:** clean environment
- **When:** `cm .settings.set v::1`
- **Then:** exit code 1; unknown parameter.; **Note:** `.processes.kill` was added to the v:: scope in TSK-099; only `.settings.set` and `.help` (universal override) do not accept v:: as a named parameter
- **Exit:** 1
- **Source:** [feature/005_cli_design.md](../../../../docs/feature/005_cli_design.md)

---

### EC-2: `verbosity::3` → exit 1 (canonical key over-range)

- **Given:** clean environment
- **When:** `cm .version.list verbosity::3`
- **Then:** exit code 1; error mentions `verbosity::`.; no silent level clamping
- **Exit:** 1
- **Source:** [feature/005_cli_design.md](../../../../docs/feature/005_cli_design.md)

---

### EC-3: `verbosity::-1` → exit 1 (canonical key negative)

- **Given:** clean environment
- **When:** `cm .version.list verbosity::-1`
- **Then:** exit code 1.
- **Exit:** 1
- **Source:** [feature/005_cli_design.md](../../../../docs/feature/005_cli_design.md)

---

### EC-4: `verbosity::0` accepted via canonical key → exit 0

- **Given:** clean environment
- **When:** `cm .version.list verbosity::0`
- **Then:** exit code 0; output without "Version:" label (v::0 minimal format).; no "Version:" prefix in output
- **Exit:** 0
- **Source:** [feature/005_cli_design.md](../../../../docs/feature/005_cli_design.md)
