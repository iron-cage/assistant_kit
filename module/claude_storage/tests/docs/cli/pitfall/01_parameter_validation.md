# Pitfall :: Parameter Validation Not Implied By Default

Contract tests verifying that every constrained parameter type performs explicit validation.

**Source:** [cli/pitfall/01_parameter_validation.md](../../../../docs/cli/pitfall/01_parameter_validation.md)

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| PF-1 | Boolean param rejects values other than `"0"` and `"1"` | Boolean Validation |
| PF-2 | Integer param rejects non-integer and negative input | Integer Validation |
| PF-3 | Enum param has exhaustive match with catch-all error arm | Enum Validation |
| PF-4 | Non-empty string param trims and rejects whitespace-only value | String Validation |

## Test Coverage Summary

- Boolean Validation: 1 test (PF-1)
- Integer Validation: 1 test (PF-2)
- Enum Validation: 1 test (PF-3)
- String Validation: 1 test (PF-4)

**Total:** 4 pitfall contract cases

**Implementation target:** `tests/cli_param_format_test.rs`

## Test Cases

---

### PF-1: Boolean param rejects values other than `"0"` and `"1"`

- **Given:** a command accepting a boolean parameter (e.g., `agent::`, `case_sensitive::`) invoked with value `"banana"`
- **When:** the command routine processes the parameter
- **Then:** command returns an argument error; no partial result is produced

---

### PF-2: Integer param rejects non-integer and negative input

- **Given:** a command accepting an integer parameter (e.g., `min_entries::`, `limit::`) invoked with value `"abc"` or `"-5"`
- **When:** the command routine processes the parameter
- **Then:** command returns an argument error; no partial result is produced

---

### PF-3: Enum param has exhaustive match with catch-all error arm

- **Given:** a command accepting an enum parameter (e.g., `target::`, `entry_type::`) invoked with an unrecognized value `"invalid_enum"`
- **When:** the command routine processes the parameter
- **Then:** command returns an argument error; no default or fallback behavior is applied silently

---

### PF-4: Non-empty string param trims and rejects whitespace-only value

- **Given:** a command accepting a non-empty string parameter (e.g., `query::`) invoked with value `"   "` (whitespace only)
- **When:** the command routine trims and validates the value
- **Then:** command returns an argument error; search is not executed with an empty effective query
