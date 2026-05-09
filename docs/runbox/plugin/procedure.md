# Plugin Analysis Operations

- **Actor:** Developer
- **Trigger:** A runbox plugin slot is added, removed, or its status, mechanism, or behavior changes.
- **Emits:** —

## Add Plugin

1. Assign the next available ID (check `readme.md` Overview Table for current highest ID, increment by 1)
2. Create `NNN_{snake_case_name}.md` with Status, Controls, Mechanism, and Notes
3. Register in `readme.md` Overview Table: add row with ID, Plugin, Status, Category

## Update Plugin

1. Edit the target `NNN_*.md` file
2. If Status changed: update `readme.md` Overview Table Status column

## Example

Adding plugin `007_git_plugin`:

1. Check `readme.md` Overview Table — current highest ID is `006`
2. Create `007_git_plugin.md` with Status, Controls, Mechanism, and Notes sections
3. Add row: `| [007](007_git_plugin.md) | \`git_plugin\` | ⚠️ | VCS Integration |`
