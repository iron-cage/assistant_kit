# Pattern Documentation Operations

- **Actor:** Developer
- **Trigger:** Discovery of a design solution reused (or planned for reuse) at more than one call site, or changes to an existing pattern's problem, solution, applicability, or trade-offs.
- **Emits:** —

## Add New Pattern Instance

1. Verify the pattern is not already covered by an existing instance (check `readme.md` Overview Table)
2. Confirm the solution applies (or is planned to apply) at more than one call site — a single-site solution belongs in `feature/`, not here
3. Assign the next available ID (current highest ID in `readme.md` + 1)
4. Create `NNN_{snake_case_name}.md`; include: `### Scope`, `### Problem`, `### Solution`, `### Applicability`, `### Consequences`, typed reference sections
5. Add a row to `readme.md` Overview Table: `| NNN | [Name](NNN_file.md) | One-sentence purpose | ✅ |`

## Update Existing Pattern Instance

1. Edit the target `NNN_*.md` file to reflect the revised problem, solution, applicability, or trade-offs
2. Update all typed reference sections if cross-references or divergences changed
3. If the pattern gains a new application site: add it to `### Applicability` and to the relevant typed reference section
4. If the pattern's purpose changed materially: update `readme.md` Overview Table row Purpose column

## Retire Pattern Instance

1. If a pattern is subsumed by another or no longer applies, prepend a `> **Status: Retired** — superseded by [pattern/NNN](NNN_file.md)` blockquote to the file
2. Update `readme.md` Overview Table Status column to `🗄️` and append `(retired)` to the Name link
3. Remove all cross-references to the retired instance from other doc instances
