# Test: `VersionSpec`

Type validation cases for the `VersionSpec` type. See [03_version_spec.md](../../../../docs/cli/type/03_version_spec.md) for specification.

### Scope

- **Purpose**: Type validation tests for `VersionSpec` (named aliases + semver string).
- **Responsibility**: Valid alias and semver acceptance, format validation rejection, and case-sensitivity.
- **Used by:** `version::` parameter
- **In Scope**: Named alias recognition, semver format rules, rejection conditions.
- **Out of Scope**: Install behavior (→ `../command/`), parameter-level default (→ `../param/001_version.md`).

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| TC-1 | `version::stable` → accepted named alias | Valid: named alias |
| TC-2 | `version::month` → accepted named alias | Valid: named alias |
| TC-3 | `version::latest` → accepted named alias | Valid: named alias |
| TC-4 | `version::1.2.3` → accepted semver | Valid: semver |
| TC-5 | `version::0.0.0` → accepted zero-patch semver | Valid: semver boundary |
| TC-6 | `version::1.2` → exit 1, two-part rejected | Invalid: format (below range) |
| TC-7 | `version::1.2.3.4` → exit 1, four-part rejected | Invalid: format (above range) |
| TC-8 | `version::01.02.03` → exit 1, leading zeros | Invalid: format |
| TC-9 | `version::STABLE` → exit 1, wrong case | Invalid: case-sensitive |
| TC-10 | `version::x` → exit 1, unknown alias | Invalid: unknown |

## Test Coverage Summary

- Named aliases (stable, month, latest): 3 tests (TC-1, TC-2, TC-3)
- Valid semver: 2 tests (TC-4, TC-5)
- Invalid two-part: 1 test (TC-6)
- Invalid four-part: 1 test (TC-7)
- Invalid leading zeros: 1 test (TC-8)
- Invalid case: 1 test (TC-9)
- Invalid unknown: 1 test (TC-10)

**Total:** 10 type cases

**Behavioral Divergence Pair:** TC-1 (`version::stable dry::1` → output contains stable alias version string) ↔ TC-2 (`version::month dry::1` → output contains month alias version string). Both valid; resolved version value differs.

---

### TC-1: `version::stable` → accepted

- **Given:** clean environment
- **When:** `cm .version.install version::stable dry::1`
- **Then:** exit 0; dry-run output references stable alias; no error
- **Exit:** 0
- **Source:** [03_version_spec.md — Named Aliases: stable](../../../../docs/cli/type/03_version_spec.md)

---

### TC-2: `version::month` → accepted

- **Given:** clean environment
- **When:** `cm .version.install version::month dry::1`
- **Then:** exit 0; dry-run output references month alias pinned version; distinct from stable
- **Exit:** 0
- **Source:** [03_version_spec.md — Named Aliases: month](../../../../docs/cli/type/03_version_spec.md)

---

### TC-3: `version::latest` → accepted

- **Given:** clean environment
- **When:** `cm .version.install version::latest dry::1`
- **Then:** exit 0; latest alias accepted; resolves dynamically
- **Exit:** 0
- **Source:** [03_version_spec.md — Named Aliases: latest](../../../../docs/cli/type/03_version_spec.md)

---

### TC-4: `version::1.2.3` → accepted semver

- **Given:** clean environment
- **When:** `cm .version.install version::1.2.3 dry::1`
- **Then:** exit 0; three-part semver accepted; no format error
- **Exit:** 0
- **Source:** [03_version_spec.md — Valid values: semver like '1.2.3'](../../../../docs/cli/type/03_version_spec.md)

---

### TC-5: `version::0.0.0` → accepted zero-patch semver

- **Given:** clean environment
- **When:** `cm .version.install version::0.0.0 dry::1`
- **Then:** exit 0; all-zero semver is structurally valid
- **Exit:** 0
- **Source:** [03_version_spec.md — semver validated by dot-count and digit check](../../../../docs/cli/type/03_version_spec.md)

---

### TC-6: `version::1.2` → exit 1, two-part rejected

- **Given:** clean environment
- **When:** `cm .version.install version::1.2`
- **Then:** exit 1; two-part version string rejected; error references version format
- **Exit:** 1
- **Source:** [03_version_spec.md — validation errors](../../../../docs/cli/type/03_version_spec.md)

---

### TC-7: `version::1.2.3.4` → exit 1, four-part rejected

- **Given:** clean environment
- **When:** `cm .version.install version::1.2.3.4`
- **Then:** exit 1; four-part semver rejected; only three-part accepted
- **Exit:** 1
- **Source:** [03_version_spec.md — validation: rejects 4-part semver](../../../../docs/cli/type/03_version_spec.md)

---

### TC-8: `version::01.02.03` → exit 1, leading zeros

- **Given:** clean environment
- **When:** `cm .version.install version::01.02.03`
- **Then:** exit 1; leading zeros in semver rejected
- **Exit:** 1
- **Source:** [03_version_spec.md — validation: rejects leading zeros](../../../../docs/cli/type/03_version_spec.md)

---

### TC-9: `version::STABLE` → exit 1, wrong case

- **Given:** clean environment
- **When:** `cm .version.install version::STABLE`
- **Then:** exit 1; alias names are case-sensitive; STABLE is not a recognized alias
- **Exit:** 1
- **Source:** [03_version_spec.md — validation: checked against named alias list](../../../../docs/cli/type/03_version_spec.md)

---

### TC-10: `version::x` → exit 1, unknown alias

- **Given:** clean environment
- **When:** `cm .version.install version::x`
- **Then:** exit 1; single-character string is neither a named alias nor valid semver
- **Exit:** 1
- **Source:** [03_version_spec.md — validation errors: unknown version](../../../../docs/cli/type/03_version_spec.md)

---

### Source Functions

| Function | File |
|----------|------|
| `tc304_version_install_wrong_case_exits_1` | `integration/mutation_commands_test.rs` |
| `tc305_version_install_empty_version_exits_1` | `integration/mutation_commands_test.rs` |
| `tc306_version_install_two_part_semver_exits_1` | `integration/mutation_commands_test.rs` |
| `tc307_version_install_unknown_alias_exits_1` | `integration/mutation_commands_test.rs` |
| `tc354_version_install_leading_zeros_exits_1` | `integration/mutation_commands_test.rs` |
| `tc355_version_install_zero_parts_valid_dry` | `integration/mutation_commands_test.rs` |
| `tc028_four_part_semver_rejected` | `cli_args_test.rs` |
| `tc029_leading_zero_semver_rejected` | `cli_args_test.rs` |
| ⏳ `tc_version_spec_month_alias_accepted` | `cli_args_test.rs` |
| ⏳ `tc_version_spec_latest_alias_accepted` | `cli_args_test.rs` |
