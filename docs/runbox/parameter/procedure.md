# Parameter Analysis Operations

- **Actor:** Developer
- **Trigger:** A runbox parameter slot is added, removed, or its status, state, or flow changes.
- **Emits:** —

## Add Parameter

1. Assign the next available ID (check `readme.md` Overview Table for current highest ID, increment by 1)
2. Create `NNN_{snake_case_key}.md` with Status, Current State, Where It Flows, and Notes
3. Register in `readme.md` Overview Table: add row with ID, Parameter, Status, Category

## Update Parameter

1. Edit the target `NNN_*.md` file
2. If Status changed: update `readme.md` Overview Table Status column

## Example

Adding parameter `011_verbosity`:

1. Check `readme.md` Overview Table — current highest ID is `010`
2. Create `011_verbosity.md` with Status, Current State, Where It Flows, and Notes sections
3. Add row: `| [011](011_verbosity.md) | \`verbosity\` | ⚠️ | Test Execution |`
