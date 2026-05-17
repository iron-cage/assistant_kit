# Type :: `JsonSchemaText`

Validation tests for the `JsonSchemaText` type. See [type.md](../../../../docs/cli/type.md#type--10-jsonschematext) for specification.

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| TC-1 | Simple object schema → accepted and forwarded | Valid Input |
| TC-2 | Empty string → forwarded (validation deferred to claude) | Edge Case |
| TC-3 | Complex nested schema → forwarded verbatim | Valid Input |
| TC-4 | Schema from file via command substitution → forwarded | Valid Input |

## Test Coverage Summary

- Valid Input: 3 tests
- Edge Case: 1 test

**Total:** 4 test cases
