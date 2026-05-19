# Feature Tests

Feature behavioral requirement test cases for `claude_profile`. Each file covers one feature doc instance from `docs/feature/` and maps its acceptance criteria to `FT-N` test cases.

### Scope

- **Purpose**: Document FT-N test cases for each feature behavioral requirement.
- **Responsibility**: Index of per-feature test case planning files covering AC coverage.
- **In Scope**: All feature doc instances in `docs/feature/` that have been validated or are under active work.
- **Out of Scope**: CLI command tests (→ `cli/command/`), parameter edge cases (→ `cli/param/`), automated test implementations (→ `tests/cli/`).

### Responsibility Table

| File | Responsibility |
|------|----------------|
| 09_token_usage.md | FT cases for Feature 009 — All-Accounts Live Quota Reporting (FR-14) |

### Coverage Summary

| Feature | File | Cases | Status |
|---------|------|-------|--------|
| 009_token_usage | [09_token_usage.md](09_token_usage.md) | FT-01 … FT-05 | ✅ |

**Total:** 1 feature spec (1 of 18 feature instances covered — expanded by TSK-153)
