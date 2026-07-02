# Feature Documentation Operations

- **Actor:** Developer
- **Trigger:** A new feature requirement is identified, or an existing feature's scope, acceptance criteria, or design changes materially.
- **Emits:** —

## Add New Feature Instance

1. Verify the feature is not already covered by an existing instance (check `readme.md` Overview Table)
2. Assign the next available ID (current highest ID + 1; IDs 041–060 are reserved — skip to 061 or higher as needed)
3. Create `NNN_{snake_case_name}.md`; include: `### Scope` (4 bullets), design/content sections (H3), `### Acceptance Criteria` (H3), typed reference sections (`### Features`, `### Invariants`, etc.)
4. Add a row to `readme.md` Overview Table: `| NNN | [Name](NNN_file.md) | One-sentence purpose | ✅ |`
5. Add a row to `entity/readme.md` Master Doc Instances Table: `| feature | NNN | Name | [feature/NNN_file.md](../feature/NNN_file.md) |`

## Update Existing Feature Instance

1. Edit the target `NNN_*.md` file to reflect revised design, updated acceptance criteria, or new cross-references
2. Update typed reference sections if cross-references to other features, invariants, or algorithms changed
3. If the feature's purpose changed materially: update `readme.md` Overview Table row Purpose column
4. Update `doc_graph.yml` if edges to or from this feature node changed

## Retire Feature Instance

1. If a feature is superseded or removed from scope, prepend a `> **Status: Deprecated** — [brief description]` blockquote to the file
2. Update `readme.md` Overview Table Status column to `❌` and suffix the Name with `— DEPRECATED`
3. Remove or update all cross-references to the retired instance from other doc instances and `doc_graph.yml`
