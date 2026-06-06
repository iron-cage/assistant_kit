# User Story Tests: Enum Output Validation

**User Story:** [024 — Enum Output Validation](../../../../docs/cli/user_story/024_enum_output_validation.md)

### Test Case Index

| ID | Scenario | Expected | Status |
|----|----------|----------|--------|
| US24-1 | Output matches `--expect "yes\|no"` (case-insensitive, trimmed) | Exit 0 | ✅ |
| US24-2 | Output does not match `--expect "yes\|no"`, default strategy | Exit 3 | ✅ |
| US24-3 | Mismatch with `--expect-strategy default:no` | stdout = `"no"`, exit 0 | ✅ |
| US24-4 | Parse error: `--expect-strategy bogus` | Exit 1; stderr contains error message | ✅ |
