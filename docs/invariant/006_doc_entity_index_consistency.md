# Invariant: Doc Entity Index Consistency

### Scope

- **Purpose**: Guarantee that every doc entity registry accurately reflects the documentation instances present on disk.
- **Responsibility**: State the correctness contract for a registry's Master Doc Entities Table and Master Doc Instances Table.
- **In Scope**: Both registry shapes — `docs/entity.md` and `docs/entity/readme.md`; instance counts in the Master Doc Entities Table, file existence for every row in the Master Doc Instances Table, entity directory resolution via each entity row's Master File link.
- **Out of Scope**: Content quality of individual doc instances (→ `docs/invariant/003_testing_strategy.md`); registries outside the `assistant` workspace; prefix naming conventions of instance files — those vary by entity family by design (`NNN_` per doc_des for general doc collections; `NN_` and `cmd_NNN_` for CLI families per cli_doc_des) and are governed by each family's own design ruleset, not by this invariant.

### Invariant Statement

A **registry** is `docs/entity.md` or `docs/entity/readme.md` — the same index, in the two shapes the workspace uses. Both satisfy this invariant identically; the shape is not itself a subject of it. For every registry across the workspace:

1. **Count accuracy**: The `Instances` column for each entity row equals the number of instance files — every `*.md` except `readme.md` (the registry) and `procedure.md` (the ops doc) — present in the entity directory, resolved as the parent of the row's own Master File link target (entities outside `docs/`, e.g. `tests/docs/*`, link with a `../` prefix).
2. **File existence**: Every file path listed in the Master Doc Instances Table resolves to an existing file on disk, relative to the registry's parent directory.

**The count means files on disk, not live instances.** A retired instance keeps its row, annotated REMOVED / DEPRECATED / SUPERSEDED, and keeps being counted. A registry that instead omits retired instances makes its own figure mean something else, and the check silently stops applying to it — which is how `claude_profile` accumulated eight divergences while reporting a convention that explained only some of them.

### Measurement

| Check | Method | Target |
|-------|--------|--------|
| Count accuracy | Count `*.md` files (excluding `readme.md`, `procedure.md`) per entity dir; diff against `Instances` column | Delta: 0 |
| File existence | `stat` each path listed in Master Doc Instances Table | Missing files: 0 |

### Violation Consequences

A stale instance count misleads contributors about documentation coverage, causes navigational errors when editors use the index to discover instances, and silently breaks tools that rely on the index for cross-reference validation. A listed-but-absent file causes broken links.

### Sources

| File | Relationship |
|------|--------------|
| `docs/entity.md` | Workspace-level registry — primary verification target |
| `module/*/docs/entity.md` | Per-crate registries, flat shape — each must satisfy this invariant |
| `module/*/docs/entity/readme.md` | Per-crate registries, directory shape — same contract; `claude_journal`, `claude_journal_viewer`, `claude_profile` |
| `doc_des.rulebook.md § Collection : Module Index : entities.md` | Defines registry schema and maintenance rules |
