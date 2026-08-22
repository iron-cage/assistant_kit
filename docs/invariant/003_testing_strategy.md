# Invariant: Testing Strategy

### Scope

- **Purpose**: Document the testing constraints and TDD baseline rule for the assistant workspace.
- **Responsibility**: State the test directory placement rule, TDD baseline enforcement, test categories, and what constitutes a regression.
- **In Scope**: tests/ placement rule, TDD baseline, test categories, skip count as regression proxy.
- **Out of Scope**: Performance constraints (→ `invariant/004_performance.md`), versioning (→ `invariant/002_versioning_strategy.md`).

### Invariant Statement

Black-box integration and CLI tests live in each crate's `tests/` directory, exercising only the
crate's public API or its compiled binary. White-box unit tests that need access to private
internals may live inline in `src/`, colocated with the code they test inside a
`#[ cfg( test ) ] mod tests { ... }` block. Manual tests live in `tests/manual/readme.md`.

**TDD baseline rule:** Before any change, record the passing test count. After the change, the passing count must be ≥ baseline. The skipped count must not increase — skips are a proxy for capability loss. A skip increase is treated as a regression even if the passing count is stable.

**Target:** 10/10 crates pass L3 (nextest + doc tests + clippy) at all times.

### Enforcement Mechanism

**Test placement:** `cargo nextest run` discovers both kinds: `#[ test ]` functions in `tests/*.rs` (one binary per file) and `#[ test ]` functions inline in `src/` (compiled into the crate's own `--lib` test binary, e.g. `claude_storage_core project::tests::test_project_id_path`). Doc tests are a separate, third mechanism — ```` ```rust ```` fences inside `///`/`//!` doc comments, wherever the comment lives — run only via `cargo test --doc`, never by nextest.

**Baseline enforcement:** Before committing any change, run `ctest3`. Record the pass/skip/fail counts. The change is not complete until pass count ≥ baseline and skip count = baseline.

**Test categories:**
- **Unit tests:** Pure logic (JSONL parsing, path resolution, builder state). No filesystem access.
- **Integration tests:** Filesystem reads from real `~/.claude/`. Skipped in CI unless the user has used Claude Code (directory exists). Must not be disabled — if environment absent, test skips with a clear message.
- **CLI integration:** `assert_cmd`-based tests for binary invocation. Require the binary to be built.

### Violation Consequences

- A skip count increase signals a test was silently disabled to make the suite pass — this masks real capability loss
- A test placed outside `tests/` and outside the crate's own `src/` module tree (e.g. a stray file never declared with `mod`) is invisible to `cargo nextest run`/`cargo test` and will not be discovered

### Features

| File | Relationship |
|------|--------------|
| [feature/001_workspace_design.md](../feature/001_workspace_design.md) | Workspace that these tests cover |

### Sources

| File | Relationship |
|------|--------------|
| `../../Cargo.toml` | Workspace lint configuration (missing_inline_in_public_items) |

### Provenance

| File | Relationship |
|------|--------------|
| `spec.md` (deleted — migrated here) | Testing Strategy section |
