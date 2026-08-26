# Invariant Documentation Operations

- **Actor:** Developer
- **Trigger:** A new constraint is identified, or an existing one changes scope.
- **Emits:** —

## Add Invariant Documentation

1. Assign the next available ID (check `readme.md` Overview Table for current highest ID, increment by 1)
2. Create `NNN_{snake_case_name}.md` in this directory
3. State the rule, its rationale, and the mechanical check that enforces it
4. Register in `readme.md` Overview Table: add row with ID, Name, Purpose, Status
5. Register in `../entity.md` Master Doc Instances Table and increment the `invariant/` row's `Instances` count

## Update Invariant Documentation

1. Edit the target `NNN_*.md` file
2. If the enforcing test moved or was renamed: update the Cross-References table
3. If name or purpose changed: update `readme.md` Overview Table row and the `../entity.md` instance row

## Rule

An invariant without a mechanical check is a comment. Every instance in this directory names the test that fails when the constraint is violated; adding one without that test is incomplete work.

## Example

Adding invariant document `003_no_blocking_in_drop`:

1. Check `readme.md` Overview Table — current highest ID is `002`
2. Create `003_no_blocking_in_drop.md` in this directory
3. Name `tests/drop_test.rs` as the enforcing check
4. Add row: `| 003 | No Blocking In Drop | `Drop` never joins a thread that can block | ✅ |`
5. Add the matching `../entity.md` instance row and bump `invariant/` Instances from 2 to 3
