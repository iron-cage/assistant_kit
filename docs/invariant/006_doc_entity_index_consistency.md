# Invariant: Doc Entity Index Consistency

### Scope

- **Purpose**: Guarantee that every `entity.md` module index file accurately reflects the documentation instances present on disk.
- **Responsibility**: State the correctness contract for the `entity.md` Master Doc Entities Table and Master Doc Instances Table.
- **In Scope**: Instance counts in `entity.md` Master Doc Entities Table, file existence for every row in Master Doc Instances Table, NNN prefix naming for all listed instances.
- **Out of Scope**: Content quality of individual doc instances (→ `docs/invariant/003_testing_strategy.md`); entity.md files outside the `assistant` workspace.

### Invariant Statement

For every `entity.md` file across the workspace:

1. **Count accuracy**: The `Instances` column for each entity row equals the number of `NNN_*.md` files present in the corresponding entity directory (ignoring `readme.md`, `procedure.md`, and non-NNN files).
2. **File existence**: Every file path listed in the Master Doc Instances Table resolves to an existing file on disk.
3. **NNN naming**: All instance files referenced by `entity.md` follow the `NNN_snake_case_name.md` naming convention (three-digit zero-padded prefix).

### Measurement

| Check | Method | Target |
|-------|--------|--------|
| Count accuracy | Count NNN-prefixed `.md` files per entity dir; diff against `Instances` column | Delta: 0 |
| File existence | `stat` each path listed in Master Doc Instances Table | Missing files: 0 |
| NNN naming | Regex `^[0-9]{3}_[a-z]` on all listed filenames | Non-conforming: 0 |

### Violation Consequences

A stale instance count misleads contributors about documentation coverage, causes navigational errors when editors use the index to discover instances, and silently breaks tools that rely on the index for cross-reference validation. A listed-but-absent file causes broken links.

### Sources

| File | Relationship |
|------|--------------|
| `docs/entity.md` | Workspace-level entity.md — primary verification target |
| `module/*/docs/entity.md` | Per-crate entity.md files — each must satisfy this invariant |
| `doc_des.rulebook.md § Collection : Module Index : entities.md` | Defines entity.md schema and maintenance rules |
