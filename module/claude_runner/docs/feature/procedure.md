# Feature Documentation Operations

- **Actor:** Developer
- **Trigger:** A new feature is added or an existing one is significantly changed.
- **Emits:** —

## Add Feature Documentation

1. Assign the next available ID (check `readme.md` Overview Table for current highest ID, increment by 1)
2. Create `NNN_{snake_case_name}.md` in this directory
3. Register in `readme.md` Overview Table: add row with ID, Name, File link, Status

## Update Feature Documentation

1. Edit the target `NNN_*.md` file
2. If name or purpose changed: update `readme.md` Overview Table row

## Example

Adding feature document `005_output_format`:

1. Check `readme.md` Overview Table — current highest ID is `004`
2. Create `005_output_format.md` in this directory
3. Add row: `| 005 | Feature Name | [005_output_format.md](005_output_format.md) | ✅ |`
