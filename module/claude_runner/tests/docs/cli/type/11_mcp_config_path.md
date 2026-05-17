# Type :: `McpConfigPath`

Validation tests for the `McpConfigPath` type. See [type.md](../../../../docs/cli/type.md#type--11-mcpconfigpath) for specification.

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| TC-1 | Absolute path → accepted and forwarded | Valid Input |
| TC-2 | Relative path → accepted and forwarded | Valid Input |
| TC-3 | Multiple paths → each forwarded as separate `--mcp-config` | Valid Input |
| TC-4 | Non-existent path → forwarded (validation deferred to claude) | Edge Case |

## Test Coverage Summary

- Valid Input: 3 tests
- Edge Case: 1 test

**Total:** 4 test cases
