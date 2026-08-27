# docs/

### Scope

**Responsibilities:** Documentation for the `claude_version_core` crate — the config-resolution algorithm and the parameter-trace instrumentation convention it implements.
**In Scope:** 4-layer settings resolution algorithm (`algorithm/`), unconditional stderr parameter-trace convention (`pattern/`).
**Out of Scope:** Source code (→ `src/`), automated tests (→ `tests/`), CLI command surface and user-facing feature/story docs (→ `claude_version/docs/`, the consuming Layer 2 crate — this crate is "pure domain operations with no CLI framework dependencies" per `src/version.rs`'s own module doc).

### Responsibility Table

| File | Responsibility |
|------|----------------|
| `algorithm/` | 4-layer settings resolution algorithm (env → project → user → catalog default) |
| `pattern/` | Unconditional stderr parameter-trace convention on mutating functions |
