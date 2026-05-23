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
| 009_token_usage.md | FT cases for Feature 009 — All-Accounts Live Quota Reporting (FR-14) |
| 017_token_refresh.md | FT cases for Feature 017 — Expired Token Refresh via Isolated Subprocess |
| 018_live_monitor.md | FT cases for Feature 018 — Live Quota Monitor Mode |

### Coverage Summary

| Feature | File | Cases | Status |
|---------|------|-------|--------|
| 009_token_usage | [009_token_usage.md](009_token_usage.md) | FT-01 … FT-05 | ✅ |
| 017_token_refresh | [017_token_refresh.md](017_token_refresh.md) | FT-01 … FT-16 | ✅ |
| 018_live_monitor | [018_live_monitor.md](018_live_monitor.md) | FT-01 … FT-09 | ✅ |

**Total:** 3 feature specs, all fully implemented. 16 of 19 feature instances not yet covered by dedicated FT specs.
