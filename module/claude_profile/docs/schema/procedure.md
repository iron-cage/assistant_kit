# Schema Documentation Operations

- **Actor:** Developer
- **Trigger:** Addition of a new on-disk file format, changes to existing field definitions (names, types, defaults), or addition of new path resolution schemas in `PersistPaths` or `ClaudePaths`.
- **Emits:** —

## Add New Schema Instance

1. Verify the file format is not already covered by an existing instance (check `readme.md` Overview Table)
2. Confirm the file is written or read by `claude_profile` — read-only paths owned by the `claude` binary are out of scope unless `clp` reads them
3. Assign the next available ID (current highest ID in `readme.md` + 1)
4. Create `NNN_{file_basename}.md`; include: `### Scope` (4 bullets including `**Responsibility**`), `### Fields` table, optional `### Notes`, typed reference sections
5. Add a row to `readme.md` Overview Table: `| NNN | [Name](NNN_file.md) | One-sentence purpose | ✅ |`

## Update Existing Schema Instance

1. Edit the target `NNN_*.md` file to add, remove, or revise field rows in `### Fields`
2. If field semantics changed: update `### Notes` to explain migration or compatibility implications
3. Update typed reference sections if feature or invariant cross-references changed
4. If the file's path or basename changed: rename the instance file to match and update `readme.md` Overview Table row

## Retire Schema Instance

1. If a file format is no longer used by `claude_profile`, prepend a `> **Status: Retired** — file format removed` blockquote to the file
2. Update `readme.md` Overview Table Status column to `🗄️` and append `(retired)` to the Name link
3. Remove all cross-references to the retired instance from other doc instances
