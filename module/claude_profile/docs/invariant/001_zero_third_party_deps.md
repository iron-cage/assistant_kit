# Invariant: Zero Third-Party Dependencies

### Scope

- **Purpose**: Prevent dependency sprawl and ensure the library path remains lightweight and auditable.
- **Responsibility**: Documents the zero-crates.io-dep constraint on the `claude_profile` library path (NFR-1).
- **In Scope**: Library path dependency policy; permitted exceptions for the CLI binary under `enabled` feature.
- **Out of Scope**: Internal workspace crate dependencies (always permitted), CLI binary optional deps (see exceptions below).

### Invariant Statement

The `claude_profile` **library path** must have zero third-party (crates.io) dependencies.

**Permitted:**
- Internal workspace crates: `claude_common`, `claude_profile_core`
- Under the `enabled` feature (CLI binary only): `unilang`, `error_tools`, `serde_json`, `ureq`
  - `serde_json` is gated behind `dep:serde_json` in the `enabled` feature and used exclusively by the `.usage` command for parsing `stats-cache.json`
  - `ureq` is gated behind `dep:ureq` in the `enabled` feature and used exclusively by `.account.limits` for fetching rate-limit headers via `POST /v1/messages`

**Forbidden:**
- Any crates.io dependency in the library path (non-feature-gated)

**Measurable threshold:** `cargo tree --no-dev-dependencies` without `--features enabled` must show zero crates.io entries.

### Enforcement Mechanism

- `Cargo.toml` structure: all permitted optional deps are gated under the `enabled` feature
- Code review: reject any PR adding a non-feature-gated `[dependencies]` entry
- CI: `cargo tree` audit without `--features enabled` to verify zero crates.io deps

### Violation Consequences

- Adds transitive dependencies that may carry security vulnerabilities
- Increases compile time and binary size for callers who only use the library API
- Reduces auditability — harder to verify the crate does what it claims
- Contradicts the "stdlib-only" design principle that makes `claude_profile` a trusted building block

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| source | `Cargo.toml` | Dependency declarations — `enabled` feature gates optional CLI deps |
| test | `tests/responsibility_no_process_execution_test.rs` | Verifies no std::process imports (related boundary) |
