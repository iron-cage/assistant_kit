# docs/

### Scope

**Responsibilities:** Documentation for the `claude_version` crate covering behavioral requirements, design patterns, and algorithms.
**In Scope:** CLI reference (`cli/`), feature requirements (`feature/`), version lock design (`pattern/`), settings inference algorithm (`algorithm/`), design rationale, and doc cross-reference graph.
**Out of Scope:** Source code (→ `src/`), automated tests (→ `tests/`), build scripts (→ `verb/`).

### Responsibility Table

| File | Responsibility |
|------|----------------|
| `algorithm/` | Settings type inference and config resolution algorithms |
| `cli/` | CLI reference: commands, flags, types, parameter groups, user stories, formats |
| `collection/` | Design decision registry |
| `feature/` | Version management, process lifecycle, settings, dry-run, CLI design, config command |
| `pattern/` | 5-layer version lock design pattern |
| `pitfall/` | Confirmed design traps: chmod side effects, symlink retarget bypass |
| `doc_graph.yml` | Cross-reference graph for all doc instances |
| `entities.md` | Master index of doc entity types and instances |
