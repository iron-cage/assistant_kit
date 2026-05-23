# Parameter Group Test Spec Operations

- **Actor:** Developer
- **Trigger:** A new parameter group is defined or an existing group test spec needs revision.
- **Emits:** —

## Add Parameter Group Test Spec

1. Assign the next available ID (check `readme.md` Responsibility Table for current highest ID, increment by 1)
2. Create `NNN_{snake_case_group_name}.md` in this directory
3. Include: group summary, interaction test index, cross-parameter semantics
4. Register in `readme.md` Responsibility Table: add row with filename and responsibility
5. Add Navigation entry in parent `readme.md` Parameter Groups list

## Update Parameter Group Test Spec

1. Edit the target `NNN_*.md` file
2. If group name or responsibility changed: update `readme.md` Responsibility Table row
3. If interaction tests added/removed: update test case index and coverage summary

## Example

Adding parameter group test spec for Credential Control:

1. Check `readme.md` Responsibility Table — current highest ID is `003`
2. Create `004_credential_control.md` with interaction test index
3. Add row: `| 004_credential_control.md | Interaction tests for Group 4 (Credential Control) |`
4. Add to parent Navigation: `- [Credential Control](param_group/004_credential_control.md)`
