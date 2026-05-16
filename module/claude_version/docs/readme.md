# docs/

### Scope

**Responsibilities:** Documentation for the `claude_version` crate covering behavioral requirements, design patterns, and algorithms.
**In Scope:** CLI reference (`cli/`), feature requirements (`feature/`), version lock design (`pattern/`), settings inference algorithm (`algorithm/`), design rationale, and doc cross-reference graph.
**Out of Scope:** Source code (→ `src/`), automated tests (→ `tests/`), build scripts (→ `verb/`).

### Responsibility Table

| File | Responsibility |
|------|----------------|
| `cli/` | CLI reference: commands, flags, types, parameter groups, workflows |
| `design_decisions.md` | Key design rationale for the CLI redesign |
| `feature/` | Version management, process lifecycle, settings, dry-run, CLI design |
| `pattern/` | 5-layer version lock design pattern |
| `algorithm/` | Settings type inference algorithm |
| `doc_graph.yml` | Cross-reference graph for all doc instances |
