# Feature Documentation Operations

- **Actor:** Developer
- **Trigger:** A new feature is added or an existing one is significantly changed.
- **Emits:** —

## Add Feature Documentation

1. Assign the next available ID (check `readme.md` Overview Table for current highest ID, increment by 1)
2. Create `NNN_{snake_case_name}.md` in this directory
3. Register in `readme.md` Overview Table: add row with ID, Name, Purpose, Status
4. Register in `../entity.md` Master Doc Instances Table and increment the `feature/` row's `Instances` count

## Update Feature Documentation

1. Edit the target `NNN_*.md` file
2. If name or purpose changed: update `readme.md` Overview Table row and the `../entity.md` instance row

## Rule

This crate reads a format owned by another program. When a documented field's on-disk shape is observed to differ from what a reasonable reader would assume — a number stored as a string, a status that lies under a known condition — record the observation and the date, not just the workaround. The next reader needs to know which claims were verified and when.

## Example

Adding feature document `003_registry_watch`:

1. Check `readme.md` Overview Table — current highest ID is `002`
2. Create `003_registry_watch.md` in this directory
3. Add row: `| 003 | Registry Watch | Notify on registry change without polling | ✅ |`
4. Add the matching `../entity.md` instance row and bump `feature/` Instances from 2 to 3
