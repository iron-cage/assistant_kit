# Invariant Documentation Operations

- **Actor:** Developer
- **Trigger:** A new non-functional constraint or architectural guarantee is identified, or an existing invariant's statement, enforcement mechanism, or violation consequences change.
- **Emits:** —

## Add New Invariant Instance

1. Verify the constraint is not already covered by an existing instance (check `readme.md` Overview Table)
2. Assign the next available ID (current highest ID in `readme.md` + 1)
3. Create `NNN_{snake_case_name}.md`; include: `### Scope` (4 bullets), `### Invariant Statement` (with measurable threshold), `### Enforcement Mechanism`, `### Violation Consequences`, typed reference sections (`### Features`, `### Sources`, etc.)
4. Add a row to `readme.md` Overview Table: `| NNN | [Name](NNN_file.md) | One-sentence constraint statement | ✅ |`
5. Add a row to `entity/readme.md` Master Doc Instances Table: `| invariant | NNN | Name | [invariant/NNN_file.md](../invariant/NNN_file.md) |`

## Update Existing Invariant Instance

1. Edit the target `NNN_*.md` file to reflect a revised statement, updated enforcement mechanism, or new violation examples
2. Update `### Enforcement Mechanism` if the detection method or responsible call sites changed
3. Update typed reference sections if cross-references to features, sources, or sibling invariants changed
4. If the invariant's purpose changed materially: update `readme.md` Overview Table row Purpose column

## Retire Invariant Instance

1. If an architectural change eliminates the constraint entirely, prepend a `> **Status: Resolved** — [description of change]` blockquote to the file
2. Update `readme.md` Overview Table Status column to `🗄️` and append `(resolved)` to the Name link
3. Remove all cross-references to the retired instance from other doc instances
