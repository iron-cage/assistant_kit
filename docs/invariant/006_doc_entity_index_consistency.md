# Invariant: Doc Entity Index Consistency

### Scope

- **Purpose**: Guarantee that every `entity.md` module index file accurately reflects the documentation instances present on disk.
- **Responsibility**: State the correctness contract for the `entity.md` Master Doc Entities Table and Master Doc Instances Table.
- **In Scope**: Instance counts in `entity.md` Master Doc Entities Table, file existence for every row in Master Doc Instances Table, entity directory resolution via each entity row's Master File link.
- **Out of Scope**: Content quality of individual doc instances (→ `docs/invariant/003_testing_strategy.md`); entity.md files outside the `assistant` workspace; prefix naming conventions of instance files — those vary by entity family by design (`NNN_` per doc_des for general doc collections; `NN_` and `cmd_NNN_` for CLI families per cli_doc_des) and are governed by each family's own design ruleset, not by this invariant.

### Invariant Statement

For every `entity.md` file across the workspace:

1. **Count accuracy**: The `Instances` column for each entity row equals the number of instance files — every `*.md` except `readme.md` (the registry) and `procedure.md` (the ops doc) — present in the entity directory, resolved as the parent of the row's own Master File link target (entities outside `docs/`, e.g. `tests/docs/*`, link with a `../` prefix).
2. **File existence**: Every file path listed in the Master Doc Instances Table resolves to an existing file on disk, relative to the `entity.md` parent directory.

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
| `docs/entity.md` | Workspace-level entity.md — primary verification target |
| `module/*/docs/entity.md` | Per-crate entity.md files — each must satisfy this invariant |
| `doc_des.rulebook.md § Collection : Module Index : entities.md` | Defines entity.md schema and maintenance rules |
