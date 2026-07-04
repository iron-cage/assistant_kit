# Test: Invariant — Container-Only Test Execution

Test case planning for [invariant/010_container_only_test_execution.md](../../../docs/invariant/010_container_only_test_execution.md). Tests verify that the two enforcement layers (shell and nextest setup script) are structurally present and correctly configured.

**Source:** [invariant/010_container_only_test_execution.md](../../../docs/invariant/010_container_only_test_execution.md)
**Related:** [claude_profile/docs/invariant/009_container_only_test_execution.md](../../../../claude_profile/docs/invariant/009_container_only_test_execution.md)

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| IT-1 | `workspace_nextest_toml_registers_setup_script` — workspace `.config/nextest.toml` contains `"setup-scripts"` in the `experimental` array and references `require-container` | Structural |
| IT-2 | `setup_script_file_exists` — `.config/setup-require-container` file exists at workspace root | Structural |
| IT-3 | `setup_script_checks_dockerenv` — setup script body contains `/.dockerenv` detection | Structural |
| IT-4 | `setup_script_checks_containerenv` — setup script body contains `/run/.containerenv` detection | Structural |
| IT-5 | `setup_script_checks_runbox_var` — setup script body contains `RUNBOX_CONTAINER` detection | Structural |

## Test Coverage Summary

- Structural: 5 tests (IT-1–IT-5)

**Total:** 5 invariant test cases

## Notes

This invariant is **self-verifying at runtime**: if any test binary executes at all, the setup script ran successfully inside the container, confirming signals 1–3 were satisfied. The structural tests above guard against accidental removal or misconfiguration of the enforcement scripts.

No Rust guard layer exists for `claude_runner` (unlike `claude_profile` which adds an in-process `run_cs()` assertion). Structural tests are the only Rust-level coverage for this invariant.

## Source Functions Table

| Function / File | Notes |
|-----------------|-------|
| `../../tests/invariant_container_test.rs` | IT-1–IT-5 implementations |
| `../../.config/nextest.toml` (workspace) | IT-1 verification target |
| `../../.config/setup-require-container` (workspace) | IT-2–IT-5 verification target |
