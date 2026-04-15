# Invariant: Testing Strategy

### Scope

- **Purpose**: Document the testing constraints and TDD baseline rule for the dream workspace.
- **Responsibility**: State the test directory placement rule, TDD baseline enforcement, test categories, and what constitutes a regression.
- **In Scope**: tests/ placement rule, TDD baseline, test categories, skip count as regression proxy.
- **Out of Scope**: Performance constraints (→ `invariant/004_performance.md`), versioning (→ `invariant/002_versioning_strategy.md`).

### Invariant Statement

All tests live in each crate's `tests/` directory. Manual tests live in `tests/manual/readme.md`.

**TDD baseline rule:** Before any change, record the passing test count. After the change, the passing count must be ≥ baseline. The skipped count must not increase — skips are a proxy for capability loss. A skip increase is treated as a regression even if the passing count is stable.

**Target:** 10/10 crates pass L3 (nextest + doc tests + clippy) at all times.

### Enforcement Mechanism

**Test placement:** Only `tests/` directories are scanned by nextest. Tests in `src/` are doc tests, run separately via `cargo test --doc`. Tests in any other location are not discovered.

**Baseline enforcement:** Before committing any change, run `ctest3`. Record the pass/skip/fail counts. The change is not complete until pass count ≥ baseline and skip count = baseline.

**Test categories:**
- **Unit tests:** Pure logic (JSONL parsing, path resolution, builder state). No filesystem access.
- **Integration tests:** Filesystem reads from real `~/.claude/`. Skipped in CI unless the user has used Claude Code (directory exists). Must not be disabled — if environment absent, test skips with a clear message.
- **CLI integration:** `assert_cmd`-based tests for binary invocation. Require the binary to be built.

### Violation Consequences

- A skip count increase signals a test was silently disabled to make the suite pass — this masks real capability loss
- Tests outside `tests/` are invisible to the standard test commands and will not be discovered

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| feature | [feature/001_workspace_design.md](../feature/001_workspace_design.md) | Workspace that these tests cover |
| source | `../../Cargo.toml` | workspace lint configuration (missing_inline_in_public_items) |

### Sources

| File | Notes |
|------|-------|
| `spec.md` (deleted — migrated here) | Testing Strategy section |
