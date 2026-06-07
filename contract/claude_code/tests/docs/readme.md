# Test Spec Documents

Test case planning documents for the `claude_code` contract crate. Each subdirectory covers one doc-entity surface.

### Scope

- **Purpose**: Host test spec docs (FT-N) for fault classification surfaces documented in `docs/fault/readme.md`.
- **Responsibility**: Index of per-surface test planning files; specs drive test implementation in `tests/behavior/` and `claude_runner_core/tests/`.

### Responsibility Table

| Directory | Responsibility |
|-----------|----------------|
| `fault/` | FT cases for `classify_error()` priority algorithm and dual-channel scanning |
