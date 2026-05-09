# API Documentation Operations

- **Actor:** Developer
- **Trigger:** A public API surface is defined or an existing API changes.
- **Emits:** —

## Add API Documentation

1. Assign the next available ID (check `readme.md` Overview Table for current highest ID, increment by 1)
2. Create `NNN_{snake_case_name}.md` in this directory
3. Register in `readme.md` Overview Table: add row with ID, Name, File link, Status

## Update API Documentation

1. Edit the target `NNN_*.md` file
2. If name or purpose changed: update `readme.md` Overview Table row

## Example

Adding API document `003_runner_config`:

1. Check `readme.md` Overview Table — current highest ID is `002`
2. Create `003_runner_config.md` in this directory
3. Add row: `| 003 | Api Name | [003_runner_config.md](003_runner_config.md) | Active |`
