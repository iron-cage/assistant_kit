# docs/

### Scope

**Responsibilities:** Library contracts and structural constraints for `claude_version_core` — the Layer 1 domain crate behind Claude Code version management, settings resolution, and the parameter catalog.
**In Scope:** Public API contracts (`api/`), measurable constraints (`invariant/`).
**Out of Scope:** Source code (→ `src/`), automated tests (→ `tests/`), build scripts (→ `verb/`).

### Relationship to `claude_version`'s docs

This crate documents its **library contract**; the CLI *behavior* built on top of it is
specified in the Layer 2 crate, `../../claude_version/docs/`. `feature/`, `algorithm/`,
`cli/`, `pattern/`, and `pitfall/` instances live there and are **not** duplicated here.

Note that several doc comments in `src/` cite bare paths such as
`docs/algorithm/002_config_resolution.md`. Those resolve under
`../../claude_version/docs/`, not under this directory — see
[invariant/001_layer_one_boundary.md](invariant/001_layer_one_boundary.md) § Known Documentation
Split for the full mapping.

### Responsibility Table

| File | Responsibility |
|------|----------------|
| `entity.md` | Master registry of doc entities and instances |
| `api/` | Public library API contracts, one instance per surface group |
| `invariant/` | Constraints that must hold for every build |
