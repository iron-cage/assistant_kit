# Parameter Group :: Claude-Native Flags

Interaction tests for Group 1 (Claude-Native Flags): `--print`, `--model`, `--verbose`,
`--effort`, `--no-persist`, `--json-schema`, `--mcp-config`. Tests validate these flags
coexist correctly and are forwarded to the claude subprocess.

**Source:** [param_group.md#group--1-claude-native-flags](../../../../docs/cli/param_group.md#group--1-claude-native-flags)

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| CC-1 | All seven claude-native flags forwarded together | Combined |
| CC-2 | `--model` + `--verbose` coexist without conflict | Interaction |
| CC-3 | `--verbose` + `--effort max` → both in assembled command | Interaction |
| CC-4 | None of the group flags set → only defaults injected | Default |
| CC-5 | `--no-persist` + `--json-schema` + `--mcp-config` → all forwarded | Combined |

## Test Coverage Summary

- Combined: 2 tests (CC-1, CC-5)
- Interaction: 2 tests (CC-2, CC-3)
- Default: 1 test (CC-4)

**Total:** 5 edge cases

## Test Cases
---

### CC-1: All seven claude-native flags forwarded together

- **Given:** clean environment
- **When:** `clr --dry-run --print --model sonnet --verbose --effort high --no-persist --json-schema '{"type":"string"}' --mcp-config /tmp/mcp.json "Fix bug"`
- **Then:** Assembled command contains `--print`, `--model sonnet`, `--verbose`, `--effort high`, `--no-session-persistence`, `--json-schema`, and `--mcp-config /tmp/mcp.json`
- **Exit:** 0
- **Source:** [param_group.md](../../../../docs/cli/param_group.md#group--1-claude-native-flags)
---

### CC-2: `--model` + `--verbose` coexist

- **Given:** clean environment
- **When:** `clr --dry-run --model opus --verbose "Fix bug"`
- **Then:** Assembled command contains both `--model opus` and `--verbose`; no conflict
- **Exit:** 0
- **Source:** [param_group.md](../../../../docs/cli/param_group.md#group--1-claude-native-flags)
---

### CC-3: `--verbose` + `--effort max` → both present

- **Given:** clean environment
- **When:** `clr --dry-run --verbose --effort max "Fix bug"`
- **Then:** Assembled command contains both `--verbose` and `--effort max`
- **Exit:** 0
- **Source:** [param_group.md](../../../../docs/cli/param_group.md#group--1-claude-native-flags)
---

### CC-4: No group flags → only defaults injected

- **Given:** clean environment
- **When:** `clr --dry-run "Fix bug"`
- **Then:** Assembled command has default `--effort max` and `--print`; no `--verbose`, `--model`, `--no-session-persistence`, `--json-schema`, or `--mcp-config`
- **Exit:** 0
- **Source:** [param_group.md](../../../../docs/cli/param_group.md#group--1-claude-native-flags)
---

### CC-5: `--no-persist` + `--json-schema` + `--mcp-config` → all forwarded

- **Given:** clean environment
- **When:** `clr --dry-run --no-persist --json-schema '{"type":"object"}' --mcp-config /tmp/servers.json "Fix bug"`
- **Then:** Assembled command contains `--no-session-persistence`, `--json-schema`, and `--mcp-config /tmp/servers.json`; all three new members coexist without conflict
- **Exit:** 0
- **Source:** [param_group.md](../../../../docs/cli/param_group.md#group--1-claude-native-flags)
