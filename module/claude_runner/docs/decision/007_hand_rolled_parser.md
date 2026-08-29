# Decision: Hand-Rolled Parser

**ID:** D7 · **Category:** Parsing · **Status:** ✅ Adopted

### Scope

- **Purpose**: Record why CLI parsing is hand-rolled rather than delegated to clap or unilang.
- **Responsibility**: Rationale for taking on parsing directly, and the surface-size argument that makes it cheap.
- **In Scope**: Dependency cost, error-message control, and the flag-surface size that justifies the choice.
- **Out of Scope**: Dependency constraints as an enforced rule (→ [`../invariant/002_dep_constraints.md`](../invariant/002_dep_constraints.md)); the whitelist the parser enforces (→ [005_unknown_flags_rejected.md](005_unknown_flags_rejected.md)).

### Decision

A hand-rolled parser. Zero external dependencies for CLI parsing.

### Rationale

Three reasons, in order of weight:

1. **Zero external dependencies** for the parsing layer.
2. **Exact control over error messages and behavior** — the error text a user sees on a typo is written here, not shaped by a framework's formatting.
3. **The flag surface is small enough that a framework adds complexity without benefit** — 24 flags plus one positional at the time of the decision.

The third reason is the one that can expire. A framework earns its cost at a surface size this one has not reached; the decision is a judgement about the present surface, not a rejection of frameworks in principle.

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| entity | [readme.md](readme.md) | Decision collection index |
| decision | [005_unknown_flags_rejected.md](005_unknown_flags_rejected.md) | The whitelist rule this parser enforces |
| invariant | [`../invariant/002_dep_constraints.md`](../invariant/002_dep_constraints.md) | Dependency constraints for the crate |
| source | `../../src/cli/parse.rs` | The parser itself |
| test | `../../tests/cli_args_test.rs` | Flag parsing coverage |
