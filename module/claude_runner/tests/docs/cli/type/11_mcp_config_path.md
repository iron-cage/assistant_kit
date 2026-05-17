# Type :: `McpConfigPath`

Validation tests for the `McpConfigPath` semantic type (String: filesystem path to an MCP
server config JSON file). Tests verify that paths are accepted and forwarded verbatim;
file existence and JSON validity are deferred to the claude subprocess, not validated by
the runner.

**Source:** [type.md](../../../../docs/cli/type.md#type--11-mcpconfigpath)

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| TC-1 | Absolute path → accepted and forwarded | Valid Input |
| TC-2 | Relative path → accepted and forwarded | Valid Input |
| TC-3 | Multiple paths → each forwarded as separate `--mcp-config` | Valid Input |
| TC-4 | Non-existent path → forwarded (validation deferred to claude) | Edge Case |

## Test Coverage Summary

- Valid Input: 3 tests (TC-1, TC-2, TC-3)
- Edge Case: 1 test (TC-4)

**Total:** 4 test cases

## Test Cases

---

### TC-1: Absolute path → accepted and forwarded

- **Given:** clean environment
- **When:** `clr --dry-run --mcp-config /home/user/.config/mcp/servers.json "task"`
- **Then:** Exit 0; assembled command contains `--mcp-config /home/user/.config/mcp/servers.json`
- **Exit:** 0
- **Source:** [type.md — McpConfigPath](../../../../docs/cli/type.md#type--11-mcpconfigpath)

---

### TC-2: Relative path → accepted and forwarded

- **Given:** clean environment
- **When:** `clr --dry-run --mcp-config ./mcp.json "task"`
- **Then:** Exit 0; assembled command contains `--mcp-config ./mcp.json`; no normalisation or resolution performed by the runner
- **Exit:** 0
- **Source:** [type.md — McpConfigPath](../../../../docs/cli/type.md#type--11-mcpconfigpath)

---

### TC-3: Multiple paths → each forwarded as a separate `--mcp-config` flag

- **Given:** clean environment
- **When:** `clr --dry-run --mcp-config /tmp/s1.json --mcp-config /tmp/s2.json "task"`
- **Then:** Exit 0; assembled command contains `--mcp-config /tmp/s1.json` and `--mcp-config /tmp/s2.json` as two independent occurrences (not merged or deduplicated)
- **Exit:** 0
- **Source:** [type.md — McpConfigPath](../../../../docs/cli/type.md#type--11-mcpconfigpath)

---

### TC-4: Non-existent path → forwarded without validation error

- **Given:** `/tmp/no_such_mcp.json` does not exist on the filesystem
- **When:** `clr --dry-run --mcp-config /tmp/no_such_mcp.json "task"`
- **Then:** Exit 0; assembled command contains `--mcp-config /tmp/no_such_mcp.json`; clr does not check whether the file exists — that responsibility belongs to claude
- **Exit:** 0
- **Source:** [type.md — McpConfigPath](../../../../docs/cli/type.md#type--11-mcpconfigpath)
