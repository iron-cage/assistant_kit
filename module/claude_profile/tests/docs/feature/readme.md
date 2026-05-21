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
| 17_token_refresh.md | FT cases for Feature 017 — Expired Token Refresh via Isolated Subprocess |
| 18_live_monitor.md | FT cases for Feature 018 — Live Quota Monitor Mode |

### Coverage Summary

| Feature | File | Cases | Status |
|---------|------|-------|--------|
| 009_token_usage | [09_token_usage.md](09_token_usage.md) | FT-01 … FT-05 | ✅ |
| 017_token_refresh | [17_token_refresh.md](17_token_refresh.md) | FT-01 … FT-14 | ✅ |
| 018_live_monitor | [18_live_monitor.md](18_live_monitor.md) | FT-01 … FT-09 | ✅ |

**Total:** 3 feature specs, all fully implemented. 15 of 18 feature instances not yet covered by dedicated FT specs.
