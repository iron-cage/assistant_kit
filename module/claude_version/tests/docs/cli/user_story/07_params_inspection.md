# Test: Params Inspection

Acceptance tests for User Story 007. See [user_story/007_params_inspection.md](../../../../docs/cli/user_story/007_params_inspection.md) for specification.

### Scope

- **Purpose**: Verify `.params` provides full Claude Code parameter catalog inspection with observable values and form annotations.
- **Responsibility**: Acceptance criteria coverage for the params inspection workflow.
- **Commands:** `.params`
- **In Scope**: Show-all mode, single-param deep-dive, kind filter, env var reads, CLI-only annotation, JSON output.
- **Out of Scope**: Config write operations (-> `../command/13_config.md`), parameter edge cases (-> `../param/`).

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| AT-1 | `.params` shows ≥35 entries with source annotations | Acceptance: show-all |
| AT-2 | `.params key::model` shows all forms and default | Acceptance: single-param |
| AT-3 | `.params kind::config` shows only config-key params | Acceptance: kind filter |
| AT-4 | `.params kind::env` shows only env-var params | Acceptance: kind filter |
| AT-5 | `.params key::model` with env override shows (env) annotation | Acceptance: env priority |
| AT-6 | `.params key::print` shows CLI-only annotation | Acceptance: CLI-only marking |
| AT-7 | `.params format::json` returns valid JSON array with required fields | Acceptance: JSON output |
| AT-8 | `.params key::UNKNOWN` exits 2 | Acceptance: error handling |
| AT-9 | `.params kind::bad` exits 1 | Acceptance: error handling |
| AT-10 | Show-all output is alphabetically sorted | Acceptance: ordering |

## Test Coverage Summary

- Show-all: 2 tests (AT-1, AT-10)
- Single-param: 2 tests (AT-2, AT-5)
- Kind filter: 2 tests (AT-3, AT-4)
- CLI-only: 1 test (AT-6)
- JSON output: 1 test (AT-7)
- Error paths: 2 tests (AT-8, AT-9)

**Total:** 10 tests

## Source Functions Table

| Function | File | Test Cases |
|----------|------|------------|
| *(none yet — implementation pending)* | — | — |
