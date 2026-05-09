# Pattern Documentation Operations

- **Actor:** Developer
- **Trigger:** A new architectural pattern is established or an existing one changes.
- **Emits:** —

## Add Pattern Documentation

1. Assign the next available ID (check `readme.md` Overview Table for current highest ID, increment by 1)
2. Create `NNN_{snake_case_name}.md` in this directory
3. Register in `readme.md` Overview Table: add row with ID, Name, File link, Status

## Update Pattern Documentation

1. Edit the target `NNN_*.md` file
2. If name or purpose changed: update `readme.md` Overview Table row

## Example

Adding pattern document `003_builder`:

1. Check `readme.md` Overview Table — current highest ID is `002`
2. Create `003_builder.md` in this directory
3. Add row: `| 003 | Pattern Name | [003_builder.md](003_builder.md) | Active |`
