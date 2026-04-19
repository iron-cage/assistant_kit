# Invariant: Zero Own Logic

### Scope

- **Purpose**: Document the verification approach for the zero-own-logic structural constraint on `dream`.
- **Responsibility**: Specify how INV-1, INV-2, and INV-3 are enforced and verified.
- **In Scope**: Grep-based INV-1 check, compile-time INV-2 verification, `Cargo.toml` INV-3 inspection.
- **Out of Scope**: Feature-gate smoke tests (→ `feature/001_aggregation.md`).

### Verification

**INV-1 — No own definitions in `src/`:**

Enforced by grep acceptance criterion (expected: empty output):

```bash
grep -rn "^pub struct\|^pub fn\|^pub trait\|^pub enum\|^pub type" module/dream/src/
```

Not an automated test; verified during code review.

**INV-2 — All exports originate from a core crate:**

Verified structurally: every test in `facade_test.rs` imports through a named dep crate
(`dream::common::ClaudePaths`, `dream::runner::ClaudeCommand`, etc.). A successful
`--all-features` build proves no items exist outside feature-gated re-export modules.

**INV-3 — No Layer 2 or Layer 3 dep:**

Verified by inspection of `Cargo.toml [dependencies]`: only `claude_core`,
`claude_storage_core`, `claude_profile_core`, `claude_runner_core`,
`claude_version_core`, `claude_assets_core` are listed. A forbidden dep would cause
`cargo build` to fail unless it were also hidden behind an optional feature gate.

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| doc | [../../docs/invariant/001_no_own_logic.md](../../docs/invariant/001_no_own_logic.md) | Invariant spec this lens documents verification for |
| doc | [../feature/001_aggregation.md](../feature/001_aggregation.md) | Feature test lens complementing this invariant verification |
| source | `../../tests/integration/facade_test.rs` | Integration tests providing INV-2 compile-time evidence |
| source | `../../src/lib.rs` | Implementation that must satisfy INV-1 and INV-2 |
| source | `../../Cargo.toml` | Dep declarations that must satisfy INV-3 |
