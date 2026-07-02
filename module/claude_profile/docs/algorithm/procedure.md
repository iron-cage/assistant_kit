# Algorithm Documentation Operations

- **Actor:** Developer
- **Trigger:** Discovery of a new multi-feature decision algorithm, or changes to an existing algorithm's inputs, decision table, output values, or primary source location.
- **Emits:** —

## Add New Algorithm Instance

1. Verify the algorithm is not already covered by an existing instance (check `readme.md` Overview Table)
2. Confirm the algorithm has cross-feature applicability or is complex enough to warrant isolation from its host feature doc
3. Assign the next available ID (current highest ID in `readme.md` + 1)
4. Create `NNN_{snake_case_name}.md`; include: `### Scope`, `### Abstract`, `### Algorithm` (H4 sub-sections for inputs, decision table, output), typed reference sections
5. Add a row to `readme.md` Overview Table: `| NNN | [Name](NNN_file.md) | One-sentence purpose | ✅ |`

## Update Existing Algorithm Instance

1. Edit the target `NNN_*.md` file to reflect revised inputs, decision logic, or output definitions
2. Update all typed reference sections if cross-references changed
3. If the algorithm's primary source location changed: update the entry point reference in `### Abstract`
4. If the algorithm's purpose changed materially: update `readme.md` Overview Table row Purpose column

## Retire Algorithm Instance

1. If an algorithm is subsumed by another or no longer applies, prepend a `> **Status: Retired** — superseded by [algorithm/NNN](NNN_file.md)` blockquote to the file
2. Update `readme.md` Overview Table Status column to `🗄️` and append `(retired)` to the Name link
3. Remove all cross-references to the retired instance from other doc instances
