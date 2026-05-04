# Parameter Group :: Claude-Native Flags

Interaction tests for Group 1 (Claude-Native Flags): `--print`, `--model`, `--verbose`, `--effort`. Tests validate these flags coexist correctly and are forwarded to the claude subprocess.

**Source:** [parameter_groups.md#group--1-claude-native-flags](../../../../docs/cli/parameter_groups.md#group--1-claude-native-flags)

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| EC-1 | All claude-native flags forwarded together | Combined |
| EC-2 | `--model` + `--verbose` coexist without conflict | Interaction |
| EC-3 | `--verbose` + `--effort max` → both in assembled command | Interaction |
| EC-4 | None of the group flags set → only defaults injected | Default |

## Test Coverage Summary

- Combined: 1 test (EC-1)
- Interaction: 2 tests (EC-2, EC-3)
- Default: 1 test (EC-4)

**Total:** 4 edge cases

## Test Cases
---

### EC-1: All claude-native flags forwarded together:

- **Given:** clean environment
- **When:** `clr --dry-run --model sonnet --verbose --effort high "Fix bug"`
- **Then:** Assembled command contains `--model sonnet`, `--verbose`, and `--effort high`
- **Exit:** 0
- **Source:** [parameter_groups.md](../../../../docs/cli/parameter_groups.md#group--1-claude-native-flags)
---

### EC-2: `--model` + `--verbose` coexist:

- **Given:** clean environment
- **When:** `clr --dry-run --model opus --verbose "Fix bug"`
- **Then:** Assembled command contains both `--model opus` and `--verbose`; no conflict
- **Exit:** 0
- **Source:** [parameter_groups.md](../../../../docs/cli/parameter_groups.md#group--1-claude-native-flags)
---

### EC-3: `--verbose` + `--effort max` → both present:

- **Given:** clean environment
- **When:** `clr --dry-run --verbose --effort max "Fix bug"`
- **Then:** Assembled command contains both `--verbose` and `--effort max`
- **Exit:** 0
- **Source:** [parameter_groups.md](../../../../docs/cli/parameter_groups.md#group--1-claude-native-flags)
---

### EC-4: No group flags → only defaults injected:

- **Given:** clean environment
- **When:** `clr --dry-run "Fix bug"`
- **Then:** Assembled command has default `--effort max` and `--print`; no `--verbose` or `--model`
- **Exit:** 0
- **Source:** [parameter_groups.md](../../../../docs/cli/parameter_groups.md#group--1-claude-native-flags)
