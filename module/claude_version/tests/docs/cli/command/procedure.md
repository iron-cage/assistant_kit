# Command Test Spec Operations

- **Actor:** Developer
- **Trigger:** A new command is implemented or an existing command test spec needs revision.
- **Emits:** —

## Add Command Test Spec

1. Assign the next available ID (check `readme.md` Responsibility Table for current highest ID, increment by 1)
2. Create `NNN_{snake_case_command_name}.md` in this directory
3. Include: test factor analysis, test case index with IDs, expected behavior per case
4. Register in `readme.md` Responsibility Table: add row with filename and responsibility
5. Add Navigation entry in parent `readme.md` Commands list

## Update Command Test Spec

1. Edit the target `NNN_*.md` file
2. If command name or responsibility changed: update `readme.md` Responsibility Table row
3. If test cases added/removed: update test case index and coverage summary

## Example

Adding command test spec for `.version.prune`:

1. Check `readme.md` Responsibility Table — current highest ID is `012`
2. Create `013_version_prune.md` with test factor analysis and test case index
3. Add row: `| 013_version_prune.md | Integration tests for .version.prune command |`
4. Add to parent Navigation: `- [.version.prune](command/013_version_prune.md)`
