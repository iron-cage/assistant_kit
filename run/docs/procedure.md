# Runbox Documentation Operations

- **Actor:** Developer
- **Trigger:** A new dimension of runbox variability is identified requiring its own collection (beyond the existing `parameter/` and `plugin/` sub-collections).
- **Emits:** —

## Add Analysis Dimension

1. Create `run/docs/{dimension}/` directory
2. Create `readme.md` with Scope and Overview Table, following the pattern in `parameter/readme.md`
3. Create `procedure.md` with Add and Update procedures for the new instance type
4. Register in `run/docs/readme.md` Responsibility Table: add row for the new directory

**Note:** To add or update an individual parameter, use `parameter/procedure.md`. To add or update an individual plugin, use `plugin/procedure.md`.

## Example

Adding a `secret/` analysis dimension for secret injection slots:

1. Create `run/docs/secret/` directory
2. Create `secret/readme.md` with Scope and Overview Table (follow `parameter/readme.md` pattern)
3. Create `secret/procedure.md` with Add and Update procedures for secret instances
4. Add row to `run/docs/readme.md`: `| \`secret/\` | Per-secret reference for all secret injection slots |`
