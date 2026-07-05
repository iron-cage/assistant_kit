# docs/

### Scope

**Responsibilities:** Behavioral requirements, API contracts, CLI reference, and structural constraints for the `claude_runner` crate.
**In Scope:** Feature requirements (`feature/`), invariant constraints (`invariant/`), public API contracts (`api/`), CLI reference (`cli/`), algorithm specifications (`algorithm/`), variable definitions (`variable/`), design rationale, cross-entity index, and doc graph.
**Out of Scope:** Source code (→ `src/`), automated tests (→ `tests/`), manual testing plans (→ `tests/manual/`), build scripts (→ `verb/`).

### Responsibility Table

| File | Responsibility |
|------|----------------|
| `algorithm/` | Path computation algorithms: Df() encoding, git root detection, session file selection |
| `api/` | Public library API contracts (COMMANDS_YAML, register_commands) |
| `cli/` | CLI reference: commands, flags, modes, examples |
| `variable/` | Output variable definitions for the six CLAUDE_* paths computed by `scope_for()` |
| `001_design_decisions.md` | Rationale for key design choices made during the `--flag value` CLI redesign |
| `entity.md` | Cross-entity index: Master Doc Entities Table and Master Doc Instances Table |
| `feature/` | Runner tool design: execution modes, defaults, YAML library |
| `invariant/` | Default flag injection, dependency constraints, and command naming convention |
| `doc_graph.yml` | Cross-reference graph for all doc instances |

### Related Crates

| Crate | Relationship |
|-------|--------------|
| `claude_storage_core` | Depends on `algorithm/001_path_encoding.md` (Df() path encoding) via `claude_storage_core::scope::scope_for()` and `claude_storage_core::continuation::to_storage_path_for()`; see `algorithm/001_path_encoding.md` in this crate for the consumption-side documentation |
