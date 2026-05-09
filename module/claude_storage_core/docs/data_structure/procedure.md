# Data Structure Documentation Operations

- **Actor:** Developer
- **Trigger:** A new core data structure is documented or an existing one changes.
- **Emits:** —

## Add Data Structure Documentation

1. Assign the next available ID (check `readme.md` for the current highest ID, increment by 1)
2. Create `NNN_{snake_case_name}.md` in this directory
3. Register in `readme.md` Overview Table: add row with ID, Name, File link, Status

## Update Data Structure Documentation

1. Edit the target `NNN_*.md` file
2. If name or purpose changed: update `readme.md` Overview Table row

## Example

Adding data structure document `004_storage_config`:

1. Check `readme.md` Overview Table — current highest ID is `003`
2. Create `004_storage_config.md` in this directory
3. Add row: `| 004 | Data Structure Name | [004_storage_config.md](004_storage_config.md) | Active |`
