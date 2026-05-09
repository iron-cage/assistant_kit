# Integration Documentation Operations

- **Actor:** Developer
- **Trigger:** A new cross-workspace integration protocol is defined or an existing one changes.
- **Emits:** —

## Add Integration Documentation

1. Assign the next available ID (check `readme.md` Overview Table for current highest ID, increment by 1)
2. Create `NNN_{snake_case_name}.md` in this directory
3. Register in `readme.md` Overview Table: add row with ID, Name, File link, Status

## Update Integration Documentation

1. Edit the target `NNN_*.md` file
2. If name or purpose changed: update `readme.md` Overview Table row

## Example

Adding integration document `004_claude_api`:

1. Check `readme.md` Overview Table — current highest ID is `003`
2. Create `004_claude_api.md` in this directory
3. Add row: `| 004 | Integration Name | [004_claude_api.md](004_claude_api.md) | Active |`
