# CLI Type Tests

Type acceptance and boundary test cases for `claude_profile` CLI types from `docs/cli/type/`.
Each file covers one type doc instance and maps its constraints to `TC-N` test cases.

### Scope

- **Purpose**: Verify each CLI type correctly accepts valid inputs, rejects invalid inputs, and
  enforces all documented constraints.
- **Responsibility**: Index of per-type boundary case files (TC-N entries).
- **In Scope**: All 4 CLI type definitions from `docs/cli/type/`.
- **Out of Scope**: Parameter edge cases (→ `../param/`), command integration tests (→ `../command/`).

### Responsibility Table

| File | Responsibility |
|------|----------------|
| `01_account_name.md` | TC- spec for AccountName type — email validation and path-safety |
| `02_output_format.md` | TC- spec for OutputFormat type — mode parsing and rejection |
| `03_warning_threshold.md` | TC- spec for WarningThreshold type — u64 boundary parsing |
| `04_account_selector.md` | TC- spec for AccountSelector resolution — prefix and email forms |

### Coverage Summary

| Type | Source | Cases | Status |
|------|--------|-------|--------|
| AccountName | [docs/cli/type/001_account_name.md](../../../../docs/cli/type/001_account_name.md) | TC-1 … TC-6 | ✅ |
| OutputFormat | [docs/cli/type/002_output_format.md](../../../../docs/cli/type/002_output_format.md) | TC-1 … TC-5 | ✅ |
| WarningThreshold | [docs/cli/type/003_warning_threshold.md](../../../../docs/cli/type/003_warning_threshold.md) | TC-1 … TC-4 | ✅ |
| AccountSelector | [docs/cli/type/004_account_selector.md](../../../../docs/cli/type/004_account_selector.md) | TC-1 … TC-4 | ✅ |

**Total:** 4 specs, 19 TC cases.
