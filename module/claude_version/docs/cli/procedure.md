# CLI Reference Documentation Operations

- **Actor:** Developer
- **Trigger:** A new CLI command is added or an existing reference document changes.
- **Emits:** —

## Add CLI Reference Document

1. Assign the next available ID (check `readme.md` Completion Matrix for current highest ID, increment by 1)
2. Create `NNN_{snake_case_name}.md` in this directory
3. Register in `readme.md` Completion Matrix: add row with file name and level indicators
4. Add a Navigation entry in `readme.md`

## Update CLI Reference Document

1. Edit the target `NNN_*.md` file
2. If name or purpose changed: update `readme.md` Completion Matrix row and Navigation entry

## Example

Adding CLI reference document `008_error_catalog`:

1. Check `readme.md` Completion Matrix — current highest ID is `007`
2. Create `008_error_catalog.md` in this directory
3. Add Completion Matrix row: `| 008_error_catalog.md | ... | Complete |`
4. Add Navigation entry: `- [Error Catalog](008_error_catalog.md) — error code reference`
