# Test: `v::` / `verbosity::` (verbosity)

Edge case coverage for the `v::` alias and `verbosity::` canonical key. See [005_params.md](../../../../docs/cli/param/readme.md) and [006_types.md](../../../../docs/cli/type/readme.md) for specification.

### Scope

- **Purpose**: Edge case tests for the `v::` / `verbosity::` parameter.
- **Responsibility**: Boundary values, invalid inputs, type violations, and default behavior for `v::`.
- **Commands:** `.status`, `.version.show`, `.version.install`, `.version.list`, `.version.guard`, `.version.history`, `.processes`, `.processes.kill`, `.settings.show`, `.settings.get`
- **In Scope**: Single-parameter edge cases, validation errors, alias resolution, type checking.
- **Out of Scope**: Command integration (→ `../command/`), group interactions (→ `../param_group/`).

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| EC-12 | `.status v::0` → 3 bare lines | Explicit 0 |
| EC-13 | `.status v::1` → labeled lines | Explicit 1 |
| EC-14 | `.version.show v::0` → bare semver | Explicit 0 |
| EC-15 | `.version.show v::1` → "Version: X.Y.Z" | Explicit 1 |
| EC-16 | `.version.list v::0` → names only | Explicit 0 |
| EC-17 | `.version.list v::1` → names + descriptions | Explicit 1 |
| EC-18 | `.settings.show v::0` → `key=value` format | Explicit 0 |
| EC-19 | `.settings.get v::0` → bare value only | Explicit 0 |
| EC-1 | Last `v::` wins when duplicated | Duplication |
| EC-20 | `.version.history v::0` → bare version+date | Explicit 0 |
| EC-21 | `.version.history v::2` → full changelog | Explicit 2 |
| EC-5 | Default (absent) resolves to `v::1` | Default Behavior |
| EC-6 | `v::0` consistently minimal across all commands | Cross-Command |
| EC-7 | `v::3` → exit 1, out of range | Invalid: out-of-range |
| EC-8 | `v::-1` → exit 1, out of range | Invalid: negative |
| EC-9 | `v::abc` → exit 1, non-integer | Format Violation |
| EC-10 | `v::` (empty) → exit 1 | Empty Value |
| EC-11 | `v::` accepted by 10 commands, rejected by 2 | Command Scope |
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

**Behavioral Divergence Pair:** EC-12 (`.status v::0` → 3 bare lines, exit 0) ↔ EC-13 (`.status v::1` → labeled lines, exit 0)

---

### EC-1: Last `v::` wins when duplicated

- **Given:** clean environment
- **When:** `clv .version.list v::0 v::1` (last is v::1)
- **Then:** Output shows descriptions (v::1 behavior, not v::0).; Last `v::` value applied
- **Exit:** 0
- **Source:** [feature/005_cli_design.md](../../../../docs/feature/005_cli_design.md)

---

### EC-5: Default (absent) → `v::1`

- **Given:** clean environment
- **When:** `clv .version.list` (no v:: param).
- **Then:** Behavior identical to `v::1` (names with descriptions).; Default equals explicit v::1
- **Exit:** 0
- **Source:** [005_params.md — v:: default: 1](../../../../docs/cli/param/readme.md)

---

### EC-6: `v::0` consistently minimal across commands

- **Given:** clean environment
- **When:** `clv .status v::0`, `clv .version.list v::0`, `clv .version.history v::0`
- **Then:** each command produces bare data without label prefixes; no "Version:", "Status:", or other label strings in output
- **Exit:** 0
- **Source:** [006_types.md — verbosity levels](../../../../docs/cli/type/readme.md)

---

### EC-7: `v::3` → exit 1

- **Given:** clean environment
- **When:** `clv .version.list v::3`
- **Then:** exit code 1.
- **Exit:** 1
- **Source:** [feature/005_cli_design.md](../../../../docs/feature/005_cli_design.md)

---

### EC-8: `v::-1` → exit 1

- **Given:** clean environment
- **When:** `clv .version.list v::-1`
- **Then:** exit code 1.
- **Exit:** 1
- **Source:** [feature/005_cli_design.md](../../../../docs/feature/005_cli_design.md)

---

### EC-9: `v::abc` → exit 1

- **Given:** clean environment
- **When:** `clv .version.list v::abc`
- **Then:** exit code 1.
- **Exit:** 1
- **Source:** [feature/005_cli_design.md](../../../../docs/feature/005_cli_design.md)

---

### EC-10: `v::` (empty) → exit 1

- **Given:** clean environment
- **When:** `clv .version.list v::`
- **Then:** exit code 1; error about v:: requiring a value.
- **Exit:** 1
- **Source:** [feature/005_cli_design.md](../../../../docs/feature/005_cli_design.md)

---

### EC-11: `v::` only for output-formatting commands

- **Given:** clean environment
- **When:** `clv .settings.set v::1`
- **Then:** exit code 1; unknown parameter.; **Note:** `.processes.kill` was added to the v:: scope in TSK-099; only `.settings.set` and `.help` (universal override) do not accept v:: as a named parameter
- **Exit:** 1
- **Source:** [feature/005_cli_design.md](../../../../docs/feature/005_cli_design.md)

---

### EC-2: `verbosity::3` → exit 1 (canonical key over-range)

- **Given:** clean environment
- **When:** `clv .version.list verbosity::3`
- **Then:** exit code 1; error mentions `verbosity::`.; no silent level clamping
- **Exit:** 1
- **Source:** [feature/005_cli_design.md](../../../../docs/feature/005_cli_design.md)

---

### EC-3: `verbosity::-1` → exit 1 (canonical key negative)

- **Given:** clean environment
- **When:** `clv .version.list verbosity::-1`
- **Then:** exit code 1.
- **Exit:** 1
- **Source:** [feature/005_cli_design.md](../../../../docs/feature/005_cli_design.md)

---

### EC-4: `verbosity::0` accepted via canonical key → exit 0

- **Given:** clean environment
- **When:** `clv .version.list verbosity::0`
- **Then:** exit code 0; output without "Version:" label (v::0 minimal format).; no "Version:" prefix in output
- **Exit:** 0
- **Source:** [feature/005_cli_design.md](../../../../docs/feature/005_cli_design.md)

---

### Source Functions

| Function | File |
|----------|------|
| `tc005_verbosity_empty_value` | `cli_args_test.rs` |
| `tc006_verbosity_out_of_range` | `cli_args_test.rs` |
| `tc007_verbosity_non_integer` | `cli_args_test.rs` |
| `tc008_verbosity_0_accepted` | `cli_args_test.rs` |
| `tc010_last_verbosity_wins` | `cli_args_test.rs` |
| `tc022_v_param_consistent` | `cli_args_test.rs` |
| `tc484_verbosity_canonical_out_of_range_rejected` | `cli_args_test.rs` |
| `tc485_verbosity_canonical_negative_rejected` | `cli_args_test.rs` |
| `tc486_verbosity_canonical_zero_accepted` | `cli_args_test.rs` |
| `tc245_last_occurrence_wins_for_verbosity` | `integration/read_commands_test.rs` |
| `tc503_verbosity_out_of_range_error_message` | `integration/error_messages_test.rs` |
