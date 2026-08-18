# Type Documentation Operations

- **Actor:** Developer
- **Trigger:** Introduction of a new Domain Type (struct/enum with domain validation semantics), a change to an existing type's identity/validation rules, or retirement of a type from the domain model.
- **Emits:** —

## Add New Type Instance

1. Verify the type is not already covered (check `readme.md` Overview Table) and is not on the Deliberately Not Instances list — if listed there, re-argue the exclusion first
2. Confirm it is a Domain Type: domain-meaningful validation or identity rules beyond its code definition (generic structures → not here; on-disk formats → `schema/`; CLI param value types → `cli/type/`)
3. Assign the next available ID (current highest ID in `readme.md` + 1)
4. Create `NNN_{type_name}.md`; include: `### Scope` (4 bullets including `**Responsibility**`), `### Definition`, `### Validation`, optional `### Relationships` / `### Serialization`, typed reference sections
5. Classify `domain` and `ddd` (value_object | entity | aggregate_root | dto) consistently with the type's identity and mutability semantics
6. Add a row to `readme.md` Overview Table and to `entity/readme.md`'s Master Doc Instances Table; increment the `type/` Instances count in the Master Doc Entities Table

## Update Existing Type Instance

1. Edit the target `NNN_*.md` to revise Definition/Validation/Relationships/Serialization
2. If the type's status changed (📋 planned → ✅ implemented, or retirement): update the Status column in `readme.md` Overview Table
3. Update typed reference sections in co-occurring type instances if relationships changed

## Retire Type Instance

1. Prepend a `> **Status: Retired** — type removed from domain model` blockquote to the file
2. Update `readme.md` Overview Table Status to `🗄️` and append `(retired)` to the Name link; keep the `entity/readme.md` row per that file's lifecycle conventions
