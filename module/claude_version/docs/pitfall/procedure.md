# Pitfall Documentation Operations

- **Actor:** Developer
- **Trigger:** A design trap is confirmed through implementation or testing.
- **Emits:** —

## Add Pitfall Instance

1. Assign the next available ID (check `readme.md` Overview Table for current highest ID, increment by 1)
2. Create `NNN_{snake_case_name}.md` in this directory — include Trap, Failure, Mitigation, and cross-reference sections
3. Register in `readme.md` Overview Table: add row with ID, Status, Name
4. Add `### Pitfalls` cross-reference row to any related pattern or feature doc instances

## Update Pitfall Instance

1. Edit the target `NNN_*.md` file
2. If name changed: update `readme.md` Overview Table row

## Retire Pitfall Instance

1. Remove the `NNN_*.md` file
2. Remove its row from `readme.md` Overview Table
3. Remove any `### Pitfalls` cross-reference rows pointing to this instance from pattern and feature doc instances
