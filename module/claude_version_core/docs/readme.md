# docs/

### Scope

**Responsibilities:** Library contracts and structural constraints for `claude_version_core` — the Layer 1 domain crate behind Claude Code version management, settings resolution, and the parameter catalog — including the 4-layer settings-resolution algorithm and the parameter-trace instrumentation convention it implements.
**In Scope:** Public API contracts (`api/`), measurable constraints (`invariant/`), the 4-layer settings-resolution algorithm (`algorithm/`), the unconditional stderr parameter-trace convention (`pattern/`).
**Out of Scope:** Source code (→ `src/`), automated tests (→ `tests/`), build scripts (→ `verb/`), CLI command surface and user-facing feature/story docs (→ `claude_version/docs/`, the consuming Layer 2 crate — see Relationship section below).

### Relationship to `claude_version`'s docs

This crate documents its own library contract (`api/`), constraints (`invariant/`), settings-
resolution algorithm (`algorithm/`), and stderr instrumentation convention (`pattern/`). The CLI
*behavior* built on top of it is specified in the Layer 2 crate, `../../claude_version/docs/`.
`feature/`, `cli/`, and `pitfall/` instances live there and are **not** duplicated here;
`algorithm/` and `pattern/` exist at both layers by design — this crate documents what it
itself implements, Layer 2 documents how the CLI command uses/renders it (see
`algorithm/002_config_resolution.md`'s own Out of Scope line).

Note that two of `src/`'s bare `docs/…` citations resolve locally now that
`algorithm/002_config_resolution.md` and `pattern/002_parameter_trace.md` exist in this
directory; `feature/001_version_management.md` and `feature/007_params_command.md` still
resolve only at Layer 2 — see
[invariant/001_layer_one_boundary.md](invariant/001_layer_one_boundary.md) § Known Documentation
Split, which needs a matching update.

### Responsibility Table

| File | Responsibility |
|------|----------------|
| `entity.md` | Master registry of doc entities and instances |
| `api/` | Public library API contracts, one instance per surface group |
| `invariant/` | Constraints that must hold for every build |
| `algorithm/` | 4-layer settings resolution algorithm (env → project → user → catalog default) |
| `pattern/` | Unconditional stderr parameter-trace convention on mutating functions |
