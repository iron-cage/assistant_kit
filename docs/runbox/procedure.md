# Runbox Documentation Operations

- **Actor:** Developer
- **Trigger:** A new dimension of runbox variability is identified requiring its own collection (beyond the existing `parameter/` and `plugin/` sub-collections).
- **Emits:** —

## Add Analysis Dimension

1. Create `docs/runbox/{dimension}/` directory
2. Create `readme.md` with Scope and Overview Table, following the pattern in `parameter/readme.md`
3. Create `procedure.md` with Add and Update procedures for the new instance type
4. Register in `docs/runbox/readme.md` Responsibility Table: add row for the new directory

**Note:** To add or update an individual parameter, use `parameter/procedure.md`. To add or update an individual plugin, use `plugin/procedure.md`.
