# Operation Documentation Operations

- **Actor:** Developer
- **Trigger:** A new CLI operation is documented or an existing one changes.
- **Emits:** —

## Add Operation Documentation

1. Assign the next available ID (check `readme.md` for the current highest ID, increment by 1)
2. Create `NNN_{snake_case_name}.md` in this directory
3. Register in `readme.md` Overview Table: add row with ID, Name, File link, Status

## Update Operation Documentation

1. Edit the target `NNN_*.md` file
2. If name or purpose changed: update `readme.md` Overview Table row

## Example

Adding operation document `003_run`:

1. Check `readme.md` Overview Table — current highest ID is `002`
2. Create `003_run.md` in this directory
3. Add row: `| 003 | Operation Name | [003_run.md](003_run.md) | Active |`
