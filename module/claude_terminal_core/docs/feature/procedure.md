# Feature Documentation Operations

- **Actor:** Developer
- **Trigger:** A new feature is added or an existing one is significantly changed.
- **Emits:** —

## Add Feature Documentation

1. Assign the next available ID (check `readme.md` Overview Table for current highest ID, increment by 1)
2. Create `NNN_{snake_case_name}.md` in this directory
3. Register in `readme.md` Overview Table: add row with ID, Name, Purpose, Status
4. Register in `../entity.md` Master Doc Instances Table and increment the `feature/` row's `Instances` count

## Update Feature Documentation

1. Edit the target `NNN_*.md` file
2. If name or purpose changed: update `readme.md` Overview Table row and the `../entity.md` instance row

## Example

Adding feature document `002_hyperlink_extraction`:

1. Check `readme.md` Overview Table — current highest ID is `001`
2. Create `002_hyperlink_extraction.md` in this directory
3. Add row: `| 002 | Hyperlink Extraction | Surface OSC 8 targets instead of dropping them | ✅ |`
4. Add the matching `../entity.md` instance row and bump `feature/` Instances from 1 to 2
