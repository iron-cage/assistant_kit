# Invariant: Zero Third-Party Dependencies

### Scope

- **Purpose**: Prevent dependency sprawl and ensure the library path remains lightweight and auditable.
- **Responsibility**: Documents the zero-crates.io-dep constraint on the `claude_profile` library path (NFR-1).
- **In Scope**: Library path dependency policy; permitted exceptions for the CLI binary under `enabled` feature.
- **Out of Scope**: Internal workspace crate dependencies (always permitted), CLI binary optional deps (see exceptions below).

### Invariant Statement

The `claude_profile` **library path** must have zero third-party (crates.io) dependencies.

**Permitted:**
- Internal workspace crates: `claude_core`, `claude_profile_core`
  - `claude_profile_core` unconditionally depends on `serde_json` (not feature-gated); this is a known gap in the zero-crates.io-deps threshold below — see Enforcement Mechanism
- Under the `enabled` feature (CLI binary only): `cli_fmt`, `unilang`, `error_tools`, `claude_quota`, `data_fmt`
  - `cli_fmt` is gated behind `dep:cli_fmt` in the `enabled` feature; it renders CLI help text via `cli_fmt::help::CliHelpTemplate`
  - `claude_quota` is gated behind `dep:claude_quota` in the `enabled` feature; it is an internal workspace crate that encapsulates the HTTP transport and exposes `fetch_rate_limits(token: &str)`
  - `data_fmt` is gated behind `dep:data_fmt` in the `enabled` feature; it is used for all table rendering in the CLI binary

**Forbidden:**
- Any crates.io dependency in the library path (non-feature-gated)

**Measurable threshold:** `cargo tree --no-dev-dependencies` without `--features enabled` must show zero crates.io entries. **Known gap:** `claude_profile_core`'s unconditional `serde_json` dependency currently makes this threshold not hold in practice; the enforcement test below cannot detect it (see below).

### Enforcement Mechanism

- `Cargo.toml` structure: all permitted optional deps are gated under the `enabled` feature
- Code review: reject any PR adding a non-feature-gated `[dependencies]` entry
- Automated (`tests/cli/invariant_test.rs`): `zero_third_party_deps_in1_library_deps_are_workspace_only` text-parses `claude_profile`'s own `[dependencies]` section and asserts every entry contains `workspace`; `zero_third_party_deps_in2_enabled_feature_activates_workspace_deps_only` asserts every `dep:xxx` entry in the `enabled` feature references a workspace-aliased dependency. Neither test runs `cargo tree`, and neither can detect a crates.io dependency introduced transitively by a workspace-internal crate (e.g. `claude_profile_core`'s unconditional `serde_json`) — both text-parse `claude_profile`'s own manifest only.

### Violation Consequences

- Adds transitive dependencies that may carry security vulnerabilities
- Increases compile time and binary size for callers who only use the library API
- Reduces auditability — harder to verify the crate does what it claims
- Contradicts the "stdlib-only" design principle that makes `claude_profile` a trusted building block

### Sources

| File | Relationship |
|------|-------------|
| `Cargo.toml` | Dependency declarations — `enabled` feature gates optional CLI deps |

### Tests

| File | Relationship |
|------|-------------|
| `tests/responsibility_no_process_execution_test.rs` | Verifies no std::process imports (related boundary) |
