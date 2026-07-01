# State Machine Documentation Operations

- **Actor:** Developer
- **Trigger:** Addition of a new lifecycle domain type, or changes to an existing type's states, valid transitions, terminal states, or behavioral invariants.
- **Emits:** —

## Add New State Machine Instance

1. Verify the lifecycle is not already covered by an existing instance (check `readme.md` Overview Table)
2. Assign the next available ID (current highest ID in `readme.md` + 1)
3. Create `NNN_{snake_case_type}.md`; include: `### Scope`, `### States`, `### Transitions`, `### Behavioral Invariants`, typed reference sections
4. Add a row to `readme.md` Overview Table: `| NNN | [Name](NNN_file.md) | One-sentence purpose | ✅ |`

## Update Existing State Machine Instance

1. Edit the target `NNN_*.md` file to reflect new states, revised transitions, or updated invariants
2. Update `### Behavioral Invariants` if invariant count or semantics changed
3. Update typed reference sections if cross-references to features, schemas, or subprocesses changed
4. If the domain type name changed: rename the file to match and update `readme.md` Overview Table row

## Retire State Machine Instance

1. If a lifecycle model is no longer applicable (e.g., the domain type was removed), prepend a `> **Status: Retired** — domain type removed` blockquote to the file
2. Update `readme.md` Overview Table Status column to `🗄️` and append `(retired)` to the Name link
3. Remove all cross-references to the retired instance from other doc instances
