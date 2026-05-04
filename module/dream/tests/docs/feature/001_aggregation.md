# Feature: Facade Aggregation

### Scope

- **Purpose**: Document smoke-test coverage for the feature-gate re-export behavior of the `dream` facade crate.
- **Responsibility**: Specify which tests verify which FRs, the test approach, and acceptance criteria.
- **In Scope**: Per-feature compile-check (FR-1–FR-5, FR-10), type-access verification, feature isolation (FR-8, FR-9), zero-dep compile (FR-6), bundle activation (FR-7).
- **Out of Scope**: Zero-own-logic verification (→ `invariant/001_no_own_logic.md`).

### Test Coverage

| FR | Test | Approach |
|----|------|----------|
| FR-1 | `common_re_exports_accessible` | `use dream::common::ClaudePaths` — compile + `TypeId` check |
| FR-2 | `storage_re_exports_accessible` | `use dream::storage::Storage` — compile + `TypeId` check |
| FR-3 | `profile_re_exports_accessible` | `use dream::profile::token` + `account` — compile + `TypeId` check |
| FR-4 | `runner_re_exports_accessible` | `use dream::runner::ClaudeCommand` — compile + `TypeId` check |
| FR-5 | `version_re_exports_accessible` | `use dream::version::CoreError` — compile + `TypeId` check |
| FR-10 | `assets_re_exports_accessible` | `use dream::assets::artifact::ArtifactKind` — compile + `TypeId` check |
| FR-6 | *(compile gate)* | `--no-default-features` build produces zero dep warnings |
| FR-7 | all six tests pass | `--features full` activates all six modules simultaneously |
| FR-8 | feature isolation | `--no-default-features --features storage` — only `claude_storage_core` dep active |
| FR-9 | per-feature runs | Each `--features X` run activates exactly one test |

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| doc | [../../docs/feature/001_aggregation.md](../../docs/feature/001_aggregation.md) | Feature spec these tests verify |
| doc | [../invariant/001_no_own_logic.md](../invariant/001_no_own_logic.md) | Invariant test lens complementing this coverage |
| source | `../../tests/integration/facade_test.rs` | Test implementation for all FR checks |
