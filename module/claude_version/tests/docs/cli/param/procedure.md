# Parameter Test Spec Operations

- **Actor:** Developer
- **Trigger:** A new parameter is added or an existing parameter test spec needs revision.
- **Emits:** —

## Add Parameter Test Spec

1. Assign the next available ID (check `readme.md` Responsibility Table for current highest ID, increment by 1)
2. Create `NNN_{snake_case_param_name}.md` in this directory
3. Include: edge case index with IDs, boundary values, invalid input cases
4. Register in `readme.md` Responsibility Table: add row with filename and responsibility
5. Add Navigation entry in parent `readme.md` Parameters list

## Update Parameter Test Spec

1. Edit the target `NNN_*.md` file
2. If parameter name or responsibility changed: update `readme.md` Responsibility Table row
3. If edge cases added/removed: update edge case index and coverage summary

## Example

Adding parameter test spec for `timeout::`:

1. Check `readme.md` Responsibility Table — current highest ID is `010`
2. Create `11_timeout.md` with edge case index and boundary tests
3. Add row: `| 11_timeout.md | Edge case tests for timeout:: parameter |`
4. Add to parent Navigation: `- [timeout::](param/11_timeout.md)`
