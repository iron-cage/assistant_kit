# Invariant: Zero Workspace Dependencies

### Scope

- **Purpose**: Keep `claude_auth` a Layer `*` standalone primitive — usable by any crate in the workspace without creating a dependency cycle or dragging in unrelated machinery.
- **Responsibility**: State the dependency constraints, define the enforcement commands, and document violation consequences.
- **In Scope**: Workspace-crate exclusion (INV-1), the single optional third-party dep (INV-2), zero-dep default build (INV-3).
- **Out of Scope**: Which parts of the API stay reachable without features (→ `002_offline_parse_core.md`).

### Invariant Statement

| ID | Invariant |
|----|-----------|
| INV-1 | `claude_auth` depends on no crate in this workspace |
| INV-2 | `ureq` is the only third-party dependency, declared `optional` and reachable only through feature `enabled` |
| INV-3 | With no features enabled, the crate builds with zero runtime dependency edges |

### Enforcement Mechanism

**INV-1** — no workspace crate may appear in the manifest:

```bash
grep -nE '^(claude_|dream|assistant)' module/claude_auth/Cargo.toml
# Expected: empty output
```

**INV-2** — the dependency set under full activation is exactly one crate:

```bash
cargo tree -p claude_auth --features enabled --edges normal --depth 1
# Expected:
#   claude_auth v0.3.1 (…/module/claude_auth)
#   └── ureq v3.3.0
```

**INV-3** — the default build stands alone:

```bash
cargo tree -p claude_auth --no-default-features --edges normal
# Expected: claude_auth alone, no dependency lines
```

### Violation Consequences

- **INV-1 violated:** `claude_auth` sits below every consumer that needs a token — `claude_profile`
  most directly. A dependency on any of them inverts that and creates a cycle the workspace
  cannot resolve. It would also make the crate unpublishable in isolation.
- **INV-2 violated:** A second third-party dep is linked into every consumer that enables
  `enabled`, for a crate whose entire job is one HTTP POST. The narrow surface is the reason
  this crate is separable from `claude_profile` at all.
- **INV-3 violated:** A non-optional dep would be linked even by consumers that only want
  `parse_response`, `TOKEN_URL`, or the error type — defeating the split described in
  [002_offline_parse_core.md](002_offline_parse_core.md).

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| doc | [002_offline_parse_core.md](002_offline_parse_core.md) | The feature split this dependency shape exists to serve |
| doc | [feature/001_token_refresh.md](../feature/001_token_refresh.md) | The one capability that needs `ureq` |
| doc | `../../../readme.md` | Workspace layer map placing this crate at Layer `*` |
| source | `../../Cargo.toml` | Dep declarations that must satisfy INV-1 through INV-3 |
