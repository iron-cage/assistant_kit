# docs/

### Scope

**Responsibilities:** Behavioral requirements, API contracts, CLI reference, and structural constraints for the `claude_runner` crate.
**In Scope:** Feature requirements (`feature/`), invariant constraints (`invariant/`), public API contracts (`api/`), CLI reference (`cli/`), design rationale, cross-entity index, and doc graph.
**Out of Scope:** Source code (→ `src/`), automated tests (→ `tests/`), manual testing plans (→ `tests/manual/`), build scripts (→ `verb/`).

### Responsibility Table

| File | Responsibility |
|------|----------------|
| `api/` | Public library API contracts (COMMANDS_YAML, VerbosityLevel) |
| `cli/` | CLI reference: commands, flags, modes, examples |
| `design_decisions.md` | Design rationale for `--flag value` CLI redesign |
| `entities.md` | Cross-entity index: Master Doc Entities Table and Master Doc Instances Table |
| `feature/` | Runner tool design: execution modes, defaults, YAML library |
| `invariant/` | Default flag injection and dependency constraint rules |
| `doc_graph.yml` | Cross-reference graph for all doc instances |
