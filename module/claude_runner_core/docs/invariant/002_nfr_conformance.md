# Invariant: NFR Conformance

### Scope

- **Purpose**: Document the non-functional constraints that claude_runner_core must always satisfy.
- **Responsibility**: State dependency count limits, performance targets, type-safety requirements, and feature flag structure.
- **In Scope**: Minimal dependencies (NFR-1), fast execution setup (NFR-2), type safety (NFR-3), no deprecated API (NFR-5), feature flag conformance (NFR-7).
- **Out of Scope**: Single execution point rule (→ `invariant/001_single_execution_point.md`), API contracts (→ `api/001_execution_api.md`).

### Invariant Statement

claude_runner_core must satisfy all of the following constraints simultaneously. Violation of any constraint is a conformance failure.

### Enforcement Mechanism

#### NFR-1: Minimal Dependencies

claude_runner_core depends only on:
- `error_tools` (workspace crate, optional, feature-gated)
- Rust standard library

**Zero** direct crates.io dependencies are permitted. No other workspace crates may be listed as dependencies.

The `enabled` feature must activate both `"error_tools/enabled"` and `"error_tools/error_untyped"`. Without explicit propagation, `error_tools::Result` and `error_tools::Error` (from anyhow) are unavailable, causing compile failure. This is required because `error_tools` is published with `default-features = false`.

**Enforced by:** `Cargo.toml` structure; `cargo +nightly udeps` detects unused dependencies.

#### NFR-2: Fast Execution Setup

Builder construction and `Command` setup must complete in under 10ms. Process execution time (waiting for Claude Code to finish) is excluded from this constraint.

**Enforced by:** Performance benchmark test.

#### NFR-3: Type Safety

Configuration parameters that have a fixed value set must use enum types, not raw strings:
- Tool approval: `ActionMode` (not `&str`)
- Logging: `LogLevel` (not `&str`)
- Output: `OutputFormat` (not `&str`)
- Input: `InputFormat` (not `&str`)
- Permissions: `PermissionMode` (not `&str`)
- Effort: `EffortLevel` (not `&str`)

Negative token limits are prevented by using `u32` or similar unsigned types where applicable.

**Enforced by:** Type signatures in `src/types.rs` and `src/command.rs`.

#### NFR-5: No Deprecated API

The following methods must not exist in this crate:
- `ClaudeCommand::generate()` (old factory method)
- `execute_non_interactive()` (old execution method)

Their absence is proven by compile-time absence and verified by `tests/verification_negative_criteria_test.rs`.

#### NFR-7: Feature Flag Structure

`Cargo.toml` must declare exactly:
```
default = ["enabled"]
full    = ["enabled"]
enabled = ["dep:error_tools", "error_tools/enabled", "error_tools/error_untyped"]
```

This structure follows the dream workspace standard and ensures correct error type availability.

### Violation Consequences

- **NFR-1 violation:** Introduces transitive dependency risk and binary bloat; `cargo udeps` fails
- **NFR-2 violation:** Automation workflows pay unexpected CPU overhead on every command construction
- **NFR-3 violation:** Callers can pass invalid string values; bugs surface at runtime not compile time
- **NFR-5 violation:** Old API re-introduced breaks the verified migration; verification tests fail
- **NFR-7 violation:** Compilation fails when `error_tools::Result`/`Error` are referenced

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| invariant | [invariant/001_single_execution_point.md](001_single_execution_point.md) | Complementary structural invariant |
| data_structure | [data_structure/001_command_types.md](../data_structure/001_command_types.md) | Type-safe enums that satisfy NFR-3 |
| source | `../../Cargo.toml` | Dependency and feature flag definitions |
| source | `../../src/types.rs` | Enum type definitions |
| test | `../../tests/verification_negative_criteria_test.rs` | NFR-5 enforcement |

### Sources

| File | Notes |
|------|-------|
| `spec.md` (deleted — migrated here) | NFR-1 through NFR-7, External Dependencies (error_tools feature propagation rationale), Conformance Checklist (NFR section) |
