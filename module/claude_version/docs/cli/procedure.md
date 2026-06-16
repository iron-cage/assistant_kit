# CLI Documentation Operations

- **Actor:** Developer
- **Trigger:** A CLI entity or instance is added, removed, or significantly changed.
- **Emits:** —

## Add Entity Directory

When introducing a new entity type (e.g., `command_noun`, `entities`):

1. Verify the entity is in `cli_doc_des.rulebook.md § Completion Levels : Docs CLI Entities Table`, or is a registered project extension per `§ Architecture : Entity Type Extension Protocol`
2. Create `<entity>/readme.md` with a Scope block and empty Responsibility Table
3. Create `<entity>/procedure.md` with an Add/Update/Remove procedure for that entity's instances
4. Register in `cli/readme.md` Completion Matrix: add a row with name, level indicators, and Status
5. Add a Navigation entry in `cli/readme.md`

## Add Instance File

When adding a new instance to an existing entity directory (e.g., a new command, parameter, or user story):

1. Open `<entity>/readme.md` — run the One-Second Test: does any existing file have the same responsibility? → Yes = use existing; No = proceed
2. Add a row to the Responsibility Table: `| NNN_name.md | <single-sentence responsibility> |`
3. Add a row to the `### All <Entities>` navigation table
4. Create `<entity>/NNN_name.md` following the canonical template for that entity type
5. Update bidirectional cross-references in all affected entity files

## Update Instance File

1. Edit the target `<entity>/NNN_name.md`
2. If the instance's purpose changed: update the Responsibility Table row in `<entity>/readme.md`
3. Update all affected cross-references in other entity files (bidirectional)

## Remove Instance File

1. Delete `<entity>/NNN_name.md`
2. Remove its row from `<entity>/readme.md` Responsibility Table and navigation table
3. Remove all cross-references to the deleted instance across all entity files

## Update Completion Matrix

In `cli/readme.md`, when an entity reaches a new level milestone:

1. Update the entity row's level indicator for that column (➖ → ✅)
2. Update `**Current Level:**` only after ALL entity rows at that level are ✅ and gate validation passes
3. Gate validation requires: all criteria checked, automated checks pass, maintainer approval
